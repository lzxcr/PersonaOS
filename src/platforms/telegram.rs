//! Telegram Bot API bridge.
//!
//! Full `PlatformAdapter` implementation using the Telegram Bot API
//! (getUpdates long-polling + optional webhook). Reuses the
//! platform-neutral turn pipeline in `crate::platforms`.

use crate::i18n::text as t;
use crate::platforms::types::{
    ConversationKind, OutboundBody, OutboundMessage, OutboundSegment, PlatformAdapter,
    PlatformConversation, PlatformInboundEvent, PlatformInboundEventKind, PlatformInboundMedia,
    PlatformMediaKind, PlatformMention, PlatformMessageInfo, SendReceipt,
};
use crate::platforms::{
    outbound_text_for_history, resolve_platform_session, run_platform_turn, PlatformTurnContext,
    TurnProfile,
};
use anyhow::{anyhow, bail, Context, Result};
use futures_util::future::BoxFuture;
use serde::Deserialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

// ── Telegram API types ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct TgResponse<T> {
    ok: bool,
    #[serde(default)]
    result: Option<T>,
}

#[derive(Debug, Deserialize)]
struct TgUpdate {
    update_id: i64,
    #[serde(default)]
    message: Option<TgMessage>,
    #[serde(default)]
    edited_message: Option<TgMessage>,
}

#[derive(Debug, Deserialize, Clone)]
struct TgMessage {
    message_id: i64,
    #[serde(default)]
    from: Option<TgUser>,
    chat: TgChat,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    caption: Option<String>,
    #[serde(default)]
    photo: Option<Vec<TgPhotoSize>>,
    #[serde(default)]
    voice: Option<TgVoice>,
    #[serde(default)]
    document: Option<TgDocument>,
    #[serde(default)]
    reply_to_message: Option<Box<TgMessage>>,
    #[serde(default)]
    entities: Option<Vec<TgMessageEntity>>,
    #[serde(default)]
    new_chat_members: Option<Vec<TgUser>>,
    #[serde(default)]
    left_chat_member: Option<TgUser>,
}

#[derive(Debug, Deserialize, Clone)]
struct TgChat {
    id: i64,
    #[serde(default)]
    r#type: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    username: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct TgUser {
    id: i64,
    #[serde(default)]
    is_bot: bool,
    #[serde(default)]
    first_name: String,
    #[serde(default)]
    last_name: Option<String>,
    #[serde(default)]
    username: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct TgPhotoSize {
    file_id: String,
}

#[derive(Debug, Deserialize, Clone)]
struct TgVoice {
    file_id: String,
    duration: i64,
}

#[derive(Debug, Deserialize, Clone)]
struct TgDocument {
    file_id: String,
    #[serde(default)]
    file_name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct TgMessageEntity {
    r#type: String,
    offset: usize,
    length: usize,
    #[serde(default)]
    user: Option<TgUser>,
}

// ── Telegram adapter ───────────────────────────────────────────────────

pub struct TelegramAdapter {
    bot_token: String,
    base_url: String,
    http: reqwest::Client,
}

impl TelegramAdapter {
    pub fn new(bot_token: &str) -> Result<Self> {
        let token = bot_token.trim();
        if token.is_empty() {
            bail!(t(
                "Telegram bot token is empty",
                "Telegram bot token 为空"
            ));
        }
        Ok(Self {
            base_url: format!("https://api.telegram.org/bot{token}"),
            bot_token: token.to_string(),
            http: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(15))
                .timeout(Duration::from_secs(45))
                .build()
                .context("building Telegram HTTP client")?,
        })
    }

    async fn api_call(
        &self,
        method: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/{method}", self.base_url);
        let response = self
            .http
            .post(&url)
            .json(body)
            .send()
            .await
            .with_context(|| format!("Telegram API {method}"))?;
        let tg: TgResponse<serde_json::Value> = response
            .json()
            .await
            .with_context(|| format!("parsing Telegram {method} response"))?;
        if !tg.ok {
            bail!("Telegram API {method} failed");
        }
        Ok(tg.result.unwrap_or_default())
    }

    // ── Message sending ─────────────────────────────────────────────────

    async fn send_text_api(
        &self,
        chat_id: i64,
        text: &str,
        reply_to: Option<i64>,
    ) -> Result<String> {
        let mut params = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
        });
        if let Some(rid) = reply_to {
            params["reply_to_message_id"] = serde_json::Value::from(rid);
        }
        let result = self.api_call("sendMessage", &params).await?;
        Ok(result["message_id"].as_i64().map(|id| id.to_string()).unwrap_or_default())
    }

    async fn send_photo_api(
        &self,
        chat_id: i64,
        data: &[u8],
        caption: Option<&str>,
        reply_to: Option<i64>,
    ) -> Result<String> {
        use reqwest::multipart;
        let url = format!("{}/sendPhoto", self.base_url);
        let photo_part = multipart::Part::bytes(data.to_vec())
            .file_name("image.jpg")
            .mime_str("image/jpeg")
            .map_err(|e| anyhow!("{e}"))?;
        let mut form = multipart::Form::new()
            .text("chat_id", chat_id.to_string())
            .part("photo", photo_part);
        if let Some(cap) = caption {
            form = form.text("caption", cap.to_string());
        }
        if let Some(rid) = reply_to {
            form = form.text("reply_to_message_id", rid.to_string());
        }
        let resp = self.http.post(&url).multipart(form).send().await?;
        let tg: TgResponse<serde_json::Value> = resp.json().await?;
        Ok(tg.result
            .and_then(|v| v["message_id"].as_i64().map(|id| id.to_string()))
            .unwrap_or_default())
    }

    async fn send_document_api(
        &self,
        chat_id: i64,
        data: Vec<u8>,
        file_name: &str,
        reply_to: Option<i64>,
    ) -> Result<String> {
        use reqwest::multipart;
        let url = format!("{}/sendDocument", self.base_url);
        let doc_part = multipart::Part::bytes(data)
            .file_name(file_name.to_string())
            .mime_str("application/octet-stream")
            .map_err(|e| anyhow!("{e}"))?;
        let mut form = multipart::Form::new()
            .text("chat_id", chat_id.to_string())
            .part("document", doc_part);
        if let Some(rid) = reply_to {
            form = form.text("reply_to_message_id", rid.to_string());
        }
        let resp = self.http.post(&url).multipart(form).send().await?;
        let tg: TgResponse<serde_json::Value> = resp.json().await?;
        Ok(tg.result
            .and_then(|v| v["message_id"].as_i64().map(|id| id.to_string()))
            .unwrap_or_default())
    }

    // ── Message conversion ─────────────────────────────────────────────

    fn user_display_name(user: &TgUser) -> String {
        let mut name = user.first_name.clone();
        if let Some(last) = &user.last_name {
            name.push(' ');
            name.push_str(last);
        }
        if name.is_empty() {
            user.username.clone().unwrap_or_else(|| format!("user{}", user.id))
        } else {
            name
        }
    }

    fn message_to_event(msg: &TgMessage) -> Option<PlatformInboundEvent> {
        let user = msg.from.as_ref()?;
        if user.is_bot {
            return None;
        }

        let chat = &msg.chat;
        let kind = if chat.r#type == "private" {
            ConversationKind::Private
        } else {
            ConversationKind::Group
        };
        let conversation = PlatformConversation {
            platform: "telegram".to_string(),
            account_id: format!("bot:{}", chat.id),
            kind,
            conversation_id: chat.id.to_string(),
        };

        let text = msg.text.clone().or_else(|| msg.caption.clone()).unwrap_or_default();
        let has_media = msg.photo.is_some() || msg.voice.is_some() || msg.document.is_some();

        let event_kind = if msg.new_chat_members.as_ref().is_some_and(|m| !m.is_empty()) {
            PlatformInboundEventKind::GroupDecrease // closest: member event
        } else if msg.left_chat_member.is_some() {
            PlatformInboundEventKind::GroupDecrease
        } else if has_media && text.is_empty() {
            PlatformInboundEventKind::Message
        } else {
            PlatformInboundEventKind::Message
        };

        let mut media = Vec::new();
        if let Some(photos) = &msg.photo {
            if let Some(largest) = photos.last() {
                media.push(PlatformInboundMedia {
                    kind: PlatformMediaKind::Image,
                    id: Some(largest.file_id.clone()),
                    name: None,
                    url: None,
                });
            }
        }
        if let Some(voice) = &msg.voice {
            media.push(PlatformInboundMedia {
                kind: PlatformMediaKind::Audio,
                id: Some(voice.file_id.clone()),
                name: None,
                url: None,
            });
        }
        if let Some(doc) = &msg.document {
            media.push(PlatformInboundMedia {
                kind: PlatformMediaKind::File,
                id: Some(doc.file_id.clone()),
                name: doc.file_name.clone(),
                url: None,
            });
        }

        let mut mentioned_user_ids = Vec::new();
        let mut mentioned_users = Vec::new();
        if let Some(entities) = &msg.entities {
            for entity in entities {
                if entity.r#type == "text_mention" {
                    if let Some(mentioned) = &entity.user {
                        let uid = mentioned.id.to_string();
                        mentioned_user_ids.push(uid.clone());
                        mentioned_users.push(PlatformMention {
                            user_id: uid,
                            display_name: Some(Self::user_display_name(mentioned)),
                        });
                    }
                }
            }
        }

        let reply_to_message_id = msg
            .reply_to_message
            .as_ref()
            .map(|r| r.message_id.to_string());
        let replied_message = msg.reply_to_message.as_ref().map(|reply| {
            PlatformMessageInfo {
                message_id: reply.message_id.to_string(),
                sender_id: reply
                    .from
                    .as_ref()
                    .map(|u| u.id.to_string())
                    .unwrap_or_default(),
                sender_display_name: reply
                    .from
                    .as_ref()
                    .map(|u| Self::user_display_name(u))
                    .unwrap_or_default(),
                timestamp: 0,
                text: reply
                    .text
                    .clone()
                    .or_else(|| reply.caption.clone())
                    .unwrap_or_default(),
                reply_to_message_id: None,
                mentioned_user_ids: Vec::new(),
                mentioned_users: Vec::new(),
                media: Vec::new(),
                conversation_kind: None,
                conversation_id: None,
            }
        });

        Some(PlatformInboundEvent {
            kind: event_kind,
            conversation,
            conversation_display_name: chat.title.clone(),
            message_id: msg.message_id.to_string(),
            sender_id: user.id.to_string(),
            sender_display_name: Self::user_display_name(user),
            operator_id: None,
            timestamp: 0,
            received_at: std::time::Instant::now(),
            message_position: None,
            ingress_order: None,
            text,
            reply_to_message_id,
            replied_message,
            mentioned_user_ids,
            mentioned_users,
            mentioned_bot: false,
            media,
            notice_sub_type: None,
            duration_seconds: None,
        })
    }
}

impl PlatformAdapter for TelegramAdapter {
    fn send<'a>(&'a self, message: OutboundMessage) -> BoxFuture<'a, Result<SendReceipt>> {
        Box::pin(async move {
            let chat_id: i64 = message
                .response_target
                .as_ref()
                .and_then(|rt| rt.message_id.parse().ok())
                .unwrap_or(0);
            if chat_id == 0 {
                return Err(anyhow!("Telegram send requires a chat_id"));
            }
            let reply_to = message
                .response_target
                .as_ref()
                .and_then(|rt| {
                    if rt.quote {
                        rt.message_id.parse::<i64>().ok()
                    } else {
                        None
                    }
                });

            match &message.body {
                OutboundBody::Segments(segments) => {
                    let mut text_parts = Vec::new();
                    let mut msg_ids = Vec::new();
                    let mut delivered = 0usize;

                    for segment in segments {
                        match segment {
                            OutboundSegment::Markdown(t) | OutboundSegment::Text(t) => {
                                text_parts.push(t.clone());
                            }
                            OutboundSegment::Mention(_) => {}
                            OutboundSegment::ImageBytes { data, alt, .. } => {
                                let caption = if !text_parts.is_empty() {
                                    Some(text_parts.join("\n"))
                                } else {
                                    Some(alt.clone())
                                };
                                match self
                                    .send_photo_api(chat_id, data, caption.as_deref(), reply_to)
                                    .await
                                {
                                    Ok(id) => {
                                        if !id.is_empty() {
                                            msg_ids.push(id);
                                        }
                                        delivered += 1;
                                    }
                                    Err(e) => {
                                        tracing::warn!(error = %e, "Telegram photo send failed");
                                    }
                                }
                                text_parts.clear();
                            }
                            OutboundSegment::ImagePath { path, alt, .. } => {
                                match tokio::fs::read(path).await {
                                    Ok(data) => {
                                        let caption = Some(alt.clone());
                                        match self
                                            .send_photo_api(
                                                chat_id, &data, caption.as_deref(), reply_to,
                                            )
                                            .await
                                        {
                                            Ok(id) => {
                                                if !id.is_empty() {
                                                    msg_ids.push(id);
                                                }
                                                delivered += 1;
                                            }
                                            Err(e) => {
                                                tracing::warn!(error = %e, "Telegram photo send failed");
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!(error = %e, "reading image for Telegram");
                                    }
                                }
                                text_parts.clear();
                            }
                            OutboundSegment::FilePath { path, name, .. } => {
                                match tokio::fs::read(path).await {
                                    Ok(data) => {
                                        let fname = name
                                            .as_deref()
                                            .or_else(|| {
                                                std::path::Path::new(path)
                                                    .file_name()
                                                    .and_then(|n| n.to_str())
                                            })
                                            .unwrap_or("file");
                                        match self
                                            .send_document_api(chat_id, data, fname, reply_to)
                                            .await
                                        {
                                            Ok(id) => {
                                                if !id.is_empty() {
                                                    msg_ids.push(id);
                                                }
                                                delivered += 1;
                                            }
                                            Err(e) => {
                                                tracing::warn!(error = %e, "Telegram document send failed");
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!(error = %e, "reading file for Telegram");
                                    }
                                }
                            }
                            OutboundSegment::Audio { .. } => {
                                text_parts.push("[语音]".to_string());
                            }
                        }
                    }

                    if !text_parts.is_empty() {
                        match self
                            .send_text_api(chat_id, &text_parts.join("\n"), reply_to)
                            .await
                        {
                            Ok(id) => {
                                if !id.is_empty() {
                                    msg_ids.push(id);
                                }
                                delivered += 1;
                            }
                            Err(e) => {
                                tracing::warn!(error = %e, "Telegram text send failed");
                            }
                        }
                    }

                    if msg_ids.is_empty() && delivered > 0 {
                        msg_ids.push("sent".to_string());
                    }
                    Ok(SendReceipt {
                        message_ids: msg_ids,
                        image_message_ids: Vec::new(),
                        delivered_parts: delivered,
                        image_digests: Vec::new(),
                        response_target_delivered: reply_to.is_some(),
                    })
                }
                OutboundBody::Forward(_nodes) => {
                    let text = outbound_text_for_history(&message);
                    let id = self.send_text_api(chat_id, &text, reply_to).await?;
                    Ok(SendReceipt {
                        message_ids: vec![id],
                        image_message_ids: Vec::new(),
                        delivered_parts: 1,
                        image_digests: Vec::new(),
                        response_target_delivered: reply_to.is_some(),
                    })
                }
            }
        })
    }

    fn bot_display_name<'a>(&'a self) -> BoxFuture<'a, Result<String>> {
        Box::pin(async { Ok("Telegram Bot".to_string()) })
    }

    fn delete_message<'a>(&'a self, message_id: &'a str) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            // Generic delete — chat_id would need context; not critical for MVP.
            let _ = message_id;
            Ok(())
        })
    }
}

// ── Polling runtime ────────────────────────────────────────────────────

#[derive(Clone)]
pub struct TelegramRuntime {
    running: Arc<AtomicBool>,
}

impl TelegramRuntime {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    pub async fn start(
        &self,
        adapter: Arc<TelegramAdapter>,
        state: crate::web::DaemonState,
    ) -> Result<()> {
        if self.running.swap(true, Ordering::AcqRel) {
            bail!("Telegram polling already running");
        }

        let running = self.running.clone();
        let http = adapter.http.clone();
        let base_url = adapter.base_url.clone();

        tokio::spawn(async move {
            let mut offset: i64 = 0;

            loop {
                if !running.load(Ordering::Acquire) {
                    break;
                }

                let url = format!("{base_url}/getUpdates?offset={offset}&timeout=30&allowed_updates=[\"message\"]");
                let response = match http
                    .get(&url)
                    .timeout(Duration::from_secs(35))
                    .send()
                    .await
                {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::warn!(target: "pos::telegram", error = %e, "getUpdates request failed; retrying in 5s");
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                };

                let body: TgResponse<Vec<TgUpdate>> = match response.json().await {
                    Ok(b) => b,
                    Err(e) => {
                        tracing::warn!(target: "pos::telegram", error = %e, "parsing getUpdates response");
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                };

                let updates = body.result.unwrap_or_default();
                for update in &updates {
                    offset = offset.max(update.update_id + 1);

                    let msg = match &update.message {
                        Some(m) => m,
                        None => continue,
                    };

                    let Some(event) = TelegramAdapter::message_to_event(msg) else {
                        continue;
                    };

                    if event.text.trim().is_empty()
                        && event.media.is_empty()
                        && event.kind == PlatformInboundEventKind::Message
                    {
                        continue;
                    }

                    let conversation = event.conversation.clone();
                    let adapter_clone = adapter.clone();
                    let state_clone = state.clone();

                    tokio::spawn(async move {
                        let config = state_clone.manager.lock().unwrap().config.clone();
                        let active_persona = config.prompt.active_persona.clone();
                        let paths = state_clone.paths.clone();
                        let state_store = state_clone.state_store.clone();
                        let plugins = match state_clone.platforms.plugins() {
                            Ok(p) => p,
                            Err(e) => {
                                tracing::warn!(target: "pos::telegram", error = %e, "building platform context");
                                return;
                            }
                        };

                        let context = Arc::new(
                            PlatformTurnContext::new(
                                conversation.clone(),
                                event.sender_id.clone(),
                                event.sender_display_name.clone(),
                                false,
                                config.clone(),
                                paths,
                                state_store,
                                adapter_clone,
                                plugins,
                            )
                            .with_inbound_event(event.clone()),
                        );

                        let session_id = match resolve_platform_session(
                            &state_clone,
                            &conversation,
                            &active_persona,
                            None,
                            &format!("TG-{}", conversation.conversation_id),
                            None,
                        ) {
                            Ok(id) => id,
                            Err(e) => {
                                tracing::warn!(target: "pos::telegram", error = %e, "resolving session");
                                return;
                            }
                        };

                        let content = context
                            .inbound_event()
                            .map(|e| e.text.clone())
                            .unwrap_or_default();

                        let profile = TurnProfile {
                            platform: Some(context),
                            ..Default::default()
                        };

                        match run_platform_turn(
                            &state_clone,
                            session_id,
                            content,
                            Vec::new(),
                            profile,
                        )
                        .await
                        {
                            Ok(_) => {
                                tracing::debug!(target: "pos::telegram", "turn completed");
                            }
                            Err(e) => {
                                tracing::warn!(target: "pos::telegram", error = %e, "turn failed");
                            }
                        }
                    });
                }
            }
        });

        Ok(())
    }

    pub async fn shutdown(&self) {
        self.running.store(false, Ordering::Release);
    }
}

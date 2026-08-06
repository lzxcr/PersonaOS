//! Telegram Bot API bridge (stub — full implementation in progress).
//!
//! This module defines the `TelegramAdapter` skeleton implementing
//! `PlatformAdapter` for the Telegram Bot API.  The polling loop and
//! message conversion logic will be completed once the platform-neutral
//! core plumbing (daemon integration, config reload) is finalised.

use crate::platforms::types::{
    BotSendAvailability, OutboundBody, OutboundMessage, OutboundSegment, PlatformAdapter,
    SendReceipt,
};
use anyhow::Result;
use futures_util::future::BoxFuture;

pub struct TelegramAdapter;

impl TelegramAdapter {
    pub fn new(_bot_token: &str) -> Result<Self> {
        Ok(Self)
    }
}

impl PlatformAdapter for TelegramAdapter {
    fn send<'a>(&'a self, message: OutboundMessage) -> BoxFuture<'a, Result<SendReceipt>> {
        Box::pin(async move {
            let text = match &message.body {
                OutboundBody::Segments(segments) => {
                    let mut parts = Vec::new();
                    for segment in segments {
                        match segment {
                            OutboundSegment::Markdown(text) | OutboundSegment::Text(text) => {
                                parts.push(text.clone());
                            }
                            OutboundSegment::Mention(user_id) => {
                                parts.push(format!("@{user_id}"));
                            }
                            OutboundSegment::ImageBytes { alt, .. }
                            | OutboundSegment::ImagePath { alt, .. } => {
                                parts.push(format!("[图片: {alt}]"));
                            }
                            OutboundSegment::FilePath { name, .. } => {
                                parts.push(format!(
                                    "[文件: {}]",
                                    name.as_deref().unwrap_or("file")
                                ));
                            }
                            OutboundSegment::Audio { .. } => {
                                parts.push("[语音]".to_string());
                            }
                        }
                    }
                    parts.join("\n")
                }
                OutboundBody::Forward(_) => "[转发消息]".to_string(),
            };
            Ok(SendReceipt::text(text))
        })
    }

    fn bot_display_name<'a>(&'a self) -> BoxFuture<'a, Result<String>> {
        Box::pin(async { Ok("Telegram Bot".to_string()) })
    }
}

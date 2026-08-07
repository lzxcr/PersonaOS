//! 多模态模型选择页 — 从多模态池选择当前激活的模型。

use crate::config::ActiveProviderModelConfig;
use ratatui::widgets::ListState;

/// 页面状态。
#[derive(Default)]
pub struct MultimodalPage {
    pub state: ListState,
    /// 编辑模式：true 时在底部输入框输入 provider/model。
    pub editing: bool,
    pub edit_buffer: String,
}

impl MultimodalPage {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            state,
            editing: false,
            edit_buffer: String::new(),
        }
    }
}

/// 从编辑缓冲解析 "provider/model"；语义同文本模型页。
pub fn parse_multimodal_input(input: &str) -> Option<ActiveProviderModelConfig> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    let (provider, model) = match trimmed.split_once('/') {
        Some((p, m)) => (p.trim().to_string(), m.trim().to_string()),
        None => (trimmed.to_string(), String::new()),
    };
    if provider.is_empty() {
        return None;
    }
    Some(ActiveProviderModelConfig {
        provider_id: provider,
        model,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_multimodal_input_handles_forms() {
        assert!(parse_multimodal_input("").is_none());

        let full = parse_multimodal_input("openai/gpt-4o").unwrap();
        assert_eq!(full.provider_id, "openai");
        assert_eq!(full.model, "gpt-4o");

        let bare = parse_multimodal_input("gemini").unwrap();
        assert_eq!(bare.provider_id, "gemini");
        assert!(bare.model.is_empty());
    }
}

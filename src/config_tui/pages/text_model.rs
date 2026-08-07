//! 文本模型选择页 — 从模型池选择当前激活的文本模型。

use crate::config::ActiveProviderModelConfig;
use ratatui::widgets::ListState;

/// 页面状态。
#[derive(Default)]
pub struct TextModelPage {
    pub selected: usize,
    pub state: ListState,
    /// 编辑模式：true 时在底部输入框输入 provider/model。
    pub editing: bool,
    pub edit_buffer: String,
}

impl TextModelPage {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            selected: 0,
            state,
            editing: false,
            edit_buffer: String::new(),
        }
    }
}

/// 将模型选择标签格式化为带序号的显示行。
pub fn model_rows(labels: &[String]) -> Vec<String> {
    labels
        .iter()
        .enumerate()
        .map(|(i, label)| format!("{:>2}. {label}", i + 1))
        .collect()
}

/// 从编辑缓冲解析 "provider/model" 组合；缺省一侧可空。
pub fn parse_model_input(input: &str) -> Option<ActiveProviderModelConfig> {
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
    fn model_rows_add_indexes() {
        let labels = vec![
            "openai/gpt-4o-mini".to_string(),
            "deepseek/v3".to_string(),
        ];
        let rows = model_rows(&labels);
        assert_eq!(rows.len(), 2);
        assert!(rows[0].starts_with(" 1."));
        assert!(rows[1].starts_with(" 2."));
        assert!(rows[0].contains("openai/gpt-4o-mini"));
    }

    #[test]
    fn parse_model_input_handles_forms() {
        assert!(parse_model_input("").is_none());
        assert!(parse_model_input("   ").is_none());

        let full = parse_model_input("openai/gpt-4o-mini").unwrap();
        assert_eq!(full.provider_id, "openai");
        assert_eq!(full.model, "gpt-4o-mini");

        let bare = parse_model_input("openai").unwrap();
        assert_eq!(bare.provider_id, "openai");
        assert!(bare.model.is_empty());

        let spaced = parse_model_input("  deepseek / v3  ").unwrap();
        assert_eq!(spaced.provider_id, "deepseek");
        assert_eq!(spaced.model, "v3");
    }

    #[test]
    fn empty_labels_produce_empty_rows() {
        assert!(model_rows(&[]).is_empty());
    }
}

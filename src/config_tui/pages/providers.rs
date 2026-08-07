//! 供应商与模型页 — 供应商列表 CRUD + 选择激活。

use crate::config::ProviderConfig;
use ratatui::widgets::ListState;

/// 页面状态。
pub struct ProvidersPage {
    pub state: ListState,
    /// 字段总览/编辑位置（持久 ListState）。
    pub field_state: ListState,
    /// 编辑模式：true 时在内联表单编辑选中供应商字段。
    pub editing: bool,
    pub edit_field: usize,
    pub edit_buffer: String,
    /// 确认删除模式。
    pub confirming_delete: bool,
    /// 字段总览模式：true 时显示选中供应商的字段值列表。
    pub viewing: bool,
    pub error_msg: Option<String>,
}

impl ProvidersPage {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        let mut field_state = ListState::default();
        field_state.select(Some(0));
        Self {
            state,
            field_state,
            editing: false,
            edit_field: 0,
            edit_buffer: String::new(),
            confirming_delete: false,
            viewing: false,
            error_msg: None,
        }
    }
}

/// 供应商字段总览行：`标签 = 当前值`。
pub fn provider_field_rows(provider: &ProviderConfig) -> Vec<String> {
    EDITABLE_FIELDS
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let value = field_value(provider, i);
            let label = match *name {
                "id" => "ID",
                "display_name" => "显示名称",
                "base_url" => "Base URL",
                "api_key" => "API Key",
                "default_model" => "默认模型",
                "models" => "模型列表",
                _ => name,
            };
            format!("{label} = {value}")
        })
        .collect()
}

/// 供应商列表行：`id  display_name  base_url  (N 模型)  [active]`。
pub fn provider_rows(providers: &[ProviderConfig], active_id: &str) -> Vec<String> {
    providers
        .iter()
        .map(|provider| {
            let active = if provider.id == active_id { " ⭐" } else { "" };
            let model_count = if provider.models.is_empty()
                && !provider.default_model.trim().is_empty()
            {
                1
            } else {
                provider.models.len()
            };
            format!(
                "{:<14} {:<16} {:<40} ({} 模型){active}",
                provider.id,
                provider.display_name,
                provider.base_url,
                model_count
            )
        })
        .collect()
}

/// 可编辑字段的定义顺序（id/display_name/base_url/api_key/default_model/models）。
pub const EDITABLE_FIELDS: [&str; 6] = [
    "id",
    "display_name",
    "base_url",
    "api_key",
    "default_model",
    "models",
];

/// 读取字段当前值。
pub fn field_value(provider: &ProviderConfig, field: usize) -> String {
    match EDITABLE_FIELDS[field.min(EDITABLE_FIELDS.len() - 1)] {
        "id" => provider.id.clone(),
        "display_name" => provider.display_name.clone(),
        "base_url" => provider.base_url.clone(),
        "api_key" => provider
            .api_key
            .as_deref()
            .map(|key| key.trim_start_matches("$env:").to_string())
            .unwrap_or_default(),
        "default_model" => provider.default_model.clone(),
        "models" => provider.models.join(","),
        _ => String::new(),
    }
}

/// 将编辑结果写回供应商；返回是否发生变更。
pub fn apply_field(provider: &mut ProviderConfig, field: usize, value: &str) -> bool {
    let value = value.trim().to_string();
    match EDITABLE_FIELDS[field.min(EDITABLE_FIELDS.len() - 1)] {
        "id" => {
            if value.is_empty() {
                return false;
            }
            provider.id = value
        }
        "display_name" => provider.display_name = value,
        "base_url" => {
            if value.is_empty() {
                return false;
            }
            provider.base_url = value
        }
        "api_key" => provider.api_key = Some(value),
        "default_model" => provider.default_model = value,
        "models" => {
            provider.models = value
                .split(',')
                .map(str::trim)
                .filter(|m| !m.is_empty())
                .map(str::to_string)
                .collect()
        }
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_provider() -> ProviderConfig {
        let mut p = ProviderConfig::default_anthropic();
        p.id = "test".to_string();
        p.display_name = "Test Provider".to_string();
        p.base_url = "https://test.example.com/v1".to_string();
        p.api_key = Some("$env:TEST_KEY".to_string());
        p.default_model = "model-a".to_string();
        p.models = vec!["model-a".to_string(), "model-b".to_string()];
        p
    }

    #[test]
    fn rows_mark_active_and_count_models() {
        let p = test_provider();
        let rows = provider_rows(&[p], "test");
        assert_eq!(rows.len(), 1);
        assert!(rows[0].contains("⭐"));
        assert!(rows[0].contains("(2 模型)"));
    }

    #[test]
    fn rows_do_not_mark_inactive() {
        let p = test_provider();
        let rows = provider_rows(&[p], "other");
        assert!(!rows[0].contains("⭐"));
    }

    #[test]
    fn field_value_reads_all_fields() {
        let p = test_provider();
        assert_eq!(field_value(&p, 0), "test");
        assert_eq!(field_value(&p, 1), "Test Provider");
        assert_eq!(field_value(&p, 2), "https://test.example.com/v1");
        assert_eq!(field_value(&p, 3), "TEST_KEY");
        assert_eq!(field_value(&p, 4), "model-a");
        assert_eq!(field_value(&p, 5), "model-a,model-b");
    }

    #[test]
    fn apply_field_updates_and_validates() {
        let mut p = test_provider();
        assert!(apply_field(&mut p, 1, "  Renamed  "));
        assert_eq!(p.display_name, "Renamed");
        assert!(apply_field(&mut p, 3, "new-key"));
        assert_eq!(p.api_key.as_deref(), Some("new-key"));
        assert!(apply_field(&mut p, 5, "x, y ,z"));
        assert_eq!(p.models, vec!["x", "y", "z"]);
        assert!(!apply_field(&mut p, 0, "  "));
        assert!(!apply_field(&mut p, 2, ""));
    }
}

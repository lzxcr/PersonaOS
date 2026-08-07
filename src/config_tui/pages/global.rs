//! 全局参数设置页 — 工具/技能/显示选项。

use crate::config::AppConfig;
use ratatui::widgets::ListState;

/// 页面状态。
#[derive(Default)]
pub struct GlobalPage {
    pub state: ListState,
    /// 编辑模式：true 时在底部输入框输入新值。
    pub editing: bool,
    pub edit_buffer: String,
}

impl GlobalPage {
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

/// 全局设置行定义。
pub fn global_rows(config: &AppConfig) -> Vec<String> {
    vec![
        format!("工具启用            = {}", config.tools.enabled),
        format!("工具最大轮数        = {}", config.tools.max_rounds),
        format!("工具加载模式        = {}", config.tools.loading_mode),
        format!("记住已加载工具      = {}", config.tools.persist_loaded_tools),
        format!("Skills 启用         = {}", config.skills.enabled),
        format!("允许执行命令        = {}", config.skills.allow_command_execution),
        format!("界面语言            = {}", config.display.language),
        format!("显示思考过程        = {}", config.display.reasoning),
        format!("显示工具调用信息    = {}", config.display.tool_calls),
        format!("命令输出显示行数    = {}", config.display.command_output_lines),
        format!("工具名可读显示      = {}", config.display.readable_tool_names),
    ]
}

/// 全局设置字段（编辑用）。
pub const GLOBAL_FIELDS: [&str; 11] = [
    "tools.enabled",
    "tools.max_rounds",
    "tools.loading_mode",
    "tools.persist_loaded_tools",
    "skills.enabled",
    "skills.allow_command_execution",
    "display.language",
    "display.reasoning",
    "display.tool_calls",
    "display.command_output_lines",
    "display.readable_tool_names",
];

/// 读取字段当前值。
pub fn field_value(config: &AppConfig, field: usize) -> String {
    match GLOBAL_FIELDS[field.min(GLOBAL_FIELDS.len() - 1)] {
        "tools.enabled" => config.tools.enabled.to_string(),
        "tools.max_rounds" => config.tools.max_rounds.to_string(),
        "tools.loading_mode" => config.tools.loading_mode.clone(),
        "tools.persist_loaded_tools" => config.tools.persist_loaded_tools.to_string(),
        "skills.enabled" => config.skills.enabled.to_string(),
        "skills.allow_command_execution" => config.skills.allow_command_execution.to_string(),
        "display.language" => config.display.language.clone(),
        "display.reasoning" => config.display.reasoning.clone(),
        "display.tool_calls" => config.display.tool_calls.clone(),
        "display.command_output_lines" => config.display.command_output_lines.to_string(),
        "display.readable_tool_names" => config.display.readable_tool_names.to_string(),
        _ => String::new(),
    }
}

/// 将编辑结果写回配置；返回是否发生变更。
pub fn apply_field(config: &mut AppConfig, field: usize, value: &str) -> bool {
    let value = value.trim().to_string();
    match GLOBAL_FIELDS[field.min(GLOBAL_FIELDS.len() - 1)] {
        "tools.enabled" => {
            let Ok(v) = value.parse::<bool>() else {
                return false;
            };
            config.tools.enabled = v
        }
        "tools.max_rounds" => {
            let Ok(v) = value.parse::<usize>() else {
                return false;
            };
            config.tools.max_rounds = v
        }
        "tools.loading_mode" => {
            if !["full", "hybrid", "stub"].contains(&value.as_str()) {
                return false;
            }
            config.tools.loading_mode = value
        }
        "tools.persist_loaded_tools" => {
            let Ok(v) = value.parse::<bool>() else {
                return false;
            };
            config.tools.persist_loaded_tools = v
        }
        "skills.enabled" => {
            let Ok(v) = value.parse::<bool>() else {
                return false;
            };
            config.skills.enabled = v
        }
        "skills.allow_command_execution" => {
            let Ok(v) = value.parse::<bool>() else {
                return false;
            };
            config.skills.allow_command_execution = v
        }
        "display.language" => {
            let v = value.to_lowercase();
            if !["zh", "en", "auto"].contains(&v.as_str()) {
                return false;
            }
            config.display.language = v
        }
        "display.reasoning" => {
            if !["summary", "full", "hidden"].contains(&value.as_str()) {
                return false;
            }
            config.display.reasoning = value
        }
        "display.tool_calls" => {
            if !["summary", "full", "hidden"].contains(&value.as_str()) {
                return false;
            }
            config.display.tool_calls = value
        }
        "display.command_output_lines" => {
            let Ok(v) = value.parse::<usize>() else {
                return false;
            };
            config.display.command_output_lines = v
        }
        "display.readable_tool_names" => {
            let Ok(v) = value.parse::<bool>() else {
                return false;
            };
            config.display.readable_tool_names = v
        }
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> AppConfig {
        AppConfig::default()
    }

    #[test]
    fn global_rows_show_current_values() {
        let rows = global_rows(&test_config());
        assert_eq!(rows.len(), 11);
        assert!(rows[0].contains("工具启用"));
        assert!(rows[2].contains("full") || rows[2].contains("hybrid") || rows[2].contains("stub"));
    }

    #[test]
    fn apply_field_validates_values() {
        let mut config = test_config();
        assert!(apply_field(&mut config, 0, "false"));
        assert!(!config.tools.enabled);
        assert!(apply_field(&mut config, 6, "zh"));
        assert_eq!(config.display.language, "zh");
        assert!(!apply_field(&mut config, 6, "xx")); // invalid language
        assert!(apply_field(&mut config, 2, "stub"));
        assert_eq!(config.tools.loading_mode, "stub");
        assert!(!apply_field(&mut config, 2, "bogus")); // invalid loading mode
        assert!(apply_field(&mut config, 7, "hidden"));
        assert_eq!(config.display.reasoning, "hidden");
        assert!(!apply_field(&mut config, 1, "abc")); // invalid usize
        assert!(apply_field(&mut config, 9, "200"));
        assert_eq!(config.display.command_output_lines, 200);
    }
}

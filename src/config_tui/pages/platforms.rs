//! IM 平台页 — 平台启用/禁用 + 基础字段编辑。

use crate::config::AppConfig;
use ratatui::widgets::ListState;

/// 页面状态。
#[derive(Default)]
pub struct PlatformsPage {
    pub state: ListState,
    /// 编辑模式：true 时在内联表单编辑选中平台的字段。
    pub editing: bool,
    pub edit_field: usize,
    pub edit_buffer: String,
}

impl PlatformsPage {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            state,
            editing: false,
            edit_field: 0,
            edit_buffer: String::new(),
        }
    }
}

/// 平台列表：id、显示名、说明。
pub const PLATFORMS: [(&str, &str, &str); 3] = [
    ("qq", "QQ (NapCat)", "OneBot v11 反向 WebSocket"),
    ("telegram", "Telegram", "Bot API 长轮询"),
    ("qq_official", "QQ 官方机器人", "官方 WebSocket API"),
];

/// 平台是否启用。
pub fn platform_enabled(config: &AppConfig, index: usize) -> bool {
    match index {
        0 => config.platforms.qq.enabled,
        1 => config.platforms.telegram.as_ref().is_some_and(|t| t.enabled),
        2 => config.platforms.qq_official.as_ref().is_some_and(|q| q.enabled),
        _ => false,
    }
}

/// 切换平台启用状态。
pub fn toggle_platform(config: &mut AppConfig, index: usize) {
    match index {
        0 => config.platforms.qq.enabled = !config.platforms.qq.enabled,
        1 => {
            let t = config.platforms.telegram.get_or_insert_with(Default::default);
            t.enabled = !t.enabled;
        }
        2 => {
            let q = config.platforms.qq_official.get_or_insert_with(Default::default);
            q.enabled = !q.enabled;
        }
        _ => {}
    }
}

/// 平台列表行。
pub fn platform_rows(config: &AppConfig) -> Vec<String> {
    PLATFORMS
        .iter()
        .enumerate()
        .map(|(i, (id, name, desc))| {
            let mark = if platform_enabled(config, i) { "✓" } else { " " };
            format!("[{mark}] {name:<18} — {desc} ({id})")
        })
        .collect()
}

/// 平台可编辑字段。
pub fn platform_fields(config: &AppConfig, index: usize) -> Vec<String> {
    match index {
        0 => vec![
            "enabled".to_string(),
            "reverse_ws_port".to_string(),
            "access_token".to_string(),
            "admin_users".to_string(),
            "max_reply_chars".to_string(),
        ],
        1 => {
            let t = config.platforms.telegram.as_ref();
            vec![
                "enabled".to_string(),
                "bot_token".to_string(),
                "webhook_path".to_string(),
                "admin_users".to_string(),
                "max_reply_chars".to_string(),
            ]
            .into_iter()
            .map(|f| if f == "enabled" { format!("{f} = {}", t.is_some_and(|t| t.enabled)) } else { f })
            .collect()
        }
        2 => {
            let q = config.platforms.qq_official.as_ref();
            vec![
                "enabled".to_string(),
                "app_id".to_string(),
                "client_secret".to_string(),
                "admin_users".to_string(),
                "max_reply_chars".to_string(),
            ]
            .into_iter()
            .map(|f| if f == "enabled" { format!("{f} = {}", q.is_some_and(|q| q.enabled)) } else { f })
            .collect()
        }
        _ => Vec::new(),
    }
}

/// 读取平台字段当前值。
pub fn platform_field_value(config: &AppConfig, index: usize, field: usize) -> String {
    let fields = platform_fields(config, index);
    let name = fields
        .get(field)
        .map(|f| f.split('=').next().unwrap_or(f).trim().to_string())
        .unwrap_or_default();
    match (index, name.as_str()) {
        (0, "reverse_ws_port") => config.platforms.qq.reverse_ws_port.to_string(),
        (0, "access_token") => config.platforms.qq.access_token.clone(),
        (0, "admin_users") => config
            .platforms
            .qq
            .admin_users
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join(","),
        (0, "max_reply_chars") => config.platforms.qq.max_reply_chars.to_string(),
        (1, "bot_token") => config
            .platforms
            .telegram
            .as_ref()
            .map(|t| t.bot_token.clone())
            .unwrap_or_default(),
        (1, "webhook_path") => config
            .platforms
            .telegram
            .as_ref()
            .map(|t| t.webhook_path.clone())
            .unwrap_or_default(),
        (1, "admin_users") => config
            .platforms
            .telegram
            .as_ref()
            .map(|t| {
                t.admin_users
                    .iter()
                    .map(i64::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .unwrap_or_default(),
        (1, "max_reply_chars") => config
            .platforms
            .telegram
            .as_ref()
            .map(|t| t.max_reply_chars.to_string())
            .unwrap_or_default(),
        (2, "app_id") => config
            .platforms
            .qq_official
            .as_ref()
            .map(|q| q.app_id.clone())
            .unwrap_or_default(),
        (2, "client_secret") => config
            .platforms
            .qq_official
            .as_ref()
            .map(|q| q.client_secret.clone())
            .unwrap_or_default(),
        (2, "admin_users") => config
            .platforms
            .qq_official
            .as_ref()
            .map(|q| {
                q.admin_users
                    .iter()
                    .map(i64::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .unwrap_or_default(),
        (2, "max_reply_chars") => config
            .platforms
            .qq_official
            .as_ref()
            .map(|q| q.max_reply_chars.to_string())
            .unwrap_or_default(),
        _ => String::new(),
    }
}

/// 写回平台字段；返回是否发生变更。
pub fn apply_platform_field(config: &mut AppConfig, index: usize, field: usize, value: &str) -> bool {
    let fields = platform_fields(config, index);
    let name = fields
        .get(field)
        .map(|f| f.split('=').next().unwrap_or(f).trim().to_string())
        .unwrap_or_default();
    let value = value.trim().to_string();
    match (index, name.as_str()) {
        (0, "reverse_ws_port") => {
            let Ok(v) = value.parse::<u16>() else {
                return false;
            };
            config.platforms.qq.reverse_ws_port = v
        }
        (0, "access_token") => config.platforms.qq.access_token = value,
        (0, "admin_users") => {
            config.platforms.qq.admin_users = parse_id_list(&value);
        }
        (0, "max_reply_chars") => {
            let Ok(v) = value.parse::<usize>() else {
                return false;
            };
            config.platforms.qq.max_reply_chars = v
        }
        (1, "bot_token") => {
            let t = config.platforms.telegram.get_or_insert_with(Default::default);
            t.bot_token = value;
        }
        (1, "webhook_path") => {
            let t = config.platforms.telegram.get_or_insert_with(Default::default);
            t.webhook_path = value;
        }
        (1, "admin_users") => {
            let t = config.platforms.telegram.get_or_insert_with(Default::default);
            t.admin_users = parse_id_list(&value);
        }
        (1, "max_reply_chars") => {
            let Ok(v) = value.parse::<usize>() else {
                return false;
            };
            let t = config.platforms.telegram.get_or_insert_with(Default::default);
            t.max_reply_chars = v;
        }
        (2, "app_id") => {
            let q = config.platforms.qq_official.get_or_insert_with(Default::default);
            q.app_id = value;
        }
        (2, "client_secret") => {
            let q = config.platforms.qq_official.get_or_insert_with(Default::default);
            q.client_secret = value;
        }
        (2, "admin_users") => {
            let q = config.platforms.qq_official.get_or_insert_with(Default::default);
            q.admin_users = parse_id_list(&value);
        }
        (2, "max_reply_chars") => {
            let Ok(v) = value.parse::<usize>() else {
                return false;
            };
            let q = config.platforms.qq_official.get_or_insert_with(Default::default);
            q.max_reply_chars = v;
        }
        _ => return false,
    }
    true
}

/// 解析逗号分隔的用户 id 列表。
fn parse_id_list(value: &str) -> Vec<i64> {
    value
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<i64>().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platforms_list_has_3_entries() {
        assert_eq!(PLATFORMS.len(), 3);
        assert_eq!(PLATFORMS[0].0, "qq");
        assert_eq!(PLATFORMS[2].0, "qq_official");
    }

    #[test]
    fn toggle_qq_flips_state() {
        let mut config = AppConfig::default();
        assert!(!platform_enabled(&config, 0));
        toggle_platform(&mut config, 0);
        assert!(platform_enabled(&config, 0));
    }

    #[test]
    fn toggle_telegram_creates_config() {
        let mut config = AppConfig::default();
        assert!(config.platforms.telegram.is_none());
        toggle_platform(&mut config, 1);
        assert!(platform_enabled(&config, 1));
        assert!(config.platforms.telegram.is_some());
    }

    #[test]
    fn rows_mark_enabled_platforms() {
        let mut config = AppConfig::default();
        toggle_platform(&mut config, 0);
        let rows = platform_rows(&config);
        assert!(rows[0].starts_with("[✓]"));
        assert!(rows[1].starts_with("[ ]"));
    }

    #[test]
    fn edit_qq_port_field() {
        let mut config = AppConfig::default();
        assert!(apply_platform_field(&mut config, 0, 1, "8400"));
        assert_eq!(config.platforms.qq.reverse_ws_port, 8400);
        assert!(!apply_platform_field(&mut config, 0, 1, "not-a-port"));
        assert_eq!(config.platforms.qq.reverse_ws_port, 8400);
    }

    #[test]
    fn edit_telegram_token_creates_config() {
        let mut config = AppConfig::default();
        assert!(apply_platform_field(&mut config, 1, 1, "123:abc"));
        assert_eq!(
            config.platforms.telegram.as_ref().map(|t| t.bot_token.as_str()),
            Some("123:abc")
        );
    }

    #[test]
    fn parse_id_list_filters_invalid() {
        let ids = parse_id_list("1, 2, x, 3");
        assert_eq!(ids, vec![1, 2, 3]);
    }
}

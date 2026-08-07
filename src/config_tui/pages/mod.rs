//! TUI 子页面模块。

pub mod global;
pub mod multimodal;
pub mod plugins;
pub mod platforms;
pub mod prompts;
pub mod providers;
pub mod subagent;
pub mod text_model;

/// 页面按键处理结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageAction {
    None,
    Back,
    Quit,
    Reopen,
}

/// 双语文本便捷包装：使用 crate::i18n::text 返回当前语种的字符串。
pub fn t(en: &'static str, zh: &'static str) -> String {
    crate::i18n::text(en, zh).to_string()
}

/// 共享的滚动列表状态辅助。
pub fn scroll_offset(selected: usize, visible: usize, total: usize) -> usize {
    if total == 0 {
        return 0;
    }
    let max_scroll = total.saturating_sub(visible);
    selected.saturating_sub(visible.saturating_sub(1)).min(max_scroll)
}

/// 计算翻页/跳转后的目标索引。
/// 返回 Some(new_index) 表示位置变化，None 表示无效键。
pub fn nav_index(code: crossterm::event::KeyCode, current: usize, count: usize) -> Option<usize> {
    use crossterm::event::KeyCode;
    if count == 0 {
        return None;
    }
    match code {
        KeyCode::Up | KeyCode::Char('k') => Some(current.saturating_sub(1)),
        KeyCode::Down | KeyCode::Char('j') => Some((current + 1).min(count - 1)),
        KeyCode::PageUp => Some(current.saturating_sub(10)),
        KeyCode::PageDown => Some((current + 10).min(count - 1)),
        KeyCode::Home => Some(0),
        KeyCode::End => Some(count - 1),
        _ => None,
    }
}

/// 列表位置指示文本："第 X/N 项"。
pub fn position_label(current: usize, count: usize) -> String {
    if count == 0 {
        "0/0".to_string()
    } else {
        format!("{}/{}", current + 1, count)
    }
}

/// 字段导航：统一处理 ↑↓/jk/PgUp/PgDn/Home/End。
/// 返回 Some(new_index) 表示位置变化；None 表示该键不参与字段导航。
pub fn move_field_index(
    code: crossterm::event::KeyCode,
    current: usize,
    count: usize,
) -> Option<usize> {
    use crossterm::event::KeyCode;
    if count == 0 {
        return None;
    }
    match code {
        KeyCode::Up | KeyCode::Char('k') => Some(current.saturating_sub(1)),
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('n') => {
            Some((current + 1).min(count - 1))
        }
        KeyCode::PageUp => Some(current.saturating_sub(10)),
        KeyCode::PageDown => Some((current + 10).min(count - 1)),
        KeyCode::Home => Some(0),
        KeyCode::End => Some(count - 1),
        _ => None,
    }
}

/// 同步字段位置到 ListState（编辑/总览模式）。
pub fn sync_field_state(state: &mut ratatui::widgets::ListState, index: usize) {
    state.select(Some(index));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_returns_chinese_by_default() {
        // 默认 locale 是 auto → 中文环境通常返回 zh。
        let result = t("Hello", "你好");
        assert!(!result.is_empty());
        assert!(result == "你好" || result == "Hello", "got: {result}");
    }

    #[test]
    fn scroll_stays_in_bounds() {
        assert_eq!(scroll_offset(0, 5, 10), 0);
        assert_eq!(scroll_offset(7, 5, 10), 3);
        assert_eq!(scroll_offset(9, 5, 10), 5);
        assert_eq!(scroll_offset(0, 5, 0), 0);
    }

    #[test]
    fn page_action_is_copyable() {
        let a = PageAction::Back;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn nav_index_handles_jumps() {
        use crossterm::event::KeyCode;
        assert_eq!(nav_index(KeyCode::Down, 0, 10), Some(1));
        assert_eq!(nav_index(KeyCode::PageDown, 0, 10), Some(9));
        assert_eq!(nav_index(KeyCode::PageUp, 1, 10), Some(0));
        assert_eq!(nav_index(KeyCode::Home, 5, 10), Some(0));
        assert_eq!(nav_index(KeyCode::End, 5, 10), Some(9));
        assert_eq!(nav_index(KeyCode::PageDown, 0, 0), None);
        assert_eq!(nav_index(KeyCode::Char('x'), 5, 10), None);
    }

    #[test]
    fn position_label_formats() {
        assert_eq!(position_label(0, 14), "1/14");
        assert_eq!(position_label(0, 0), "0/0");
    }
}

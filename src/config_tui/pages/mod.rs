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

/// 共享的滚动列表状态辅助。
pub fn scroll_offset(selected: usize, visible: usize, total: usize) -> usize {
    if total == 0 {
        return 0;
    }
    let max_scroll = total.saturating_sub(visible);
    selected.saturating_sub(visible.saturating_sub(1)).min(max_scroll)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

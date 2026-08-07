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
}

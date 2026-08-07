//! 子代理档位池页 — cheap/balanced/strong 三档模型池配置。

use crate::config::ModelTier;
use ratatui::widgets::ListState;

/// 页面状态。
#[derive(Default)]
pub struct SubagentPage {
    pub tab_index: usize,
    pub state: ListState,
}

impl SubagentPage {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            tab_index: 0,
            state,
        }
    }

    /// 当前激活档位；越界时钳制到 Strong。
    pub fn active_tier(&self) -> ModelTier {
        ModelTier::ALL[self.tab_index.clamp(0, ModelTier::ALL.len() - 1)]
    }
}

/// 档位切换标签（用于 Tabs 组件）。
pub fn tier_labels() -> [&'static str; 3] {
    ["cheap", "balanced", "strong"]
}

/// 为列表项添加选中标记。
pub fn choice_mark(selected: bool) -> &'static str {
    if selected { "✓" } else { "  " }
}

/// 档位的中文说明。
pub fn tier_hint(tier: ModelTier) -> &'static str {
    match tier {
        ModelTier::Cheap => "低成本任务",
        ModelTier::Balanced => "普通任务",
        ModelTier::Strong => "复杂任务",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_tier_cycles_through_all() {
        let mut page = SubagentPage::new();
        page.tab_index = 0;
        assert_eq!(page.active_tier(), ModelTier::Cheap);
        page.tab_index = 1;
        assert_eq!(page.active_tier(), ModelTier::Balanced);
        page.tab_index = 2;
        assert_eq!(page.active_tier(), ModelTier::Strong);
        page.tab_index = 99; // out of range clamps
        assert_eq!(page.active_tier(), ModelTier::Strong);
    }

    #[test]
    fn tier_labels_match_all_tiers() {
        assert_eq!(tier_labels(), ["cheap", "balanced", "strong"]);
    }

    #[test]
    fn choice_mark_marks_selection() {
        assert_eq!(choice_mark(true), "✓");
        assert_eq!(choice_mark(false), "  ");
    }
}

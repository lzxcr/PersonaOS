//! 插件配置页 — 插件列表 + 启用/禁用切换。

use crate::config::AppConfig;
use ratatui::widgets::ListState;

/// 页面状态。
#[derive(Default)]
pub struct PluginsPage {
    pub state: ListState,
}

impl PluginsPage {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self { state }
    }
}

/// 插件定义：id、显示名、描述。
pub const PLUGINS: [(&str, &str, &str); 14] = [
    ("web", "网络搜索", "搜索 API 与脚本 fallback"),
    ("deep_research", "深度研究", "长任务研究并输出 Markdown"),
    ("vision", "识图", "图片理解和终端预览"),
    ("image_generation", "生图", "文本生成图片"),
    ("web_images", "搜图", "网络图片搜索、下载与审核"),
    ("print_image", "打印图片", "终端图片打印尺寸"),
    ("memes", "表情包", "人格表情库与发送尺寸"),
    ("knowledge_base", "知识库", "本地文件检索与语义索引"),
    ("archlinux", "Arch Linux", "AUR 状态与 ArchWiki 查询"),
    ("man", "在线手册", "在线 man 手册搜索与读取"),
    ("memory", "记忆", "长期记忆与联想"),
    ("package_advisor", "AUR 审查", "PKGBUILD/AUR 安全审查"),
    ("deep_research_linux_game_compatibility", "Linux 游戏兼容", "Proton/反作弊/兼容性查询"),
    ("api_quota", "大模型额度查询", "查询 DeepSeek 与 OpenRouter API 额度"),
];

/// 插件是否启用。
pub fn plugin_enabled(config: &AppConfig, index: usize) -> bool {
    match index {
        0 => config.plugins.web.enabled,
        1 => config.plugins.deep_research.enabled,
        2 => config.plugins.vision.enabled,
        3 => config.plugins.image_generation.enabled,
        4 => config.plugins.web_images.enabled,
        5 => config.plugins.print_image.enabled,
        6 => config.plugins.memes.enabled,
        7 => config.plugins.knowledge_base.enabled,
        8 => config.plugins.archlinux.enabled,
        9 => config.plugins.man.enabled,
        10 => config.plugins.memory.enabled,
        11 => config.plugins.package_advisor.enabled,
        12 => config.plugins.deep_research_linux_game_compatibility.enabled,
        13 => config.plugins.api_quota.enabled,
        _ => false,
    }
}

/// 切换插件启用状态；返回新状态。
pub fn toggle_plugin(config: &mut AppConfig, index: usize) -> bool {
    let value = !plugin_enabled(config, index);
    match index {
        0 => config.plugins.web.enabled = value,
        1 => config.plugins.deep_research.enabled = value,
        2 => config.plugins.vision.enabled = value,
        3 => config.plugins.image_generation.enabled = value,
        4 => config.plugins.web_images.enabled = value,
        5 => config.plugins.print_image.enabled = value,
        6 => config.plugins.memes.enabled = value,
        7 => config.plugins.knowledge_base.enabled = value,
        8 => config.plugins.archlinux.enabled = value,
        9 => config.plugins.man.enabled = value,
        10 => config.plugins.memory.enabled = value,
        11 => config.plugins.package_advisor.enabled = value,
        12 => config.plugins.deep_research_linux_game_compatibility.enabled = value,
        13 => config.plugins.api_quota.enabled = value,
        _ => {}
    }
    value
}

/// 插件列表行：`[✓/ ] 显示名 — 描述 (id)`。
pub fn plugin_rows(config: &AppConfig) -> Vec<String> {
    PLUGINS
        .iter()
        .enumerate()
        .map(|(i, (id, name, desc))| {
            let mark = if plugin_enabled(config, i) { "✓" } else { " " };
            format!("[{mark}] {name:<14} — {desc} ({id})")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugins_list_has_14_entries() {
        assert_eq!(PLUGINS.len(), 14);
        assert_eq!(PLUGINS[0].0, "web");
        assert_eq!(PLUGINS[13].0, "api_quota");
    }

    #[test]
    fn toggle_plugin_flips_state() {
        let mut config = AppConfig::default();
        let before = plugin_enabled(&config, 0);
        let after = toggle_plugin(&mut config, 0);
        assert_ne!(before, after);
        assert_eq!(after, plugin_enabled(&config, 0));
    }

    #[test]
    fn rows_mark_enabled_plugins() {
        let mut config = AppConfig::default();
        // 找一个默认禁用的插件做翻转测试。
        let mut disabled_index = None;
        for i in 0..PLUGINS.len() {
            if !plugin_enabled(&config, i) {
                disabled_index = Some(i);
                break;
            }
        }
        let Some(index) = disabled_index else {
            // 全部默认启用时，翻转第一个即可验证。
            let rows = plugin_rows(&config);
            assert!(rows[0].starts_with("[✓]"));
            toggle_plugin(&mut config, 0);
            let rows = plugin_rows(&config);
            assert!(rows[0].starts_with("[ ]"));
            return;
        };
        let rows = plugin_rows(&config);
        assert!(rows[index].starts_with("[ ]"), "应默认禁用: {}", rows[index]);
        toggle_plugin(&mut config, index);
        let rows = plugin_rows(&config);
        assert!(rows[index].starts_with("[✓]"));
    }
}

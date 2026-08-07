//! 插件配置页 — 插件列表 + 启用/禁用切换 + 字段详情编辑。

use super::t;
use crate::config::AppConfig;
use anyhow::{bail, Result};
use ratatui::widgets::ListState;

/// 页面状态。
pub struct PluginsPage {
    pub state: ListState,
    /// 字段总览/编辑位置（持久 ListState）。
    pub field_state: ListState,
    /// 字段总览模式：true 时显示选中插件的字段值列表。
    pub viewing: bool,
    pub editing: bool,
    pub edit_field: usize,
    pub edit_buffer: String,
    pub error_msg: Option<String>,
    /// API quota 子页
    pub quota_active: bool,
    pub quota_provider: usize, // 0=deepseek, 1=openrouter
    pub quota_field_idx: usize, // 0=name, 1=api_key
}

impl PluginsPage {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            state,
            field_state: {
                let mut fs = ListState::default();
                fs.select(Some(0));
                fs
            },
            viewing: false,
            editing: false,
            edit_field: 0,
            edit_buffer: String::new(),
            error_msg: None,
            quota_active: false,
            quota_provider: 0,
            quota_field_idx: 0,
        }
    }

    /// 当前选中插件的字段总览行：`标签 = 显示值`。
    pub fn field_rows(&self, config: &AppConfig) -> Vec<String> {
        self.current_fields(config)
            .iter()
            .map(|f| format!("{} = {}", f.label, f.display_value()))
            .collect()
    }

    /// 当前选中插件的字段列表。
    pub fn current_fields(&self, config: &AppConfig) -> Vec<Field> {
        let i = self.state.selected().unwrap_or(0);
        plugin_fields(config, i)
    }
}

// ── Plugin list / toggle ──────────────────────────────────────────────

/// 插件定义：id、显示名、描述。
pub fn plugins() -> [(&'static str, String, String); 14] {
    [
        ("web", t("Web search", "网络搜索"), t("Search APIs with script fallback", "搜索 API 与脚本 fallback")),
        ("deep_research", t("Deep research", "深度研究"), t("Run long research tasks and output Markdown", "长任务研究并输出 Markdown")),
        ("vision", t("Vision", "识图"), t("Image understanding and terminal preview", "图片理解和终端预览")),
        ("image_generation", t("Image generation", "生图"), t("Generate images from text", "文本生成图片")),
        ("web_images", t("Image search", "搜图"), t("Search, download, and review web images", "网络图片搜索、下载与审核")),
        ("print_image", t("Print image", "打印图片"), t("Terminal image print size", "终端图片打印尺寸")),
        ("memes", t("Memes", "表情包"), t("Persona meme library and send size", "人格表情库与发送尺寸")),
        ("knowledge_base", t("Knowledge base", "知识库"), t("Local file search and semantic index", "本地文件检索与语义索引")),
        ("archlinux", "Arch Linux".to_string(), t("AUR status and ArchWiki lookup", "AUR 状态与 ArchWiki 查询")),
        ("man", t("Online manuals", "在线手册"), t("Search and read online man pages", "在线 man 手册搜索与读取")),
        ("memory", t("Memory", "记忆"), t("Long-term memory and association", "长期记忆与联想")),
        ("package_advisor", t("AUR review", "AUR 审查"), t("PKGBUILD/AUR security review", "PKGBUILD/AUR 安全审查")),
        ("deep_research_linux_game_compatibility", t("Linux game compatibility", "Linux 游戏兼容"), t("Proton/anti-cheat/compatibility lookup", "Proton/反作弊/兼容性查询")),
        ("api_quota", t("LLM API quota", "大模型额度查询"), t("Query DeepSeek and OpenRouter API quota", "查询 DeepSeek 与 OpenRouter API 额度")),
    ]
}

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
    plugins()
        .into_iter()
        .enumerate()
        .map(|(i, (id, name, desc))| {
            let mark = if plugin_enabled(config, i) { "✓" } else { " " };
            format!("[{mark}] {name:<14} — {desc} ({id})")
        })
        .collect()
}

// ── API Quota management ──────────────────────────────────────────────

/// Quota 供应商标签。
pub const QUOTA_PROVIDERS: [&str; 2] = ["DeepSeek", "OpenRouter"];

/// 获取指定供应商的账号列表。
pub fn quota_accounts<'a>(config: &'a AppConfig, provider: usize) -> &'a [crate::config::ApiQuotaAccountConfig] {
    match provider {
        0 => &config.plugins.api_quota.deepseek.accounts,
        1 => &config.plugins.api_quota.openrouter.accounts,
        _ => &[],
    }
}

/// 删除指定账号；返回删除的账号名。
pub fn quota_delete_account(config: &mut AppConfig, provider: usize, index: usize) -> Option<String> {
    let accounts = match provider {
        0 => &mut config.plugins.api_quota.deepseek.accounts,
        1 => &mut config.plugins.api_quota.openrouter.accounts,
        _ => return None,
    };
    if index < accounts.len() {
        let name = accounts[index].name.clone();
        accounts.remove(index);
        Some(name)
    } else {
        None
    }
}

/// 添加新账号（自动生成 id 和默认名）。
pub fn quota_add_account(config: &mut AppConfig, provider: usize) {
    let accounts = match provider {
        0 => &mut config.plugins.api_quota.deepseek.accounts,
        1 => &mut config.plugins.api_quota.openrouter.accounts,
        _ => return,
    };
    let next_id = (accounts.len() + 1).to_string();
    let next_name = format!("账号 {next_id}");
    accounts.push(crate::config::ApiQuotaAccountConfig {
        id: next_id,
        name: next_name,
        api_key: String::new(),
    });
}

/// 获取指定账号的字段值（0=name, 1=api_key）。
pub fn quota_account_field(
    config: &AppConfig,
    provider: usize,
    index: usize,
    field: usize,
) -> String {
    let accounts = match provider {
        0 => &config.plugins.api_quota.deepseek.accounts,
        1 => &config.plugins.api_quota.openrouter.accounts,
        _ => return String::new(),
    };
    match (accounts.get(index), field) {
        (Some(account), 0) => account.name.clone(),
        (Some(account), 1) => account.api_key.clone(),
        _ => String::new(),
    }
}

/// 设置指定账号的字段值。
pub fn quota_set_account_field(
    config: &mut AppConfig,
    provider: usize,
    index: usize,
    field: usize,
    value: &str,
) {
    let accounts = match provider {
        0 => &mut config.plugins.api_quota.deepseek.accounts,
        1 => &mut config.plugins.api_quota.openrouter.accounts,
        _ => return,
    };
    if let Some(account) = accounts.get_mut(index) {
        match field {
            0 => account.name = value.to_string(),
            1 => account.api_key = value.to_string(),
            _ => {}
        }
    }
}

// ── Field abstraction ──────────────────────────────────────────────────

/// 可编辑字段的声明式描述。
#[derive(Debug, Clone)]
pub struct Field {
    pub label: String,
    pub value: String,
    pub is_bool: bool,
    pub is_sensitive: bool,
    pub choices: Vec<String>,
}

impl Field {
    pub fn new(label: String, value: String) -> Self {
        Self { label, value, is_bool: false, is_sensitive: false, choices: Vec::new() }
    }

    pub fn boolean(label: String, value: bool) -> Self {
        Self { label, value: value.to_string(), is_bool: true, is_sensitive: false, choices: Vec::new() }
    }

    pub fn sensitive(mut self) -> Self {
        self.is_sensitive = true;
        self
    }

    pub fn choices(mut self, choices: &[&str]) -> Self {
        self.choices = choices.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 显示值（脱敏处理）。
    pub fn display_value(&self) -> String {
        if self.is_sensitive && !self.value.is_empty() {
            if self.value.starts_with("sk-") {
                format!("{}…", &self.value[..6.min(self.value.len())])
            } else if self.value.starts_with("$env:") {
                self.value.clone()
            } else {
                "***".to_string()
            }
        } else {
            self.value.clone()
        }
    }
}

pub fn parse_bool_field(value: &str) -> Result<bool> {
    match value.trim().to_lowercase().as_str() {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        other => bail!("invalid boolean: {other}"),
    }
}

// ── Plugin field definitions ───────────────────────────────────────────

pub fn plugin_fields(config: &AppConfig, index: usize) -> Vec<Field> {
    match index {
        0 => vec![
            Field::boolean(t("Enabled", "启用"), config.plugins.web.enabled),
            Field::new(t("Results per request", "每次返回数量"), config.plugins.web.max_results.to_string()),
            Field::new("Tavily API Keys".to_string(), config.plugins.web.tavily_api_keys.join("\n")).sensitive(),
            Field::new("Firecrawl API Keys".to_string(), config.plugins.web.firecrawl_api_keys.join("\n")).sensitive(),
            Field::new("AnySearch API Keys".to_string(), config.plugins.web.anysearch_api_keys.join("\n")).sensitive(),
            Field::new("SearXNG URL".to_string(), config.plugins.web.searxng_base_url.clone()),
        ],
        1 => vec![
            Field::boolean(t("Enabled", "启用"), config.plugins.deep_research.enabled),
            Field::new(t("Output directory", "输出目录"), config.plugins.deep_research.output_dir.clone()),
            Field::new(t("Thinking depth", "思考深度"), config.plugins.deep_research.thinking_depth.clone())
                .choices(&["minimal", "low", "medium", "high", "xhigh"]),
            Field::new(t("Maximum review revisions", "最大审视修正次数"), config.plugins.deep_research.max_review_revisions.to_string()),
            Field::new(t("Tool steps per round", "每轮工具步数"), config.plugins.deep_research.max_tool_steps_per_round.to_string()),
            Field::new(t("Final answer character limit", "最终字数上限"), config.plugins.deep_research.max_final_answer_chars.to_string()),
            Field::new(t("Tool timeout (seconds)", "工具超时秒数"), config.plugins.deep_research.tool_call_timeout_seconds.to_string()),
            Field::boolean(t("Show progress", "显示过程进度"), config.plugins.deep_research.show_progress),
        ],
        2 => vec![
            Field::boolean(t("Enabled", "启用"), config.plugins.vision.enabled),
            Field::boolean(t("Prefer current model for images", "优先使用当前模型识图"), config.plugins.vision.prefer_current_multimodal_model),
            Field::new(t("Vision provider", "识图供应商"), config.plugins.vision.vision_provider_id.clone()),
            Field::new(t("Vision model", "识图模型"), config.plugins.vision.vision_model.clone()),
            Field::new(t("Response header timeout (s)", "响应头超时秒"), config.plugins.vision.response_header_timeout_seconds.to_string()),
            Field::new(t("Stream idle timeout (s)", "流空闲超时秒"), config.plugins.vision.stream_idle_timeout_seconds.to_string()),
            Field::new(t("Image timeout (s)", "图片超时秒"), config.plugins.vision.image_timeout_seconds.to_string()),
            Field::boolean(t("Preview with chafa", "终端图片预览"), config.plugins.vision.preview_with_chafa),
        ],
        3 => vec![
            Field::boolean(t("Enabled", "启用"), config.plugins.image_generation.enabled),
            Field::new(t("Provider type", "供应商类型"), config.plugins.image_generation.provider_type.clone()),
            Field::new(t("Base URL", "基础 URL"), config.plugins.image_generation.base_url.clone()),
            Field::new(t("API Keys", "API 密钥"), config.plugins.image_generation.api_keys.join("\n")).sensitive(),
            Field::new(t("Model", "模型"), config.plugins.image_generation.model.clone()),
            Field::new(t("Default aspect ratio", "默认宽高比"), config.plugins.image_generation.default_aspect_ratio.clone()),
            Field::new(t("Default resolution", "默认分辨率"), config.plugins.image_generation.default_resolution.clone()),
            Field::new(t("Output dir", "输出目录"), config.plugins.image_generation.output_dir.clone()),
            Field::boolean(t("Auto print", "自动打印"), config.plugins.image_generation.auto_print),
            Field::new(t("Timeout (s)", "超时秒"), config.plugins.image_generation.timeout_seconds.to_string()),
        ],
        4 => vec![
            Field::boolean(t("Enabled", "启用"), config.plugins.web_images.enabled),
            Field::new(t("Source mode", "来源模式"), config.plugins.web_images.source_mode.clone()),
            Field::new(t("Max results", "最大结果数"), config.plugins.web_images.max_results.to_string()),
            Field::new(t("Max download (MB)", "最大下载(MB)"), config.plugins.web_images.max_download_mb.to_string()),
            Field::boolean(t("Safe search", "安全搜索"), config.plugins.web_images.safe_search),
            Field::boolean(t("Vision screening", "画面审核"), config.plugins.web_images.vision_screening_enabled),
            Field::boolean(t("Auto preview", "自动预览"), config.plugins.web_images.auto_preview),
            Field::new(t("Preview count", "预览数量"), config.plugins.web_images.preview_count.to_string()),
            Field::new(t("Timeout (s)", "超时秒"), config.plugins.web_images.timeout_seconds.to_string()),
        ],
        5 => vec![
            Field::boolean(t("Enabled", "启用"), config.plugins.print_image.enabled),
            Field::new(t("Width (%)", "宽度(%)"), config.plugins.print_image.width_percent.to_string()),
            Field::new(t("Height (%)", "高度(%)"), config.plugins.print_image.height_percent.to_string()),
        ],
        6 => vec![
            Field::boolean(t("Enabled", "启用"), config.plugins.memes.enabled),
            Field::new(t("Search max results", "搜索最大数"), config.plugins.memes.search_max_results.to_string()),
            Field::new(t("Width (%)", "宽度(%)"), config.plugins.memes.width_percent.to_string()),
            Field::new(t("Height (%)", "高度(%)"), config.plugins.memes.height_percent.to_string()),
            Field::new(t("Max image (MB)", "图片上限(MB)"), config.plugins.memes.max_image_mb.to_string()),
            Field::boolean(t("Allow GIF", "允许动图"), config.plugins.memes.allow_gif_animation),
            Field::boolean(t("Auto send", "自动发送"), config.plugins.memes.auto_send_enabled),
            Field::new(t("Auto send probability", "自动发送概率"), config.plugins.memes.auto_send_probability.to_string()),
        ],
        7 => vec![
            Field::boolean(t("Enabled", "启用"), config.plugins.knowledge_base.enabled),
            Field::new(t("Data dir", "数据目录"), config.plugins.knowledge_base.data_dir.clone()),
            Field::new(t("Max search results", "最大搜索结果"), config.plugins.knowledge_base.max_search_results.to_string()),
            Field::new(t("Snippet context chars", "片段上下文字符"), config.plugins.knowledge_base.snippet_context_chars.to_string()),
            Field::new(t("Max read lines", "最大读取行数"), config.plugins.knowledge_base.max_read_lines.to_string()),
            Field::new(t("Max file size (KB)", "最大文件(KB)"), config.plugins.knowledge_base.max_file_size_kb.to_string()),
            Field::boolean(t("Upload tool", "上传工具"), config.plugins.knowledge_base.upload_tool_enabled),
            Field::boolean(t("Embedding", "向量嵌入"), config.plugins.knowledge_base.embedding_enabled),
        ],
        8 => vec![
            Field::boolean(t("Enabled", "启用"), config.plugins.archlinux.enabled),
        ],
        9 => vec![
            Field::boolean(t("Enabled", "启用"), config.plugins.man.enabled),
        ],
        10 => vec![
            Field::boolean(t("Enabled", "启用"), config.plugins.memory.enabled),
        ],
        11 => vec![
            Field::boolean(t("Enabled", "启用"), config.plugins.package_advisor.enabled),
        ],
        12 => vec![
            Field::boolean(t("Enabled", "启用"), config.plugins.deep_research_linux_game_compatibility.enabled),
        ],
        13 => vec![
            Field::boolean(t("Enabled", "启用"), config.plugins.api_quota.enabled),
        ],
        _ => Vec::new(),
    }
}

// ── Apply plugin fields ────────────────────────────────────────────────

pub fn apply_plugin_fields(config: &mut AppConfig, index: usize, fields: &[Field]) -> Result<()> {
    match index {
        0 => {
            config.plugins.web.enabled = parse_bool_field(&fields[0].value)?;
            config.plugins.web.max_results = fields[1].value.trim().parse::<usize>()?.clamp(1, 10);
            config.plugins.web.tavily_api_keys = fields[2].value.lines().map(String::from).filter(|s| !s.trim().is_empty()).collect();
            config.plugins.web.firecrawl_api_keys = fields[3].value.lines().map(String::from).filter(|s| !s.trim().is_empty()).collect();
            config.plugins.web.anysearch_api_keys = fields[4].value.lines().map(String::from).filter(|s| !s.trim().is_empty()).collect();
            config.plugins.web.searxng_base_url = fields[5].value.trim().to_string();
        }
        1 => {
            config.plugins.deep_research.enabled = parse_bool_field(&fields[0].value)?;
            config.plugins.deep_research.output_dir = fields[1].value.trim().to_string();
            config.plugins.deep_research.thinking_depth = fields[2].value.trim().to_string();
            config.plugins.deep_research.max_review_revisions = fields[3].value.trim().parse::<usize>()?;
            config.plugins.deep_research.max_tool_steps_per_round = fields[4].value.trim().parse::<usize>()?;
            config.plugins.deep_research.max_final_answer_chars = fields[5].value.trim().parse::<usize>()?;
            config.plugins.deep_research.tool_call_timeout_seconds = fields[6].value.trim().parse::<u64>()?;
            config.plugins.deep_research.show_progress = parse_bool_field(&fields[7].value)?;
        }
        2 => {
            config.plugins.vision.enabled = parse_bool_field(&fields[0].value)?;
            config.plugins.vision.prefer_current_multimodal_model = parse_bool_field(&fields[1].value)?;
            config.plugins.vision.vision_provider_id = fields[2].value.trim().to_string();
            config.plugins.vision.vision_model = fields[3].value.trim().to_string();
            config.plugins.vision.response_header_timeout_seconds = fields[4].value.trim().parse::<u64>()?;
            config.plugins.vision.stream_idle_timeout_seconds = fields[5].value.trim().parse::<u64>()?;
            config.plugins.vision.image_timeout_seconds = fields[6].value.trim().parse::<u64>()?;
            config.plugins.vision.preview_with_chafa = parse_bool_field(&fields[7].value)?;
        }
        3 => {
            config.plugins.image_generation.enabled = parse_bool_field(&fields[0].value)?;
            config.plugins.image_generation.provider_type = fields[1].value.trim().to_string();
            config.plugins.image_generation.base_url = fields[2].value.trim().to_string();
            config.plugins.image_generation.api_keys = fields[3].value.lines().map(String::from).filter(|s| !s.trim().is_empty()).collect();
            config.plugins.image_generation.model = fields[4].value.trim().to_string();
            config.plugins.image_generation.default_aspect_ratio = fields[5].value.trim().to_string();
            config.plugins.image_generation.default_resolution = fields[6].value.trim().to_string();
            config.plugins.image_generation.output_dir = fields[7].value.trim().to_string();
            config.plugins.image_generation.auto_print = parse_bool_field(&fields[8].value)?;
            config.plugins.image_generation.timeout_seconds = fields[9].value.trim().parse::<u64>()?;
        }
        4 => {
            config.plugins.web_images.enabled = parse_bool_field(&fields[0].value)?;
            config.plugins.web_images.source_mode = fields[1].value.trim().to_string();
            config.plugins.web_images.max_results = fields[2].value.trim().parse::<usize>()?;
            config.plugins.web_images.max_download_mb = fields[3].value.trim().parse::<f64>()?;
            config.plugins.web_images.safe_search = parse_bool_field(&fields[4].value)?;
            config.plugins.web_images.vision_screening_enabled = parse_bool_field(&fields[5].value)?;
            config.plugins.web_images.auto_preview = parse_bool_field(&fields[6].value)?;
            config.plugins.web_images.preview_count = fields[7].value.trim().parse::<usize>()?;
            config.plugins.web_images.timeout_seconds = fields[8].value.trim().parse::<u64>()?;
        }
        5 => {
            config.plugins.print_image.enabled = parse_bool_field(&fields[0].value)?;
            config.plugins.print_image.width_percent = fields[1].value.trim().parse::<u8>()?;
            config.plugins.print_image.height_percent = fields[2].value.trim().parse::<u8>()?;
        }
        6 => {
            config.plugins.memes.enabled = parse_bool_field(&fields[0].value)?;
            config.plugins.memes.search_max_results = fields[1].value.trim().parse::<usize>()?;
            config.plugins.memes.width_percent = fields[2].value.trim().parse::<u8>()?;
            config.plugins.memes.height_percent = fields[3].value.trim().parse::<u8>()?;
            config.plugins.memes.max_image_mb = fields[4].value.trim().parse::<u64>()?;
            config.plugins.memes.allow_gif_animation = parse_bool_field(&fields[5].value)?;
            config.plugins.memes.auto_send_enabled = parse_bool_field(&fields[6].value)?;
            config.plugins.memes.auto_send_probability = fields[7].value.trim().parse::<f32>()?;
        }
        7 => {
            config.plugins.knowledge_base.enabled = parse_bool_field(&fields[0].value)?;
            config.plugins.knowledge_base.data_dir = fields[1].value.trim().to_string();
            config.plugins.knowledge_base.max_search_results = fields[2].value.trim().parse::<usize>()?;
            config.plugins.knowledge_base.snippet_context_chars = fields[3].value.trim().parse::<usize>()?;
            config.plugins.knowledge_base.max_read_lines = fields[4].value.trim().parse::<usize>()?;
            config.plugins.knowledge_base.max_file_size_kb = fields[5].value.trim().parse::<usize>()?;
            config.plugins.knowledge_base.upload_tool_enabled = parse_bool_field(&fields[6].value)?;
            config.plugins.knowledge_base.embedding_enabled = parse_bool_field(&fields[7].value)?;
        }
        8 => {
            config.plugins.archlinux.enabled = parse_bool_field(&fields[0].value)?;
        }
        9 => {
            config.plugins.man.enabled = parse_bool_field(&fields[0].value)?;
        }
        10 => {
            config.plugins.memory.enabled = parse_bool_field(&fields[0].value)?;
        }
        11 => {
            config.plugins.package_advisor.enabled = parse_bool_field(&fields[0].value)?;
        }
        12 => {
            config.plugins.deep_research_linux_game_compatibility.enabled = parse_bool_field(&fields[0].value)?;
        }
        13 => {
            config.plugins.api_quota.enabled = parse_bool_field(&fields[0].value)?;
        }
        _ => {}
    }
    Ok(())
}
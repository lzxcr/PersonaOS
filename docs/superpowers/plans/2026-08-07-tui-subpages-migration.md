# TUI 子页面迁移（ratatui）实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: 使用 superpowers:executing-plans 逐任务执行。步骤用 `- [ ]` 跟踪。

**Goal:** 将旧 crossterm 实现的 8 个 TUI 配置子页面迁移到 ratatui，恢复并现代化 `pos config` 的完整功能。

**Architecture:** 每个页面独立实现为 ratatui widget 组合（Table/List/Paragraph/Tabs），页面状态保存在 `App` 结构中，通过 `Screen` 枚举路由。配置逻辑（字段定义、验证、默认值）复用旧 `config_tui_old.rs` 中的纯函数。

**Tech Stack:** ratatui 0.29 + crossterm 0.28 + 现有 `AppConfig` 类型

**背景**：主菜单已迁移（`src/config_tui/mod.rs`，360 行），子页面当前是占位符。旧实现 `src/config_tui_old.rs`（8403 行）保留为逻辑参考，**不编译**。

---

## 文件结构

```
src/config_tui/
├── mod.rs          # 已有：App 结构、Screen 枚举、主菜单、事件循环
├── theme.rs        # 已有：MD3 主题
├── pages/
│   ├── mod.rs      # 页面模块聚合
│   ├── providers.rs    # 供应商与模型页（CRUD + 选择）
│   ├── text_model.rs   # 文本模型选择页
│   ├── multimodal.rs   # 多模态模型选择页
│   ├── subagent.rs     # 子代理档位池页
│   ├── plugins.rs      # 插件配置页
│   ├── prompts.rs      # 自定义提示词/人格页
│   ├── platforms.rs    # IM 平台页（QQ 配置）
│   └── global.rs       # 全局参数设置页
```

每个页面：`pub struct XxxPage` + `pub fn draw(&mut self, frame, area, app, theme)` + `pub fn handle_key(&mut self, code, app) -> PageAction`。

`PageAction` 枚举（`pages/mod.rs`）：
```rust
pub enum PageAction {
    None,
    Back,            // 返回主菜单
    Quit,            // 退出配置
    Reopen,          // 重新打开页面（状态重置后）
}
```

---

### Task 1: 页面框架

**Files:**
- Create: `src/config_tui/pages/mod.rs`

- [ ] **Step 1: 创建 `pages/mod.rs` 定义共享类型**

```rust
//! TUI 子页面模块。

pub mod global;
pub mod multimodal;
pub mod plugins;
pub mod platforms;
pub mod prompts;
pub mod providers;
pub mod subagent;
pub mod text_model;

use crate::config_tui::theme::Theme;

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
```

- [ ] **Step 2: 运行测试验证失败**

Run: `cargo test config_tui::pages::tests`
Expected: FAIL（模块不存在）

- [ ] **Step 3: 实现模块（见 Step 1 代码）+ 各页面 stub**

每个页面文件先建最小 stub（`pub struct XxxPage { pub selected: usize }` + `Default`），保证编译通过。

- [ ] **Step 4: 运行测试验证通过**

Run: `cargo test config_tui::pages::tests`
Expected: PASS（2 个测试）

- [ ] **Step 5: 提交**

```bash
git add src/config_tui/
git commit -m "feat(tui): 页面框架与滚动辅助"
```

---

### Task 2: 文本模型选择页

**Files:**
- Create: `src/config_tui/pages/text_model.rs`
- Modify: `src/config_tui/mod.rs`（路由 + 状态）

- [ ] **Step 1: 写失败测试（纯逻辑：从 choices 计算显示列表）**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_choices_render_with_indexes() {
        let labels = vec!["openai/gpt-4o-mini".to_string(), "deepseek/v3".to_string()];
        let rows = model_rows(&labels);
        assert_eq!(rows.len(), 2);
        assert!(rows[0].contains("1"));
        assert!(rows[1].contains("2"));
    }
}
```

- [ ] **Step 2: 运行验证失败** → 实现 → 验证通过

页面行为：
- 列出 `config.active_provider_model_choices()` 的所有选择
- ↑↓ 导航，Enter 选中（`config.active_provider_models = Some(vec![...])`）
- 按 `n` 新建模型条目（内联文本输入）
- Esc 返回主菜单

- [ ] **Step 3: 接入 `mod.rs`**：`Screen::TextModel` 持有 `pages::text_model::TextModelPage`，draw/handle_key 分发。

- [ ] **Step 4: 提交**

---

### Task 3: 多模态模型选择页

**Files:**
- Create: `src/config_tui/pages/multimodal.rs`

同 Task 2 模式，使用 `active_multimodal_provider_model_choices()`。

---

### Task 4: 子代理档位池页

**Files:**
- Create: `src/config_tui/pages/subagent.rs`

Tabs 切换 cheap/balanced/strong 三档，每档列出模型选择（同 Task 2 交互）。

---

### Task 5: 供应商与模型页（核心）

**Files:**
- Create: `src/config_tui/pages/providers.rs`

行为（复用旧实现 `config_tui_old.rs` 的纯逻辑函数）：
- Table 列出 `config.providers`：ID、显示名、Base URL、模型数
- 操作：添加（模板列表）、编辑（内联字段）、删除（确认）、设为 active
- 字段编辑：`ProviderConfig` 的 id/display_name/base_url/api_key(default_model/models)
- API 配额管理入口（`edit_api_quota` 逻辑）

验证逻辑复用：`config.provider(None)`、`config.save()` 的既有校验。

---

### Task 6: 全局参数设置页

**Files:**
- Create: `src/config_tui/pages/global.rs`

字段（对照旧实现）：
- language、active_persona（字符串）、always_allow_tools（bool）
- theme、matugen_scheme
- 保存按钮（写盘）

---

### Task 7: 插件配置页

**Files:**
- Create: `src/config_tui/pages/plugins.rs`

- List 列出 14 个插件（`plugin_names()` 复用），空格/Enter 切换 enabled
- 选中插件进入详情：字段编辑（`plugin_fields()` + `apply_plugin_fields()` 复用）
- API 配额子页

---

### Task 8: IM 平台页

**Files:**
- Create: `src/config_tui/pages/platforms.rs`

- Tabs：QQ / Telegram / QQ Official
- QQ 子页：enabled、reverse_ws_port、access_token、admin_users、限流、模型路由、real_context 等（量大，分批）
- 平台命令前缀/权限

---

### Task 9: 自定义提示词页

**Files:**
- Create: `src/config_tui/pages/prompts.rs`

- 人格管理：内置/自定义列表 + 激活/重命名/删除/新建
- 身份管理
- 提示词文件编辑器

---

### Task 10: 收尾

- [ ] 删除 `src/config_tui_old.rs`
- [ ] `cargo build` 零警告
- [ ] `cargo test` 回归全绿
- [ ] 手动验证 `pos config` 各页面

---

## 执行顺序（按依赖与价值）

1. Task 1（框架）→ 2. Task 2/3/4（模型选择，小页面建立模式）→ 3. Task 5（供应商，核心）→ 4. Task 6/7（全局/插件）→ 5. Task 8/9（平台/提示词，大页面）→ 6. Task 10（收尾）

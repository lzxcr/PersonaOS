# TUI 功能补齐：6 项缺失功能恢复

> **For agentic workers:** 使用 superpowers:executing-plans 逐任务执行。

**Goal:** 将 ratatui TUI 从"基础配置面板"恢复到"全量配置编辑器"，补齐 i18n、插件详情、API quota、QQ 深度配置、人格 CRUD、全局缺失字段。

**Architecture:** 从旧 crossterm TUI（git `HEAD~3:src/config_tui_old.rs`）提取纯逻辑函数到页面模块，ratatui 绘制层调用这些函数。模式：`pages/X.rs` 放纯逻辑 + 返回结构化数据，`mod.rs` 放渲染 + 事件处理。

**Tech Stack:** ratatui 0.29, crossterm 0.28, 现有 i18n 模块 (`src/i18n.rs`)

---

## 文件结构

```
src/config_tui/
├── mod.rs              # 路由/渲染/事件（现有，+新增页面分支）
├── theme.rs            # 不变
└── pages/
    ├── mod.rs           # PageAction 枚举（现有）
    ├── text_model.rs    # 现有
    ├── multimodal.rs    # 现有
    ├── subagent.rs      # 现有
    ├── providers.rs     # 现有 + API quota 子页
    ├── plugins.rs       # 现有 + 插件字段编辑
    ├── platforms.rs     # 现有 + QQ 深度配置子页
    ├── prompts.rs       # 现有 + 人格 CRUD
    └── global.rs        # 现有 + 3 个缺失字段
```

---

### Task 1: i18n 双语恢复（核心基础）

**Files:**
- Modify: `src/config_tui/mod.rs`（全部字符串）
- Modify: `src/config_tui/pages/*.rs`（全部字符串）

**Approach:** 在 `pages/mod.rs` 添加 `t(en, zh) -> &'static str` 包装。所有硬编码中文字符串替换为 `t("en", "zh")` 调用。

- [ ] **Step 1: 添加 i18n 包装函数**
  在 `pages/mod.rs` 新增：
  ```rust
  use crate::i18n::text;
  /// 双语文本便捷包装
  pub fn t(en: &'static str, zh: &'static str) -> String {
      text(en, zh)
  }
  ```

- [ ] **Step 2: 替换页面模块中的字符串**
  逐文件替换硬编码字符串。以 `plugins.rs` 为例：
  - `"网络搜索"` → `t("Web search", "网络搜索")`
  - `"搜索 API 与脚本 fallback"` → `t("Search APIs with script fallback", "搜索 API 与脚本 fallback")`

- [ ] **Step 3: 替换 mod.rs 中的字符串**
  主菜单、状态栏、按钮、提示语全部替换。示例：
  ```rust
  // 旧: format!(" 文本模型 (当前: {active})")
  // 新: format!(" {} ({})", t("Text model", "文本模型"), active)
  ```

- [ ] **Step 4: 验证** `cargo check` 零警告 + `cargo test`

---

### Task 2: 全局设置补 3 字段

**Files:**
- Modify: `src/config_tui/pages/global.rs`

**Approach:** 在 `GLOBAL_FIELDS` 追加 3 个条目，`field_value`/`apply_field` 添加对应分支。

**新增字段：**
```rust
"tools.subagent_concurrency" → config.tools.subagent_concurrency: usize (parse/validate)
"display.show_token_usage" → config.display.show_token_usage: bool
"display.mixed_model_endpoint_display" → config.display.mixed_model_endpoint_display: String
```

验证 `subagent_concurrency` 范围 1-16，`mixed_model_endpoint_display` 仅限 `["hidden","append","replace"]`。

- [ ] **Step 1: TDD** 在 global.rs 测试中添加 3 个新字段的验证测试
- [ ] **Step 2: 实现** 更新 `GLOBAL_FIELDS`(11→14)、`field_value`、`apply_field`
- [ ] **Step 3: 验证** `cargo test config_tui::pages::global`

---

### Task 3: 插件详情字段编辑

**Files:**
- Modify: `src/config_tui/pages/plugins.rs`
- Modify: `src/config_tui/mod.rs`

**Approach:** 从旧代码（`git show HEAD~3:src/config_tui_old.rs | sed -n '668,1094p'`）提取 `plugin_fields()` 和 `apply_plugin_fields()` 到 plugins.rs 的 Pub API。mod.rs 的 Plugin 页增加 Enter→详情编辑模式（复用现有的 Field 编辑模式）。

**插件列表（14 个）完整字段摘要：**
- web: max_results(usize), tavily_api_keys(textarea), firecrawl_api_keys(textarea), searxng_url
- deep_research: 8 字段含 output_dir, thinking_depth(choices), max_review_revisions 等
- vision: vision_provider_id(choices), vision_model, 3 个 timeout
- image_generation: 10 字段含 api_key, endpoint, 模型 parameters
- web_images: 8 字段含 search providers, api keys
- print_image/pixels: 2 字段
- memes: meme_library, send_max_width_pixels, provider/model
- knowledge_base: 4 字段
- archlinux/man/package_advisor/Linux game compat: 各 1-2 字段
- memory: 通过 config.memory 直接操作
- api_quota: 跳转到 API quota 管理（见 Task 4）

- [ ] **Step 1: 提取 plugin_fields + apply_plugin_fields 到 plugins.rs**
- [ ] **Step 2: 添加 TDD 测试**（至少覆盖 web、deep_research 两个插件的字段读写）
- [ ] **Step 3: mod.rs 添加插件详情绘制 + 事件处理**（复用现有内联字段编辑器模式）
- [ ] **Step 4: 验证** `cargo check` + `cargo test`

---

### Task 4: API Quota 多账号管理

**Files:**
- Modify: `src/config_tui/pages/plugins.rs`
- Modify: `src/config_tui/mod.rs`

**Approach:** 从旧代码提取 `edit_api_quota*` 系列函数。在 plugins.rs 的 api_quota 详情入口实现账号列表 + 新建/删除/编辑。

- [ ] **Step 1: 提取 Pure Logic** — `api_quota_accounts()` 和 CRUD 辅助
- [ ] **Step 2: TDD 测试**
- [ ] **Step 3: mod.rs 绘制 + 事件**
- [ ] **Step 4: 验证**

---

### Task 5: 人格 CRUD

**Files:**
- Modify: `src/config_tui/pages/prompts.rs`
- Modify: `src/config_tui/mod.rs`

**Approach:** 从旧代码提取 `edit_personas`/`new_persona`/`apply_persona_delete`/`move_persona_scope`。prompts.rs 新增 Create/Edit/Delete/Rename 函数。mod.rs 添加子操作模式（PromptAction）。

- [ ] **Step 1: 提取 Pure Logic**
- [ ] **Step 2: TDD 测试**
- [ ] **Step 3: mod.rs 绘制 + 事件**
- [ ] **Step 4: 验证**

---

### Task 6: QQ 深度配置

**Files:**
- Modify: `src/config_tui/pages/platforms.rs`
- Modify: `src/config_tui/mod.rs`

**Approach:** 从旧代码提取 QQ 专用配置函数。platforms.rs 新增 `qq_advanced_fields()` 等纯逻辑。mod.rs 的 QQ 平台子页增加 Enter→高级配置（多子页 navigation）。

**子页清单：**
1. 命令前缀 / 权限
2. 会话限流
3. 模型路由（model_route）
4. 会话级人格覆盖
5. real_context 引擎配置（judge_advanced、triggers、continuation、reply_target 等）
6. 消息历史配置
7. 表情包管理
8. 历史归档

- [ ] **Step 1: 提取 Pure Logic** 分 8 个子模块
- [ ] **Step 2: TDD 测试**
- [ ] **Step 3: mod.rs 增强 platforms 绘制 + 事件**
- [ ] **Step 4: 验证**

---

### Task 7: 收尾

- [ ] `cargo check` 零警告
- [ ] `cargo test` 全绿
- [ ] 更新 docs/ROADMAP.md 标记完成
- [ ] git commit

---

## 执行顺序

Task 1（i18n）→ Task 2（全局字段补全）→ Task 3（插件详情）→ Task 4（API quota）→ Task 5（人格 CRUD）→ Task 6（QQ 深度配置）→ Task 7（收尾）

优先级：Task 1/2 快速见效 → Task 3/4 中等规模 → Task 5/6 大工作量

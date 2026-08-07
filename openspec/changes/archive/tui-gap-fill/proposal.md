## 背景

ratatui TUI 重写（`tui-overhaul-and-analysis`）相对旧 crossterm TUI 存在 6 项功能回退：
1. i18n 双语（新 TUI 全中文硬编码）
2. 全局设置缺 3 字段
3. 插件详情字段编辑缺失
4. API quota 多账号管理缺失
5. 人格 CRUD 缺失
6. QQ 深度配置缺失

本变更补齐全部 6 项，使 ratatui TUI 达到并超过旧 crossterm TUI 的功能覆盖。

## 决策

- 从旧代码（git `HEAD~5:src/config_tui_old.rs`）提取纯逻辑函数，而非重新实现
- 插件字段定义匹配**当前** config 结构体（旧 TUI 的字段名已过时）
- 页面交互沿用既有 ratatui 模式：列表导航 + 内联字段编辑 + 布尔切换

## 实现

| 功能 | 位置 | 说明 |
|---|---|---|
| i18n 包装 | `pages/mod.rs` `t()` | 包装 `crate::i18n::text`，全 TUI 双语化 |
| 全局 3 字段 | `pages/global.rs` | `subagent_concurrency`(1-16)/`show_token_usage`/`mixed_model_endpoint_display` |
| 插件详情 | `pages/plugins.rs` + `mod.rs` | Field 抽象 + 14 插件全字段 + apply 校验 |
| API quota | `pages/plugins.rs` + `mod.rs` | DeepSeek/OpenRouter 账号 CRUD，Tab 切换 |
| 人格 CRUD | `pages/prompts.rs` + `mod.rs` | 新建/重命名/删除(确认+清理作用域)/激活 |
| QQ 深度配置 | `pages/platforms.rs` + `mod.rs` | 9 项高级字段（权限/限流/模型路由等） |

## 验证

- `cargo check` 零警告
- `cargo test` 1235 passed / 0 failed / 3 ignored
- `cargo build` 通过

## 后续

- QQ real_context 引擎全量参数（~3000 行旧代码）未迁移，保留为 roadmap
- Telegram/QQ 官方平台深度配置同

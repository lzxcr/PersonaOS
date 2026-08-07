## ADDED Requirements

### Requirement: TUI 功能完备
TUI 配置界面 SHALL 覆盖旧 crossterm 版本的全部功能。

#### Scenario: 双语界面
- **WHEN** 运行 `pos config` 且系统 locale 为中文
- **THEN** 界面显示中文
- **AND** 系统 locale 为英文时显示英文

#### Scenario: 全局设置完整
- **WHEN** 打开「全局参数设置」
- **THEN** 可编辑 `subagent_concurrency`（范围 1-16）、`show_token_usage`（bool）、`mixed_model_endpoint_display`（hidden/append/replace）

#### Scenario: 插件详情编辑
- **WHEN** 在插件列表按 Enter
- **THEN** 进入该插件的全部可编辑字段（web API keys、deep_research 参数、生图配置等）
- **AND** 修改后校验写入

#### Scenario: API 额度管理
- **WHEN** 在 `api_quota` 插件按 Enter
- **THEN** 进入 DeepSeek/OpenRouter 多账号管理（Tab 切换、a 添加、d 删除、Enter 编辑）

#### Scenario: 人格管理
- **WHEN** 在提示词页按 n/r/d
- **THEN** 分别新建/重命名/删除人格，删除时清理作用域目录
- **AND** Enter 激活选中人格

#### Scenario: QQ 深度配置
- **WHEN** 在 IM 平台页对 QQ 按 Enter
- **THEN** 进入高级配置（权限/中间消息/用户识别/会话限流等）

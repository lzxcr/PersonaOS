## MODIFIED Requirements

### Requirement: 项目身份
项目 SHALL 不预置任何默认模型供应商。首次启动时 `default_providers()` 返回空列表，用户必须自行配置模型供应商和 API key 后才能使用。

#### Scenario: 无预置供应商
- **WHEN** 首次运行 `pos init`
- **THEN** `config.jsonc` 中 `providers` 为空对象
- **AND** 不包含 `opencode` 或任何其他供应商的预置条目

### Requirement: 品牌纯净 (扩展)
文档 SHALL 不将特定供应商描述为"默认接入"。

## ADDED Requirements

### Requirement: Telegram 平台完整实现
`src/platforms/telegram.rs` SHALL 完整实现 `PlatformAdapter` trait，支持 getUpdates 长轮询和 webhook 两种接入模式。

#### Scenario: 长轮询
- **WHEN** `platforms.telegram.enabled = true` 且 `webhook_path` 为空
- **THEN** daemon 启动 Telegram getUpdates 长轮询
- **AND** 入站消息经 `run_platform_turn` 处理

#### Scenario: 消息转换
- **WHEN** Telegram 用户发送文本/图片/语音/文件消息
- **THEN** `message_to_event()` 产出正确的 `PlatformInboundEvent`
- **AND** 支持 mention 解析、回复引用、群聊/私聊识别

### Requirement: 编译零警告
`cargo build` SHALL 产出零警告。

#### Scenario: 零警告
- **WHEN** 执行 `cargo build --release`
- **THEN** 无 warning 输出
- **AND** `cargo test` 全部通过

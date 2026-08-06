## Why

1. **Opencode 预接入**：项目硬编码了 opencode.ai 作为默认模型供应商（`src/default_models.rs`、`src/config.rs` 的 `default_opencodezen()`）。PersonaOS 应不绑定任何特定供应商，用户自行配置。
2. **Telegram 仅 stub**：当前 `src/platforms/telegram.rs` 只有 63 行骨架代码，无法实际收发消息。
3. **编译警告**：`cargo build` 有 2 个 unused import 警告。

## What Changes

### Task 1 — 移除 Opencode 预接入

#### 1.1 删除默认供应商常量和逻辑
- 删除 `src/default_models.rs` 中 `OPENCODE_PROVIDER_ID` / `OPENCODE_ZEN_BASE_URL` / `OPENCODE_DEFAULT_CHAT_MODEL` / `OPENCODE_DEFAULT_VISION_MODEL`
- 保留文件作为占位（后续可放通用默认值）

#### 1.2 清理 config.rs
- 删除 `default_opencodezen()` 方法
- 删除 `is_opencode_zen()` 方法
- `default_providers()` 返回空 Vec（不预置任何供应商）
- 删除 opencodezen → opencode 迁移逻辑
- 更新所有涉及 opencode 的测试，改用通用测试 provider

#### 1.3 更新测试
- `cli.rs` 测试中的 opencode 字符串 → 通用测试名
- `config.rs` 测试中的 opencode 默认值 → 空默认验证
- `llm/openai_compatible.rs` 测试中的 opencode URL → 通用测试 URL
- `render/mod.rs` / `state/mod.rs` / `tools/diagnostics.rs` 中的 opencode 引用 → 清理

#### 1.4 更新文档
- README.md 中"默认接入了 opencode" → "需自行配置模型供应商"
- docs/CONFIG.md 移除 Opencode 作为默认供应商的暗示

### Task 2 — 完整实现 Telegram 平台

#### 2.1 重写 `src/platforms/telegram.rs`
- 完整 `TelegramAdapter`：HTTP client + bot token
- `PlatformAdapter::send`：真实调用 Telegram Bot API（sendMessage/sendPhoto/sendDocument/sendVoice）
- Telegram API 类型定义（Update/Message/Chat/User 等 serde 结构体）
- `message_to_event()`：TgMessage → PlatformInboundEvent 转换
- getUpdates 长轮询循环（`run_polling`）
- Webhook 支持（可选，当配置 `webhook_path` 时）

#### 2.2 daemon 集成
- `PlatformRuntime` 添加 `telegram: TelegramRuntime` 字段
- daemon 启动时若 `config.platforms.telegram.enabled`，spawn 轮询 task
- 热重载时若配置变更，重启 Telegram 监听

#### 2.3 消息段映射
- 出站：Text/Markdown → Telegram MarkdownV2，ImageBytes → sendPhoto，FilePath → sendDocument
- 入站：photo/voice/document → PlatformInboundMedia，entities → mentions
- 群聊/私聊识别（chat.type → ConversationKind）
- 回复引用（reply_to_message → ResponseTarget.quote）

### Task 3 — 编译零警告

#### 3.1 清理未使用导入
- `src/platforms/telegram.rs` — 移除未使用的 import
- `src/platforms/qq_official.rs` — 移除未使用的 import

#### 3.2 其他警告排查
- 运行 `cargo build 2>&1 | grep warning` 确认零警告
- 修复任何残留警告

## 不做的

- 不实现 QQ 官方机器人完整版（保持 stub）
- 不实现 Telegram 群管插件（保持平台核心功能）
- 不添加 TTS/STT provider 实现

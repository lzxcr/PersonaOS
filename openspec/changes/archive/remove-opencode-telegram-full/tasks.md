## 1. 移除 Opencode 预接入
- [ ] 1.1 清空 `src/default_models.rs` 常量
- [ ] 1.2 删除 `config.rs` 中 `default_opencodezen()` / `is_opencode_zen()`
- [ ] 1.3 `default_providers()` 返回空 Vec
- [ ] 1.4 更新 `config.rs` 测试（opencode → 通用测试 provider）
- [ ] 1.5 更新 `cli.rs` 测试
- [ ] 1.6 更新 `llm/openai_compatible.rs` 测试
- [ ] 1.7 清理 `render/mod.rs` / `state/mod.rs` / `tools/diagnostics.rs` 中的 opencode 引用
- [ ] 1.8 更新 `config_tui.rs` 中的 `is_opencode_zen()` 调用
- [ ] 1.9 更新 README.md（移除 opencode 默认接入描述）

## 2. 完整实现 Telegram
- [ ] 2.1 重写 `TelegramAdapter`（HTTP client + API 调用）
- [ ] 2.2 实现 `message_to_event()` 入站转换
- [ ] 2.3 实现 `send()` 出站映射（Text/Image/File/Audio）
- [ ] 2.4 实现 getUpdates 长轮询循环
- [ ] 2.5 Webhook 支持
- [ ] 2.6 daemon 集成（PlatformRuntime + 启动 task）
- [ ] 2.7 热重载支持

## 3. 编译零警告
- [ ] 3.1 清理 telegram.rs 未使用 import
- [ ] 3.2 清理 qq_official.rs 未使用 import
- [ ] 3.3 `cargo build 2>&1 | grep warning` 确认零警告

## 4. 验证
- [ ] 4.1 `cargo build` 通过且零警告
- [ ] 4.2 `cargo test` 全部通过
- [ ] 4.3 全仓库 Grep 确认无 opencode 硬编码残留

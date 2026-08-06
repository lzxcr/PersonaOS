## 1. Phase 1 — 清理
- [ ] 1.1 README 重写：清除三舟描述，新增致谢章节
- [ ] 1.2 `web/index.html:287` — MIYU → POS
- [ ] 1.3 `src/config_tui.rs:119` — MIYU CONFIG → POS CONFIG
- [ ] 1.4 `src/tools/descriptions/create_artifact.json:4` — Miyu → PersonaOS
- [ ] 1.5 `src/tools/descriptions/set_alarm.json:18` — Miyu → PersonaOS
- [ ] 1.6 `src/config.rs` — 注释 + SANZHOU_PERSONA_FILE → DEFAULT_PLATFORM_PERSONA_FILE
- [ ] 1.7 `src/prompts.rs:90` — 注释 Sanzhou → 中性示例
- [ ] 1.8 `src/tools/knowledge_base.rs:259` — 注释 Sanzhou → 中性示例
- [ ] 1.9 `assets/fonts/README.md:7` — MIYU_RENDERER_FONTS_DIR → POS_RENDERER_FONTS_DIR

## 2. Phase 2 — 平台配置泛化
- [ ] 2.1 `config.rs`: `PlatformsConfig` 支持多平台注册（`HashMap<String, PlatformConfig>`）
- [ ] 2.2 `config.rs`: 向后兼容旧 `platforms.qq` 格式（自动迁移）
- [ ] 2.3 插件系统 `plugin_enabled()` 等按 `context.conversation.platform` 路由
- [ ] 2.4 `message_history` 插件 db 路径动态化（`platforms/{platform}/...`）
- [ ] 2.5 `plugins/mod.rs` 中 `require_ai_confirmation` key 平台化

## 3. Phase 3 — Telegram 平台
- [ ] 3.1 `src/platforms/telegram.rs` — PlatformAdapter 实现
- [ ] 3.2 传输层：getUpdates 轮询 + webhook 模式
- [ ] 3.3 Update → PlatformInboundEvent 转换
- [ ] 3.4 OutboundSegment → Telegram API 映射
- [ ] 3.5 `config.rs`: `TelegramConfig` 配置结构
- [ ] 3.6 daemon 启动/热重载集成

## 4. Phase 3b — QQ 官方机器人
- [ ] 4.1 `src/platforms/qq_official.rs` — PlatformAdapter 实现
- [ ] 4.2 QQ 官方 WebSocket 事件管线
- [ ] 4.3 消息段转换（Markdown/图片/音频）
- [ ] 4.4 `config.rs`: `QqOfficialConfig` 配置结构
- [ ] 4.5 daemon 启动/热重载集成

## 5. Phase 4 — 媒体能力接口
- [ ] 5.1 定义 `SpeechSynthesis` trait（TTS 接口）
- [ ] 5.2 定义 `SpeechRecognition` trait（STT 接口）
- [ ] 5.3 `OutboundSegment::Audio` 消息段
- [ ] 5.4 `PlatformInboundMedia::Audio` 入站语音转写管线
- [ ] 5.5 OneBot/Telegram/QQ Official 平台的 Audio 段处理

## 6. Phase 5 — Live2D WebUI
- [ ] 6.1 WebUI 新增 Live2D canvas 容器
- [ ] 6.2 Cubism SDK 加载逻辑（CDN + 本地回退）
- [ ] 6.3 模型文件配置路径（`~/.pos/data/live2d/`）
- [ ] 6.4 通过 WebSocket 事件驱动表情/动作

## 7. Phase 6 — 子代理持久化
- [ ] 7.1 `resume_id` 检查点迁移到 SQLite
- [ ] 7.2 daemon 重启后可恢复子代理会话
- [ ] 7.3 TTL 清理 + 上限管理

## 8. 验证
- [ ] 8.1 `cargo build` 通过
- [ ] 8.2 `cargo test` 全部通过
- [ ] 8.3 全仓库 Grep 确认无 MIYU / 三舟 / sanzhou 残留

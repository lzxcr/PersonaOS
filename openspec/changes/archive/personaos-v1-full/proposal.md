## Why

PersonaOS 作为 Miyu 的通用人格化平台重构，目前存在以下问题：

1. **品牌残留**：代码中有 4 处运行时可见的 `MIYU`/`Miyu` 标识 + 6 处注释/文档残留 + `SANZHOU_PERSONA_FILE` 遗留功能常量。
2. **功能缺位**：Miyu 规划中的 5 项功能（TTS、STT、Live2D、Telegram、QQ 官方机器人）及子代理持久化在 PersonaOS 中均未实现。
3. **架构绑定**：平台插件配置 `config.platforms.qq.plugins` 硬编码 QQ 平台，新增平台无法复用同一插件体系。
4. **缺少致谢**：README 未向前身项目 Miyu 致谢。

本次改动以"统一可扩展接口"为原则，完成品牌清理 + 功能补全 + 架构泛化，不引入人格特化。

## What Changes

### Phase 1 — 清理 (品牌 + 遗留代码)

#### 1.1 README 重写
- 清除末尾"三舟/Miyu 角色"段落
- 新增「致谢」章节（Miyu + 参考项目列表）
- 重写项目渊源描述为中性版本

#### 1.2 品牌残留清理
- `web/index.html:287` — `MIYU` → `POS`
- `src/config_tui.rs:119` — `MIYU CONFIG` / `MIYU 配置` → `POS CONFIG` / `POS 配置`
- `src/tools/descriptions/create_artifact.json:4` — `Miyu` → `PersonaOS`
- `src/tools/descriptions/set_alarm.json:18` — `Miyu` → `PersonaOS`
- 清理全部注释中的 Sanzhou/miyu/MIYU 引用

#### 1.3 遗留功能代码清理
- `src/config.rs` 中的 `SANZHOU_PERSONA_FILE` 常量 → 重命名为 `DEFAULT_PLATFORM_PERSONA_FILE`（中性命名，机制保留）
- `PlatformPersonaOverride::Builtin` 的 doc/serde 注释中的 `miyu`/`Sanzhou` 示例 → 中性名称
- `assets/fonts/README.md` — `MIYU_RENDERER_FONTS_DIR` → `POS_RENDERER_FONTS_DIR`

### Phase 2 — 架构泛化（平台插件配置解耦）

#### 2.1 平台配置重构
- `PlatformsConfig` 从单一 `qq: OneBotConfig` 扩展为多平台注册表
- 每种平台有自己的 `PlatformConfig`（含 `plugins` 子配置），共享 plugin id 命名空间
- 迁移现有 `config.platforms.qq.plugins.*` 到新结构，向后兼容旧配置文件

#### 2.2 插件系统泛化
- `plugin_enabled()` / `observe_ingress()` 等从 `config.platforms.qq.plugins` 读取改为按 `context.conversation.platform` 路由
- `message_history` 插件 db 路径从 `platforms/onebot/` 改为 `platforms/{platform}/` 动态路径

### Phase 3 — 新平台实现

#### 3.1 Telegram 平台 (`src/platforms/telegram.rs`)
- 实现 `PlatformAdapter` trait（发送/接收/群管理/表情回应）
- 传输层：轮询 getUpdates + webhook 双模式
- 入站：Telegram Update → `PlatformInboundEvent`
- 出站：`OutboundSegment` → Telegram MarkdownV2 / Photo / Audio
- 配置：`config.platforms.telegram`

#### 3.2 QQ 官方机器人 (`src/platforms/qq_official.rs`)
- 实现 `PlatformAdapter` trait
- QQ 官方 API（WebSocket + HTTP）+ 事件管线
- 配置：`config.platforms.qq_official`

### Phase 4 — 媒体能力接口（TTS + STT）

#### 4.1 TTS 接口
- 定义 `SpeechSynthesis` trait（`synthesize(text) -> AudioData`）
- 默认实现：无（需用户配置 provider）
- Outbound 扩展：`OutboundSegment::Audio`
- 集成：LLM 可通过 tool 调用 TTS，Telegram/QQ 原生发送语音消息

#### 4.2 STT 接口
- 定义 `SpeechRecognition` trait（`transcribe(audio) -> String`）
- 默认实现：无
- Inbound 扩展：`PlatformInboundMedia::Audio` 自动转写进对话文本

### Phase 5 — Live2D WebUI 集成
- WebUI 中新增 Live2D 展示容器（canvas + Cubism SDK）
- 模型文件由用户配置（`~/.pos/data/live2d/`）
- 通过现有 `PlatformPlugin` 钩子驱动表情/动作切换
- 完全可选：无模型文件时 WebUI 保持现状

### Phase 6 — 子代理持久化
- `resume_id` 从进程内 HashMap 迁移到 SQLite（`state` 表）
- daemon 重启后子代理会话可恢复
- 保留 TTL 清理机制

## 不做的
- 不引入任何人格特化提示词
- 不预置 Live2D 模型/TTS 模型文件
- 不在本次改动中实现 WeChat 平台（架构已预留）
- 不改变现有 OneBot/NapCat 的 QQ 群管插件行为

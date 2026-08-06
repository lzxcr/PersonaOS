## MODIFIED Requirements

### Requirement: 品牌纯净
代码、前端、文档 SHALL 不含 Miyu / 三舟 / sanzhou / miyu 品牌标识（历史归档与明确标注的来源致谢除外）。

#### Scenario: 品牌检查
- **WHEN** 全仓库搜索 `Miyu` / `三舟` / `sanzhou`（排除 git 历史、openspec 归档、README 致谢章节）
- **THEN** 无运行时可见的残留（代码、WebUI、工具描述、TUI 界面、注释）
- **AND** 不存在 `SANZHOU_PERSONA_FILE` / `MIYU_RENDERER_FONTS_DIR` 等遗留常量

#### Scenario: 致谢保留
- **WHEN** README 致谢章节引用 Miyu
- **THEN** 明确标注为「前身项目」
- **AND** 不构成品牌身份混淆

### Requirement: 平台插件配置
平台插件配置 SHALL 按 `platform` 字段路由，而非硬编码 `platforms.qq.plugins`。

#### Scenario: 多平台插件
- **WHEN** Telegram 和 OneBot 同时运行
- **THEN** 各平台独立读取自己的 `platforms.{platform}.plugins` 配置
- **AND** 旧 `platforms.qq.plugins` 格式自动迁移

## ADDED Requirements

### Requirement: 平台适配器注册
系统 SHALL 支持通过 `PlatformAdapter` trait 注册多种 IM 平台（OneBot、Telegram、QQ Official）。

#### Scenario: Telegram 接入
- **WHEN** 配置 `platforms.telegram.bot_token`
- **THEN** daemon 启动 Telegram 监听
- **AND** 入站消息经 `PlatformTurnContext` 管线处理

#### Scenario: QQ 官方机器人接入
- **WHEN** 配置 `platforms.qq_official`
- **THEN** daemon 连接 QQ 官方 WebSocket
- **AND** 入站消息复用平台无关核心

### Requirement: 语音合成接口
系统 SHALL 定义 `SpeechSynthesis` trait 作为 TTS 的统一抽象，不内置实现。

#### Scenario: TTS 工具调用
- **WHEN** LLM 调用 TTS 工具并传入文本
- **THEN** 系统调用用户配置的 `SpeechSynthesis` 实现
- **AND** 产出可通过 `OutboundSegment::Audio` 发送的音频数据

### Requirement: 语音识别接口
系统 SHALL 定义 `SpeechRecognition` trait 作为 STT 的统一抽象。

#### Scenario: 语音消息转写
- **WHEN** 入站消息包含 `PlatformInboundMedia::Audio`
- **THEN** 若配置了 STT provider，自动转写文本并入对话
- **AND** 未配置时音频以元数据形式保留

### Requirement: Live2D 展示
WebUI SHALL 支持可选的 Live2D 模型展示，通过 WebSocket 事件驱动。

#### Scenario: 无模型文件
- **WHEN** `~/.pos/data/live2d/` 为空
- **THEN** WebUI 保持现有布局，不展示 Live2D 容器

### Requirement: 子代理持久化
子代理会话的 `resume_id` SHALL 持久化到 SQLite，daemon 重启后可恢复。

#### Scenario: 重启恢复
- **WHEN** daemon 重启后收到恢复请求
- **THEN** SQLite 中的子代理检查点可被重新加载
- **AND** 超出 TTL 的检查点自动清理

### Requirement: 功能基线记录
openspec 基线 SHALL 记录 PersonaOS 的功能实现状态。

#### Scenario: 功能对比清单
- **WHEN** 查看 `personaos-framework` 基线
- **THEN** 记录 TTS/STT/Live2D/Telegram/QQ Official/子代理持久化的实现状态

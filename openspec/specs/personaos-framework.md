# PersonaOS 框架

## Requirements

### Requirement: 项目身份
项目包名 SHALL 为 `persona-os`，默认二进制名 SHALL 为 `pos`。

#### Scenario: 构建产物
- **WHEN** 执行 `cargo build`
- **THEN** 生成 `pos` 二进制
- **AND** Cargo.lock 中包名为 `persona-os`

### Requirement: 路径基础设施命名
路径基础设施类型 SHALL 命名为 `PersonaPaths`。

#### Scenario: 类型引用
- **WHEN** 代码引用路径基础设施
- **THEN** 使用 `PersonaPaths` 类型
- **AND** 项目中不存在 `MiyuPaths` 标识符

### Requirement: 数据与安装目录
用户数据目录 SHALL 为 `~/.pos`，系统安装资源目录 SHALL 为 `/usr/share/pos`。

#### Scenario: 目录解析
- **WHEN** `PersonaPaths::new()` 解析用户目录
- **THEN** 返回 `~/.pos`（非 `~/.miyu`）
- **AND** 系统资源路径使用 `/usr/share/pos`

### Requirement: 环境变量前缀
环境变量 SHALL 使用 `POS_` 前缀。

#### Scenario: 环境变量命名
- **WHEN** 构建或运行时读取环境变量
- **THEN** 使用 `POS_BUILD_ID` / `POS_HOME` / `POS_LANG` / `POS_LOG` / `POS_DIRECT` 等
- **AND** 不存在 `MIYU_*` 前缀环境变量

### Requirement: 日志标识
日志 target SHALL 使用 `pos`。

#### Scenario: 日志输出
- **WHEN** 系统记录日志
- **THEN** target 为 `pos` / `pos::qq` 等
- **AND** 不存在 `miyu` 日志 target

### Requirement: 内置人格注册表为空
`BUILTIN_PERSONAS` 注册表 SHALL 保留机制但默认不含任何人格条目。

#### Scenario: 注册表为空
- **WHEN** 系统启动且注册表为空
- **THEN** `default_builtin_persona()` 返回错误，提示「未注册任何内置人格」
- **AND** 用户必须配置自定义人格或注册内置人格后才能使用

#### Scenario: 无预置人格文件
- **WHEN** 查看 `src/prompts/`
- **THEN** 不存在 `builtin-miyu.md` / `builtin-sanzhou.md`
- **AND** 不存在任何预置人格提示词

### Requirement: 品牌纯净
代码、前端、文档 SHALL 不含 Miyu / 三舟 / sanzhou / miyu 品牌标识（git 历史与致谢章节除外）。

#### Scenario: 品牌检查
- **WHEN** 全仓库搜索 `Miyu` / `三舟` / `sanzhou`（排除 git 历史、openspec 归档、README 致谢）
- **THEN** 无运行时可见的残留
- **AND** 不存在 `SANZHOU_PERSONA_FILE` / `MIYU_RENDERER_FONTS_DIR` 等遗留常量

#### Scenario: 致谢保留
- **WHEN** README 致谢章节引用 Miyu
- **THEN** 明确标注为「前身项目」
- **AND** 不构成品牌身份混淆

### Requirement: 默认知识库为空
默认知识库 SHALL 不预置任何内容，用户按需导入。

#### Scenario: 首次初始化
- **WHEN** 首次运行初始化
- **THEN** 不拉取 Arch Linux ShorinWiki
- **AND** 知识库机制可用但为空

### Requirement: 平台插件配置
平台插件配置 SHALL 按 `platform` 字段路由，而非硬编码平台。

#### Scenario: 多平台插件
- **WHEN** 多个平台同时运行
- **THEN** 各平台独立读取自己的 `platforms.{platform}.plugins` 配置
- **AND** 旧 `platforms.qq.plugins` 格式向后兼容

### Requirement: 平台适配器注册
系统 SHALL 通过 `PlatformAdapter` trait 支持多种 IM 平台（OneBot、Telegram、QQ Official）。

#### Scenario: 已有平台
- **WHEN** 查看 `src/platforms/` 目录
- **THEN** 包含 `onebot.rs`、`telegram.rs`、`qq_official.rs` 三个平台模块
- **AND** 每个模块实现 `PlatformAdapter` trait

### Requirement: 语音合成接口
系统 SHALL 定义 `SpeechSynthesis` trait 作为 TTS 的统一抽象，`src/media/mod.rs` 中定义。

### Requirement: 语音识别接口
系统 SHALL 定义 `SpeechRecognition` trait 作为 STT 的统一抽象。

### Requirement: 音频消息段
`OutboundSegment` SHALL 包含 `Audio` 变体，支持语音消息发送。

### Requirement: Live2D 展示
WebUI SHALL 包含可选的 Live2D canvas 容器，无模型文件时隐藏。

#### Scenario: 无模型文件
- **WHEN** `~/.pos/data/live2d/` 为空
- **THEN** WebUI 保持现有布局，不展示 Live2D

### Requirement: 子代理持久化
子代理会话检查点 SHALL 持久化到磁盘，daemon 重启后可恢复。

#### Scenario: 重启恢复
- **WHEN** daemon 重启后收到 `resume_id` 请求
- **THEN** 磁盘中的检查点可被重新加载
- **AND** 超出 TTL 的检查点自动清理

### Requirement: 功能基线记录
openspec 基线 SHALL 记录功能实现状态。

#### Scenario: 已实现
- **WHEN** 查看本基线
- **THEN** artifact 增强、通讯平台命令、上下文触限压缩、manage_platform_access、web/daemon 拆分、Telegram 桩、QQ Official 桩、TTS/STT 接口、Live2D 容器、子代理持久化均已实现
- **AND** TTS/STT/Live2D 的具体 provider 实现待用户配置

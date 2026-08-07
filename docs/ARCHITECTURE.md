# PersonaOS 架构

PersonaOS 是一个本地 AI 人格操作系统——单二进制 `pos`，集 CLI REPL、守护进程、WebUI 于一体。本文档描述进程模型、模块拓扑和核心数据流。

## 进程模型

```
┌─────────────┐   IPC (Unix socket)   ┌──────────────────┐
│  pos CLI    │ ◄──────────────────► │  pos daemon       │
│  (前台)     │   core.sock           │  (后台常驻)       │
│             │                       │                   │
│  · REPL     │                       │  · Agent Actor    │
│  · 一次性   │                       │  · WebUI HTTP/WS  │
│  · config   │                       │  · IM 桥接(QQ等)  │
│  · shell钩  │                       │  · 记忆整理线程   │
└─────────────┘                       └──────────────────┘
```

`pos` 二进制以不同参数自我复制实现所有角色：

| 角色 | 启动方式 | 说明 |
|---|---|---|
| CLI 前台 | `pos [msg]` | REPL 或一次性对话 |
| Daemon | `pos __daemon` | 后台引擎（`src/daemon.rs`，`src/cli.rs:855`） |
| Web 服务 | daemon 内置 | 无独立进程，HTTP/WS 端口由 daemon 承载 |
| 闹钟 worker | `pos __alarm-worker` | 独立子进程播放闹钟音频 |
| 渲染器 worker | `pos --renderer` | 文本转图片卡片（QQ 群聊渲染，`src/platforms/plugins/renderer.rs`） |

CLI 与 daemon 通过本地 Unix socket（`~/.pos/runtime/core.sock`）通信。`pos web` 命令仅打印 URL；实际 Web 服务在 daemon 内运行。

## 模块拓扑

```
src/
├── main.rs            # 入口：解析 CLI → 分发
├── cli.rs             # CLI 解析、子命令路由、REPL 循环
├── cli.rs:config.rs   # 配置结构体（序列化/反序列化/验证）
├── ipc.rs             # CLI ↔ daemon Unix socket 通信
├── daemon.rs          # daemon 启动、web 监听、热重载
├── web.rs             # HTTP/WebSocket 服务、Actor 命令调度
│
├── agent/             # 对话主循环（~274KB）
│   ├── mod.rs         #   主循环、tool loop、stream 消费
│   ├── compact.rs     #   上下文压缩（LLM 摘要 + 多轮合并）
│   ├── overflow.rs    #   溢出检测（trim_at_ratio 阈值）
│   └── conversation.rs#   会话消息管理
│
├── llm/               # 模型供应商
│   ├── mod.rs         #   ChatMessage / ChatContent / ToolCall 类型
│   └── openai_compatible.rs  # OpenAI 兼容协议客户端
│
├── tools/             # 工具系统（80+ 工具）
│   ├── mod.rs         #   注册入口 builtin_registry()
│   ├── registry.rs    #   ToolSpec / ToolRegistry / call
│   ├── load_tools.rs  #   按需懒加载 (group:xxx / stub)
│   ├── descriptions/  #   工具描述 JSON 文件
│   ├── task.rs        #   子代理 (task 工具)
│   └── subagent_runner.rs  # 子代理执行引擎
│
├── platforms/         # IM 平台桥接
│   ├── mod.rs         #   平台无关核心：turn 驱动、限流、会话解析
│   ├── types.rs       #   PlatformAdapter trait、消息段、事件类型
│   ├── onebot.rs      #   NapCat/QQ (OneBot v11 WebSocket)
│   ├── telegram.rs    #   Telegram Bot API (stub)
│   ├── qq_official.rs #   QQ 官方机器人 (stub)
│   ├── commands.rs    #   平台内建命令 (/reset /stop /models)
│   ├── access_control.rs  # 动态准入授权
│   ├── tool.rs        #   平台工具 (send_message_to_user 等)
│   └── plugins/       #   QQ 群管插件体系
│       ├── mod.rs     #     PlatformPlugin trait + 注册表
│       ├── access_manager.rs   # 准入管理
│       ├── real_context/       # 真人语境引擎（最复杂插件）
│       ├── group_management.rs # 群管工具
│       ├── message_history/    # 消息归档
│       ├── message_recall.rs   # 撤回
│       ├── meme_collector.rs   # 表情包
│       ├── reply_processor.rs  # 回复加工
│       └── renderer.rs        # 文本转图渲染器
│
├── memory/            # 记忆系统
│   ├── mod.rs         #   MemoryStore (SQLite 双库)
│   └── organizer.rs   #   后台整理线程（短期→长期提炼）
│
├── state/             # 持久化状态
│   ├── conversation_db.rs  # 会话/会话绑定 SQLite
│   ├── migrations.rs       # 数据库迁移
│   └── usage.rs            # token 用量统计
│
├── media/             # 媒体能力接口 (新增)
│   └── mod.rs         #   SpeechSynthesis / SpeechRecognition trait
│
├── skills/            # Skill 系统
├── prompts/           # 提示词模板
├── render/            # 终端渲染 (cosmic-text)
├── shell/             # fish/bash/zsh 集成
├── config_tui/        # TUI 配置界面 (ratatui + MD3 主题)
│   ├── mod.rs         #   主应用路由 / 事件循环
│   ├── theme.rs       #   MD3 暗/亮双主题
│   └── pages/         #   8 个子页面 (模型/子代理/供应商/插件/平台/提示词/全局)
│       ├── providers.rs   # 供应商 CRUD + 模型池
│       ├── plugins.rs     # 插件开关 + 详情字段编辑 + API 额度多账号
│       ├── platforms.rs   # 平台开关 + QQ 高级配置
│       ├── prompts.rs     # 人格列表 + CRUD
│       ├── global.rs      # 14 项全局参数
│       └── ...            # 文本/多模态/子代理档位
├── question.rs        # 终端交互式问答
├── clipboard.rs       # 剪切板
├── token_counter.rs   # token 计数
└── i18n.rs            # 中英双语
```

## 核心数据流：一次对话的完整链路

### CLI REPL 对话

```
用户输入 (REPL)
  │
  ▼
cli.rs: 读取输入 ──► 通过 IPC (core.sock) 发送 ActorCommand
  │
  ▼
web.rs: Actor 主循环接收 StartTurn 命令
  │
  ▼
agent/mod.rs: run_agent_turn()
  │
  ├─ 1. 解析系统提示词 (prompts.rs → BUILTIN_PROMPTS / 自定义)
  ├─ 2. 解析记忆上下文 (memory/mod.rs → 短期/长期/知识点联想)
  ├─ 3. 组装工具注册表 (tools/mod.rs → builtin_registry)
  ├─ 4. 构造 OpenAI 请求 (llm/openai_compatible.rs)
  │
  ├─ 5. LLM 流式响应 ──► 解析 tool_calls
  │     │
  │     ├─ 工具调用 → tools/registry.rs → handler
  │     │              └─ 结果回注 messages
  │     │
  │     └─ 文本回复 → 流式输出到 IPC
  │
  └─ 6. overflow 检测 → 必要时触发 compact.rs 压缩
       │
       └─ 最终回复 → IPC → cli.rs 渲染
```

### IM 平台对话

```
QQ/TG 消息
  │
  ▼
onebot.rs / telegram.rs: 协议适配
  │
  ├─ 解析 → PlatformInboundEvent (types.rs:171)
  ├─ 准入检查 → platforms/mod.rs::RateWindow
  ├─ 插件预处理 → plugins/mod.rs (real_context 活跃判断、message_history 归档)
  │
  ▼
PlatformTurnContext (mod.rs:552)
  │
  ├─ resolve_platform_session → 会话绑定
  ├─ prepare_turn → 组装 TurnProfile
  │
  ▼
run_platform_turn (mod.rs:1899)
  │
  └─ ActorCommand::StartTurn → agent/ (同上 CLI 流程)
       │
       └─ 出站经过 reply_processor / 分帧 → adapter.send()
```

## 平台管线架构

```
          ┌──────────────────────┐
          │   PlatformAdapter     │  ← 每个平台实现此 trait
          │   (types.rs:385)     │
          ├──────────────────────┤
          │ send(message)        │  必实现
          │ bot_display_name()   │  必实现
          │ message_info()       │  可选
          │ group_members()      │  可选
          │ set_group_ban()      │  可选
          │ delete_message()     │  可选
          │ ...                  │
          └──────────────────────┘
                    ▲
         ┌─────────┼─────────┐
         │         │         │
    OneBotAdapter  │  QqOfficialAdapter
              TelegramAdapter
```

平台无关核心（`mod.rs`）通过 `PlatformTurnContext` 统一驱动所有平台，插件系统 (`plugins/`) 通过 `PlatformPlugin` trait 提供 16 个可重写钩子。

每个平台的插件配置按 `platform` 字段独立路由（`PlatformsConfig::plugin_config()`）。

## 记忆系统三层结构

```
┌─────────────────────────────────────────────────┐
│              记忆系统 (SQLite)                    │
├─────────────────────────────────────────────────┤
│                                                 │
│  ┌─────────┐  后台整理线程  ┌─────────┐          │
│  │短期日记 │ ──────►────── │长期日记 │          │
│  │(14天保留)│   (organizer) │(永久)   │          │
│  └─────────┘               └─────────┘          │
│       │                         │               │
│       └────── 联想召回 ─────────┘               │
│                  │                              │
│            ┌─────────┐                          │
│            │ 知识点  │  ← 提炼的事实             │
│            │(可遗忘) │                          │
│            └─────────┘                          │
│                                                 │
│  ┌──────────────┐                               │
│  │ 挤出上下文   │  ← 超限被移除的轮次            │
│  │ (单独 SQLite)│     可显式搜索找回             │
│  └──────────────┘                               │
└─────────────────────────────────────────────────┘
```

- **短期日记**：每轮对话完成后立即写入；14 天保留期，联想命中刷新
- **长期日记**：累计 14 条未整理日记后，独立后台线程提炼（不阻塞回复）
- **知识点**：提炼的事实，随时间衰减为"已遗忘"，不物理删除
- **联想召回**：以 jieba 中文分词做低成本匹配（三种记忆联合检索）

## Agent 生命周期

```
Turn 开始
  │
  ├─ overflow.rs: OverflowCheck
  │   └─ 若 token 超出 trim_at_ratio → compact.rs::perform_compact
  │       └─ LLM 摘要 + 多轮合并 → 压缩历史
  │
  ├─ 组装 ToolRegistry (按 config.plugins.* 条件注册)
  │
  ├─ LLM 流式响应循环
  │   ├─ tool_calls → 本地工具执行
  │   │   └─ 结果回到 messages → 继续 LLM
  │   └─ 文本 delta → 流式输出
  │
  ├─ 对话完成 → memory 写入
  │
  └─ Turn 结束
```

## 新增模块

### media/ — 媒体能力接口 (Phase 4)

```rust
pub trait SpeechSynthesis: Send + Sync {
    fn synthesise(&self, text: &str, language: Option<&str>) -> Result<SynthesisedAudio>;
}

pub trait SpeechRecognition: Send + Sync {
    fn transcribe(&self, audio: &[u8], mime: &str) -> Result<String>;
}
```

框架不内置实现。用户配置 provider 后，工具可通过这些 trait 调用 TTS/STT。

### telegram.rs / qq_official.rs — 新平台桩

当前为 `PlatformAdapter` trait 的骨架实现（stub）。`PlatformsConfig` 已预留 `telegram: Option<TelegramConfig>` 和 `qq_official: Option<QqOfficialConfig>` 配置字段。完整传输层实现在路线图中。

<p align="center">
  <img src="pics/pos-logo.png" alt="PersonaOS" width="180">
</p>

# PersonaOS

一个让 AI 拥有人格的本地 AI 操作系统。开箱即用的开源框架，为大语言模型提供**人格、记忆、技能、工具**能力，支持接入通讯平台。

> Operating System for AI Personas —— 不是 AI Linux 发行版，而是运行 AI 人格的本地平台。

## 定位

PersonaOS 将「人格」从应用代码中抽象出来，作为一等公民：

```
PersonaOS
├── Core
│   ├── LLM Runtime
│   ├── Memory System
│   ├── Tool System
│   └── Plugin Framework
│
├── Personas          ← 人格即数据，注册即用
├── Skills
├── Knowledge Bases
└── Interfaces
    ├── Terminal
    ├── WebUI
    └── IM Platforms
```

- 框架不内置任何人格。`BUILTIN_PERSONAS` 注册表为空，未注册人格时直接报错提示
- 人格通过注册表条目 + 提示词文件（`src/prompts/builtin-<name>.md`）定义，也可在 `~/.pos/data/prompts/` 放自定义人格文件
- 每个内建人格拥有独立的记忆作用域、表情库与知识库视图

## 有什么功能？

`pos` 由大模型驱动，默认接入了 [opencode](https://github.com/anomalyco/opencode) 的公共模型服务，你也可以配置自己的大模型服务。框架能力与人格解耦，任何人格都能使用完整工具集。

- 终端集成：与 `fish`、`zsh`、`bash` 集成，终端打字直接对话
- REPL 交互模式、TUI 配置界面、WebUI 网页
- NapCat 接入 QQ：私聊、群聊、群管理
- 记忆系统：短期/长期日记、知识点、联想召回（jieba 分词）
- 工具全家桶：天气、汇率、闹钟、玄学、骰子、表情包、搜图、生图、网络搜索、深度研究、文件操作、计算器、哈希编解码
- Skill 系统：可复用的子-agent 编排能力
- 知识库：按需导入本地内容，支持关键词 + 语义检索

## 如何构建？

```bash
git clone <your-fork>/PersonaOS.git
cd PersonaOS
cargo build --release
```

构建产物为 `pos` 二进制。首次运行自动初始化：

```bash
pos init          # 初始化配置与状态
pos               # 进入 REPL
pos config        # TUI 配置
pos web           # WebUI
pos persona <name> # 切换人格
```

## 配置与数据

| 路径 | 用途 |
|---|---|
| `~/.pos/config` | `config.jsonc`、主题、shell 集成 |
| `~/.pos/data` | prompts、identities、persona-avatars、scripts、skills、kb |
| `~/.pos/state` | 运行状态、会话、记忆 |

## 添加人格

1. 在 `src/prompts/builtin-<name>.md` 编写人格提示词，并在 `src/prompts.rs` 的 `BUILTIN_PERSONAS` 注册表中添加条目（`is_default: true` 标记默认人格）；
2. 或直接在 `~/.pos/data/prompts/` 放置 `<Name>.md` 人格文件，通过 `pos persona <Name>` 切换。

## 文档

| 文档 | 说明 |
|---|---|
| [架构](docs/ARCHITECTURE.md) | 进程模型、模块拓扑、数据流、平台管线 |
| [CLI 参考](docs/CLI.md) | 全部子命令、参数、Shell 集成 |
| [配置指南](docs/CONFIG.md) | config.jsonc 字段树、模型池、插件 |
| [开发指南](docs/DEVELOPMENT.md) | 新增工具/平台/插件的分步教程 |
| [路线图](docs/ROADMAP.md) | 功能全景、改进方向、技术债务 |

## 许可

PersonaOS 使用 MIT License 发布，见 `LICENSE`。

## 致谢

PersonaOS 的前身是 [Miyu](https://github.com/SHORiN-KiWATA/Miyu)（一个终端 AI 助手），感谢 SHORiN-KiWATA 的开源贡献。本项目在其基础上重构为不绑定具体人格的通用平台。

同时致谢以下参考项目：

- [Opencode](https://github.com/anomalyco/opencode) — 公共模型服务
- [Claude Code](https://github.com/anthropics/claude-code) — agent 交互范式
- [NapCatQQ](https://github.com/NapNeko/NapCatQQ) — QQ 协议适配
- [Astrbot](https://github.com/AstrBotDevs/AstrBot) — 插件设计参考

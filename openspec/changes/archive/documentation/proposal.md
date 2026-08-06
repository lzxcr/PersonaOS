## Why

项目当前缺少开发者和用户文档。仅有 README.md 简要介绍和 openspec 内部规格，没有以下关键文档：

1. **架构文档** — 模块拓扑、数据流、进程模型
2. **API/集成文档** — CLI 命令参考、IPC 协议、WebUI WebSocket 事件
3. **配置文档** — config.jsonc 完整字段说明、模型池、插件开关
4. **开发指南** — 新增平台/插件/工具的步骤与范例
5. **路线图** — 当前功能全景 + 未来改进方向

## What Changes

### 1. `docs/ARCHITECTURE.md` — 项目架构

- 进程模型：pos 二进制 / daemon 模式 / IPC 通信
- 模块拓扑图（src/ 模块依赖关系）
- 核心数据流：CLI → IPC → Agent Actor → LLM → 工具循环 → 回复
- 平台流水线：入站 → PlatformAdapter → PlatformTurnContext → turn → 出站
- 记忆系统架构：短期/长期/知识点/联想召回
- Agent 生命周期：compact / overflow / plan mode
- 新增模块说明（media、telegram、qq_official）

### 2. `docs/CLI.md` — CLI 命令参考

- 全局选项
- 所有子命令（带参数、示例）
- Shell 集成（fish/bash/zsh）
- 环境变量

### 3. `docs/CONFIG.md` — 配置指南

- config.jsonc 完整字段树
- 模型供应商与模型池
- 平台配置（QQ/Telegram/QQ Official）
- 插件开关与参数
- TUI 配置界面说明
- JSONC 注释语法

### 4. `docs/DEVELOPMENT.md` — 开发指南

- 新增工具：3 步（写 handler → 注册 → 可选描述文件）
- 新增平台适配器：实现 PlatformAdapter trait
- 新增平台插件：实现 PlatformPlugin trait + 注册
- 工具描述文件格式
- Skill 脚本格式
- 代码风格与约定

### 5. `docs/ROADMAP.md` — 功能全景与路线图

- 当前功能分类清单（终端/WebUI/IM/记忆/工具/Skill/知识库）
- 已实现的 Miyu 功能 vs 待完善
- 建议的改进方向：
  - Phase A: Telegram/QQ Official 从 stub → 完整实现
  - Phase B: TTS/STT provider 实现（OpenAI TTS / Whisper）
  - Phase C: Live2D SDK 集成 + 模型加载
  - Phase D: 平台插件配置 UI（config TUI 中按平台管理）
  - Phase E: 性能优化（编译缓存、token 计数、SQLite WAL）
  - Phase F: 安全性增强（沙箱执行、工具权限细化）
  - Phase G: 多语言/国际化完善

### 6. README.md 更新

- 添加文档索引章节，指向 docs/ 目录

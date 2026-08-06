## Why

将已有的 Miyu 项目（/home/lzx/项目/sanzhou）重构迁移为**全新的 PersonaOS 项目**——一个纯净、通用的本地 AI 人格平台框架。

用户明确指示：
- 在 /home/lzx/项目 新开 PersonaOS 仓库，不保留对旧 Miyu 的任何兼容支持
- 不迁移 Miyu、三舟人格（它们将来重新作为 PersonaOS 的扩展人格接入）
- 清理所有配置、品牌、人格数据，保证项目纯净性与泛用性

## What Changes

### 1. 新仓库结构

```
/home/lzx/项目/PersonaOS/
├── Cargo.toml          # package=persona-os, bin=pos
├── build.rs
├── src/                # 迁移自 sanzhou，去掉全部 Miyu/三舟 硬编码
├── assets/             # 字体/jieba/tiktoken（通用基础设施）
├── web/                # 前端（去品牌化）
├── tests/
└── openspec/
```

### 2. 命名映射（全局替换）

| 旧 (Miyu) | 新 (PersonaOS) |
|---|---|
| 包名 `miyu` | `persona-os` |
| 二进制 `miyu` | `pos` |
| `MiyuPaths` | `PersonaPaths` |
| `~/.miyu` | `~/.pos` |
| `/usr/share/miyu` | `/usr/share/pos` |
| 环境变量 `MIYU_*` | `POS_*` |
| 日志 target `"miyu"` | `"pos"` |
| MCP clientInfo `"miyu"` | `"personaos"` |
| cookie `miyu_session` | `pos_session` |
| User-Agent `miyu/0.1` | `pos/0.3` |
| WebUI 标题 `Miyu` | `PersonaOS` |
| 构建标识 `MIYU_BUILD_ID` | `POS_BUILD_ID` |

### 3. 人格系统纯净化

- **移除** `src/prompts/builtin-miyu.md` 与 `builtin-sanzhou.md`
- **清空** `BUILTIN_PERSONAS` 注册表（保留机制，不保留条目）
- **无默认回退**：注册表为空时，`default_builtin_persona()` 直接返回错误，明确提示「未注册任何内置人格，请先在 BUILTIN_PERSONAS 注册或配置自定义人格」
- 人格机制完整保留：用户可自由添加人格文件、通过 TUI/CLI/WebUI 管理

### 4. 内容清理

删除不迁移的文件/目录：
- `kb/`（Arch Linux 默认知识库）
- `pics/`（Miyu 相关截图与 logo）
- `src/memes/miyu/`（Miyu 表情包）
- `docs/`（Miyu 内部规划文档）
- `todolist.md`
- `openspec/`（旧仓库的规格；PersonaOS 新建自己的）
- `resources/matugen/miyu-theme.css` → 改为 `personaos-theme.css`
- 旧 `.git` 历史（新仓库干净开始）

### 5. 默认知识库策略

默认知识库（Arch Linux ShorinWiki）不迁移。PersonaOS 保留知识库**机制**，但默认不预置任何内容，用户按需导入。

### 6. 保留的框架能力（完整迁移）

- LLM Runtime（多 provider、thinking、缓存、工具调用）
- Agent 循环（会话、压缩、溢出处理、重试）
- 记忆系统（短期/长期日记、知识点、联想）
- 工具系统（天气/汇率/闹钟/算卦/骰子/搜索/生图/文件等，去品牌文案）
- 平台适配（终端 REPL、WebUI、QQ/OneBot）
- Skill 系统、知识库工具、深度研究、诊断工具
- TUI/CLI/WebUI 配置管理

## 不做的事

- ❌ 不做 `~/.miyu` → `~/.pos` 数据迁移（全新开始）
- ❌ 不保留 Miyu/三舟 人格文件（将来以扩展形式重新接入）
- ❌ 不保留旧 git 历史
- ❌ 不修改 API 行为语义（只改名/清理）

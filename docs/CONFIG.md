# 配置指南

PersonaOS 使用 JSONC（带注释的 JSON）配置文件。运行 `pos config` 打开 TUI 编辑，或直接编辑 `~/.pos/config/config.jsonc`。

运行 `pos reload` 热重载配置（无需重启 daemon）。

## 配置文件结构

```jsonc
{
  // ═══ 全局 ═══
  "language": "auto",           // "zh" | "en" | "auto"
  "active_persona": "",         // 空 = 默认内置人格
  "always_allow_tools": false,  // 跳过工具执行确认

  // ═══ 模型供应商 ═══
  "providers": {
    "opencode": {
      "url": "https://api.opencode.ai/v1",
      "api_key": "sk-...",
      "enabled": true
    }
  },

  // ═══ 模型池 ═══
  "text_models": [
    {
      "provider": "opencode",
      "model": "deepseek-v4-flash",
      "is_default": true
    }
  ],
  "multimodal_models": [
    {
      "provider": "opencode",
      "model": "gpt-4o-mini",
      "is_default": true
    }
  ],
  "plan_models": [],       // Plan 模式的文本模型池
  "chat_models": [],       // Chat 模式的文本模型池
  "subagent_tiers": {      // 子代理分 tier 模型池
    "cheap": [],           // 低成本任务
    "balanced": [],        // 均衡
    "strong": []           // 复杂任务
  },

  // ═══ 插件总开关 ═══
  "plugins": {
    "shell": { "enabled": true },
    "web_search": { "enabled": true },
    "memory": { "enabled": true },
    "knowledge_base": { "enabled": true },
    "alarms": { "enabled": true },
    "archlinux": { "enabled": false },
    "gaming": { "enabled": false },
    "divination": { "enabled": false },
    "memes": { "enabled": false },
    "diagnostics": { "enabled": true },
    "deep_research": { "enabled": true },
    "skills": { "enabled": true },
    "scripts": { "enabled": true },
    "subagent": { "enabled": true },
    "planning": { "enabled": true },
    "mcp": { "enabled": false }
  },

  // ═══ 工具加载模式 ═══
  "tools": {
    "loading_mode": "default"   // "default" | "hybrid" | "stub"
  },

  // ═══ MCP 服务器 ═══
  "mcp_servers": [],

  // ═══ 平台 ═══
  "platforms": {
    "command_prefix": "/",     // 平台命令前缀
    "commands": {},            // 自定义命令权限

    // ── QQ (OneBot / NapCat) ──
    "qq": {
      "enabled": false,
      "reverse_ws_port": 8300,
      "access_token": "",
      "admin_users": [],
      "allow_non_admin_host_tools": false,
      "group_intermediate_messages": false,
      "private_intermediate_messages": true,
      "user_identification": true,
      "show_group_name": true,
      "max_reply_chars": 3000,
      "asset_base_url": "",
      "memory": {},
      "private_chats": {
        "whitelist": [],
        "friend_requests_require_private_whitelist": false,
        "allow_non_whitelist": true,
        "non_whitelist_rate_limit": {
          "max_messages": 10,
          "window_seconds": 60
        }
      },
      "group_chats": {
        "whitelist": [],
        "allow_non_whitelist": true,
        "non_whitelist_rate_limit": {
          "max_messages": 5,
          "window_seconds": 60
        }
      },
      "session_limits": {
        "running": 1,
        "queued": 4
      },
      "text_models": null,
      "multimodal_models": null,
      "conversations": [],
      "plugins": {
        "access_manager": { "enabled": true },
        "message_history": { "enabled": true },
        "real_context": {
          "enabled": true,
          "settings": {
            "trigger_mode": "adaptive",
            "cooldown_seconds": 30,
            "max_turns_per_window": 5
          }
        },
        "message_recall": { "enabled": true },
        "meme_collector": { "enabled": false },
        "group_management": { "enabled": true },
        "reply_processor": {
          "enabled": true,
          "settings": {
            "mode": "markdown",
            "threshold": 2000
          }
        }
      }
    },

    // ── Telegram (可选) ──
    "telegram": {
      "enabled": false,
      "bot_token": "",
      "webhook_path": "",
      "admin_users": [],
      "group_intermediate_messages": false,
      "private_intermediate_messages": true,
      "max_reply_chars": 3000,
      "plugins": {}
    },

    // ── QQ 官方机器人 (可选) ──
    "qq_official": {
      "enabled": false,
      "app_id": "",
      "client_secret": "",
      "admin_users": [],
      "group_intermediate_messages": false,
      "private_intermediate_messages": true,
      "max_reply_chars": 3000,
      "plugins": {}
    }
  },

  // ═══ 外观 ═══
  "theme": "graphite",
  "matugen_scheme": "graphite"
}
```

## 关键配置详解

### 模型供应商

支持任何 OpenAI 兼容 API。常见供应商：

| 供应商 | URL 模板 |
|---|---|
| OpenCode | `https://api.opencode.ai/v1` |
| OpenAI | `https://api.openai.com/v1` |
| DeepSeek | `https://api.deepseek.com/v1` |
| 本地 Ollama | `http://localhost:11434/v1` |
| 本地 vLLM | `http://localhost:8000/v1` |

### 模型池

`text_models` 和 `multimodal_models` 支持多模型负载均衡。`is_default: true` 标记默认模型（恰好一个）。

注意事项：
- Plan 模式的 `plan_models` 若不设置则继承 `text_models`
- 子代理 `subagent_tiers` 若不设置则继承 `text_models`
- IM 平台可通过 `platforms.qq.text_models` 覆盖全局池

### 工具加载模式

| 模式 | 说明 |
|---|---|
| `default` | 全部工具常驻 |
| `hybrid` | 常用工具常驻；非常用按需拉取 |
| `stub` | 非常用工具以精简条目可见；LLM 可手动加载 |

**推荐**：日常使用 `stub`，节省 token；开发/调试使用 `default`。

### 平台插件配置

每个 IM 平台有独立的 `plugins` 配置。插件按平台路由——QQ 和 Telegram 的插件配置互不影响。

QQ 群管插件（7 个内置）：
- `access_manager` — 准入控制
- `real_context` — 真人语境引擎（控制何时主动回话）
- `group_management` — 群管工具（禁言/踢人/头衔）
- `message_history` — 消息归档
- `message_recall` — 撤回
- `meme_collector` — 表情包
- `reply_processor` — 回复格式化

### TUI 配置

运行 `pos config` 打开 MD3 主题的终端配置界面（ratatui 渲染），支持键盘导航（↑↓/Enter/Esc）。修改直接写入 `config.jsonc`。

## 配置热重载

修改配置文件后运行 `pos reload`，或通过 WebUI 配置面板保存。以下配置需要重启 daemon 才能生效：
- 模型供应商 URL 变更
- `reverse_ws_port` 变更
- `mcp_servers` 变更

其余配置支持热重载。

## 数据目录

| 路径 | 用途 |
|---|---|
| `~/.pos/config/config.jsonc` | 主配置文件 |
| `~/.pos/config/theme.json` | MD3 主题覆盖 |
| `~/.pos/data/prompts/` | 自定义人格提示词 |
| `~/.pos/data/identities/` | 用户身份文件 |
| `~/.pos/data/persona-avatars/` | 人格头像 |
| `~/.pos/data/skills/` | 已安装 skill |
| `~/.pos/data/scripts/` | 用户脚本 |
| `~/.pos/data/kb/` | 知识库文件 |
| `~/.pos/data/pictures/` | 图片（搜图/生图） |
| `~/.pos/data/live2d/` | Live2D 模型文件（可选） |
| `~/.pos/state/memory.db` | 记忆数据库 |
| `~/.pos/state/conversations.db` | 会话数据库 |
| `~/.pos/state/subagent-checkpoints/` | 子代理恢复检查点 |
| `~/.pos/cache/logs/` | 日志文件 |

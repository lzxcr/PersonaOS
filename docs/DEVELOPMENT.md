# 开发指南

本文档覆盖 PersonaOS 最常见的扩展场景：新增工具、新增平台适配器、新增平台插件。

## 前置条件

- Rust 1.89+
- 能成功运行 `cargo build`
- 理解 [架构文档](./ARCHITECTURE.md) 中的模块拓扑

## 场景一：新增工具

新增一个 AI 可调用的工具，分为 3 步。

### 步骤 1：编写工具处理函数

在 `src/tools/` 下创建新文件，例如 `src/tools/weather.rs`：

```rust
use super::{ToolProgress, ToolRegistry};
use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;

pub fn register(registry: &mut ToolRegistry) {
    registry.register(
        super::ToolSpec::new(
            "get_weather",                    // 工具名（唯一）
            "获取指定城市的天气",              // 描述（会发送给 LLM）
            serde_json::json!({               // JSON Schema 参数
                "type": "object",
                "properties": {
                    "city": {
                        "type": "string",
                        "description": "城市名"
                    }
                },
                "required": ["city"]
            }),
            Arc::new(|args, progress| {
                Box::pin(async move {
                    let city = args["city"].as_str().unwrap_or("未知");
                    // ... 实际的天气查询逻辑 ...
                    let result = format!("{city}：晴，25°C");
                    Ok(Value::String(result))
                })
            }),
        )
        .writes()                           // 权限标记：ReadOnly / Writes / Presentation
        .with_groups(&["utility"]),         // 工具分组
    );
}
```

### 步骤 2：注册到工具表

编辑 `src/tools/mod.rs`，在文件顶部添加模块声明：

```rust
mod weather;
```

然后在 `builtin_registry` 函数中添加注册调用：

```rust
pub fn builtin_registry(config: &AppConfig) -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    // ... 现有注册 ...
    if config.plugins.get("weather").map_or(true, |p| p.enabled) {
        weather::register(&mut registry);
    }
    registry
}
```

同时在 `src/config.rs` 的 `PluginsConfig` 中可选添加开关字段。

### 步骤 3（可选）：添加描述文件

在 `src/tools/descriptions/` 下创建 `get_weather.json`：

```json
{
  "name": "get_weather",
  "display_name": "天气查询",
  "description": "查询指定城市的实时天气信息。",
  "parameters": {
    "type": "object",
    "properties": {
      "city": { "type": "string", "description": "城市名称" }
    },
    "required": ["city"]
  },
  "always_loaded": false,
  "load_policy": "summary",
  "groups": ["utility"]
}
```

描述文件会被编译期 `include_str!` 汇总（`src/tools/tool_descriptions.rs`），用于懒加载和 display_name 覆盖。

## 场景二：新增平台适配器

以 Telegram 为例，实现一个 `PlatformAdapter`。

### 步骤 1：创建适配器文件

创建 `src/platforms/telegram.rs`：

```rust
use crate::platforms::types::{
    OutboundBody, OutboundMessage, OutboundSegment, PlatformAdapter,
    SendReceipt,
};
use anyhow::Result;
use futures_util::future::BoxFuture;

pub struct TelegramAdapter { /* HTTP client, bot token */ }

impl TelegramAdapter {
    pub fn new(bot_token: &str) -> Result<Self> {
        // 初始化 HTTP client，验证 token
    }
}

impl PlatformAdapter for TelegramAdapter {
    fn send<'a>(&'a self, message: OutboundMessage) -> BoxFuture<'a, Result<SendReceipt>> {
        Box::pin(async move {
            // 1. 将 OutboundBody → Telegram API 请求
            // 2. 调用 Telegram sendMessage/sendPhoto 等
            // 3. 返回 SendReceipt
        })
    }

    fn bot_display_name<'a>(&'a self) -> BoxFuture<'a, Result<String>> {
        Box::pin(async { Ok("MyBot".to_string()) })
    }

    // 可选重写：message_info、group_members、set_group_ban 等
}
```

### 步骤 2：注册模块

编辑 `src/platforms/mod.rs`：

```rust
pub(crate) mod telegram;  // 添加此行

// 在 PlatformRuntime 中添加连接管理器（参照 onebot 的 ConnectionRegistry）
```

### 步骤 3：添加配置

编辑 `src/config.rs`，在 `PlatformsConfig` 的 `plugin_config()` 方法中处理 `"telegram"` 路由（已预留）。

### 步骤 4：实现传输层

在适配器中实现 Telegram Bot API 的 getUpdates 轮询或 webhook 接收。入站消息转为 `PlatformInboundEvent`，调用 `run_platform_turn()` 驱动对话。

**PlatformAdapter trait 必选方法**：

| 方法 | 说明 |
|---|---|
| `send(message) → SendReceipt` | 发送消息到平台 |
| `bot_display_name() → String` | 机器人名称 |

**可选重写方法**（默认 bail 或 Unknown）：

| 方法 | 说明 |
|---|---|
| `message_info(id)` | 查询消息详情 |
| `message_images(id)` | 获取消息图片 |
| `group_members()` | 获取群成员列表 |
| `group_member(user_id)` | 获取单个成员信息 |
| `bot_group_role()` | 机器人自身群角色 |
| `set_message_reaction(id, reaction, active)` | 表情回应 |
| `delete_message(id)` | 撤回消息 |
| `set_group_ban(user_id, duration)` | 群禁言 |
| `set_group_kick(user_id)` | 踢人 |
| `set_group_special_title(user_id, title)` | 设置群头衔 |

## 场景三：新增平台插件

平台插件通过 `PlatformPlugin` trait 的 16 个钩子注入行为。

### 步骤 1：实现 PlatformPlugin

创建 `src/platforms/plugins/my_plugin.rs`：

```rust
use crate::platforms::plugins::{PlatformPlugin, PlatformPluginDescriptor};
use crate::platforms::{PlatformTurnContext, OutboundMessage};
use anyhow::Result;
use async_trait::async_trait;

pub struct MyPlugin;

impl MyPlugin {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl PlatformPlugin for MyPlugin {
    fn descriptor(&self) -> PlatformPluginDescriptor {
        PlatformPluginDescriptor {
            id: "my_plugin",
            display_name: "My Plugin",
            priority: 100,
            default_enabled: true,
        }
    }

    async fn before_send(
        &self,
        context: &PlatformTurnContext,
        message: OutboundMessage,
    ) -> OutboundMessage {
        // 修改/替换出站消息
        message
    }

    async fn handle_command(
        &self,
        context: &PlatformTurnContext,
        text: &str,
    ) -> Result<Option<OutboundMessage>> {
        // 处理自定义命令（如 /mycommand）
        Ok(None)
    }

    async fn register_tools(
        &self,
        registry: &mut crate::tools::ToolRegistry,
        context: Arc<PlatformTurnContext>,
    ) {
        // 注册平台特定工具
    }
}
```

### 步骤 2：注册插件

编辑 `src/platforms/plugins/mod.rs`，在 `PlatformPluginRegistry::built_in()` 中添加：

```rust
pub(crate) fn built_in() -> Result<Self> {
    Ok(Self::new(vec![
        // ... 现有插件 ...
        Arc::new(MyPlugin::new()),
    ]))
}
```

### PlatformPlugin 完整钩子列表

| 钩子 | 阶段 | 说明 |
|---|---|---|
| `observe_ingress` | 入站 | 归档原始事件（轻量，不阻塞） |
| `observe_inbound` | 入站 | 观察结构化事件 |
| `decide_trigger` | 入站 | 修改触发决策（是否回复） |
| `accept_followup` | 入站 | 判断是否接受 follow-up 消息 |
| `preempt_inbound` | 入站 | 抢占旧 turn |
| `turn_started` | Turn 开始 | turn 启动通知 |
| `before_turn` | Turn 开始 | 注入系统上下文 |
| `turn_is_superseded` | Turn 中 | 检查是否被覆盖 |
| `after_turn_aborted` | Turn 结束 | turn 中止清理 |
| `after_session_reset` | 会话重置 | 会话清空后 |
| `after_persona_reset` | 人格重置 | 人格变更后 |
| `before_send` | 出站 | 改写/替换出站消息 |
| `after_send` | 出站 | 记录已发送消息 |
| `record_external_bot_message` | 出站 | 记录外部 bot 消息 |
| `handle_command` | 命令 | 处理自定义平台命令 |
| `register_tools` | 初始化 | 注册平台特定工具 |

## 代码风格约定

- 使用 `pub(crate)` 可见性，严格控制公开 API
- `anyhow::Result` 作为通用错误类型
- `tracing` 宏（`info!/warn!/debug!`）记录日志，target 前缀 `pos::`
- 中英双语文本使用 `i18n::text("english", "中文")`
- 异步函数返回 `BoxFuture` 而非 `async fn`（trait 兼容性）
- 大型模块（>5000 行）拆分子文件
- 测试放在同文件底部 `#[cfg(test)] mod tests {}`

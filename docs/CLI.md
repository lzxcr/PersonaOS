# CLI 命令参考

`pos` 是 PersonaOS 的唯一二进制入口。无子命令时进入 REPL；带消息参数时发起一次性对话。

## 全局选项

| 选项 | 说明 |
|---|---|
| `--plan` | 只读 Plan 模式（禁止修改文件/执行命令） |
| `--debug` | 输出调试信息 |
| `--stdout` | 纯文本输出（禁用 TUI 渲染） |
| `--session <id>` | 指定本次对话使用的会话 ID |
| `--help`, `-h` | 帮助 |
| `--version`, `-V` | 版本号 |

隐藏选项（由 shell hook 内部使用）：`--shell-intercept`、`--shell-classify`、`--shell`、`--stdin`、`--clipboard-paste`

## 环境变量

| 变量 | 说明 |
|---|---|
| `POS_HOME` | 覆盖默认 `~/.pos` 数据目录 |
| `POS_LANG` | 强制语言 (`zh` / `en` / `auto`) |
| `POS_LOG` | 日志级别 |
| `POS_DIRECT` | 跳过 daemon IPC，直接本地运行 |

## 子命令

### `pos [MESSAGE]` — 默认模式

```bash
pos                          # 进入 REPL 交互模式
pos "你好"                    # 一次性对话
echo "ls" | pos              # 管道输入
pos --plan "分析这个方案"     # Plan 模式
```

### `pos init`

首次运行自动触发。生成 `~/.pos/config/config.jsonc` 默认配置，创建状态目录。

```bash
pos init
```

### `pos config`

打开 TUI 配置界面，或在终端查看配置路径。

```bash
pos config                    # TUI 配置
pos config paths              # 打印所有路径
pos config validate           # 验证配置文件
```

TUI 支持的配置视图：文本模型、多模态模型、子代理档位池、供应商与模型、插件配置（含详情字段编辑与 API 额度管理）、自定义提示词（人格 CRUD）、接入平台（QQ/Telegram/QQ 官方全量字段编辑）、全局参数设置。中英双语，快捷键 `↑↓` 导航、`Enter` 确认、`空格` 开关、`Esc` 返回。

### `pos web`

启动/访问 WebUI（实际服务由 daemon 承载）。

```bash
pos web                       # 启动 daemon 若未运行，打印 URL
pos web --port 8420           # 指定端口
pos web -p mypassword         # 设置托管密码
```

### `pos daemon`

后台服务生命周期管理。

```bash
pos daemon start              # 启动 daemon（默认端口 8410）
pos daemon stop               # 停止
pos daemon restart            # 重启
pos daemon status             # 运行状态
pos daemon logs -n 50         # 查看最近 50 行日志
pos daemon start --port 8420  # 指定端口启动
```

### `pos reload`

热重载 daemon 配置（无需重启）。

```bash
pos reload
```

### `pos models`

切换当前会话的文本模型。

```bash
pos models                    # 列出可用模型，交互式选择
pos models 2                  # 选择第 2 个模型
pos models opencode/deepseek  # 按 provider/model 切换
pos models default            # 恢复默认模型
```

### `pos list-models`

列出全局文本模型池（非交互）。

```bash
pos list-models
```

### `pos variant`

切换当前会话的思考档位（thinking variant）。

```bash
pos variant                   # 交互式选择
pos variant light             # 轻量思考
pos variant deep              # 深度思考
```

### `pos fish-init` / `pos bash-init` / `pos zsh-init`

安装 shell hook，实现终端无缝集成。

```bash
pos fish-init                 # 安装 fish hook
pos bash-init                 # 安装 bash hook
pos zsh-init                  # 安装 zsh hook
pos remove-shell-hook         # 卸载所有 hook
```

**Fish 集成效果最佳**：完整无缝对话。Bash / Zsh 支持单行对话。

在终端中直接输入自然语言即可对话（前缀触发可配置）。

### `pos history`

查看当前会话历史。

```bash
pos history                   # 最近 20 条
pos history -n 50             # 最近 50 条
pos history --raw             # 原始 JSON 输出
pos history --no-thinking     # 隐藏思考内容
```

### `pos pop <N>`

将最旧的 N 轮对话移出上下文（释放 token）。

```bash
pos pop 5                     # 移除最早 5 轮
```

### `pos memory`

记忆管理。

```bash
pos memory stats              # 记忆统计
pos memory search <query>     # 搜索记忆
pos memory reset              # 清空当前人格记忆
pos memory remember <text>    # 手动记录
```

### `pos reset [all]`

重置当前会话。`pos reset all` 同时清空长期记忆。

```bash
pos reset                     # 清空当前会话
pos reset all                 # 清空会话 + 长期记忆
```

### `pos session` / `pos new` / `pos rename` / `pos archive` / `pos delete`

会话生命周期管理。

```bash
pos new "项目讨论"             # 创建新会话
pos session                   # 列出会话，交互式切换
pos session "项目讨论"         # 切换到指定会话
pos rename "新名称"            # 重命名当前会话
pos archive                   # 归档当前会话
pos delete                    # 删除当前会话
```

### `pos workspace`

绑定会话工作目录（影响文件操作等工具的作用域）。

```bash
pos workspace                 # 查看当前工作目录
pos workspace /path/to/project # 设置工作目录
```

### `pos kb`

知识库管理。

```bash
pos kb                        # TUI 浏览
pos kb list                   # 列出知识库文件
pos kb add <file>             # 导入文件
pos kb remove <name>          # 删除文件
```

### `pos skills`

Skill（可复用子 agent 编排）管理。

```bash
pos skills list               # 列出已安装 skill
pos skills show <name>        # 查看详情
pos skills enable <name>      # 启用
pos skills disable <name>     # 禁用
pos skills remove <name>      # 删除
pos skills stats              # 统计
pos skills prune              # 清理未使用
```

### `pos paths`

显示所有路径。

```bash
pos paths
```

输出示例：
```
配置目录: /home/user/.pos/config
配置文件: /home/user/.pos/config/config.jsonc
数据目录: /home/user/.pos/data
缓存目录: /home/user/.pos/cache
状态目录: /home/user/.pos/state
...
```

### `pos ask <MSG>`（隐藏）

一次性提问（`pos "msg"` 的别名，语义更明确）。

## Shell 集成

安装 hook 后，在终端中直接输入自然语言即可对话。Fish shell 支持完整多行对话；Bash / Zsh 支持单行。

```bash
# 安装后：
$ 今天天气怎么样？
AI: 今天北京晴，15°C...

$ 帮我写个 Python 脚本
AI: [生成代码...]
```

### IPC 协议概述

CLI 与 daemon 通过 Unix domain socket（`~/.pos/runtime/core.sock`）通信，使用 JSON 消息。

主要消息类型：
- `StartTurn` — 发起对话
- `CancelTurn` — 取消进行中的对话
- `ReloadConfig` — 热重载配置
- `StreamEvent` — 流式回复事件

Daemon 在 `runtime/` 目录管理锁文件 `core.lock` 和 `starter.lock`，防止多实例冲突。

## 背景

上一轮交互改进遗留三个问题：
1. 主菜单 emoji 图标（✨🖼⚙🏭🔌👤💬🎛💾）与整体风格不协调
2. QQ 平台编辑模式割裂：Enter 被 `qq_advanced` 拦截，导致基础字段（reverse_ws_port/access_token/admin_users）完全无法编辑；且从第 6 个字段开始进入（edit_field=5）
3. 页面名"IM 平台"表述不佳

## 修复

### 1. 移除主菜单 emoji
主菜单项恢复为纯文本，可保留 `▌` 高亮色条。

### 2. 统一平台编辑模式
- 删除 `qq_advanced` 独立模式，平台编辑统一走 `editing` 模式
- QQ 字段 = 基础 5 字段（enabled/reverse_ws_port/access_token/admin_users/max_reply_chars）+ 高级 9 字段（权限/中间消息/用户识别/会话限流等），**合并为单个全量字段列表**
- 修复 `apply_platform_field` 缺失的 `enabled` 分支（列表空格已能切换，但编辑模式字段 0 目前是死字段）
- 编辑从字段 0 开始（不再跳过）

### 3. 重命名页面
"IM 平台" → 待用户确认的新名称（选项见 proposal 决策）。

## 决策（待用户确认）

- 页面名：候选 ["平台", "消息平台", "接入平台", "聊天平台"]
- QQ 全量字段顺序：基础在前、高级在后

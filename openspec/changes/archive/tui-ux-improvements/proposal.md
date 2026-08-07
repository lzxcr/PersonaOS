## 背景

ratatui TUI 迁移完成后，功能覆盖已完整（6 项 gap-fill 补齐），但交互设计存在 9 个问题（含 2 个真 bug）和视觉简陋（间距紧凑、缺乏视觉层次）。

用户要求：修复 bug → 统一交互模式 → 支持连续编辑 → 翻页导航 → 微调外观，自唤醒仅澄清现状。

## 修复清单

### P0: Bug 修复
1. platforms 切字段 buffer 错位（B1）
2. global 编辑模式 ↑↓/k/j 失效（B2）

### P1: 交互统一
3. bool 字段一律 Space 直接切换（global + platforms）
4. choices 字段一律 Enter 循环选择（带选项列表展示）
5. 校验失败显示红色错误提示

### P2: 连续编辑
6. Enter 应用值后跳到下一字段（不退出编辑模式）
7. 修正 plugins 切字段后 buffer 不清空（改为自动加载当前值）
8. 修正 platforms 编辑模式切字段时 buffer 重载

### P3: 导航增强
9. PageUp/PageDown/Home/End 翻页 + 列表行号提示
10. 统一 vi 键位（j/k/n 在所有列表/编辑模式一致）

### 视觉微调
11. 加大内边距（列表项间加空白行）
12. 选中态增加左侧色条（替代纯文字 ▸ 符号）
13. 状态栏增加当前页/总页数指示
14. 主菜单增加 icon（ASCII 符号前缀）

## 决策

- 不改动业务逻辑（config 读写 / validate）
- 不新增依赖
- 只改 `mod.rs`（渲染+事件）和 `pages/mod.rs`（scroll_offset 活用）

## MODIFIED Requirements

### Requirement: P0 — Bug 修复

#### Scenario: platforms 切字段不串值
- **WHEN** 在 platforms 编辑模式中按 `↓` 切换字段
- **THEN** 编辑缓冲自动重载为新字段的当前值
- **AND** 不会把旧字段的输入误写入新字段

#### Scenario: global 编辑模式 ↑↓ 切换字段
- **WHEN** 在 global 编辑模式中按 `↑` / `↓` / `k` / `j`
- **THEN** 切换到上一个/下一个字段（导航行为）
- **AND** 不把按键字符写入编辑缓冲

### Requirement: P1 — bool/choices 交互统一

#### Scenario: bool 字段空格切换
- **WHEN** 光标在 bool 字段上
- **AND** 编辑模式激活
- **THEN** 按 `Space` 直接翻转布尔值
- **AND** 不需要手打 "true"/"false"

#### Scenario: choices 字段 Enter 选择
- **WHEN** 光标在 choices 字段上
- **AND** 编辑模式激活
- **THEN** 按 `Enter` 循环切换选项（前进）
- **AND** 按 `←` 反向循环（后退）
- **AND** 选项列表显示在提示行
- **AND** 当前选项高亮标识

### Requirement: P2 — 连续编辑

#### Scenario: Enter 应用值后跳下一字段
- **WHEN** 在编辑模式中按 `Enter` 应用字段值
- **THEN** 自动跳转到下一个字段
- **AND** 不退出编辑模式
- **AND** 编辑缓冲预载新字段的当前值

### Requirement: P3 — 导航增强

#### Scenario: 翻页导航
- **WHEN** 在列表页按 `PageUp` / `PageDown` / `Home` / `End`
- **THEN** 跳转到相应位置
- **AND** 列表行号提示显示"第 X/N 行"

### Requirement: 视觉微调

#### Scenario: 选中态色条
- **WHEN** 列表项被高亮
- **THEN** 左侧出现彩色竖条 + 文字高亮
- **AND** 中文 ▸ 符号替换为 Unicode 箭头

#### Scenario: 状态栏增强
- **WHEN** 在任意页面
- **THEN** 状态栏显示"页面名 | 第 X/N 项 | 快捷键提示"

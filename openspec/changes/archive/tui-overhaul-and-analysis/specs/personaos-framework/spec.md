## ADDED Requirements

### Requirement: 上游跟踪
`.gitignore` SHALL 包含 `Miyu-upstream/` 条目，用于本地克隆 Miyu 上游仓库跟踪。

### Requirement: TUI 现代化
TUI 配置界面 SHALL 使用 ratatui 渲染，支持颜色主题、圆角边框、鼠标交互。

#### Scenario: 视觉一致性
- **WHEN** 运行 `pos config`
- **THEN** 界面使用一致的配色方案
- **AND** 盒子使用圆角边框
- **AND** 选中项有高亮颜色

#### Scenario: 鼠标支持
- **WHEN** 在支持鼠标的终端中运行
- **THEN** 可用鼠标点击菜单项和按钮

### Requirement: TUI 代码组织
`src/config_tui.rs` SHALL 拆分为模块目录 `src/config_tui/`，按功能分离文件。

### Requirement: 零编译警告
`cargo check` SHALL 产出零警告（维持现有标准）。

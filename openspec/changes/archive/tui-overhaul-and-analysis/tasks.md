## 1. 分析与追踪
- [ ] 1.1 .gitignore 添加 `Miyu-upstream/` 条目
- [ ] 1.2 确认无编译错误（`cargo check` 零警告）

## 2. TUI 重写（核心）
- [ ] 2.1 添加 ratatui 依赖到 Cargo.toml
- [ ] 2.2 创建 `src/config_tui/` 模块目录结构
- [ ] 2.3 实现颜色主题系统（MD3 palette, dark/light）
- [ ] 2.4 重写主菜单（List 组件 + Block 圆角边框）
- [ ] 2.5 重写供应商管理页面（Table + 内联编辑）
- [ ] 2.6 重写模型配置页面（Tabs + 选择列表）
- [ ] 2.7 重写插件配置页面
- [ ] 2.8 重写 IM 平台配置页面
- [ ] 2.9 添加状态栏（当前页 + 快捷键提示）
- [ ] 2.10 添加鼠标点击支持
- [ ] 2.11 删除旧 `config_tui.rs`（迁移完成后）

## 3. 验证
- [ ] 3.1 `cargo build` 通过且零警告
- [ ] 3.2 `cargo test` 回归通过
- [ ] 3.3 手动验证 TUI 各页面渲染正常

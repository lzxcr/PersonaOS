## 1. Miyu 上游差异分析

最新 Miyu 上游提交 `2270aac`（2026-08-06）仅修改了 `todolist.md`（移除 2 项已完成的条目），无代码变更。

```
2270aac docs: todolist 移除已完成/已证实无问题的两项
```

**结论：无需整合。** Miyu 自 PersonaOS 分叉以来无功能代码更新。

`.gitignore` 需添加 `Miyu-upstream/` 条目以便后续跟踪。

## 2. 编译状态

当前 PersonaOS `cargo check` 零错误零警告。`sanzhou/` 目录目标文件只读，无法在此环境构建验证。

## 3. TUI 配置界面问题

### 现状

`src/config_tui.rs`（8400 行）使用**裸 crossterm**直接绘制：

| 方面 | 现状 | 问题 |
|---|---|---|
| 渲染引擎 | 裸 crossterm（MoveTo/Print/Attribute） | 无布局系统，手动坐标计算 |
| 色彩 | 无颜色，仅 Bold/Reverse/Dim | 缺乏视觉层次 |
| 盒子 | ASCII 线条 `┌─┐│└┘` | 单调，无圆角/阴影 |
| 结构 | 8400 行单文件 | 难以维护 |
| 交互 | 纯键盘（↑↓ Enter Esc） | 无鼠标支持、无滚动条指示 |
| 全屏刷新 | 每帧 `Clear(All)` | 闪烁/不流畅 |
| 字段编辑 | 内联编辑器 | 功能可用但交互感粗糙 |

### 优化方案

- 引入 **ratatui** 替代裸 crossterm（项目已依赖的生态系统）
- 添加**颜色主题**（MD3 配色、亮暗切换）
- 盒子 → **ratatui Block** 带圆角边框和标题
- 列表 → **ratatui List** 带高亮样式
- 表单 → **ratatui Paragraph/Table** + 滚动条
- 添加**鼠标点击**支持（crossterm mouse events）
- 拆分文件 → `config_tui/main.rs` + `config_tui/screens/*.rs`
- 添加**状态栏**显示当前页面/快捷键提示
- 全屏重绘 → 差异更新（减少闪烁）

## 4. 整体设计问题分析

### 4.1 代码组织

| 文件 | 行数 | 建议 |
|---|---|---|
| `config_tui.rs` | 8400 | 拆分：main/screens/form/editors/widgets |
| `onebot.rs` | 9000 | 拆分：connection/handler/parser/adapter |
| `agent/mod.rs` | 6800+ | 已分模块，可进一步精简主入口 |
| `web.rs` | 10900 | 拆分：server/ws/handlers/api |
| `config.rs` | 7900 | 拆分：types/migration/validation |

### 4.2 UI/UX 问题

| 问题 | 影响 |
|---|---|
| TUI 无颜色/样式 | 视觉层次缺失，不现代 |
| WebUI 主题单一 | 虽然支持 matugen 但缺少丰富的内置主题 |
| WebUI 无响应式布局优化 | 移动端体验一般 |
| 缺少加载/进度动画 | 交互反馈不足 |
| TUI 编辑字段无光标闪烁 | 编辑体验差 |

### 4.3 架构问题

| 问题 | 建议 |
|---|---|
| 平台插件配置仍绑定 `platforms.qq.plugins` | 已在 Phase 2 部分解耦，config_tui 需同步更新 |
| media 模块无 provider 实现 | 可后续按需添加 |
| 测试覆盖率偏低 | 大量功能通过集成测试，缺少单元测试 |

## 5. 建议优先级

| 优先级 | 项目 | 工作量 |
|---|---|---|
| P0 | TUI 用 ratatui 重写 + 颜色主题 | 大 (1-2 周) |
| P1 | .gitignore 添加 Miyu-upstream | 小 |
| P1 | 大文件拆分 | 中 (持续) |
| P2 | WebUI 主题增强 | 中 |
| P2 | 鼠标支持 | 中 |
| P3 | 测试覆盖 | 持续 |

## 不做的

- 不在本轮实现 Miyu 功能集成（上游无变更）
- 不重写 WebUI（范围过大）
- 不修改核心 agent/平台逻辑

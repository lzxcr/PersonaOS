## 背景

接入平台字段总览/编辑模式方向键持续失效（四轮修复未解决）。

## 根因

分离 `state`（实体列表）+ `field_state`（字段列表）两套 ListState 方案，与 ratatui 的 `render_stateful_widget` 内部 offset/selected 同步行为不兼容——高亮位置和实际选中行不一致，方向键"看似无响应"。

而 **global 页面（内联编辑）只用一套 `state`** 直接驱动渲染——工作正常。

## 修复

**照搬 global 交互模型**：viewing/editing 时 `state` 临时用作字段位置，Esc 回列表时恢复原平台索引。

### 状态约定

```
list mode:      state = 平台列表位置（只读）
viewing/editing: state = 字段总览/编辑位置（读写）
Esc → list:     恢复 state 为保存的平台位置
```

### 方案（三页统一）

1. 进入 viewing 前保存实体索引到 `saved_entity: usize`
2. viewing/editing 中 ↑↓/Enter/←→ 直接操作 `state.select()`
3. draw：渲染 `&mut self.X.state`（不做分离）
4. Esc 从 viewing → 列表：`state.select(Some(saved_entity))`
5. Esc 从 editing → viewing：保持 state 不变（位置回总览）

### 简化实施

删除三页的 `field_state` 字段，改为 `saved_entity: Option<usize>`（进入 viewing 时保存实体索引）。draw 和 handle 全部操作 `state`，完全对标 global 页面。

## 背景

接入平台字段总览/编辑模式下方向键失效（"又"复现）。

## 根因

draw 每帧新建局部 `ListState`（`ListState::default()` + `select(edit_field)`）渲染字段列表 → **offset 每帧从 0 重置**，List 不跟随 `edit_field` 滚动 → 高亮移动到可视区外，方向键看似失效。

且 platforms/providers/plugins 三页各自手写相同的 viewing/editing 逻辑（重复代码，各起炉灶），是 bug 温床。

## 修复

### 1. 持久化 field_state
- 三个页面 struct 加 `field_state: ListState`（持久，ratatui 自动维护 offset）
- draw：渲染 `&mut self.X.field_state`，不再新建
- handle：↑↓/PgUp/Home/End 修改 `edit_field` 并同步 `field_state.select(Some(edit_field))`

### 2. 通用字段导航辅助（pages/mod.rs）
新增 `move_field_index(code, current, count) -> Option<usize>`：
- 统一处理 ↑↓/jk/↑↓/PgUp/PgDn/Home/End
- 三页 handle 的 viewing/editing 导航统一调用，消除重复

### 3. 状态约定
- `self.X.state` — 实体列表位置（永不被字段模式修改）
- `self.X.edit_field` + `self.X.field_state` — 字段总览/编辑位置（同步）
- Enter 进入 editing：edit_buffer = field_value(edit_field)
- Enter 应用/跳下一字段：edit_field+1 并同步 field_state

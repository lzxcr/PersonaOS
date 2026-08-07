## 背景

接入平台（platforms）等页面进入字段总览/编辑模式后，方向键切换的却是**平台列表位置**而非字段条目。

## 根因

`selected`（实体索引：平台/供应商/插件）从 `self.X.state.selected()` 读取，但总览/编辑模式渲染时又调用 `self.X.state.select(Some(field_row))` 把 state 改成字段位置 → 上下键在改字段位置的同时，`selected` 也变了 → 显示/取数的实体错乱。

## 修复

**状态分离原则**：
- `self.X.state` — 只跟踪**实体列表**位置（平台/供应商/插件），总览/编辑模式下永不被修改
- `edit_field` — 跟踪**字段总览/编辑**位置
- 总览/编辑渲染用**局部 `ListState`**（`let mut field_state = ListState::default(); field_state.select(Some(edit_field));`），不写回 `self.X.state`

## 实施范围

1. platforms：draw（局部 field_state）+ handle（viewing/editing 导航改 edit_field，不再动 state）
2. providers：同上
3. plugins：同上
4. 顺带修复编译警告（unused variable 等）

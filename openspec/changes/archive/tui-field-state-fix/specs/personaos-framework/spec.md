## MODIFIED Requirements

### Requirement: 字段总览/编辑状态隔离

#### Scenario: 方向键切换字段而非实体
- **WHEN** 进入字段总览（viewing）或编辑（editing）模式
- **THEN** 按 ↑↓ 切换的是字段条目
- **AND** 实体（平台/供应商/插件）保持选中不变
- **AND** 页面标题/取数仍指向原实体

#### Scenario: 渲染不污染实体状态
- **WHEN** 渲染字段列表
- **THEN** 使用局部 ListState（edit_field 位置）
- **AND** `self.X.state` 不被修改，实体列表位置保持

#### Scenario: 编译零警告
- **WHEN** 运行 cargo check
- **THEN** 无 error 无 warning

## MODIFIED Requirements

### Requirement: 平台配置页

#### Scenario: 页面改名
- **WHEN** 打开平台配置页
- **THEN** 页面标题显示「接入平台」
- **AND** 相关字符串（主菜单项、screen_title）同步更新

#### Scenario: QQ 全量字段编辑
- **WHEN** 在平台列表对 QQ 按 Enter
- **THEN** 进入单列表编辑模式，字段 = 基础 5 项 + 高级 9 项（共 14 项）
- **AND** 字段从第 1 项开始
- **AND** Enter 应用后跳到下一字段（连续编辑）
- **AND** enabled（字段 0）可编辑（apply_platform_field 补全）

#### Scenario: 移除主菜单 emoji
- **WHEN** 渲染主菜单
- **THEN** 菜单项无 emoji 前缀，保留 `▌` 高亮色条

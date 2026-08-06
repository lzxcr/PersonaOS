## ADDED Requirements

### Requirement: 架构文档
仓库 SHALL 包含 `docs/ARCHITECTURE.md`，描述进程模型、模块拓扑、核心数据流。

#### Scenario: 文档可读
- **WHEN** 新开发者阅读 ARCHITECTURE.md
- **THEN** 可理解 pos 二进制与 daemon 的关系
- **AND** 可定位到各功能的源文件

### Requirement: CLI 参考
仓库 SHALL 包含 `docs/CLI.md`，列出所有子命令、参数、示例。

### Requirement: 配置指南
仓库 SHALL 包含 `docs/CONFIG.md`，完整描述 config.jsonc 字段树。

### Requirement: 开发指南
仓库 SHALL 包含 `docs/DEVELOPMENT.md`，覆盖新增工具/平台/插件的开发步骤。

### Requirement: 路线图
仓库 SHALL 包含 `docs/ROADMAP.md`，列出当前功能全景和未来改进方向。

### Requirement: README 索引
README SHALL 包含指向 docs/ 目录的文档索引章节。

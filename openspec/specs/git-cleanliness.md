## Requirements

### Requirement: 构建产物不入库
仓库 SHALL 不包含 `target/` 等编译产物。

#### Scenario: 提交内容
- **WHEN** 列出 git 跟踪文件
- **THEN** 不含任何 `target/` 路径
- **AND** 不含 `.o`/`.rlib`/`.rmeta`/`.bin` 等编译产物

### Requirement: .gitignore 存在
仓库根 SHALL 有 `.gitignore`，排除构建产物与本地文件。

#### Scenario: 忽略规则
- **WHEN** 运行 `git status`
- **THEN** `target/` 被忽略不显示
- **AND** 源码与资源正常跟踪

### Requirement: 仓库体积合理
.git 对象库 SHALL 显著小于构建产物大小。

#### Scenario: 体积检查
- **WHEN** 执行 `git count-objects -vH`
- **THEN** size-pack 为几十 MB 级别（仅源码+资源）

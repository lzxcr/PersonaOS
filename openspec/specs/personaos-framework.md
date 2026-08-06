# PersonaOS 框架

## Requirements

### Requirement: 项目身份
项目包名 SHALL 为 `persona-os`，默认二进制名 SHALL 为 `pos`。

#### Scenario: 构建产物
- **WHEN** 执行 `cargo build`
- **THEN** 生成 `pos` 二进制
- **AND** Cargo.lock 中包名为 `persona-os`

### Requirement: 路径基础设施命名
路径基础设施类型 SHALL 命名为 `PersonaPaths`（替代 `MiyuPaths`）。

#### Scenario: 类型引用
- **WHEN** 代码引用路径基础设施
- **THEN** 使用 `PersonaPaths` 类型
- **AND** 项目中不存在 `MiyuPaths` 标识符

### Requirement: 数据与安装目录
用户数据目录 SHALL 为 `~/.pos`，系统安装资源目录 SHALL 为 `/usr/share/pos`。

#### Scenario: 目录解析
- **WHEN** `PersonaPaths::new()` 解析用户目录
- **THEN** 返回 `~/.pos`（非 `~/.miyu`）
- **AND** 系统资源路径使用 `/usr/share/pos`

### Requirement: 环境变量前缀
环境变量 SHALL 使用 `POS_` 前缀。

#### Scenario: 环境变量命名
- **WHEN** 构建或运行时读取环境变量
- **THEN** 使用 `POS_BUILD_ID` / `POS_HOME` / `POS_LANG` / `POS_LOG` / `POS_DIRECT` 等
- **AND** 不存在 `MIYU_*` 前缀环境变量

### Requirement: 日志标识
日志 target SHALL 使用 `pos`。

#### Scenario: 日志输出
- **WHEN** 系统记录日志
- **THEN** target 为 `pos` / `pos::qq` 等
- **AND** 不存在 `miyu` 日志 target

### Requirement: 内置人格注册表为空
`BUILTIN_PERSONAS` 注册表 SHALL 保留机制但默认不含任何人格条目。

#### Scenario: 注册表为空
- **WHEN** 系统启动且注册表为空
- **THEN** `default_builtin_persona()` 返回错误，提示「未注册任何内置人格」
- **AND** 用户必须配置自定义人格或注册内置人格后才能使用

#### Scenario: 无预置人格文件
- **WHEN** 查看 `src/prompts/`
- **THEN** 不存在 `builtin-miyu.md` / `builtin-sanzhou.md`
- **AND** 不存在任何预置人格提示词（无中性默认助手）

### Requirement: 品牌纯净
代码、前端、文档 SHALL 不含 Miyu / 三舟 品牌标识（人格提示词文件中的角色名除外，但本仓库不迁移它们）。

#### Scenario: 品牌检查
- **WHEN** 全仓库搜索 `Miyu` / `三舟`
- **THEN** 仅出现在 git 历史、注释说明迁移来源处
- **AND** 不出现为当前产品身份

### Requirement: 默认知识库为空
默认知识库 SHALL 不预置任何内容，用户按需导入。

#### Scenario: 首次初始化
- **WHEN** 首次运行初始化
- **THEN** 不拉取 Arch Linux ShorinWiki
- **AND** 知识库机制可用但为空

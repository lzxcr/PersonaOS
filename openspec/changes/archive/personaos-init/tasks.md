## 1. 新仓库骨架
- [ ] 1.1 创建 PersonaOS 目录结构
- [ ] 1.2 Cargo.toml：package=persona-os, bin=pos
- [ ] 1.3 git init（干净历史）

## 2. 源码迁移（sanzhou → PersonaOS）
- [ ] 2.1 复制 src/ 全部源码（排除 prompts/builtin-*.md、memes/miyu/）
- [ ] 2.2 批量替换命名：MiyuPaths→PersonaPaths, miyu→pos, MIYU_→POS_, ~/.miyu→~/.pos, /usr/share/miyu→/usr/share/pos, miyu_session→pos_session, mcp "miyu"→"personaos"
- [ ] 2.3 prompts.rs：清空 BUILTIN_PERSONAS；`default_builtin_persona()` 在注册表为空时返回错误
- [ ] 2.4 清理 PersonaOS 默认知识库初始化（不预置 ShorinWiki）
- [ ] 2.5 清理 QQ 默认触发词 "Miyu" → 空或 "PersonaOS"

## 3. 资源迁移
- [ ] 3.1 assets/（字体/jieba/tiktoken）迁移
- [ ] 3.2 web/ 前端迁移 + 去品牌化（title、登录页、默认助手名）
- [ ] 3.3 新建 PersonaOS logo/壁纸占位（或去除引用）
- [ ] 3.4 matugen 主题改 personaos-theme.css

## 4. 文档
- [ ] 4.1 重写 README.md（PersonaOS 定位）
- [ ] 4.2 删除 docs/、todolist.md、旧 openspec/
- [ ] 4.3 更新 Cargo.toml description

## 5. 验证
- [ ] 5.1 cargo check 编译通过
- [ ] 5.2 测试通过（不含预存在失败）
- [ ] 5.3 全仓库 grep 确认无 Miyu/三舟 品牌残留

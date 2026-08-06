## Why

首次提交误将 `target/` 构建产物（7902 个文件，~3.1 GiB）纳入 git，导致仓库 .git 膨胀到 3.2G。需要清理 git 历史，保证提交干净。

## What Changes

1. 重建 git 历史（唯一 commit，直接重写）：删除 .git → 重新 init
2. 添加 `.gitignore`：排除 `target/`、`*.log` 等构建/运行时产物
3. 保留 `assets/`（38M 运行时资源：字体/jieba/tiktoken，正常内容）
4. 重新提交，验证 .git 大小

## 不做

- ❌ 不删除 assets/ 运行时资源
- ❌ 不改动任何源码

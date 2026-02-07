# 贡献指南

感谢你对 rs-ali-oss 项目的关注！欢迎提交 Issue 和 Pull Request。

## 开发环境搭建

### 前置要求

- **Rust**：Edition 2024，stable 工具链
- **Git**：用于版本控制

### 初始化

```bash
git clone git@github.com:infinitete/rs-ali-oss.git
cd rs-ali-oss
./setup.sh   # 配置 pre-commit hook
```

`setup.sh` 会启用 pre-commit hook，在每次提交前自动运行 `cargo fmt --check`、`cargo clippy -- -D warnings` 和 `cargo test`。

### 常用命令

```bash
cargo build                          # 构建
cargo test                           # 运行所有测试
cargo test test_name                 # 运行匹配名称的测试
cargo clippy -- -D warnings          # Lint 检查
cargo fmt                            # 格式化代码
cargo doc --no-deps --open           # 构建并打开文档

# 完整 CI 检查（提交前建议运行）
cargo fmt -- --check && cargo clippy -- -D warnings && cargo test
```

## 代码规范

### 格式化

- 使用 `rustfmt` 默认配置，不添加 `rustfmt.toml`
- 最大行宽 100 字符
- 多行结构使用尾逗号

### 命名约定

| 项目 | 约定 | 示例 |
|------|------|------|
| 模块 | snake_case | `object_ops` |
| 类型 / Trait | PascalCase | `OssClient`, `BucketInfo` |
| 函数 / 方法 | snake_case | `put_object` |
| 常量 | SCREAMING_SNAKE_CASE | `DEFAULT_ENDPOINT` |
| Builder 方法 | snake_case，无 `set_` 前缀 | `.region("cn-hangzhou")` |

### 导入顺序

按以下顺序分组，组间空行分隔：

1. `std` / `core` / `alloc`
2. 外部 crate（来自 `Cargo.toml`）
3. crate 内部（`crate::`、`super::`、`self::`）

```rust
use std::collections::HashMap;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::OssError;
```

### 错误处理

- 库代码中**禁止**使用 `.unwrap()` 或 `.expect()`（测试代码除外）
- 使用 `?` 操作符传播错误
- 不要使用 `as any`、`@ts-ignore` 等类型抑制手段

### 安全性

- 禁止使用 `unsafe` 块，除非绝对必要且有充分文档说明
- 凭证（AccessKey ID/Secret）**绝不能**出现在日志或错误信息中

### 文档

- 所有公开项必须有 `///` 文档注释
- 关键 API 的文档注释中应包含使用示例

## 提交规范

使用 [Conventional Commits](https://www.conventionalcommits.org/) 格式：

```
<emoji> <type>[scope]: <description>
```

### 类型与 Emoji

| 类型 | Emoji | 用途 |
|------|-------|------|
| `feat` | ✨ | 新功能 |
| `fix` | 🐛 | Bug 修复 |
| `docs` | 📝 | 文档更新 |
| `style` | 🎨 | 代码格式调整 |
| `refactor` | ♻️ | 重构（不修复 bug 也不添加功能） |
| `perf` | ⚡️ | 性能优化 |
| `test` | ✅ | 添加或修改测试 |
| `chore` | 🔧 | 构建流程或辅助工具变更 |

### 示例

```
✨ feat(object): 添加 PutSymlink 操作
🐛 fix(auth): 修复 V4 签名中特殊字符编码问题
📝 docs: 更新 API 覆盖率说明
```

### 原则

- 每次提交只包含一个逻辑变更
- 使用中文撰写提交信息
- 主题行不超过 50 个字符

## Pull Request 流程

1. **Fork** 本仓库并创建功能分支
2. 在分支上进行开发，确保遵循上述代码规范
3. 提交前运行完整 CI 检查：
   ```bash
   cargo fmt -- --check && cargo clippy -- -D warnings && cargo test
   ```
4. 确认以下检查项：
   - [ ] `cargo fmt -- --check` 通过
   - [ ] `cargo clippy -- -D warnings` 通过
   - [ ] `cargo test` 通过
   - [ ] 非测试代码中没有新增 `.unwrap()`
   - [ ] 公开 API 有文档注释
5. 提交 Pull Request，描述清楚变更内容和目的

## 项目结构

```
src/
├── lib.rs           # crate 入口，公开 re-export
├── client.rs        # OssClient、重试逻辑、URL 构建
├── config.rs        # ClientBuilder、Config、Credentials
├── error.rs         # OssError 枚举、Result 类型别名
├── credential.rs    # CredentialProvider trait 及实现
├── crc64.rs         # CRC64-ECMA 校验和
├── progress.rs      # ProgressListener trait
├── encoding.rs      # URI/Query 百分号编码集（crate 内部）
├── middleware.rs     # 请求拦截器链
├── auth/            # V4 签名算法
├── ops/             # 操作实现（object、bucket、multipart、presign、transfer、paginator）
└── types/           # 请求/响应类型定义
tests/               # 集成测试
```

## 需要帮助？

如果你不确定某个改动是否合适，欢迎先开一个 Issue 讨论。

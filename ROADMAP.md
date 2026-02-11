# 开发路线图

本文档描述 rs-ali-oss SDK 的功能演进计划，按里程碑版本划分阶段。

> ✅ 已完成 · 🚧 开发中 · 📋 计划中 · 💡 远期考虑

## 当前进度

**v0.1.x (当前版本)**: 核心数据面 + 部分存储桶管理功能已实现

- ✅ 对象操作（14 个 API）：~74% 覆盖率
- ✅ 存储桶基础操作（5 个 API）：100% 覆盖率
- 🚧 存储桶管理/策略（21/~40 个 API）：~52% 覆盖率
  - ✅ ACL、CORS、Referer、Policy
  - ✅ Lifecycle、Versioning
  - ✅ Encryption、Logging
  - 📋 Website、Replication、Inventory、Tags、WORM、TransferAcceleration、RequestPayment 等
- ✅ 分片上传（6 个 API）：~86% 覆盖率
- ✅ 预签名 URL（2 个 API）：100% 覆盖率

---

---

## v0.1.0 — 核心数据面 ✅ (已发布)

首个发布版本，覆盖日常文件存取的核心操作。

| 功能 | 状态 |
|------|------|
| 对象操作（14 个 API：CRUD、复制、追加、归档恢复、ACL、标签） | ✅ |
| 存储桶基础操作（5 个 API：创建、删除、列举、信息、位置） | ✅ |
| 分片上传（6 个 API：完整生命周期管理） | ✅ |
| 预签名 URL（GET / PUT） | ✅ |
| TransferManager（自动分片上传 + CRC64 + 进度回调） | ✅ |
| 自动分页器（ListObjectsV2 / ListBuckets） | ✅ |
| V4 签名认证 | ✅ |
| 凭证管理（Static / Environment / Chain / Caching） | ✅ |
| STS 临时凭证 | ✅ |
| 自动重试（指数退避 + 抖动） | ✅ |
| 请求拦截器中间件 | ✅ |
| CRC64-ECMA 校验 | ✅ |
| 凭证安全（zeroize / Debug 脱敏 / HTTPS 强制） | ✅ |

---

## v0.2.0 — 存储桶策略与对象增强 🚧

**目标**：补全生产环境最常用的存储桶管理 API 和缺失的对象操作。

### Bucket 访问控制与策略

| 功能 | API | 优先级 |
|------|-----|--------|
| 存储桶 ACL | `PutBucketAcl`、`GetBucketAcl` | 🔴 高 |
| 跨域规则 (CORS) | `PutBucketCors`、`GetBucketCors`、`DeleteBucketCors` | 🔴 高 |
| 防盗链 | `PutBucketReferer`、`GetBucketReferer` | 🟡 中 |
| 授权策略 | `PutBucketPolicy`、`GetBucketPolicy`、`DeleteBucketPolicy` | 🟡 中 |

✅ 已完成：ACL、CORS、Referer、Policy

### 对象操作补全

| 功能 | API | 优先级 |
|------|-----|--------|
| 分片拷贝 | `UploadPartCopy` | 🔴 高 |
| 软链接 | `PutSymlink`、`GetSymlink` | 🟡 中 |
| 轻量级元数据 | `GetObjectMeta` | 🟢 低 |

### 高级工具增强

| 功能 | 说明 | 优先级 |
|------|------|--------|
| TransferManager 下载 | 大文件分片并发下载 | 🔴 高 |
| TransferManager 服务端拷贝 | 基于 `UploadPartCopy` 的大文件服务端复制 | 🔴 高 |
| 断点续传 | 上传/下载失败后恢复 | 🟡 中 |

---

## v0.3.0 — 生命周期与版本控制 🚧

**目标**：支持数据生命周期管理和版本化存储。

### 生命周期管理

| 功能 | API | 优先级 |
|------|-----|--------|
| 生命周期规则 | `PutBucketLifecycle`、`GetBucketLifecycle`、`DeleteBucketLifecycle` | 🔴 高 |

### 版本控制

| 功能 | API | 优先级 |
|------|-----|--------|
| 版本控制开关 | `PutBucketVersioning`、`GetBucketVersioning` | 🔴 高 |
| 版本列举 | `ListObjectVersions` | 🔴 高 |
| 版本化对象操作 | 现有对象 API 添加 `versionId` 参数支持 | 🔴 高 |

✅ 已完成：Lifecycle、Versioning (开关)

### 存储桶标签与统计

| 功能 | API | 优先级 |
|------|-----|--------|
| 存储桶标签 | `PutBucketTags`、`GetBucketTags`、`DeleteBucketTags` | 🟡 中 |
| 存储桶统计 | `GetBucketStat` | 🟢 低 |

---

## v0.4.0 — 安全与加密 🚧

**目标**：完善数据安全能力。

### 服务端加密

| 功能 | API | 优先级 |
|------|-----|--------|
| 存储桶默认加密 | `PutBucketEncryption`、`GetBucketEncryption`、`DeleteBucketEncryption` | 🔴 高 |
| 对象级加密 | 请求中指定 SSE-OSS / SSE-KMS / SSE-C 头 | 🔴 高 |

✅ 已完成：服务端加密（存储桶级）

### 合规保留 (WORM)

| 功能 | API | 优先级 |
|------|-----|--------|
| WORM 策略 | `InitiateBucketWorm`、`AbortBucketWorm`、`CompleteBucketWorm`、`ExtendBucketWorm`、`GetBucketWorm` | 🟡 中 |

### 请求者付费

| 功能 | API | 优先级 |
|------|-----|--------|
| 请求者付费 | `PutBucketRequestPayment`、`GetBucketRequestPayment` | 🟢 低 |

---

## v0.5.0 — 运维与监控 🚧

**目标**：支持日志、站点托管和跨区域复制等运维场景。

### 日志管理

| 功能 | API | 优先级 |
|------|-----|--------|
| 访问日志 | `PutBucketLogging`、`GetBucketLogging`、`DeleteBucketLogging` | 🟡 中 |

✅ 已完成：访问日志

### 静态网站托管

| 功能 | API | 优先级 |
|------|-----|--------|
| 静态网站 | `PutBucketWebsite`、`GetBucketWebsite`、`DeleteBucketWebsite` | 🟡 中 |

### 跨区域复制

| 功能 | API | 优先级 |
|------|-----|--------|
| 复制规则 | `PutBucketReplication`、`GetBucketReplication`、`DeleteBucketReplication` | 🟡 中 |
| 复制进度 | `GetBucketReplicationProgress` | 🟡 中 |
| 复制位置 | `GetBucketReplicationLocation` | 🟢 低 |

### 清单

| 功能 | API | 优先级 |
|------|-----|--------|
| 存储桶清单 | `PutBucketInventory`、`GetBucketInventory`、`ListBucketInventory`、`DeleteBucketInventory` | 🟢 低 |

### 传输加速

| 功能 | API | 优先级 |
|------|-----|--------|
| 传输加速 | `PutBucketTransferAcceleration`、`GetBucketTransferAcceleration` | 🟢 低 |

---

## v1.0.0 — 稳定版 💡

**目标**：API 稳定化，达到生产级 1.0 标准。

### API 稳定化

| 任务 | 说明 |
|------|------|
| 公开 API 审查 | 全面审查并冻结公开 API 接口 |
| 错误类型精化 | 细化错误枚举，确保每种失败场景有明确的错误变体 |
| Builder 模式统一 | 所有请求构建器风格一致化 |
| 文档完善 | 每个公开 API 都有完整的文档和示例 |

### 质量与性能

| 任务 | 说明 |
|------|------|
| 集成测试覆盖 | 真实 OSS 环境端到端测试（可选 CI） |
| 性能基准 | 上传/下载吞吐量基准测试 |
| 连接池优化 | 长连接复用、DNS 缓存策略 |
| 内存优化 | 大文件流式处理零拷贝优化 |

### 生态集成

| 任务 | 说明 |
|------|------|
| `tokio` / `async-std` 兼容 | 可选的运行时无关设计 |
| `tracing` 集成 | 结构化日志和链路追踪 |
| `tower` 中间件兼容 | 与 Tower 生态的互操作 |

---

## 远期规划 💡

以下功能根据社区需求和优先级择机实现。

| 功能 | 说明 |
|------|------|
| `SelectObject` | SQL 查询 OSS 中的 CSV/JSON 数据 |
| `PostObject` | 表单上传（浏览器直传场景） |
| LiveChannel | RTMP 推流相关 API（约 8 个） |
| 数据处理 | 图片处理（`ProcessObject`）、视频截帧 |
| `DescribeRegions` | 查询所有可用地域 |
| WASM 支持 | 在 WebAssembly 环境中运行 |
| 同步 API | 可选的阻塞式同步接口 |

---

## 如何参与

- 查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解开发流程
- 在 [Issues](https://github.com/infinitete/rs-ali-oss/issues) 中提出需求或报告问题
- 欢迎为路线图中的任何功能提交 Pull Request

如果你对某个功能有特别需求，请通过 Issue 告知我们，帮助调整优先级。

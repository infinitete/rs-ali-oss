# rs-ali-oss

[![CI](https://github.com/infinitete/rs-ali-oss/actions/workflows/ci.yml/badge.svg)](https://github.com/infinitete/rs-ali-oss/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[English](README.md)

阿里云对象存储服务 (OSS) 的生产级 Rust SDK。

## 特性

- **核心 OSS API 覆盖** — 对象增删改查、存储桶基础操作、分片上传、预签名 URL、标签、ACL（详见 [API 覆盖率](#api-覆盖率)）
- **V4 签名认证** — 使用 OSS V4 签名算法进行安全的请求签名
- **自动重试** — 带抖动的指数退避策略，处理瞬态错误
- **Transfer Manager** — 大文件自动分片上传，支持 CRC64 校验
- **自动分页** — `ListObjectsV2` 和 `ListBuckets` 透明分页
- **进度追踪** — 实时上传/下载进度回调
- **凭证提供者** — 静态凭证、环境变量凭证、凭证链
- **安全优先** — 密钥内存清零、`Debug` 输出脱敏、默认强制 HTTPS
- **异步/等待** — 基于 `tokio` + `reqwest` 的高性能异步 I/O
- **强类型** — Bucket 名称、Object 键、存储类型、时间戳均使用强类型

## 快速开始

添加到 `Cargo.toml`：

```toml
[dependencies]
rs-ali-oss = "0.1"
tokio = { version = "1", features = ["full"] }
```

### 上传对象

```rust
use rs_ali_oss::{OssClient, ClientBuilder, BucketName, ObjectKey, Result};
use rs_ali_oss::types::request::PutObjectRequestBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    let client = OssClient::from_builder(
        ClientBuilder::new()
            .access_key_id("your-access-key-id")
            .access_key_secret("your-access-key-secret")
            .region("cn-hangzhou"),
    )?;

    let request = PutObjectRequestBuilder::new()
        .bucket(BucketName::new("my-bucket")?)
        .key(ObjectKey::new("hello.txt")?)
        .body(b"Hello, OSS!".to_vec())
        .content_type("text/plain")
        .build()?;

    let response = client.put_object(request).await?;
    println!("ETag: {}", response.etag);
    Ok(())
}
```

### 下载对象

```rust
use rs_ali_oss::{OssClient, ClientBuilder, BucketName, ObjectKey, Result};
use rs_ali_oss::types::request::GetObjectRequestBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    let client = OssClient::from_builder(
        ClientBuilder::new()
            .access_key_id("your-access-key-id")
            .access_key_secret("your-access-key-secret")
            .region("cn-hangzhou"),
    )?;

    let request = GetObjectRequestBuilder::new()
        .bucket(BucketName::new("my-bucket")?)
        .key(ObjectKey::new("hello.txt")?)
        .build()?;

    let response = client.get_object(request).await?;
    let bytes = response.body.bytes().await?;
    println!("内容: {}", String::from_utf8_lossy(&bytes));
    Ok(())
}
```

### 生成预签名 URL

```rust
use rs_ali_oss::{OssClient, ClientBuilder, BucketName, ObjectKey, Result};
use rs_ali_oss::types::request::PresignedUrlRequestBuilder;
use std::time::Duration;

fn main() -> Result<()> {
    let client = OssClient::from_builder(
        ClientBuilder::new()
            .access_key_id("your-access-key-id")
            .access_key_secret("your-access-key-secret")
            .region("cn-hangzhou"),
    )?;

    let request = PresignedUrlRequestBuilder::new()
        .bucket(BucketName::new("my-bucket")?)
        .key(ObjectKey::new("secret-doc.pdf")?)
        .expires(Duration::from_secs(3600))
        .build()?;

    let url = client.presign_get_object(request)?;
    println!("下载链接（1 小时有效）: {url}");
    Ok(())
}
```

### 使用 Transfer Manager 上传大文件

```rust
use rs_ali_oss::{OssClient, ClientBuilder, BucketName, ObjectKey, Result};
use rs_ali_oss::ops::transfer::{TransferManagerBuilder, TransferUploadRequestBuilder};

#[tokio::main]
async fn main() -> Result<()> {
    let client = OssClient::from_builder(
        ClientBuilder::new()
            .access_key_id("your-access-key-id")
            .access_key_secret("your-access-key-secret")
            .region("cn-hangzhou"),
    )?;

    let manager = TransferManagerBuilder::new(client)
        .part_size(10 * 1024 * 1024)  // 每片 10 MB
        .enable_crc64(true)
        .build();

    let data = vec![0u8; 50_000_000]; // 50 MB 文件
    let request = TransferUploadRequestBuilder::new()
        .bucket(BucketName::new("my-bucket")?)
        .key(ObjectKey::new("large-file.bin")?)
        .data(data)
        .build()?;

    let response = manager.upload(request).await?;
    println!("ETag: {}", response.etag);
    Ok(())
}
```

## API 参考

### 对象操作

| 方法 | 说明 |
|------|------|
| `put_object` | 上传对象 |
| `get_object` | 下载对象 |
| `head_object` | 获取对象元数据 |
| `delete_object` | 删除对象 |
| `delete_multiple_objects` | 批量删除对象 |
| `copy_object` | 复制对象 |
| `list_objects_v2` | 列举存储桶中的对象 |
| `append_object` | 追加写入对象 |
| `restore_object` | 解冻归档对象 |
| `get_object_acl` / `put_object_acl` | 获取/设置对象 ACL |
| `get_object_tagging` / `put_object_tagging` / `delete_object_tagging` | 对象标签操作 |

### 存储桶操作

| 方法 | 说明 |
|------|------|
| `create_bucket` | 创建存储桶 |
| `delete_bucket` | 删除存储桶 |
| `list_buckets` | 列举所有存储桶 |
| `get_bucket_info` | 获取存储桶信息 |
| `get_bucket_location` | 获取存储桶所在数据中心 |

### 分片上传

| 方法 | 说明 |
|------|------|
| `initiate_multipart_upload` | 初始化分片上传 |
| `upload_part` | 上传单个分片 |
| `complete_multipart_upload` | 完成分片上传 |
| `abort_multipart_upload` | 取消分片上传 |
| `list_parts` | 列举已上传的分片 |
| `list_multipart_uploads` | 列举进行中的分片上传 |

### 预签名 URL

| 方法 | 说明 |
|------|------|
| `presign_get_object` | 生成下载预签名 URL |
| `presign_put_object` | 生成上传预签名 URL |

### 高级 API

| 组件 | 说明 |
|------|------|
| `TransferManager` | 自动分片上传，支持 CRC64 校验和进度追踪 |
| `ListObjectsV2Paginator` | 对象列表自动分页 |
| `ListBucketsPaginator` | 存储桶列表自动分页 |

## API 覆盖率

本 SDK 专注于**核心数据面操作**。存储桶管理/策略类 API 尚未实现。

| 类别 | 已实现 | 总计 | 覆盖率 |
|------|--------|------|--------|
| 对象操作 | 14 | ~19 | ~74% |
| 存储桶基础操作（CRUD） | 5 | ~5 | 100% |
| 存储桶管理/策略 | 0 | ~40 | 0% |
| 分片上传 | 6 | ~7 | ~86% |
| 预签名 URL | 2 | 2 | 100% |

### 尚未实现

<details>
<summary>存储桶管理类 API（生命周期、版本控制、加密、CORS 等）</summary>

- 存储桶 ACL — `PutBucketAcl`、`GetBucketAcl`
- 生命周期 — `PutBucketLifecycle`、`GetBucketLifecycle`、`DeleteBucketLifecycle`
- 版本控制 — `PutBucketVersioning`、`GetBucketVersioning`、`ListObjectVersions`
- 服务端加密 — `PutBucketEncryption`、`GetBucketEncryption`、`DeleteBucketEncryption`
- 日志管理 — `PutBucketLogging`、`GetBucketLogging`、`DeleteBucketLogging`
- 静态网站 — `PutBucketWebsite`、`GetBucketWebsite`、`DeleteBucketWebsite`
- 防盗链 — `PutBucketReferer`、`GetBucketReferer`
- 跨域规则 — `PutBucketCors`、`GetBucketCors`、`DeleteBucketCors`
- 授权策略 — `PutBucketPolicy`、`GetBucketPolicy`、`DeleteBucketPolicy`
- 清单 — `PutBucketInventory`、`GetBucketInventory`、`ListBucketInventory`、`DeleteBucketInventory`
- 跨区域复制 — `PutBucketReplication`、`GetBucketReplication`、`DeleteBucketReplication` 等
- 合规保留 (WORM) — `InitiateBucketWorm`、`CompleteBucketWorm` 等
- 传输加速 — `PutBucketTransferAcceleration`、`GetBucketTransferAcceleration`
- 请求者付费 — `PutBucketRequestPayment`、`GetBucketRequestPayment`
- 存储桶标签 — `PutBucketTags`、`GetBucketTags`、`DeleteBucketTags`

</details>

<details>
<summary>其他对象 API</summary>

- 软链接 — `PutSymlink`、`GetSymlink`
- 分片拷贝 — `UploadPartCopy`
- 轻量级元数据 — `GetObjectMeta`
- SQL 查询 — `SelectObject`
- 表单上传 — `PostObject`

</details>

<details>
<summary>其他 API</summary>

- LiveChannel（RTMP 推流）
- 数据处理（图片处理、视频截帧）
- `DescribeRegions`

</details>

欢迎贡献！请参阅 [贡献指南](CONTRIBUTING.md)。

完整的开发计划请查看 [开发路线图](ROADMAP.md)。

## 配置

### 客户端构建器

```rust
use rs_ali_oss::ClientBuilder;
use std::time::Duration;

let config = ClientBuilder::new()
    .access_key_id("LTAI5tXXXX")
    .access_key_secret("your-secret")
    .region("cn-hangzhou")
    // 可选配置
    .endpoint("https://oss-cn-hangzhou.aliyuncs.com")
    .use_path_style(false)
    .max_retries(3)
    .base_retry_delay(Duration::from_millis(200))
    .max_retry_delay(Duration::from_secs(30))
    .connect_timeout(Duration::from_secs(10))
    .read_timeout(Duration::from_secs(30))
    .request_timeout(Duration::from_secs(300))
    .pool_max_idle_per_host(10)
    .pool_idle_timeout(Duration::from_secs(90))
    .build()?;
```

### 凭证提供者

```rust
use rs_ali_oss::credential::{
    StaticProvider, EnvironmentProvider, ProviderChain,
};

// 静态凭证
let provider = StaticProvider::new("access-key-id", "access-key-secret");

// 从环境变量读取：
//   ALIBABA_CLOUD_ACCESS_KEY_ID
//   ALIBABA_CLOUD_ACCESS_KEY_SECRET
//   ALIBABA_CLOUD_SECURITY_TOKEN（可选，STS 临时凭证）
let provider = EnvironmentProvider::new();

// 凭证链（按顺序尝试每个提供者）
let chain = ProviderChain::default_chain()
    .with(StaticProvider::new("fallback-id", "fallback-secret"));
```

### STS 临时凭证

```rust
use rs_ali_oss::ClientBuilder;

let client = OssClient::from_builder(
    ClientBuilder::new()
        .access_key_id("sts-access-key-id")
        .access_key_secret("sts-access-key-secret")
        .security_token("sts-security-token")
        .region("cn-hangzhou"),
)?;
```

### 进度追踪

```rust
use rs_ali_oss::progress::{ProgressListener, TransferProgress};
use std::sync::Arc;

let listener = Arc::new(|p: &TransferProgress| {
    if let Some(frac) = p.fraction() {
        println!("进度: {:.1}%", frac * 100.0);
    }
});
```

## 错误处理

所有操作返回 `rs_ali_oss::Result<T>`，使用 `OssError` 枚举：

```rust
use rs_ali_oss::OssError;

match client.get_object(request).await {
    Ok(response) => { /* ... */ }
    Err(OssError::ServerError { status, code, message, request_id, .. }) => {
        eprintln!("OSS 错误 {status}: {code} - {message}（请求 ID: {request_id}）");
    }
    Err(OssError::Http(e)) => eprintln!("网络错误: {e}"),
    Err(OssError::RetryExhausted { attempts, last_error }) => {
        eprintln!("{attempts} 次重试后仍然失败: {last_error}");
    }
    Err(e) => eprintln!("其他错误: {e}"),
}
```

## 安全性

- **凭证保护**：通过 `zeroize` crate 在释放时清零访问密钥。`Debug` 输出将密钥显示为 `****`。
- **强制 TLS**：自定义端点默认必须使用 HTTPS。仅在本地开发时使用 `.allow_insecure(true)`。
- **防止凭证泄露**：`Credentials` 故意不实现 `Display`。
- **输入校验**：在发送请求前，对存储桶名称、对象键、元数据键、分片编号和过期时间进行校验。

## 项目结构

```
src/
├── lib.rs           # Crate 入口，公开 re-export
├── client.rs        # OssClient、重试逻辑、URL 构建
├── config.rs        # ClientBuilder、Config、Credentials
├── error.rs         # OssError 枚举、Result 类型别名
├── credential.rs    # CredentialProvider trait 及实现
├── crc64.rs         # CRC64-ECMA 校验和
├── progress.rs      # ProgressListener trait
├── encoding.rs      # URI/Query 百分号编码集
├── middleware.rs     # 请求拦截器链
├── auth/
│   └── v4.rs        # V4 签名算法
├── ops/
│   ├── object.rs    # 对象操作
│   ├── bucket.rs    # 存储桶操作
│   ├── multipart.rs # 分片上传操作
│   ├── presign.rs   # 预签名 URL 生成
│   ├── paginator.rs # 自动分页器
│   └── transfer.rs  # Transfer Manager
└── types/
    ├── common.rs    # BucketName、ObjectKey、Region、StorageClass、ObjectAcl
    ├── response.rs  # 所有响应类型
    └── request/     # 所有请求构建器
```

## 环境要求

- **Rust**：Edition 2024（stable 工具链）
- **核心依赖**：`reqwest`、`serde`、`quick-xml`、`thiserror`、`tokio`、`hmac`、`sha2`、`chrono`、`zeroize`

## 许可证

本项目采用 [MIT 许可证](LICENSE) 授权。

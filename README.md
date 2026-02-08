# rs-ali-oss

[![CI](https://github.com/infinitete/rs-ali-oss/actions/workflows/ci.yml/badge.svg)](https://github.com/infinitete/rs-ali-oss/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[中文文档](README.zh-CN.md)

A production-ready Rust SDK for [Alibaba Cloud Object Storage Service (OSS)](https://www.alibabacloud.com/product/object-storage-service).

## Features

- **Core OSS API Coverage** — Object CRUD, bucket basics, multipart upload, presigned URLs, tagging, ACL (see [API Coverage](#api-coverage) for details)
- **V4 Signature Authentication** — Secure request signing with OSS V4 signature algorithm
- **Automatic Retry** — Exponential backoff with jitter for transient errors
- **Transfer Manager** — Automatic multipart upload for large files with CRC64 checksum verification
- **Auto-Pagination** — Transparent pagination for `ListObjectsV2` and `ListBuckets`
- **Progress Tracking** — Real-time upload/download progress callbacks
- **Credential Providers** — Static, environment-based, and chainable credential sources
- **Security First** — Secrets zeroized in memory, redacted in `Debug` output, HTTPS enforced by default
- **Async/Await** — Built on `tokio` + `reqwest` for high-performance async I/O
- **Type Safety** — Strong types for bucket names, object keys, storage classes, and timestamps

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
rs-ali-oss = "0.1"
tokio = { version = "1", features = ["full"] }
```

### Upload an Object

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

### Download an Object

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
    println!("Content: {}", String::from_utf8_lossy(&bytes));
    Ok(())
}
```

### Generate a Presigned URL

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
    println!("Download URL (valid 1 hour): {url}");
    Ok(())
}
```

### Upload Large Files with Transfer Manager

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
        .part_size(10 * 1024 * 1024)  // 10 MB per part
        .enable_crc64(true)
        .build();

    let data = vec![0u8; 50_000_000]; // 50 MB file
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

## API Reference

### Object Operations

| Method | Description |
|--------|-------------|
| `put_object` | Upload an object |
| `get_object` | Download an object |
| `head_object` | Get object metadata |
| `delete_object` | Delete an object |
| `delete_multiple_objects` | Delete objects in batch |
| `copy_object` | Copy an object |
| `list_objects_v2` | List objects in a bucket |
| `append_object` | Append data to an appendable object |
| `restore_object` | Restore an archived object |
| `get_object_acl` / `put_object_acl` | Get/Set object ACL |
| `get_object_tagging` / `put_object_tagging` / `delete_object_tagging` | Object tagging operations |

### Bucket Operations

| Method | Description |
|--------|-------------|
| `create_bucket` | Create a new bucket |
| `delete_bucket` | Delete a bucket |
| `list_buckets` | List all buckets |
| `get_bucket_info` | Get bucket metadata |
| `get_bucket_location` | Get bucket data center location |

### Multipart Upload

| Method | Description |
|--------|-------------|
| `initiate_multipart_upload` | Start a multipart upload |
| `upload_part` | Upload a single part |
| `complete_multipart_upload` | Finalize the upload |
| `abort_multipart_upload` | Cancel and clean up |
| `list_parts` | List uploaded parts |
| `list_multipart_uploads` | List active multipart uploads |

### Presigned URLs

| Method | Description |
|--------|-------------|
| `presign_get_object` | Generate download URL |
| `presign_put_object` | Generate upload URL |

### High-Level APIs

| Component | Description |
|-----------|-------------|
| `TransferManager` | Automatic multipart upload with CRC64 and progress tracking |
| `ListObjectsV2Paginator` | Auto-paginated object listing |
| `ListBucketsPaginator` | Auto-paginated bucket listing |

## API Coverage

This SDK focuses on **core data-plane operations**. Bucket management/policy APIs are not yet implemented.

| Category | Implemented | Total | Coverage |
|----------|-------------|-------|----------|
| Object operations | 14 | ~19 | ~74% |
| Bucket basics (CRUD) | 5 | ~5 | 100% |
| Bucket management/policy | 0 | ~40 | 0% |
| Multipart upload | 6 | ~7 | ~86% |
| Presigned URLs | 2 | 2 | 100% |

### Not Yet Implemented

<details>
<summary>Bucket management APIs (lifecycle, versioning, encryption, CORS, etc.)</summary>

- Bucket ACL — `PutBucketAcl`, `GetBucketAcl`
- Lifecycle — `PutBucketLifecycle`, `GetBucketLifecycle`, `DeleteBucketLifecycle`
- Versioning — `PutBucketVersioning`, `GetBucketVersioning`, `ListObjectVersions`
- Server-side encryption — `PutBucketEncryption`, `GetBucketEncryption`, `DeleteBucketEncryption`
- Logging — `PutBucketLogging`, `GetBucketLogging`, `DeleteBucketLogging`
- Static website — `PutBucketWebsite`, `GetBucketWebsite`, `DeleteBucketWebsite`
- Hotlink protection — `PutBucketReferer`, `GetBucketReferer`
- CORS — `PutBucketCors`, `GetBucketCors`, `DeleteBucketCors`
- Bucket policy — `PutBucketPolicy`, `GetBucketPolicy`, `DeleteBucketPolicy`
- Inventory — `PutBucketInventory`, `GetBucketInventory`, `ListBucketInventory`, `DeleteBucketInventory`
- Cross-region replication — `PutBucketReplication`, `GetBucketReplication`, `DeleteBucketReplication`, etc.
- WORM (compliance retention) — `InitiateBucketWorm`, `CompleteBucketWorm`, etc.
- Transfer acceleration — `PutBucketTransferAcceleration`, `GetBucketTransferAcceleration`
- Requester pays — `PutBucketRequestPayment`, `GetBucketRequestPayment`
- Bucket tags — `PutBucketTags`, `GetBucketTags`, `DeleteBucketTags`

</details>

<details>
<summary>Additional object APIs</summary>

- Symlink — `PutSymlink`, `GetSymlink`
- Multipart copy — `UploadPartCopy`
- Lightweight metadata — `GetObjectMeta`
- SQL select — `SelectObject`
- Form upload — `PostObject`

</details>

<details>
<summary>Other APIs</summary>

- LiveChannel (RTMP streaming)
- Data processing (image processing, video snapshots)
- `DescribeRegions`

</details>

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

For the full development plan, see the [Roadmap](ROADMAP.md).

## Configuration

### Client Builder

```rust
use rs_ali_oss::{OssClient, ClientBuilder};
use std::time::Duration;

let client = OssClient::from_builder(
    ClientBuilder::new()
        .access_key_id("LTAI5tXXXX")
        .access_key_secret("your-secret")
        .region("cn-hangzhou")
        // Optional settings
        .endpoint("https://oss-cn-hangzhou.aliyuncs.com")
        .max_retries(3)
        .base_retry_delay(Duration::from_millis(200))
        .max_retry_delay(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(30))
        .request_timeout(Duration::from_secs(300))
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(90)),
)?;
```

### Endpoint & URL Style

By default, the SDK constructs virtual-hosted style URLs (`{bucket}.oss-{region}.aliyuncs.com`).
When you set a custom endpoint, the SDK automatically prepends the bucket name to the endpoint host:

```rust
// Custom endpoint — SDK generates: https://my-bucket.oss-cn-chengdu.aliyuncs.com/key
let client = OssClient::from_builder(
    ClientBuilder::new()
        .access_key_id("LTAI5tXXXX")
        .access_key_secret("your-secret")
        .region("cn-chengdu")
        .endpoint("https://oss-cn-chengdu.aliyuncs.com"),
)?;

// No endpoint — SDK auto-generates: https://my-bucket.oss-cn-hangzhou.aliyuncs.com/key
let client = OssClient::from_builder(
    ClientBuilder::new()
        .access_key_id("LTAI5tXXXX")
        .access_key_secret("your-secret")
        .region("cn-hangzhou"),
)?;

// Path-style (bucket in URL path instead of subdomain):
// https://oss-cn-hangzhou.aliyuncs.com/my-bucket/key
let client = OssClient::from_builder(
    ClientBuilder::new()
        .access_key_id("LTAI5tXXXX")
        .access_key_secret("your-secret")
        .region("cn-hangzhou")
        .use_path_style(true),
)?;
```

### Credential Providers

```rust
use rs_ali_oss::credential::{
    StaticProvider, EnvironmentProvider, ProviderChain,
};

// Static credentials
let provider = StaticProvider::new("access-key-id", "access-key-secret");

// From environment variables:
//   ALIBABA_CLOUD_ACCESS_KEY_ID
//   ALIBABA_CLOUD_ACCESS_KEY_SECRET
//   ALIBABA_CLOUD_SECURITY_TOKEN (optional)
let provider = EnvironmentProvider::new();

// Credential chain (tries each provider in order)
let chain = ProviderChain::default_chain()
    .with(StaticProvider::new("fallback-id", "fallback-secret"));
```

### STS Temporary Credentials

```rust
use rs_ali_oss::{OssClient, ClientBuilder};

let client = OssClient::from_builder(
    ClientBuilder::new()
        .access_key_id("sts-access-key-id")
        .access_key_secret("sts-access-key-secret")
        .security_token("sts-security-token")
        .region("cn-hangzhou"),
)?;
```

### Progress Tracking

```rust
use rs_ali_oss::progress::{ProgressListener, TransferProgress};
use std::sync::Arc;

let listener = Arc::new(|p: &TransferProgress| {
    if let Some(frac) = p.fraction() {
        println!("Progress: {:.1}%", frac * 100.0);
    }
});
```

## Error Handling

All operations return `rs_ali_oss::Result<T>`, which uses the `OssError` enum:

```rust
use rs_ali_oss::OssError;

match client.get_object(request).await {
    Ok(response) => { /* ... */ }
    Err(OssError::ServerError { status, code, message, request_id, .. }) => {
        eprintln!("OSS error {status}: {code} - {message} (request: {request_id})");
    }
    Err(OssError::Http(e)) => eprintln!("Network error: {e}"),
    Err(OssError::RetryExhausted { attempts, last_error }) => {
        eprintln!("Failed after {attempts} attempts: {last_error}");
    }
    Err(e) => eprintln!("Other error: {e}"),
}
```

## Security

- **Credential Protection**: Access keys are zeroized on drop via the `zeroize` crate. `Debug` output redacts secrets as `****`.
- **TLS Enforced**: Custom endpoints must use HTTPS by default. Use `.allow_insecure(true)` only for local development.
- **No Credential Leaks**: `Display` is intentionally not implemented for `Credentials`.
- **Input Validation**: Bucket names, object keys, metadata keys, part numbers, and expiry durations are validated before requests are sent.

## Project Structure

```
src/
├── lib.rs           # Crate root, public re-exports
├── client.rs        # OssClient, retry logic, URL construction
├── config.rs        # ClientBuilder, Config, Credentials
├── error.rs         # OssError enum, Result type alias
├── credential.rs    # CredentialProvider trait and implementations
├── crc64.rs         # CRC64-ECMA checksum
├── progress.rs      # ProgressListener trait
├── encoding.rs      # URI/Query percent-encoding sets
├── middleware.rs     # Request interceptor chain
├── auth/
│   └── v4.rs        # V4 signature algorithm
├── ops/
│   ├── object.rs    # Object operations
│   ├── bucket.rs    # Bucket operations
│   ├── multipart.rs # Multipart upload operations
│   ├── presign.rs   # Presigned URL generation
│   ├── paginator.rs # Auto-paginators
│   └── transfer.rs  # Transfer Manager
└── types/
    ├── common.rs    # BucketName, ObjectKey, Region, StorageClass, ObjectAcl
    ├── response.rs  # All response types
    └── request/     # All request builders
```

## Requirements

- **Rust**: Edition 2024 (stable toolchain)
- **Minimum Dependencies**: `reqwest`, `serde`, `quick-xml`, `thiserror`, `tokio`, `hmac`, `sha2`, `chrono`, `zeroize`

## License

This project is licensed under the [MIT License](LICENSE).

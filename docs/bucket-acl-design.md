# Bucket ACL 架构设计文档

> **版本**: 1.0
> **日期**: 2025-02-10
> **优先级**: 高 (v0.2.0 核心功能)

---

## 1. 概述

本文档定义了 rs-ali-oss SDK 中 Bucket ACL（访问控制列表）功能的架构设计。该功能允许用户设置和查询存储桶的访问权限。

### 1.1 设计目标

1. **API 一致性**: 与现有 Object ACL API 保持一致的设计模式
2. **类型安全**: 使用强类型枚举确保 ACL 值的有效性
3. **可扩展性**: 为未来 ACL 相关功能（如 Bucket Policy）预留扩展空间
4. **测试友好**: 支持完整的单元测试和集成测试

---

## 2. 模块架构

### 2.1 整体架构图

```
┌─────────────────────────────────────────────────────────────────────┐
│                            OssClient                                 │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                      put_bucket_acl()                        │    │
│  │                      get_bucket_acl()                        │    │
│  └─────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Type Definitions                             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐      │
│  │   BucketAcl     │  │   Requests      │  │   Responses     │      │
│  │   (common.rs)   │  │   (request/)    │  │   (response.rs) │      │
│  │                 │  │                 │  │                 │      │
│  │ • Private       │  │ • PutBucket...  │  │ • PutBucket...  │      │
│  │ • PublicRead    │  │ • GetBucket...  │  │ • GetBucket...  │      │
│  │ • PublicRW      │  │                 │  │ • BucketOwner   │      │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘      │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                          HTTP Layer                                  │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  PUT /?acl    x-oss-acl: private                            │    │
│  │  GET /?acl                                                  │    │
│  └─────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 文件组织

```
src/
├── types/
│   ├── common.rs          // 新增 BucketAcl 枚举
│   ├── request/
│   │   └── bucket.rs      // 新增 PutBucketAclRequest, GetBucketAclRequest
│   └── response.rs        // 新增响应类型
├── ops/
│   └── bucket.rs          // 新增 put_bucket_acl(), get_bucket_acl()
└── lib.rs                 // 导出公开类型
```

---

## 3. 类型定义

### 3.1 BucketAcl 枚举

**位置**: `src/types/common.rs`

```rust
/// Bucket access control level.
///
/// Defines the access permissions for a bucket. Each level grants
/// different permissions to the public and authenticated users.
///
/// # Variants
///
/// * `Private` - Only the bucket owner has access
/// * `PublicRead` - Owner has full access, public has read access
/// * `PublicReadWrite` - Everyone has full access (use with caution)
///
/// # Examples
///
/// ```
/// use rs_ali_oss::types::BucketAcl;
///
/// let acl = BucketAcl::PublicRead;
/// assert_eq!(acl.to_string(), "public-read");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BucketAcl {
    /// Private access (bucket owner only).
    #[serde(rename = "private")]
    Private,

    /// Public read access.
    #[serde(rename = "public-read")]
    PublicRead,

    /// Public read-write access.
    #[serde(rename = "public-read-write")]
    PublicReadWrite,
}

impl fmt::Display for BucketAcl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Private => write!(f, "private"),
            Self::PublicRead => write!(f, "public-read"),
            Self::PublicReadWrite => write!(f, "public-read-write"),
        }
    }
}
```

**设计决策**:
- 创建独立的 `BucketAcl` 而非复用 `ObjectAcl`
- 原因: `ObjectAcl` 包含 `Default` 变体，对 Bucket 无意义
- 未来两者可能有不同的值空间

### 3.2 请求类型

#### 3.2.1 PutBucketAclRequest

**位置**: `src/types/request/bucket.rs`

```rust
/// Request to set the ACL of a bucket.
#[derive(Debug)]
pub struct PutBucketAclRequest {
    pub(crate) bucket: BucketName,
    pub(crate) acl: BucketAcl,
}

/// Builder for [`PutBucketAclRequest`].
#[derive(Debug, Default)]
pub struct PutBucketAclRequestBuilder {
    bucket: Option<BucketName>,
    acl: Option<BucketAcl>,
}

impl PutBucketAclRequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bucket(mut self, bucket: BucketName) -> Self {
        self.bucket = Some(bucket);
        self
    }

    pub fn acl(mut self, acl: BucketAcl) -> Self {
        self.acl = Some(acl);
        self
    }

    pub fn build(self) -> Result<PutBucketAclRequest> {
        Ok(PutBucketAclRequest {
            bucket: self.bucket.ok_or_else(|| OssError::MissingField("bucket".into()))?,
            acl: self.acl.ok_or_else(|| OssError::MissingField("acl".into()))?,
        })
    }
}
```

#### 3.2.2 GetBucketAclRequest

**位置**: `src/types/request/bucket.rs`

```rust
/// Request to get the ACL of a bucket.
#[derive(Debug)]
pub struct GetBucketAclRequest {
    pub(crate) bucket: BucketName,
}

/// Builder for [`GetBucketAclRequest`].
#[derive(Debug, Default)]
pub struct GetBucketAclRequestBuilder {
    bucket: Option<BucketName>,
}

impl GetBucketAclRequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bucket(mut self, bucket: BucketName) -> Self {
        self.bucket = Some(bucket);
        self
    }

    pub fn build(self) -> Result<GetBucketAclRequest> {
        Ok(GetBucketAclRequest {
            bucket: self.bucket.ok_or_else(|| OssError::MissingField("bucket".into()))?,
        })
    }
}
```

### 3.3 响应类型

#### 3.3.1 PutBucketAclResponse

**位置**: `src/types/response.rs`

```rust
/// Response from a PutBucketAcl operation.
#[derive(Debug)]
pub struct PutBucketAclResponse {
    /// OSS request ID.
    pub request_id: Option<String>,
}
```

#### 3.3.2 GetBucketAclResponse

**位置**: `src/types/response.rs`

```rust
/// Response from a GetBucketAcl operation (XML-deserialized).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "AccessControlPolicy")]
pub struct GetBucketAclResponse {
    /// Bucket owner information.
    pub owner: BucketOwner,
    /// Access control list.
    #[serde(rename = "AccessControlList")]
    pub access_control_list: BucketAccessControlList,
}

/// Bucket owner information.
#[derive(Debug, Clone, Deserialize)]
pub struct BucketOwner {
    /// User ID of the bucket owner.
    pub id: String,
    /// Display name of the bucket owner.
    pub display_name: String,
}

/// Access control list for bucket ACL.
#[derive(Debug, Clone, Deserialize)]
pub struct BucketAccessControlList {
    /// The granted permission.
    pub grant: BucketAcl,
}
```

---

## 4. API 实现

### 4.1 OssClient 方法

**位置**: `src/ops/bucket.rs`

```rust
impl OssClient {
    /// Set the ACL of a bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::PutBucketAclRequestBuilder;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let request = PutBucketAclRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .acl(BucketAcl::PublicRead)
    ///     .build()?;
    /// client.put_bucket_acl(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn put_bucket_acl(
        &self,
        request: PutBucketAclRequest,
    ) -> Result<PutBucketAclResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("acl", "")])?;
        let resource_path = format!("/{}/", request.bucket);
        let http_req = self
            .http_client()
            .request(Method::PUT, url)
            .header("x-oss-acl", request.acl.to_string())
            .build()?;
        let response = self.execute(http_req, &resource_path).await?;
        let request_id = header_opt(&response, "x-oss-request-id");
        Ok(PutBucketAclResponse { request_id })
    }

    /// Get the ACL of a bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::GetBucketAclRequestBuilder;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let request = GetBucketAclRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .build()?;
    /// let response = client.get_bucket_acl(request).await?;
    /// println!("ACL: {}", response.access_control_list.grant);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_bucket_acl(
        &self,
        request: GetBucketAclRequest,
    ) -> Result<GetBucketAclResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("acl", "")])?;
        let resource_path = format!("/{}/", request.bucket);
        let http_req = self.http_client().request(Method::GET, url).build()?;
        let response = self.execute(http_req, &resource_path).await?;
        let body = response.text().await?;
        let resp: GetBucketAclResponse = parse_xml(&body)?;
        Ok(resp)
    }
}
```

### 4.2 CreateBucket 增强

**变更**: `src/types/request/bucket.rs`

```rust
/// Request to create a new bucket.
#[derive(Debug)]
pub struct CreateBucketRequest {
    pub(crate) bucket: BucketName,
    pub(crate) storage_class: Option<StorageClass>,
    pub(crate) acl: Option<BucketAcl>,  // 新增
}

impl CreateBucketRequestBuilder {
    // ... 现有方法 ...

    /// Set the ACL for the new bucket.
    pub fn acl(mut self, acl: BucketAcl) -> Self {
        self.acl = Some(acl);
        self
    }

    pub fn build(self) -> Result<CreateBucketRequest> {
        Ok(CreateBucketRequest {
            bucket: self.bucket.ok_or_else(|| OssError::MissingField("bucket".into()))?,
            storage_class: self.storage_class,
            acl: self.acl,  // 新增
        })
    }
}
```

**变更**: `src/ops/bucket.rs` - `create_bucket` 方法

```rust
pub async fn create_bucket(
    &self,
    request: CreateBucketRequest,
) -> Result<CreateBucketResponse> {
    let url = self.build_url(Some(&request.bucket), None, &[])?;
    let resource_path = format!("/{}/", request.bucket);
    let mut http_req = self.http_client().request(Method::PUT, url);

    // 处理 storage_class
    let body = match request.storage_class {
        Some(sc) => {
            let xml = format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
                 <CreateBucketConfiguration>\
                 <StorageClass>{sc}</StorageClass>\
                 </CreateBucketConfiguration>"
            );
            http_req = http_req.header("content-type", "application/xml");
            Some(xml)
        }
        None => None,
    };

    let http_req = http_req.build()?;
    let mut response = self.execute(http_req, &resource_path).await?;

    // 新增: 处理 ACL
    if let Some(acl) = request.acl {
        // 需要重新发送请求以包含 ACL 头
        // 或者修改请求构建逻辑
    }

    // ...
}
```

**注意**: CreateBucket 的 ACL 支持需要特殊处理，因为 OSS 要求在同一请求中处理 XML body 和自定义头。建议将 ACL 作为请求头添加。

---

## 5. 数据流

### 5.1 PutBucketAcl 流程

```
┌──────────────┐
│   User Code  │
└──────┬───────┘
       │ 1. Build Request
       ▼
┌─────────────────────────────┐
│ PutBucketAclRequestBuilder  │
│ .bucket(name)               │
│ .acl(BucketAcl::PublicRead) │
│ .build()                    │
└──────┬──────────────────────┘
       │ 2. PutBucketAclRequest
       ▼
┌─────────────────────────────┐
│   OssClient::put_bucket_acl │
└──────┬──────────────────────┘
       │ 3. Build URL & Headers
       ▼
┌─────────────────────────────┐
│   PUT /?acl                 │
│   x-oss-acl: public-read    │
└──────┬──────────────────────┘
       │ 4. HTTP Execute
       ▼
┌─────────────────────────────┐
│   OSS Service               │
└──────┬──────────────────────┘
       │ 5. Response (200 OK)
       ▼
┌─────────────────────────────┐
│ PutBucketAclResponse        │
│ { request_id }              │
└─────────────────────────────┘
```

### 5.2 GetBucketAcl 流程

```
┌──────────────┐
│   User Code  │
└──────┬───────┘
       │ 1. Build Request
       ▼
┌─────────────────────────────┐
│ GetBucketAclRequestBuilder  │
│ .bucket(name)               │
│ .build()                    │
└──────┬──────────────────────┘
       │ 2. GetBucketAclRequest
       ▼
┌─────────────────────────────┐
│   OssClient::get_bucket_acl │
└──────┬──────────────────────┘
       │ 3. Build URL
       ▼
┌─────────────────────────────┐
│   GET /?acl                 │
└──────┬──────────────────────┘
       │ 4. HTTP Execute
       ▼
┌─────────────────────────────┐
│   OSS Service               │
└──────┬──────────────────────┘
       │ 5. Response (XML)
       ▼
┌─────────────────────────────┐
│   <AccessControlPolicy>     │
│     <Owner>                 │
│       <ID>...</ID>          │
│       <DisplayName>...</>   │
│     </Owner>                │
│     <AccessControlList>     │
│       <Grant>public-read</> │
│     </AccessControlList>    │
│   </AccessControlPolicy>    │
└──────┬──────────────────────┘
       │ 6. Parse XML
       ▼
┌─────────────────────────────┐
│ GetBucketAclResponse        │
│ { owner, access_control_list } │
└─────────────────────────────┘
```

---

## 6. 错误处理

### 6.1 错误码映射

| OSS 错误码 | HTTP 状态 | 处理方式 |
|-----------|----------|---------|
| NoSuchBucket | 404 | 映射到 `OssError::ServerError` |
| AccessDenied | 403 | 映射到 `OssError::ServerError` |
| InvalidArgument | 400 | 映射到 `OssError::ServerError` |

**设计决策**: 不扩展 `OssError` 枚举，使用现有的 `ServerError` 变体。
- 原因: 保持错误类型稳定，OSS 错误码通过 `code` 字段传递
- 未来可根据需要添加特定的 ACL 错误变体

---

## 7. 测试策略

### 7.1 单元测试

**文件**: `src/types/common.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_acl_display() {
        assert_eq!(BucketAcl::Private.to_string(), "private");
        assert_eq!(BucketAcl::PublicRead.to_string(), "public-read");
        assert_eq!(BucketAcl::PublicReadWrite.to_string(), "public-read-write");
    }

    #[test]
    fn bucket_acl_serde_round_trip() {
        let acl = BucketAcl::PublicRead;
        let json = serde_json::to_string(&acl).unwrap();
        let deserialized: BucketAcl = serde_json::from_str(&json).unwrap();
        assert_eq!(acl, deserialized);
    }
}
```

**文件**: `src/types/request/bucket.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_bucket_acl_request_builder() {
        let req = PutBucketAclRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .acl(BucketAcl::PublicRead)
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn put_bucket_acl_missing_acl() {
        let req = PutBucketAclRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn get_bucket_acl_request_builder() {
        let req = GetBucketAclRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .build();
        assert!(req.is_ok());
    }
}
```

**文件**: `src/types/response.rs`

```rust
#[test]
fn deserialize_get_bucket_acl_response() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AccessControlPolicy>
    <Owner>
        <ID>0022012****</ID>
        <DisplayName>user_example</DisplayName>
    </Owner>
    <AccessControlList>
        <Grant>public-read</Grant>
    </AccessControlList>
</AccessControlPolicy>"#;
    let resp: GetBucketAclResponse = quick_xml::de::from_str(xml).unwrap();
    assert_eq!(resp.access_control_list.grant, BucketAcl::PublicRead);
    assert_eq!(resp.owner.id, "0022012****");
    assert_eq!(resp.owner.display_name, "user_example");
}
```

### 7.2 集成测试

**文件**: `tests/integration_bucket_acl.rs` (可选)

```rust
//! Integration tests for Bucket ACL operations.
//!
//! These tests require a real OSS environment with valid credentials.
//! Run with: cargo test --test integration_bucket_acl -- --ignored

use rs_ali_oss::*;
use rs_ali_oss::types::request::*;

#[tokio::test]
#[ignore = "requires real OSS environment"]
async fn test_put_and_get_bucket_acl() -> Result<()> {
    let client = OssClient::new(/* credentials */);

    // Set ACL
    let put_req = PutBucketAclRequestBuilder::new()
        .bucket(BucketName::new("test-bucket")?)
        .acl(BucketAcl::PublicRead)
        .build()?;
    client.put_bucket_acl(put_req).await?;

    // Get ACL
    let get_req = GetBucketAclRequestBuilder::new()
        .bucket(BucketName::new("test-bucket")?)
        .build()?;
    let resp = client.get_bucket_acl(get_req).await?;

    assert_eq!(resp.access_control_list.grant, BucketAcl::PublicRead);
    Ok(())
}
```

---

## 8. 导出配置

### 8.1 lib.rs 更新

```rust
// src/lib.rs

pub use types::common::{BucketAcl, /* ... 现有导出 ... */};
pub use types::request::{
    PutBucketAclRequest, PutBucketAclRequestBuilder,
    GetBucketAclRequest, GetBucketAclRequestBuilder,
    /* ... 现有导出 ... */
};
pub use types::response::{
    PutBucketAclResponse, GetBucketAclResponse,
    BucketOwner, BucketAccessControlList,
    /* ... 现有导出 ... */
};
```

---

## 9. 实现检查清单

### 9.1 类型定义
- [ ] `BucketAcl` 枚举 (common.rs)
- [ ] `PutBucketAclRequest` + Builder (bucket.rs)
- [ ] `GetBucketAclRequest` + Builder (bucket.rs)
- [ ] `PutBucketAclResponse` (response.rs)
- [ ] `GetBucketAclResponse` (response.rs)
- [ ] `BucketOwner` (response.rs)
- [ ] `BucketAccessControlList` (response.rs)

### 9.2 API 实现
- [ ] `OssClient::put_bucket_acl()` (bucket.rs)
- [ ] `OssClient::get_bucket_acl()` (bucket.rs)
- [ ] `CreateBucketRequest` 添加 `acl` 字段
- [ ] `CreateBucketRequestBuilder::acl()` 方法

### 9.3 测试
- [ ] `BucketAcl` Display 测试
- [ ] `BucketAcl` Serde 测试
- [ ] `PutBucketAclRequestBuilder` 测试
- [ ] `GetBucketAclRequestBuilder` 测试
- [ ] XML 反序列化测试
- [ ] 集成测试（可选）

### 9.4 文档
- [ ] 公开 API 的 rustdoc 注释
- [ ] 代码示例
- [ ] lib.rs 导出更新

---

## 10. 参考资料

1. [阿里云 OSS - PutBucketAcl](https://help.aliyun.com/zh/oss/developer-reference/putbucketacl)
2. [阿里云 OSS - GetBucketAcl](https://help.aliyun.com/zh/oss/developer-reference/getbucketacl)
3. 现有 Object ACL 实现: `src/ops/object.rs`
4. CLAUDE.md - 代码规范

---

## 附录 A: API 对比表

| 特性 | Object ACL | Bucket ACL |
|-----|-----------|-----------|
| 枚举类型 | `ObjectAcl` | `BucketAcl` |
| 权限值 | private, public-read, public-read-write, default | private, public-read, public-read-write |
| 请求头 | `x-oss-object-acl` | `x-oss-acl` |
| 响应结构 | `ObjectAccessControlList` | `BucketAccessControlList` + `BucketOwner` |
| URL 参数 | `?acl` | `?acl` |

---

## 附录 B: 向后兼容性

本设计**不破坏**任何现有 API:
- 新增类型（`BucketAcl`）不影响现有 `ObjectAcl`
- 新增方法（`put_bucket_acl`, `get_bucket_acl`）为独立 API
- `CreateBucketRequest` 的 `acl` 字段为可选，现有代码无需修改

---

**文档版本**: 1.0
**最后更新**: 2025-02-10

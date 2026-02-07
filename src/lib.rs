//! Alibaba Cloud OSS SDK for Rust.
#![deny(missing_docs)]

pub mod auth;
pub mod client;
pub mod config;
pub mod crc64;
pub mod credential;
pub(crate) mod encoding;
pub mod error;
pub mod middleware;
pub mod ops;
pub mod progress;
pub mod types;

pub use client::OssClient;
pub use config::{ClientBuilder, Config, Credentials, PoolConfig, RetryConfig, TimeoutConfig};
pub use credential::{
    CachingProvider, CredentialProvider, EnvironmentProvider, ProviderChain, StaticProvider,
};
pub use error::{OssError, Result};
pub use middleware::{Interceptor, InterceptorContext, RequestOutcome};
pub use ops::paginator::{
    ListBucketsPaginator, ListBucketsPaginatorBuilder, ListObjectsV2Paginator,
    ListObjectsV2PaginatorBuilder,
};
pub use ops::transfer::{
    TransferManager, TransferManagerBuilder, TransferUploadRequest, TransferUploadRequestBuilder,
    TransferUploadResponse,
};
pub use progress::{NoopProgressListener, ProgressListener, TransferKind, TransferProgress};
pub use types::common::{
    BucketName, MetadataDirective, ObjectAcl, ObjectKey, Region, StorageClass,
};
pub use types::response::ObjectBody;

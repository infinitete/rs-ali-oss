//! Transfer Manager for automatic multipart uploads of large files.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::client::OssClient;
use crate::error::{OssError, Result};
use crate::progress::{NoopProgressListener, ProgressListener, TransferKind, TransferProgress};
use crate::types::common::{BucketName, ObjectKey, StorageClass};
use crate::types::request::{
    AbortMultipartUploadRequestBuilder, CompleteMultipartUploadRequestBuilder, CompletedPart,
    InitiateMultipartUploadRequestBuilder, PutObjectRequestBuilder, UploadPartRequestBuilder,
};

const DEFAULT_PART_SIZE: u64 = 8 * 1024 * 1024;
const MIN_PART_SIZE: u64 = 100 * 1024;
const DEFAULT_MULTIPART_THRESHOLD: u64 = 8 * 1024 * 1024;
const DEFAULT_CONCURRENCY: usize = 8;

/// Automatic multipart upload manager.
///
/// Splits large uploads into parts, tracks progress, and computes CRC64
/// checksums when enabled. Falls back to simple `PutObject` for data
/// below the multipart threshold.
///
/// # Examples
/// ```no_run
/// # use rs_ali_oss::*;
/// # use rs_ali_oss::ops::transfer::*;
/// # async fn example(client: OssClient) -> Result<()> {
/// let manager = TransferManagerBuilder::new(client)
///     .part_size(10 * 1024 * 1024)
///     .enable_crc64(true)
///     .build();
/// let request = TransferUploadRequestBuilder::new()
///     .bucket(BucketName::new("my-bucket")?)
///     .key(ObjectKey::new("large-file.bin")?)
///     .data(vec![0u8; 20_000_000])
///     .build()?;
/// let response = manager.upload(request).await?;
/// println!("ETag: {}", response.etag);
/// # Ok(())
/// # }
/// ```
pub struct TransferManager {
    client: OssClient,
    part_size: u64,
    multipart_threshold: u64,
    concurrency: usize,
    progress_listener: Arc<dyn ProgressListener>,
    enable_crc64: bool,
}

/// Builder for [`TransferManager`].
pub struct TransferManagerBuilder {
    client: OssClient,
    part_size: u64,
    multipart_threshold: u64,
    concurrency: usize,
    progress_listener: Option<Arc<dyn ProgressListener>>,
    enable_crc64: bool,
}

impl TransferManagerBuilder {
    /// Create a new builder with the given OSS client.
    pub fn new(client: OssClient) -> Self {
        Self {
            client,
            part_size: DEFAULT_PART_SIZE,
            multipart_threshold: DEFAULT_MULTIPART_THRESHOLD,
            concurrency: DEFAULT_CONCURRENCY,
            progress_listener: None,
            enable_crc64: false,
        }
    }

    /// Set the part size in bytes for multipart uploads (minimum 100 KB).
    pub fn part_size(mut self, size: u64) -> Self {
        self.part_size = size;
        self
    }

    /// Set the size threshold above which multipart upload is used.
    pub fn multipart_threshold(mut self, threshold: u64) -> Self {
        self.multipart_threshold = threshold;
        self
    }

    /// Attach a progress listener.
    pub fn progress_listener(mut self, listener: Arc<dyn ProgressListener>) -> Self {
        self.progress_listener = Some(listener);
        self
    }

    /// Enable CRC64 checksum computation and combination across parts.
    pub fn enable_crc64(mut self, enable: bool) -> Self {
        self.enable_crc64 = enable;
        self
    }

    /// Set the maximum number of concurrent part uploads (default: 8).
    pub fn concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency;
        self
    }

    /// Build the transfer manager.
    pub fn build(self) -> TransferManager {
        let part_size = if self.part_size < MIN_PART_SIZE {
            MIN_PART_SIZE
        } else {
            self.part_size
        };
        let concurrency = self.concurrency.max(1);
        TransferManager {
            client: self.client,
            part_size,
            multipart_threshold: self.multipart_threshold,
            concurrency,
            progress_listener: self
                .progress_listener
                .unwrap_or_else(|| Arc::new(NoopProgressListener)),
            enable_crc64: self.enable_crc64,
        }
    }
}

/// Request for a managed upload.
#[derive(Debug)]
pub struct TransferUploadRequest {
    pub(crate) bucket: BucketName,
    pub(crate) key: ObjectKey,
    pub(crate) data: Vec<u8>,
    pub(crate) content_type: Option<String>,
    pub(crate) storage_class: Option<StorageClass>,
}

/// Builder for [`TransferUploadRequest`].
#[derive(Debug, Default)]
pub struct TransferUploadRequestBuilder {
    bucket: Option<BucketName>,
    key: Option<ObjectKey>,
    data: Option<Vec<u8>>,
    content_type: Option<String>,
    storage_class: Option<StorageClass>,
}

impl TransferUploadRequestBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the target bucket.
    pub fn bucket(mut self, bucket: BucketName) -> Self {
        self.bucket = Some(bucket);
        self
    }

    /// Set the object key.
    pub fn key(mut self, key: ObjectKey) -> Self {
        self.key = Some(key);
        self
    }

    /// Set the upload data.
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    /// Set the content type.
    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    /// Set the storage class.
    pub fn storage_class(mut self, storage_class: StorageClass) -> Self {
        self.storage_class = Some(storage_class);
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<TransferUploadRequest> {
        Ok(TransferUploadRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            key: self
                .key
                .ok_or_else(|| OssError::MissingField("key".into()))?,
            data: self
                .data
                .ok_or_else(|| OssError::MissingField("data".into()))?,
            content_type: self.content_type,
            storage_class: self.storage_class,
        })
    }
}

/// Response from a managed upload.
#[derive(Debug)]
pub struct TransferUploadResponse {
    /// ETag of the uploaded object.
    pub etag: String,
    /// Combined CRC64 checksum (if enabled).
    pub crc64: Option<u64>,
    /// Whether multipart upload was used.
    pub multipart: bool,
}

impl TransferManager {
    /// Upload data, automatically choosing simple or multipart upload.
    ///
    /// If the data length exceeds the multipart threshold, the upload is split
    /// into parts of the configured size, uploaded concurrently (up to the
    /// configured concurrency limit), and then completed. On any part failure
    /// the multipart upload is aborted.
    pub async fn upload(&self, request: TransferUploadRequest) -> Result<TransferUploadResponse> {
        let total_size = request.data.len() as u64;

        if total_size <= self.multipart_threshold {
            return self.simple_upload(request, total_size).await;
        }
        self.multipart_upload(request, total_size).await
    }

    async fn simple_upload(
        &self,
        request: TransferUploadRequest,
        total_size: u64,
    ) -> Result<TransferUploadResponse> {
        let crc = if self.enable_crc64 {
            Some(crate::crc64::checksum(&request.data))
        } else {
            None
        };

        let mut builder = PutObjectRequestBuilder::new()
            .bucket(request.bucket)
            .key(request.key)
            .body(request.data);

        if let Some(ct) = request.content_type {
            builder = builder.content_type(ct);
        }
        if let Some(sc) = request.storage_class {
            builder = builder.storage_class(sc);
        }

        self.progress_listener.on_progress(&TransferProgress {
            bytes_transferred: 0,
            total_bytes: Some(total_size),
            kind: TransferKind::Upload,
        });

        let resp = self.client.put_object(builder.build()?).await?;

        self.progress_listener.on_progress(&TransferProgress {
            bytes_transferred: total_size,
            total_bytes: Some(total_size),
            kind: TransferKind::Upload,
        });

        Ok(TransferUploadResponse {
            etag: resp.etag,
            crc64: crc,
            multipart: false,
        })
    }

    async fn multipart_upload(
        &self,
        request: TransferUploadRequest,
        total_size: u64,
    ) -> Result<TransferUploadResponse> {
        let bucket = request.bucket;
        let key = request.key;

        let mut init_builder = InitiateMultipartUploadRequestBuilder::new()
            .bucket(bucket.clone())
            .key(key.clone());

        if let Some(ct) = request.content_type {
            init_builder = init_builder.content_type(ct);
        }
        if let Some(sc) = request.storage_class {
            init_builder = init_builder.storage_class(sc);
        }

        let init_resp = self
            .client
            .initiate_multipart_upload(init_builder.build()?)
            .await?;
        let upload_id = init_resp.upload_id;

        self.progress_listener.on_progress(&TransferProgress {
            bytes_transferred: 0,
            total_bytes: Some(total_size),
            kind: TransferKind::Upload,
        });

        match self
            .upload_parts(&bucket, &key, &upload_id, &request.data, total_size)
            .await
        {
            Ok((parts, combined_crc)) => {
                let complete_req = CompleteMultipartUploadRequestBuilder::new()
                    .bucket(bucket)
                    .key(key)
                    .upload_id(&upload_id)
                    .parts(parts)
                    .build()?;

                let complete_resp = self.client.complete_multipart_upload(complete_req).await?;

                Ok(TransferUploadResponse {
                    etag: complete_resp.etag.trim_matches('"').to_string(),
                    crc64: combined_crc,
                    multipart: true,
                })
            }
            Err(e) => {
                let abort_req = AbortMultipartUploadRequestBuilder::new()
                    .bucket(bucket)
                    .key(key)
                    .upload_id(&upload_id)
                    .build()?;
                // Best-effort abort — ignore errors
                let _ = self.client.abort_multipart_upload(abort_req).await;
                Err(e)
            }
        }
    }

    async fn upload_parts(
        &self,
        bucket: &BucketName,
        key: &ObjectKey,
        upload_id: &str,
        data: &[u8],
        total_size: u64,
    ) -> Result<(Vec<CompletedPart>, Option<u64>)> {
        let part_size = self.part_size as usize;
        let data: Arc<[u8]> = Arc::from(data);
        let semaphore = Arc::new(Semaphore::new(self.concurrency));
        let bytes_sent = Arc::new(AtomicU64::new(0));
        let mut join_set = JoinSet::new();

        let num_parts = data.len().div_ceil(part_size);
        let mut part_crcs: Vec<(u64, u64)> = Vec::with_capacity(num_parts);

        for (i, chunk_range) in data.chunks(part_size).enumerate() {
            let part_number = (i as u32) + 1;
            let offset = i * part_size;
            let chunk_len = chunk_range.len();

            if self.enable_crc64 {
                let part_crc = crate::crc64::checksum(chunk_range);
                part_crcs.push((part_crc, chunk_len as u64));
            }

            let client = self.client.clone();
            let bucket = bucket.clone();
            let key = key.clone();
            let upload_id = upload_id.to_string();
            let data = Arc::clone(&data);
            let sem = Arc::clone(&semaphore);
            let progress = Arc::clone(&bytes_sent);
            let listener = Arc::clone(&self.progress_listener);

            join_set.spawn(async move {
                let _permit = sem
                    .acquire()
                    .await
                    .map_err(|_| OssError::Auth("semaphore closed".to_string()))?;

                let chunk = data[offset..offset + chunk_len].to_vec();

                let upload_req = UploadPartRequestBuilder::new()
                    .bucket(bucket)
                    .key(key)
                    .upload_id(&upload_id)
                    .part_number(part_number)
                    .body(chunk)
                    .build()?;

                let resp = client.upload_part(upload_req).await?;

                let sent =
                    progress.fetch_add(chunk_len as u64, Ordering::Relaxed) + chunk_len as u64;
                listener.on_progress(&TransferProgress {
                    bytes_transferred: sent,
                    total_bytes: Some(total_size),
                    kind: TransferKind::Upload,
                });

                Ok::<_, OssError>((part_number, resp.etag))
            });
        }

        let mut parts: Vec<CompletedPart> = Vec::with_capacity(num_parts);
        while let Some(result) = join_set.join_next().await {
            let (part_number, etag) =
                result.map_err(|e| OssError::Auth(format!("part upload task panicked: {e}")))??;
            parts.push(CompletedPart { part_number, etag });
        }

        parts.sort_by_key(|p| p.part_number);

        let combined_crc = if self.enable_crc64 {
            let mut crc: u64 = 0;
            for &(part_crc, len) in &part_crcs {
                crc = crate::crc64::combine(crc, part_crc, len);
            }
            Some(crc)
        } else {
            None
        };

        Ok((parts, combined_crc))
    }
}

impl std::fmt::Debug for TransferManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransferManager")
            .field("part_size", &self.part_size)
            .field("multipart_threshold", &self.multipart_threshold)
            .field("concurrency", &self.concurrency)
            .field("enable_crc64", &self.enable_crc64)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ClientBuilder;

    fn test_client() -> OssClient {
        OssClient::from_builder(
            ClientBuilder::new()
                .access_key_id("test-id")
                .access_key_secret("test-secret")
                .region("cn-hangzhou"),
        )
        .unwrap()
    }

    #[test]
    fn builder_defaults() {
        let mgr = TransferManagerBuilder::new(test_client()).build();
        assert_eq!(mgr.part_size, DEFAULT_PART_SIZE);
        assert_eq!(mgr.multipart_threshold, DEFAULT_MULTIPART_THRESHOLD);
        assert_eq!(mgr.concurrency, DEFAULT_CONCURRENCY);
        assert!(!mgr.enable_crc64);
    }

    #[test]
    fn builder_custom_part_size() {
        let mgr = TransferManagerBuilder::new(test_client())
            .part_size(16 * 1024 * 1024)
            .build();
        assert_eq!(mgr.part_size, 16 * 1024 * 1024);
    }

    #[test]
    fn builder_clamps_part_size_to_minimum() {
        let mgr = TransferManagerBuilder::new(test_client())
            .part_size(1024)
            .build();
        assert_eq!(mgr.part_size, MIN_PART_SIZE);
    }

    #[test]
    fn builder_with_crc64() {
        let mgr = TransferManagerBuilder::new(test_client())
            .enable_crc64(true)
            .build();
        assert!(mgr.enable_crc64);
    }

    #[test]
    fn builder_with_custom_threshold() {
        let mgr = TransferManagerBuilder::new(test_client())
            .multipart_threshold(1024 * 1024)
            .build();
        assert_eq!(mgr.multipart_threshold, 1024 * 1024);
    }

    #[test]
    fn builder_custom_concurrency() {
        let mgr = TransferManagerBuilder::new(test_client())
            .concurrency(4)
            .build();
        assert_eq!(mgr.concurrency, 4);
    }

    #[test]
    fn builder_concurrency_clamped_to_one() {
        let mgr = TransferManagerBuilder::new(test_client())
            .concurrency(0)
            .build();
        assert_eq!(mgr.concurrency, 1);
    }

    #[test]
    fn upload_request_builder() {
        let req = TransferUploadRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("file.bin").unwrap())
            .data(vec![1, 2, 3])
            .content_type("application/octet-stream")
            .build();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.data.len(), 3);
        assert_eq!(
            req.content_type.as_deref(),
            Some("application/octet-stream")
        );
    }

    #[test]
    fn upload_request_missing_data() {
        let req = TransferUploadRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("file.bin").unwrap())
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn upload_request_missing_bucket() {
        let req = TransferUploadRequestBuilder::new()
            .key(ObjectKey::new("file.bin").unwrap())
            .data(vec![1])
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn debug_impl_does_not_leak_internals() {
        let mgr = TransferManagerBuilder::new(test_client()).build();
        let debug = format!("{mgr:?}");
        assert!(debug.contains("TransferManager"));
        assert!(debug.contains("part_size"));
    }
}

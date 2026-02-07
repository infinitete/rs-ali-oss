//! Object operation request types: Put, Get, Delete, Head, ListV2, Copy, DeleteMultiple.

use std::collections::HashMap;

use serde::Serialize;

use crate::error::{OssError, Result};
use crate::types::common::{BucketName, MetadataDirective, ObjectAcl, ObjectKey, StorageClass};

use super::validate_metadata_key;

/// Request to upload an object to OSS.
#[derive(Debug)]
pub struct PutObjectRequest {
    pub(crate) bucket: BucketName,
    pub(crate) key: ObjectKey,
    pub(crate) body: reqwest::Body,
    pub(crate) content_type: Option<String>,
    pub(crate) storage_class: Option<StorageClass>,
    pub(crate) acl: Option<ObjectAcl>,
    pub(crate) metadata: HashMap<String, String>,
}

/// Builder for [`PutObjectRequest`].
#[derive(Debug, Default)]
pub struct PutObjectRequestBuilder {
    bucket: Option<BucketName>,
    key: Option<ObjectKey>,
    body: Option<reqwest::Body>,
    content_type: Option<String>,
    storage_class: Option<StorageClass>,
    acl: Option<ObjectAcl>,
    metadata: HashMap<String, String>,
}

impl PutObjectRequestBuilder {
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

    /// Set the request body.
    pub fn body(mut self, body: impl Into<reqwest::Body>) -> Self {
        self.body = Some(body.into());
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

    /// Set the object ACL.
    pub fn acl(mut self, acl: ObjectAcl) -> Self {
        self.acl = Some(acl);
        self
    }

    /// Add a custom metadata entry (x-oss-meta-*).
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<PutObjectRequest> {
        for key in self.metadata.keys() {
            validate_metadata_key(key)?;
        }
        Ok(PutObjectRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            key: self
                .key
                .ok_or_else(|| OssError::MissingField("key".into()))?,
            body: self
                .body
                .ok_or_else(|| OssError::MissingField("body".into()))?,
            content_type: self.content_type,
            storage_class: self.storage_class,
            acl: self.acl,
            metadata: self.metadata,
        })
    }
}

/// Request to download an object from OSS.
#[derive(Debug)]
pub struct GetObjectRequest {
    pub(crate) bucket: BucketName,
    pub(crate) key: ObjectKey,
    pub(crate) range: Option<String>,
}

/// Builder for [`GetObjectRequest`].
#[derive(Debug, Default)]
pub struct GetObjectRequestBuilder {
    bucket: Option<BucketName>,
    key: Option<ObjectKey>,
    range: Option<String>,
}

impl GetObjectRequestBuilder {
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

    /// Set the byte range (e.g., "bytes=0-999").
    pub fn range(mut self, range: impl Into<String>) -> Self {
        self.range = Some(range.into());
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<GetObjectRequest> {
        Ok(GetObjectRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            key: self
                .key
                .ok_or_else(|| OssError::MissingField("key".into()))?,
            range: self.range,
        })
    }
}

/// Request to delete an object from OSS.
#[derive(Debug)]
pub struct DeleteObjectRequest {
    pub(crate) bucket: BucketName,
    pub(crate) key: ObjectKey,
}

/// Builder for [`DeleteObjectRequest`].
#[derive(Debug, Default)]
pub struct DeleteObjectRequestBuilder {
    bucket: Option<BucketName>,
    key: Option<ObjectKey>,
}

impl DeleteObjectRequestBuilder {
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

    /// Build the request.
    pub fn build(self) -> Result<DeleteObjectRequest> {
        Ok(DeleteObjectRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            key: self
                .key
                .ok_or_else(|| OssError::MissingField("key".into()))?,
        })
    }
}

/// Request to retrieve object metadata from OSS.
#[derive(Debug)]
pub struct HeadObjectRequest {
    pub(crate) bucket: BucketName,
    pub(crate) key: ObjectKey,
}

/// Builder for [`HeadObjectRequest`].
#[derive(Debug, Default)]
pub struct HeadObjectRequestBuilder {
    bucket: Option<BucketName>,
    key: Option<ObjectKey>,
}

impl HeadObjectRequestBuilder {
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

    /// Build the request.
    pub fn build(self) -> Result<HeadObjectRequest> {
        Ok(HeadObjectRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            key: self
                .key
                .ok_or_else(|| OssError::MissingField("key".into()))?,
        })
    }
}

/// Request to list objects in a bucket using the V2 API.
#[derive(Debug)]
pub struct ListObjectsV2Request {
    pub(crate) bucket: BucketName,
    pub(crate) prefix: Option<String>,
    pub(crate) delimiter: Option<String>,
    pub(crate) max_keys: Option<u32>,
    pub(crate) continuation_token: Option<String>,
    pub(crate) start_after: Option<String>,
}

/// Builder for [`ListObjectsV2Request`].
#[derive(Debug, Default)]
pub struct ListObjectsV2RequestBuilder {
    bucket: Option<BucketName>,
    prefix: Option<String>,
    delimiter: Option<String>,
    max_keys: Option<u32>,
    continuation_token: Option<String>,
    start_after: Option<String>,
}

impl ListObjectsV2RequestBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the target bucket.
    pub fn bucket(mut self, bucket: BucketName) -> Self {
        self.bucket = Some(bucket);
        self
    }

    /// Filter results to keys beginning with this prefix.
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Group keys that share a common prefix ending with this delimiter.
    pub fn delimiter(mut self, delimiter: impl Into<String>) -> Self {
        self.delimiter = Some(delimiter.into());
        self
    }

    /// Set the maximum number of keys to return (1-1000).
    pub fn max_keys(mut self, max_keys: u32) -> Self {
        self.max_keys = Some(max_keys);
        self
    }

    /// Set the continuation token for paginated results.
    pub fn continuation_token(mut self, token: impl Into<String>) -> Self {
        self.continuation_token = Some(token.into());
        self
    }

    /// List objects that appear after this key in lexicographic order.
    pub fn start_after(mut self, start_after: impl Into<String>) -> Self {
        self.start_after = Some(start_after.into());
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<ListObjectsV2Request> {
        if let Some(max_keys) = self.max_keys
            && !(1..=1000).contains(&max_keys)
        {
            return Err(OssError::InvalidParameter {
                field: "max_keys".into(),
                reason: "must be between 1 and 1000".into(),
            });
        }
        Ok(ListObjectsV2Request {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            prefix: self.prefix,
            delimiter: self.delimiter,
            max_keys: self.max_keys,
            continuation_token: self.continuation_token,
            start_after: self.start_after,
        })
    }
}

/// Request to copy an object within OSS.
#[derive(Debug)]
pub struct CopyObjectRequest {
    pub(crate) bucket: BucketName,
    pub(crate) key: ObjectKey,
    pub(crate) source_bucket: BucketName,
    pub(crate) source_key: ObjectKey,
    pub(crate) metadata_directive: Option<MetadataDirective>,
    pub(crate) content_type: Option<String>,
    pub(crate) storage_class: Option<StorageClass>,
    pub(crate) acl: Option<ObjectAcl>,
    pub(crate) metadata: HashMap<String, String>,
}

/// Builder for [`CopyObjectRequest`].
#[derive(Debug, Default)]
pub struct CopyObjectRequestBuilder {
    bucket: Option<BucketName>,
    key: Option<ObjectKey>,
    source_bucket: Option<BucketName>,
    source_key: Option<ObjectKey>,
    metadata_directive: Option<MetadataDirective>,
    content_type: Option<String>,
    storage_class: Option<StorageClass>,
    acl: Option<ObjectAcl>,
    metadata: HashMap<String, String>,
}

impl CopyObjectRequestBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the destination bucket.
    pub fn bucket(mut self, bucket: BucketName) -> Self {
        self.bucket = Some(bucket);
        self
    }

    /// Set the destination object key.
    pub fn key(mut self, key: ObjectKey) -> Self {
        self.key = Some(key);
        self
    }

    /// Set the source bucket.
    pub fn source_bucket(mut self, bucket: BucketName) -> Self {
        self.source_bucket = Some(bucket);
        self
    }

    /// Set the source object key.
    pub fn source_key(mut self, key: ObjectKey) -> Self {
        self.source_key = Some(key);
        self
    }

    /// Set the metadata directive (COPY or REPLACE).
    pub fn metadata_directive(mut self, directive: MetadataDirective) -> Self {
        self.metadata_directive = Some(directive);
        self
    }

    /// Set the content type (only used with REPLACE directive).
    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    /// Set the storage class for the destination object.
    pub fn storage_class(mut self, storage_class: StorageClass) -> Self {
        self.storage_class = Some(storage_class);
        self
    }

    /// Set the ACL for the destination object.
    pub fn acl(mut self, acl: ObjectAcl) -> Self {
        self.acl = Some(acl);
        self
    }

    /// Add a custom metadata entry (only used with REPLACE directive).
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<CopyObjectRequest> {
        for key in self.metadata.keys() {
            validate_metadata_key(key)?;
        }
        Ok(CopyObjectRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            key: self
                .key
                .ok_or_else(|| OssError::MissingField("key".into()))?,
            source_bucket: self
                .source_bucket
                .ok_or_else(|| OssError::MissingField("source_bucket".into()))?,
            source_key: self
                .source_key
                .ok_or_else(|| OssError::MissingField("source_key".into()))?,
            metadata_directive: self.metadata_directive,
            content_type: self.content_type,
            storage_class: self.storage_class,
            acl: self.acl,
            metadata: self.metadata,
        })
    }
}

/// Request to delete multiple objects from OSS in a single request.
#[derive(Debug)]
pub struct DeleteMultipleObjectsRequest {
    pub(crate) bucket: BucketName,
    pub(crate) keys: Vec<ObjectKey>,
    pub(crate) quiet: bool,
}

/// Builder for [`DeleteMultipleObjectsRequest`].
#[derive(Debug, Default)]
pub struct DeleteMultipleObjectsRequestBuilder {
    bucket: Option<BucketName>,
    keys: Vec<ObjectKey>,
    quiet: bool,
}

impl DeleteMultipleObjectsRequestBuilder {
    /// Create a new builder (quiet mode enabled by default).
    pub fn new() -> Self {
        Self {
            quiet: true,
            ..Self::default()
        }
    }

    /// Set the target bucket.
    pub fn bucket(mut self, bucket: BucketName) -> Self {
        self.bucket = Some(bucket);
        self
    }

    /// Add an object key to delete.
    pub fn key(mut self, key: ObjectKey) -> Self {
        self.keys.push(key);
        self
    }

    /// Set all object keys to delete.
    pub fn keys(mut self, keys: Vec<ObjectKey>) -> Self {
        self.keys = keys;
        self
    }

    /// Set quiet mode (default: true). When true, only errors are returned.
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<DeleteMultipleObjectsRequest> {
        let bucket = self
            .bucket
            .ok_or_else(|| OssError::MissingField("bucket".into()))?;
        if self.keys.is_empty() {
            return Err(OssError::MissingField(
                "keys (at least one key required)".into(),
            ));
        }
        if self.keys.len() > 1000 {
            return Err(OssError::InvalidParameter {
                field: "keys".into(),
                reason: "cannot delete more than 1000 objects per request".into(),
            });
        }
        Ok(DeleteMultipleObjectsRequest {
            bucket,
            keys: self.keys,
            quiet: self.quiet,
        })
    }
}

/// Request to restore an archived object so it can be downloaded.
#[derive(Debug)]
pub struct RestoreObjectRequest {
    pub(crate) bucket: BucketName,
    pub(crate) key: ObjectKey,
    pub(crate) days: u32,
}

/// Builder for [`RestoreObjectRequest`].
#[derive(Debug, Default)]
pub struct RestoreObjectRequestBuilder {
    bucket: Option<BucketName>,
    key: Option<ObjectKey>,
    days: Option<u32>,
}

impl RestoreObjectRequestBuilder {
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

    /// Set the number of days to keep the restored copy available.
    pub fn days(mut self, days: u32) -> Self {
        self.days = Some(days);
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<RestoreObjectRequest> {
        let days = self
            .days
            .ok_or_else(|| OssError::MissingField("days".into()))?;
        if days < 1 {
            return Err(OssError::InvalidParameter {
                field: "days".into(),
                reason: "must be at least 1".into(),
            });
        }
        Ok(RestoreObjectRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            key: self
                .key
                .ok_or_else(|| OssError::MissingField("key".into()))?,
            days,
        })
    }
}

/// Request to append data to an appendable object.
#[derive(Debug)]
pub struct AppendObjectRequest {
    pub(crate) bucket: BucketName,
    pub(crate) key: ObjectKey,
    pub(crate) position: u64,
    pub(crate) body: reqwest::Body,
    pub(crate) content_type: Option<String>,
}

/// Builder for [`AppendObjectRequest`].
#[derive(Debug, Default)]
pub struct AppendObjectRequestBuilder {
    bucket: Option<BucketName>,
    key: Option<ObjectKey>,
    position: Option<u64>,
    body: Option<reqwest::Body>,
    content_type: Option<String>,
}

impl AppendObjectRequestBuilder {
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

    /// Set the byte offset to append at (0 for new objects).
    pub fn position(mut self, position: u64) -> Self {
        self.position = Some(position);
        self
    }

    /// Set the request body.
    pub fn body(mut self, body: impl Into<reqwest::Body>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Set the content type.
    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<AppendObjectRequest> {
        Ok(AppendObjectRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            key: self
                .key
                .ok_or_else(|| OssError::MissingField("key".into()))?,
            position: self
                .position
                .ok_or_else(|| OssError::MissingField("position".into()))?,
            body: self
                .body
                .ok_or_else(|| OssError::MissingField("body".into()))?,
            content_type: self.content_type,
        })
    }
}

/// Request to get the ACL of an object.
#[derive(Debug)]
pub struct GetObjectAclRequest {
    pub(crate) bucket: BucketName,
    pub(crate) key: ObjectKey,
}

/// Builder for [`GetObjectAclRequest`].
#[derive(Debug, Default)]
pub struct GetObjectAclRequestBuilder {
    bucket: Option<BucketName>,
    key: Option<ObjectKey>,
}

impl GetObjectAclRequestBuilder {
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

    /// Build the request.
    pub fn build(self) -> Result<GetObjectAclRequest> {
        Ok(GetObjectAclRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            key: self
                .key
                .ok_or_else(|| OssError::MissingField("key".into()))?,
        })
    }
}

/// Request to set the ACL of an object.
#[derive(Debug)]
pub struct PutObjectAclRequest {
    pub(crate) bucket: BucketName,
    pub(crate) key: ObjectKey,
    pub(crate) acl: ObjectAcl,
}

/// Builder for [`PutObjectAclRequest`].
#[derive(Debug, Default)]
pub struct PutObjectAclRequestBuilder {
    bucket: Option<BucketName>,
    key: Option<ObjectKey>,
    acl: Option<ObjectAcl>,
}

impl PutObjectAclRequestBuilder {
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

    /// Set the ACL to apply.
    pub fn acl(mut self, acl: ObjectAcl) -> Self {
        self.acl = Some(acl);
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<PutObjectAclRequest> {
        Ok(PutObjectAclRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            key: self
                .key
                .ok_or_else(|| OssError::MissingField("key".into()))?,
            acl: self
                .acl
                .ok_or_else(|| OssError::MissingField("acl".into()))?,
        })
    }
}

/// Request to get the tags of an object.
#[derive(Debug)]
pub struct GetObjectTaggingRequest {
    pub(crate) bucket: BucketName,
    pub(crate) key: ObjectKey,
}

/// Builder for [`GetObjectTaggingRequest`].
#[derive(Debug, Default)]
pub struct GetObjectTaggingRequestBuilder {
    bucket: Option<BucketName>,
    key: Option<ObjectKey>,
}

impl GetObjectTaggingRequestBuilder {
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

    /// Build the request.
    pub fn build(self) -> Result<GetObjectTaggingRequest> {
        Ok(GetObjectTaggingRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            key: self
                .key
                .ok_or_else(|| OssError::MissingField("key".into()))?,
        })
    }
}

/// Request to set the tags of an object.
#[derive(Debug)]
pub struct PutObjectTaggingRequest {
    pub(crate) bucket: BucketName,
    pub(crate) key: ObjectKey,
    pub(crate) tags: Vec<(String, String)>,
}

/// Builder for [`PutObjectTaggingRequest`].
#[derive(Debug, Default)]
pub struct PutObjectTaggingRequestBuilder {
    bucket: Option<BucketName>,
    key: Option<ObjectKey>,
    tags: Vec<(String, String)>,
}

impl PutObjectTaggingRequestBuilder {
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

    /// Add a tag key-value pair.
    pub fn tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.push((key.into(), value.into()));
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<PutObjectTaggingRequest> {
        if self.tags.is_empty() {
            return Err(OssError::MissingField(
                "tags (at least one tag required)".into(),
            ));
        }
        if self.tags.len() > 10 {
            return Err(OssError::InvalidParameter {
                field: "tags".into(),
                reason: "cannot set more than 10 tags per object".into(),
            });
        }
        Ok(PutObjectTaggingRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            key: self
                .key
                .ok_or_else(|| OssError::MissingField("key".into()))?,
            tags: self.tags,
        })
    }
}

/// Request to delete all tags from an object.
#[derive(Debug)]
pub struct DeleteObjectTaggingRequest {
    pub(crate) bucket: BucketName,
    pub(crate) key: ObjectKey,
}

/// Builder for [`DeleteObjectTaggingRequest`].
#[derive(Debug, Default)]
pub struct DeleteObjectTaggingRequestBuilder {
    bucket: Option<BucketName>,
    key: Option<ObjectKey>,
}

impl DeleteObjectTaggingRequestBuilder {
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

    /// Build the request.
    pub fn build(self) -> Result<DeleteObjectTaggingRequest> {
        Ok(DeleteObjectTaggingRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            key: self
                .key
                .ok_or_else(|| OssError::MissingField("key".into()))?,
        })
    }
}

/// XML wrapper for serializing the DeleteMultipleObjects body.
#[derive(Debug, Serialize)]
#[serde(rename = "Delete")]
pub(crate) struct DeleteMultipleObjectsXml {
    /// Whether to use quiet mode.
    #[serde(rename = "Quiet")]
    pub quiet: bool,
    /// The objects to delete.
    #[serde(rename = "Object")]
    pub objects: Vec<DeleteObjectXmlEntry>,
}

/// Single object entry in the DeleteMultipleObjects XML body.
#[derive(Debug, Serialize)]
pub(crate) struct DeleteObjectXmlEntry {
    /// The object key.
    #[serde(rename = "Key")]
    pub key: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_object_request_builder() {
        let req = PutObjectRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("test.txt").unwrap())
            .body(b"hello".to_vec())
            .content_type("text/plain")
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn put_object_request_missing_bucket() {
        let req = PutObjectRequestBuilder::new()
            .key(ObjectKey::new("test.txt").unwrap())
            .body(b"hello".to_vec())
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn get_object_request_with_range() {
        let req = GetObjectRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("test.txt").unwrap())
            .range("bytes=0-999")
            .build();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.range.as_deref(), Some("bytes=0-999"));
    }

    #[test]
    fn delete_object_request_builder() {
        let req = DeleteObjectRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("test.txt").unwrap())
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn head_object_request_builder() {
        let req = HeadObjectRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("test.txt").unwrap())
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn list_objects_v2_request_builder() {
        let req = ListObjectsV2RequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .prefix("photos/")
            .delimiter("/")
            .max_keys(50)
            .build();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.prefix.as_deref(), Some("photos/"));
        assert_eq!(req.delimiter.as_deref(), Some("/"));
        assert_eq!(req.max_keys, Some(50));
    }

    #[test]
    fn list_objects_v2_request_missing_bucket() {
        let req = ListObjectsV2RequestBuilder::new().prefix("test/").build();
        assert!(req.is_err());
    }

    #[test]
    fn put_object_with_metadata() {
        let req = PutObjectRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("test.txt").unwrap())
            .body(b"hello".to_vec())
            .metadata("author", "test")
            .metadata("project", "demo")
            .build()
            .unwrap();
        assert_eq!(req.metadata.len(), 2);
        assert_eq!(req.metadata.get("author").unwrap(), "test");
    }

    #[test]
    fn list_objects_v2_max_keys_zero_fails() {
        let req = ListObjectsV2RequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .max_keys(0)
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn list_objects_v2_max_keys_1001_fails() {
        let req = ListObjectsV2RequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .max_keys(1001)
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn copy_object_request_builder() {
        let req = CopyObjectRequestBuilder::new()
            .bucket(BucketName::new("dest-bucket").unwrap())
            .key(ObjectKey::new("dest/key.txt").unwrap())
            .source_bucket(BucketName::new("src-bucket").unwrap())
            .source_key(ObjectKey::new("src/key.txt").unwrap())
            .metadata_directive(MetadataDirective::Copy)
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn copy_object_request_missing_source() {
        let req = CopyObjectRequestBuilder::new()
            .bucket(BucketName::new("dest-bucket").unwrap())
            .key(ObjectKey::new("dest/key.txt").unwrap())
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn delete_multiple_objects_request_builder() {
        let req = DeleteMultipleObjectsRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("file1.txt").unwrap())
            .key(ObjectKey::new("file2.txt").unwrap())
            .build();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.keys.len(), 2);
        assert!(req.quiet);
    }

    #[test]
    fn delete_multiple_objects_request_empty_keys_fails() {
        let req = DeleteMultipleObjectsRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn delete_multiple_objects_xml_serializes() {
        let xml_body = DeleteMultipleObjectsXml {
            quiet: true,
            objects: vec![
                DeleteObjectXmlEntry {
                    key: "key1".to_string(),
                },
                DeleteObjectXmlEntry {
                    key: "key2".to_string(),
                },
            ],
        };
        let xml = quick_xml::se::to_string(&xml_body).unwrap();
        assert!(xml.contains("<Quiet>true</Quiet>"));
        assert!(xml.contains("<Key>key1</Key>"));
        assert!(xml.contains("<Key>key2</Key>"));
    }

    #[test]
    fn metadata_key_with_spaces_fails() {
        let req = PutObjectRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("test.txt").unwrap())
            .body(b"hello".to_vec())
            .metadata("invalid key", "value")
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn metadata_key_valid_passes() {
        let req = PutObjectRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("test.txt").unwrap())
            .body(b"hello".to_vec())
            .metadata("valid-key_1", "value")
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn restore_object_request_builder() {
        let req = RestoreObjectRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("archived.bin").unwrap())
            .days(3)
            .build();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.days, 3);
    }

    #[test]
    fn restore_object_missing_days() {
        let req = RestoreObjectRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("archived.bin").unwrap())
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn restore_object_days_zero_fails() {
        let req = RestoreObjectRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("archived.bin").unwrap())
            .days(0)
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn append_object_request_builder() {
        let req = AppendObjectRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("append.log").unwrap())
            .position(0)
            .body(b"log line".to_vec())
            .content_type("text/plain")
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn append_object_missing_body() {
        let req = AppendObjectRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("append.log").unwrap())
            .position(0)
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn get_object_acl_request_builder() {
        let req = GetObjectAclRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("file.txt").unwrap())
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn get_object_acl_missing_key() {
        let req = GetObjectAclRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn put_object_acl_request_builder() {
        let req = PutObjectAclRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("file.txt").unwrap())
            .acl(ObjectAcl::PublicRead)
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn put_object_acl_missing_acl() {
        let req = PutObjectAclRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("file.txt").unwrap())
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn get_object_tagging_request_builder() {
        let req = GetObjectTaggingRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("file.txt").unwrap())
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn put_object_tagging_request_builder() {
        let req = PutObjectTaggingRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("file.txt").unwrap())
            .tag("env", "prod")
            .tag("team", "backend")
            .build();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.tags.len(), 2);
    }

    #[test]
    fn put_object_tagging_empty_tags_fails() {
        let req = PutObjectTaggingRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("file.txt").unwrap())
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn put_object_tagging_over_10_tags_fails() {
        let mut builder = PutObjectTaggingRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("file.txt").unwrap());
        for i in 0..11 {
            builder = builder.tag(format!("key{i}"), format!("val{i}"));
        }
        assert!(builder.build().is_err());
    }

    #[test]
    fn delete_object_tagging_request_builder() {
        let req = DeleteObjectTaggingRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("file.txt").unwrap())
            .build();
        assert!(req.is_ok());
    }
}

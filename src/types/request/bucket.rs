//! Bucket operation request types: Create, Delete, List, GetInfo.

use crate::error::{OssError, Result};
use crate::types::common::{BucketName, StorageClass};

/// Request to create a new bucket.
#[derive(Debug)]
pub struct CreateBucketRequest {
    pub(crate) bucket: BucketName,
    pub(crate) storage_class: Option<StorageClass>,
}

/// Builder for [`CreateBucketRequest`].
#[derive(Debug, Default)]
pub struct CreateBucketRequestBuilder {
    bucket: Option<BucketName>,
    storage_class: Option<StorageClass>,
}

impl CreateBucketRequestBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the bucket name.
    pub fn bucket(mut self, bucket: BucketName) -> Self {
        self.bucket = Some(bucket);
        self
    }

    /// Set the default storage class for the bucket.
    pub fn storage_class(mut self, storage_class: StorageClass) -> Self {
        self.storage_class = Some(storage_class);
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<CreateBucketRequest> {
        Ok(CreateBucketRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            storage_class: self.storage_class,
        })
    }
}

/// Request to delete a bucket.
#[derive(Debug)]
pub struct DeleteBucketRequest {
    pub(crate) bucket: BucketName,
}

/// Builder for [`DeleteBucketRequest`].
#[derive(Debug, Default)]
pub struct DeleteBucketRequestBuilder {
    bucket: Option<BucketName>,
}

impl DeleteBucketRequestBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the bucket name.
    pub fn bucket(mut self, bucket: BucketName) -> Self {
        self.bucket = Some(bucket);
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<DeleteBucketRequest> {
        Ok(DeleteBucketRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
        })
    }
}

/// Request to list all buckets owned by the authenticated user.
#[derive(Debug, Default)]
pub struct ListBucketsRequest {
    pub(crate) prefix: Option<String>,
    pub(crate) marker: Option<String>,
    pub(crate) max_keys: Option<u32>,
}

/// Builder for [`ListBucketsRequest`].
#[derive(Debug, Default)]
pub struct ListBucketsRequestBuilder {
    prefix: Option<String>,
    marker: Option<String>,
    max_keys: Option<u32>,
}

impl ListBucketsRequestBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter buckets whose names begin with this prefix.
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Set the marker for paginated results.
    pub fn marker(mut self, marker: impl Into<String>) -> Self {
        self.marker = Some(marker.into());
        self
    }

    /// Set the maximum number of buckets to return.
    pub fn max_keys(mut self, max_keys: u32) -> Self {
        self.max_keys = Some(max_keys);
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<ListBucketsRequest> {
        Ok(ListBucketsRequest {
            prefix: self.prefix,
            marker: self.marker,
            max_keys: self.max_keys,
        })
    }
}

/// Request to get the region/location of a bucket.
#[derive(Debug)]
pub struct GetBucketLocationRequest {
    pub(crate) bucket: BucketName,
}

/// Builder for [`GetBucketLocationRequest`].
#[derive(Debug, Default)]
pub struct GetBucketLocationRequestBuilder {
    bucket: Option<BucketName>,
}

impl GetBucketLocationRequestBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the bucket name.
    pub fn bucket(mut self, bucket: BucketName) -> Self {
        self.bucket = Some(bucket);
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<GetBucketLocationRequest> {
        Ok(GetBucketLocationRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
        })
    }
}

/// Request to retrieve bucket metadata and configuration.
#[derive(Debug)]
pub struct GetBucketInfoRequest {
    pub(crate) bucket: BucketName,
}

/// Builder for [`GetBucketInfoRequest`].
#[derive(Debug, Default)]
pub struct GetBucketInfoRequestBuilder {
    bucket: Option<BucketName>,
}

impl GetBucketInfoRequestBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the bucket name.
    pub fn bucket(mut self, bucket: BucketName) -> Self {
        self.bucket = Some(bucket);
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<GetBucketInfoRequest> {
        Ok(GetBucketInfoRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_bucket_request_builder() {
        let req = CreateBucketRequestBuilder::new()
            .bucket(BucketName::new("new-bucket").unwrap())
            .storage_class(StorageClass::InfrequentAccess)
            .build();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.storage_class, Some(StorageClass::InfrequentAccess));
    }

    #[test]
    fn delete_bucket_request_builder() {
        let req = DeleteBucketRequestBuilder::new()
            .bucket(BucketName::new("old-bucket").unwrap())
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn list_buckets_request_builder() {
        let req = ListBucketsRequestBuilder::new()
            .prefix("my-")
            .marker("my-bucket-01")
            .max_keys(10)
            .build();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.prefix.as_deref(), Some("my-"));
        assert_eq!(req.marker.as_deref(), Some("my-bucket-01"));
        assert_eq!(req.max_keys, Some(10));
    }

    #[test]
    fn get_bucket_info_request_builder() {
        let req = GetBucketInfoRequestBuilder::new()
            .bucket(BucketName::new("info-bucket").unwrap())
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn get_bucket_location_request_builder() {
        let req = GetBucketLocationRequestBuilder::new()
            .bucket(BucketName::new("loc-bucket").unwrap())
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn get_bucket_location_missing_bucket() {
        let req = GetBucketLocationRequestBuilder::new().build();
        assert!(req.is_err());
    }
}

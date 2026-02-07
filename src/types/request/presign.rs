//! Presigned URL request types.

use crate::error::{OssError, Result};
use crate::types::common::{BucketName, ObjectKey};

/// Request to generate a presigned URL.
#[derive(Debug)]
pub struct PresignedUrlRequest {
    pub(crate) bucket: BucketName,
    pub(crate) key: ObjectKey,
    pub(crate) expires: std::time::Duration,
    pub(crate) content_type: Option<String>,
    pub(crate) datetime: Option<chrono::DateTime<chrono::Utc>>,
}

/// Builder for [`PresignedUrlRequest`].
#[derive(Debug, Default)]
pub struct PresignedUrlRequestBuilder {
    bucket: Option<BucketName>,
    key: Option<ObjectKey>,
    expires: Option<std::time::Duration>,
    content_type: Option<String>,
    datetime: Option<chrono::DateTime<chrono::Utc>>,
}

impl PresignedUrlRequestBuilder {
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

    /// Set the URL expiration duration (default: 1 hour, max: 7 days).
    pub fn expires(mut self, expires: std::time::Duration) -> Self {
        self.expires = Some(expires);
        self
    }

    /// Set the content type (useful for PUT presigned URLs).
    pub fn content_type(mut self, ct: impl Into<String>) -> Self {
        self.content_type = Some(ct.into());
        self
    }

    /// Set a specific datetime for signing (default: current time).
    /// Useful for testing to produce deterministic signatures.
    pub fn datetime(mut self, dt: chrono::DateTime<chrono::Utc>) -> Self {
        self.datetime = Some(dt);
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<PresignedUrlRequest> {
        let expires = self.expires.unwrap_or(std::time::Duration::from_secs(3600));
        if expires.is_zero() {
            return Err(OssError::InvalidParameter {
                field: "expires".into(),
                reason: "must be at least 1 second".into(),
            });
        }
        if expires.as_secs() > 604800 {
            return Err(OssError::InvalidParameter {
                field: "expires".into(),
                reason: "cannot exceed 7 days (604800 seconds)".into(),
            });
        }
        Ok(PresignedUrlRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            key: self
                .key
                .ok_or_else(|| OssError::MissingField("key".into()))?,
            expires,
            content_type: self.content_type,
            datetime: self.datetime,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presign_url_default_expires_one_hour() {
        let request = PresignedUrlRequestBuilder::new()
            .bucket(BucketName::new("my-bucket").unwrap())
            .key(ObjectKey::new("file.txt").unwrap())
            .build()
            .unwrap();
        assert_eq!(request.expires.as_secs(), 3600);
    }

    #[test]
    fn presign_url_rejects_expires_over_7_days() {
        let result = PresignedUrlRequestBuilder::new()
            .bucket(BucketName::new("my-bucket").unwrap())
            .key(ObjectKey::new("file.txt").unwrap())
            .expires(std::time::Duration::from_secs(604801))
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn presign_url_rejects_zero_expires() {
        let result = PresignedUrlRequestBuilder::new()
            .bucket(BucketName::new("my-bucket").unwrap())
            .key(ObjectKey::new("file.txt").unwrap())
            .expires(std::time::Duration::ZERO)
            .build();
        assert!(result.is_err());
    }
}

//! Multipart upload request types: Initiate, UploadPart, Complete, Abort, ListParts.

use serde::Serialize;

use crate::error::{OssError, Result};
use crate::types::common::{BucketName, ObjectKey, StorageClass};

/// Request to initiate a multipart upload.
#[derive(Debug)]
pub struct InitiateMultipartUploadRequest {
    pub(crate) bucket: BucketName,
    pub(crate) key: ObjectKey,
    pub(crate) content_type: Option<String>,
    pub(crate) storage_class: Option<StorageClass>,
}

/// Builder for [`InitiateMultipartUploadRequest`].
#[derive(Debug, Default)]
pub struct InitiateMultipartUploadRequestBuilder {
    bucket: Option<BucketName>,
    key: Option<ObjectKey>,
    content_type: Option<String>,
    storage_class: Option<StorageClass>,
}

impl InitiateMultipartUploadRequestBuilder {
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
    pub fn build(self) -> Result<InitiateMultipartUploadRequest> {
        Ok(InitiateMultipartUploadRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            key: self
                .key
                .ok_or_else(|| OssError::MissingField("key".into()))?,
            content_type: self.content_type,
            storage_class: self.storage_class,
        })
    }
}

/// Request to upload a single part in a multipart upload.
#[derive(Debug)]
pub struct UploadPartRequest {
    pub(crate) bucket: BucketName,
    pub(crate) key: ObjectKey,
    pub(crate) upload_id: String,
    pub(crate) part_number: u32,
    pub(crate) body: reqwest::Body,
}

/// Builder for [`UploadPartRequest`].
#[derive(Debug, Default)]
pub struct UploadPartRequestBuilder {
    bucket: Option<BucketName>,
    key: Option<ObjectKey>,
    upload_id: Option<String>,
    part_number: Option<u32>,
    body: Option<reqwest::Body>,
}

impl UploadPartRequestBuilder {
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

    /// Set the upload ID from InitiateMultipartUpload.
    pub fn upload_id(mut self, upload_id: impl Into<String>) -> Self {
        self.upload_id = Some(upload_id.into());
        self
    }

    /// Set the part number (1-10000).
    pub fn part_number(mut self, part_number: u32) -> Self {
        self.part_number = Some(part_number);
        self
    }

    /// Set the part body.
    pub fn body(mut self, body: impl Into<reqwest::Body>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<UploadPartRequest> {
        let part_number = self
            .part_number
            .ok_or_else(|| OssError::MissingField("part_number".into()))?;
        if !(1..=10000).contains(&part_number) {
            return Err(OssError::InvalidParameter {
                field: "part_number".into(),
                reason: "must be between 1 and 10000".into(),
            });
        }
        Ok(UploadPartRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            key: self
                .key
                .ok_or_else(|| OssError::MissingField("key".into()))?,
            upload_id: self
                .upload_id
                .ok_or_else(|| OssError::MissingField("upload_id".into()))?,
            part_number,
            body: self
                .body
                .ok_or_else(|| OssError::MissingField("body".into()))?,
        })
    }
}

/// A completed part reference used when completing a multipart upload.
#[derive(Debug, Clone, Serialize)]
#[serde(rename = "Part")]
pub struct CompletedPart {
    /// The part number.
    #[serde(rename = "PartNumber")]
    pub part_number: u32,
    /// The ETag returned when the part was uploaded.
    #[serde(rename = "ETag")]
    pub etag: String,
}

/// Request to complete a multipart upload.
#[derive(Debug)]
pub struct CompleteMultipartUploadRequest {
    pub(crate) bucket: BucketName,
    pub(crate) key: ObjectKey,
    pub(crate) upload_id: String,
    pub(crate) parts: Vec<CompletedPart>,
}

/// Builder for [`CompleteMultipartUploadRequest`].
#[derive(Debug, Default)]
pub struct CompleteMultipartUploadRequestBuilder {
    bucket: Option<BucketName>,
    key: Option<ObjectKey>,
    upload_id: Option<String>,
    parts: Vec<CompletedPart>,
}

impl CompleteMultipartUploadRequestBuilder {
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

    /// Set the upload ID.
    pub fn upload_id(mut self, upload_id: impl Into<String>) -> Self {
        self.upload_id = Some(upload_id.into());
        self
    }

    /// Add a completed part.
    pub fn part(mut self, part: CompletedPart) -> Self {
        self.parts.push(part);
        self
    }

    /// Set all completed parts at once.
    pub fn parts(mut self, parts: Vec<CompletedPart>) -> Self {
        self.parts = parts;
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<CompleteMultipartUploadRequest> {
        Ok(CompleteMultipartUploadRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            key: self
                .key
                .ok_or_else(|| OssError::MissingField("key".into()))?,
            upload_id: self
                .upload_id
                .ok_or_else(|| OssError::MissingField("upload_id".into()))?,
            parts: self.parts,
        })
    }
}

/// Request to abort a multipart upload.
#[derive(Debug)]
pub struct AbortMultipartUploadRequest {
    pub(crate) bucket: BucketName,
    pub(crate) key: ObjectKey,
    pub(crate) upload_id: String,
}

/// Builder for [`AbortMultipartUploadRequest`].
#[derive(Debug, Default)]
pub struct AbortMultipartUploadRequestBuilder {
    bucket: Option<BucketName>,
    key: Option<ObjectKey>,
    upload_id: Option<String>,
}

impl AbortMultipartUploadRequestBuilder {
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

    /// Set the upload ID.
    pub fn upload_id(mut self, upload_id: impl Into<String>) -> Self {
        self.upload_id = Some(upload_id.into());
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<AbortMultipartUploadRequest> {
        Ok(AbortMultipartUploadRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            key: self
                .key
                .ok_or_else(|| OssError::MissingField("key".into()))?,
            upload_id: self
                .upload_id
                .ok_or_else(|| OssError::MissingField("upload_id".into()))?,
        })
    }
}

/// Request to list parts of a multipart upload.
#[derive(Debug)]
pub struct ListPartsRequest {
    pub(crate) bucket: BucketName,
    pub(crate) key: ObjectKey,
    pub(crate) upload_id: String,
    pub(crate) max_parts: Option<u32>,
    pub(crate) part_number_marker: Option<u32>,
}

/// Builder for [`ListPartsRequest`].
#[derive(Debug, Default)]
pub struct ListPartsRequestBuilder {
    bucket: Option<BucketName>,
    key: Option<ObjectKey>,
    upload_id: Option<String>,
    max_parts: Option<u32>,
    part_number_marker: Option<u32>,
}

impl ListPartsRequestBuilder {
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

    /// Set the upload ID.
    pub fn upload_id(mut self, upload_id: impl Into<String>) -> Self {
        self.upload_id = Some(upload_id.into());
        self
    }

    /// Set the maximum number of parts to return.
    pub fn max_parts(mut self, max_parts: u32) -> Self {
        self.max_parts = Some(max_parts);
        self
    }

    /// Set the part number marker for pagination.
    pub fn part_number_marker(mut self, marker: u32) -> Self {
        self.part_number_marker = Some(marker);
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<ListPartsRequest> {
        Ok(ListPartsRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            key: self
                .key
                .ok_or_else(|| OssError::MissingField("key".into()))?,
            upload_id: self
                .upload_id
                .ok_or_else(|| OssError::MissingField("upload_id".into()))?,
            max_parts: self.max_parts,
            part_number_marker: self.part_number_marker,
        })
    }
}

/// Request to list in-progress multipart uploads for a bucket.
#[derive(Debug)]
pub struct ListMultipartUploadsRequest {
    pub(crate) bucket: BucketName,
    pub(crate) prefix: Option<String>,
    pub(crate) delimiter: Option<String>,
    pub(crate) max_uploads: Option<u32>,
    pub(crate) key_marker: Option<String>,
    pub(crate) upload_id_marker: Option<String>,
}

/// Builder for [`ListMultipartUploadsRequest`].
#[derive(Debug, Default)]
pub struct ListMultipartUploadsRequestBuilder {
    bucket: Option<BucketName>,
    prefix: Option<String>,
    delimiter: Option<String>,
    max_uploads: Option<u32>,
    key_marker: Option<String>,
    upload_id_marker: Option<String>,
}

impl ListMultipartUploadsRequestBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the target bucket.
    pub fn bucket(mut self, bucket: BucketName) -> Self {
        self.bucket = Some(bucket);
        self
    }

    /// Filter uploads to keys beginning with this prefix.
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Group uploads by this delimiter.
    pub fn delimiter(mut self, delimiter: impl Into<String>) -> Self {
        self.delimiter = Some(delimiter.into());
        self
    }

    /// Set the maximum number of uploads to return (1-1000).
    pub fn max_uploads(mut self, max_uploads: u32) -> Self {
        self.max_uploads = Some(max_uploads);
        self
    }

    /// Set the key marker for paginated results.
    pub fn key_marker(mut self, key_marker: impl Into<String>) -> Self {
        self.key_marker = Some(key_marker.into());
        self
    }

    /// Set the upload ID marker for paginated results.
    pub fn upload_id_marker(mut self, upload_id_marker: impl Into<String>) -> Self {
        self.upload_id_marker = Some(upload_id_marker.into());
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<ListMultipartUploadsRequest> {
        if let Some(max_uploads) = self.max_uploads
            && !(1..=1000).contains(&max_uploads)
        {
            return Err(OssError::InvalidParameter {
                field: "max_uploads".into(),
                reason: "must be between 1 and 1000".into(),
            });
        }
        Ok(ListMultipartUploadsRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            prefix: self.prefix,
            delimiter: self.delimiter,
            max_uploads: self.max_uploads,
            key_marker: self.key_marker,
            upload_id_marker: self.upload_id_marker,
        })
    }
}

/// XML wrapper for serializing the CompleteMultipartUpload body.
#[derive(Debug, Serialize)]
#[serde(rename = "CompleteMultipartUpload")]
pub struct CompleteMultipartUploadXml {
    /// The completed parts.
    #[serde(rename = "Part")]
    pub parts: Vec<CompletedPart>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initiate_multipart_upload_request_builder() {
        let req = InitiateMultipartUploadRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("large-file.bin").unwrap())
            .content_type("application/octet-stream")
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn upload_part_request_builder() {
        let req = UploadPartRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("large-file.bin").unwrap())
            .upload_id("test-upload-id")
            .part_number(1)
            .body(b"part-data".to_vec())
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn upload_part_request_missing_upload_id() {
        let req = UploadPartRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("large-file.bin").unwrap())
            .part_number(1)
            .body(b"part-data".to_vec())
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn complete_multipart_upload_request_builder() {
        let req = CompleteMultipartUploadRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("large-file.bin").unwrap())
            .upload_id("test-upload-id")
            .part(CompletedPart {
                part_number: 1,
                etag: "etag1".to_string(),
            })
            .part(CompletedPart {
                part_number: 2,
                etag: "etag2".to_string(),
            })
            .build();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.parts.len(), 2);
    }

    #[test]
    fn abort_multipart_upload_request_builder() {
        let req = AbortMultipartUploadRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("large-file.bin").unwrap())
            .upload_id("test-upload-id")
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn list_parts_request_builder() {
        let req = ListPartsRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("large-file.bin").unwrap())
            .upload_id("test-upload-id")
            .max_parts(100)
            .part_number_marker(5)
            .build();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.max_parts, Some(100));
        assert_eq!(req.part_number_marker, Some(5));
    }

    #[test]
    fn list_parts_request_missing_upload_id() {
        let req = ListPartsRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("large-file.bin").unwrap())
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn completed_part_serializes_to_xml() {
        let parts = vec![
            CompletedPart {
                part_number: 1,
                etag: "\"etag1\"".to_string(),
            },
            CompletedPart {
                part_number: 2,
                etag: "\"etag2\"".to_string(),
            },
        ];
        let wrapper = CompleteMultipartUploadXml { parts };
        let xml = quick_xml::se::to_string(&wrapper).unwrap();
        assert!(xml.contains("<PartNumber>1</PartNumber>"));
        assert!(xml.contains("<PartNumber>2</PartNumber>"));
        assert!(xml.contains("<ETag>\"etag1\"</ETag>"));
    }

    #[test]
    fn upload_part_request_part_number_zero_fails() {
        let req = UploadPartRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("file.bin").unwrap())
            .upload_id("uid")
            .part_number(0)
            .body(b"data".to_vec())
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn upload_part_request_part_number_10001_fails() {
        let req = UploadPartRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("file.bin").unwrap())
            .upload_id("uid")
            .part_number(10001)
            .body(b"data".to_vec())
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn upload_part_request_part_number_10000_ok() {
        let req = UploadPartRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .key(ObjectKey::new("file.bin").unwrap())
            .upload_id("uid")
            .part_number(10000)
            .body(b"data".to_vec())
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn list_multipart_uploads_request_builder() {
        let req = ListMultipartUploadsRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .prefix("uploads/")
            .max_uploads(50)
            .build();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.prefix.as_deref(), Some("uploads/"));
        assert_eq!(req.max_uploads, Some(50));
    }

    #[test]
    fn list_multipart_uploads_missing_bucket() {
        let req = ListMultipartUploadsRequestBuilder::new()
            .prefix("test/")
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn list_multipart_uploads_max_uploads_zero_fails() {
        let req = ListMultipartUploadsRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .max_uploads(0)
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn list_multipart_uploads_max_uploads_1001_fails() {
        let req = ListMultipartUploadsRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .max_uploads(1001)
            .build();
        assert!(req.is_err());
    }
}

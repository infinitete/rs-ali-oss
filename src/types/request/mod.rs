//! Request types and builders for OSS operations.

mod bucket;
mod multipart;
mod object;
mod presign;

use crate::error::{OssError, Result};

/// Validate that a metadata key contains only ASCII alphanumeric, hyphens, and underscores.
fn validate_metadata_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(OssError::InvalidParameter {
            field: "metadata key".into(),
            reason: "must not be empty".into(),
        });
    }
    if !key
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(OssError::InvalidParameter {
            field: "metadata key".into(),
            reason: format!(
                "contains invalid characters: '{}'. Only ASCII alphanumeric, hyphens, and underscores are allowed",
                key
            ),
        });
    }
    Ok(())
}

pub use bucket::{
    CreateBucketRequest, CreateBucketRequestBuilder, DeleteBucketRequest,
    DeleteBucketRequestBuilder, GetBucketInfoRequest, GetBucketInfoRequestBuilder,
    GetBucketLocationRequest, GetBucketLocationRequestBuilder, ListBucketsRequest,
    ListBucketsRequestBuilder,
};
pub use multipart::{
    AbortMultipartUploadRequest, AbortMultipartUploadRequestBuilder,
    CompleteMultipartUploadRequest, CompleteMultipartUploadRequestBuilder,
    CompleteMultipartUploadXml, CompletedPart, InitiateMultipartUploadRequest,
    InitiateMultipartUploadRequestBuilder, ListMultipartUploadsRequest,
    ListMultipartUploadsRequestBuilder, ListPartsRequest, ListPartsRequestBuilder,
    UploadPartRequest, UploadPartRequestBuilder,
};
pub use object::{
    AppendObjectRequest, AppendObjectRequestBuilder, CopyObjectRequest, CopyObjectRequestBuilder,
    DeleteMultipleObjectsRequest, DeleteMultipleObjectsRequestBuilder, DeleteObjectRequest,
    DeleteObjectRequestBuilder, DeleteObjectTaggingRequest, DeleteObjectTaggingRequestBuilder,
    GetObjectAclRequest, GetObjectAclRequestBuilder, GetObjectRequest, GetObjectRequestBuilder,
    GetObjectTaggingRequest, GetObjectTaggingRequestBuilder, HeadObjectRequest,
    HeadObjectRequestBuilder, ListObjectsV2Request, ListObjectsV2RequestBuilder,
    PutObjectAclRequest, PutObjectAclRequestBuilder, PutObjectRequest, PutObjectRequestBuilder,
    PutObjectTaggingRequest, PutObjectTaggingRequestBuilder, RestoreObjectRequest,
    RestoreObjectRequestBuilder,
};
pub(crate) use object::{DeleteMultipleObjectsXml, DeleteObjectXmlEntry};
pub use presign::{PresignedUrlRequest, PresignedUrlRequestBuilder};

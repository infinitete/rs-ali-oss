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

pub(crate) use bucket::{
    CorsConfigurationXml, CorsRuleXml, EncryptionConfigurationXml, EncryptionRuleXml,
    LifecycleConfigurationXml, LifecycleExpirationXml, LifecycleRuleXml, LifecycleTransitionXml,
    LoggingConfigurationXml, LoggingEnabledXml, RefererBlacklistXml, RefererConfigurationXml,
    RefererListXml, VersioningConfigurationXml,
};
pub use bucket::{
    CorsRule, CreateBucketRequest, CreateBucketRequestBuilder, DeleteBucketCorsRequest,
    DeleteBucketCorsRequestBuilder, DeleteBucketEncryptionRequest,
    DeleteBucketEncryptionRequestBuilder, DeleteBucketLifecycleRequest,
    DeleteBucketLifecycleRequestBuilder, DeleteBucketLoggingRequest,
    DeleteBucketLoggingRequestBuilder, DeleteBucketPolicyRequest, DeleteBucketPolicyRequestBuilder,
    DeleteBucketRequest, DeleteBucketRequestBuilder, GetBucketAclRequest,
    GetBucketAclRequestBuilder, GetBucketCorsRequest, GetBucketCorsRequestBuilder,
    GetBucketEncryptionRequest, GetBucketEncryptionRequestBuilder, GetBucketInfoRequest,
    GetBucketInfoRequestBuilder, GetBucketLifecycleRequest, GetBucketLifecycleRequestBuilder,
    GetBucketLocationRequest, GetBucketLocationRequestBuilder, GetBucketLoggingRequest,
    GetBucketLoggingRequestBuilder, GetBucketPolicyRequest, GetBucketPolicyRequestBuilder,
    GetBucketRefererRequest, GetBucketRefererRequestBuilder, GetBucketVersioningRequest,
    GetBucketVersioningRequestBuilder, LifecycleExpiration, LifecycleRule, LifecycleRuleStatus,
    LifecycleTransition, ListBucketsRequest, ListBucketsRequestBuilder, PutBucketAclRequest,
    PutBucketAclRequestBuilder, PutBucketCorsRequest, PutBucketCorsRequestBuilder,
    PutBucketEncryptionRequest, PutBucketEncryptionRequestBuilder, PutBucketLifecycleRequest,
    PutBucketLifecycleRequestBuilder, PutBucketLoggingRequest, PutBucketLoggingRequestBuilder,
    PutBucketPolicyRequest, PutBucketPolicyRequestBuilder, PutBucketRefererRequest,
    PutBucketRefererRequestBuilder, PutBucketVersioningRequest, PutBucketVersioningRequestBuilder,
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

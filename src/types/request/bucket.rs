//! Bucket operation request types: Create, Delete, List, GetInfo, ACL, CORS, Referer, Policy, Versioning, Lifecycle, Encryption, Logging.

use serde::{Deserialize, Serialize};

use crate::error::{OssError, Result};
use crate::types::common::{BucketAcl, BucketName, ServerSideEncryption, StorageClass};

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
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the bucket name.
    pub fn bucket(mut self, bucket: BucketName) -> Self {
        self.bucket = Some(bucket);
        self
    }

    /// Set the ACL to apply.
    pub fn acl(mut self, acl: BucketAcl) -> Self {
        self.acl = Some(acl);
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<PutBucketAclRequest> {
        Ok(PutBucketAclRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            acl: self
                .acl
                .ok_or_else(|| OssError::MissingField("acl".into()))?,
        })
    }
}

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
    pub fn build(self) -> Result<GetBucketAclRequest> {
        Ok(GetBucketAclRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
        })
    }
}

/// Request to set the CORS configuration of a bucket.
#[derive(Debug)]
pub struct PutBucketCorsRequest {
    pub(crate) bucket: BucketName,
    pub(crate) cors_rules: Vec<CorsRule>,
}

/// A single CORS rule.
#[derive(Debug, Clone, Default)]
pub struct CorsRule {
    /// Allowed origins for CORS requests.
    pub allowed_origins: Vec<String>,
    /// Allowed HTTP methods.
    pub allowed_methods: Vec<crate::types::common::CorsHttpMethod>,
    /// Allowed headers in OPTIONS request (optional).
    pub allowed_headers: Option<Vec<String>>,
    /// Exposed headers in responses (optional).
    pub expose_headers: Option<Vec<String>>,
    /// Cache time for OPTIONS response in seconds (optional).
    pub max_age_seconds: Option<u32>,
}

/// Builder for [`PutBucketCorsRequest`].
#[derive(Debug, Default)]
pub struct PutBucketCorsRequestBuilder {
    bucket: Option<BucketName>,
    cors_rules: Vec<CorsRule>,
}

impl PutBucketCorsRequestBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the bucket name.
    pub fn bucket(mut self, bucket: BucketName) -> Self {
        self.bucket = Some(bucket);
        self
    }

    /// Add a CORS rule.
    pub fn add_rule(mut self, rule: CorsRule) -> Self {
        self.cors_rules.push(rule);
        self
    }

    /// Set multiple CORS rules.
    pub fn rules(mut self, rules: Vec<CorsRule>) -> Self {
        self.cors_rules = rules;
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<PutBucketCorsRequest> {
        if self.cors_rules.is_empty() {
            return Err(OssError::InvalidParameter {
                field: "cors_rules".into(),
                reason: "at least one CORS rule is required".into(),
            });
        }
        Ok(PutBucketCorsRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            cors_rules: self.cors_rules,
        })
    }
}

impl CorsRule {
    /// Create a new CORS rule.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an allowed origin.
    pub fn add_allowed_origin(mut self, origin: impl Into<String>) -> Self {
        self.allowed_origins.push(origin.into());
        self
    }

    /// Add an allowed HTTP method.
    pub fn add_allowed_method(mut self, method: crate::types::common::CorsHttpMethod) -> Self {
        self.allowed_methods.push(method);
        self
    }

    /// Set allowed headers.
    pub fn allowed_headers(mut self, headers: Vec<String>) -> Self {
        self.allowed_headers = Some(headers);
        self
    }

    /// Set expose headers.
    pub fn expose_headers(mut self, headers: Vec<String>) -> Self {
        self.expose_headers = Some(headers);
        self
    }

    /// Set max age seconds.
    pub fn max_age_seconds(mut self, seconds: u32) -> Self {
        self.max_age_seconds = Some(seconds);
        self
    }
}

/// Request to get the CORS configuration of a bucket.
#[derive(Debug)]
pub struct GetBucketCorsRequest {
    pub(crate) bucket: BucketName,
}

/// Builder for [`GetBucketCorsRequest`].
#[derive(Debug, Default)]
pub struct GetBucketCorsRequestBuilder {
    bucket: Option<BucketName>,
}

impl GetBucketCorsRequestBuilder {
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
    pub fn build(self) -> Result<GetBucketCorsRequest> {
        Ok(GetBucketCorsRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
        })
    }
}

/// Request to delete the CORS configuration of a bucket.
#[derive(Debug)]
pub struct DeleteBucketCorsRequest {
    pub(crate) bucket: BucketName,
}

/// Builder for [`DeleteBucketCorsRequest`].
#[derive(Debug, Default)]
pub struct DeleteBucketCorsRequestBuilder {
    bucket: Option<BucketName>,
}

impl DeleteBucketCorsRequestBuilder {
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
    pub fn build(self) -> Result<DeleteBucketCorsRequest> {
        Ok(DeleteBucketCorsRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
        })
    }
}

/// Request to set the Referer (hotlink protection) configuration of a bucket.
#[derive(Debug)]
pub struct PutBucketRefererRequest {
    pub(crate) bucket: BucketName,
    pub(crate) allow_empty_referer: bool,
    pub(crate) allow_truncate_query_string: Option<bool>,
    pub(crate) truncate_path: Option<bool>,
    pub(crate) referer_list: Vec<String>,
    pub(crate) referer_blacklist: Option<Vec<String>>,
}

/// Builder for [`PutBucketRefererRequest`].
#[derive(Debug, Default)]
pub struct PutBucketRefererRequestBuilder {
    bucket: Option<BucketName>,
    allow_empty_referer: Option<bool>,
    allow_truncate_query_string: Option<bool>,
    truncate_path: Option<bool>,
    referer_list: Vec<String>,
    referer_blacklist: Option<Vec<String>>,
}

impl PutBucketRefererRequestBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the bucket name.
    pub fn bucket(mut self, bucket: BucketName) -> Self {
        self.bucket = Some(bucket);
        self
    }

    /// Set whether to allow empty Referer.
    pub fn allow_empty_referer(mut self, allow: bool) -> Self {
        self.allow_empty_referer = Some(allow);
        self
    }

    /// Set whether to truncate query string when matching Referer.
    pub fn allow_truncate_query_string(mut self, allow: bool) -> Self {
        self.allow_truncate_query_string = Some(allow);
        self
    }

    /// Set whether to truncate path when matching Referer.
    pub fn truncate_path(mut self, truncate: bool) -> Self {
        self.truncate_path = Some(truncate);
        self
    }

    /// Add a Referer to the whitelist.
    pub fn add_referer(mut self, referer: impl Into<String>) -> Self {
        self.referer_list.push(referer.into());
        self
    }

    /// Set the Referer whitelist.
    pub fn referer_list(mut self, list: Vec<String>) -> Self {
        self.referer_list = list;
        self
    }

    /// Set the Referer blacklist.
    pub fn referer_blacklist(mut self, list: Vec<String>) -> Self {
        self.referer_blacklist = Some(list);
        self
    }

    /// Add a single Referer to the blacklist.
    pub fn add_referer_blacklist(mut self, referer: impl Into<String>) -> Self {
        self.referer_blacklist
            .get_or_insert_with(Vec::new)
            .push(referer.into());
        self
    }

    /// Build the request.
    ///
    /// If `allow_empty_referer` is not explicitly set, it defaults to `true`
    /// (matching the OSS API default behavior).
    pub fn build(self) -> Result<PutBucketRefererRequest> {
        Ok(PutBucketRefererRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            allow_empty_referer: self.allow_empty_referer.unwrap_or(true),
            allow_truncate_query_string: self.allow_truncate_query_string,
            truncate_path: self.truncate_path,
            referer_list: self.referer_list,
            referer_blacklist: self.referer_blacklist,
        })
    }
}

/// Request to get the Referer configuration of a bucket.
#[derive(Debug)]
pub struct GetBucketRefererRequest {
    pub(crate) bucket: BucketName,
}

/// Builder for [`GetBucketRefererRequest`].
#[derive(Debug, Default)]
pub struct GetBucketRefererRequestBuilder {
    bucket: Option<BucketName>,
}

impl GetBucketRefererRequestBuilder {
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
    pub fn build(self) -> Result<GetBucketRefererRequest> {
        Ok(GetBucketRefererRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
        })
    }
}

/// Request to set the authorization policy of a bucket.
///
/// The policy is a JSON string containing the bucket policy rules.
#[derive(Debug)]
pub struct PutBucketPolicyRequest {
    pub(crate) bucket: BucketName,
    pub(crate) policy: String,
}

/// Builder for [`PutBucketPolicyRequest`].
#[derive(Debug, Default)]
pub struct PutBucketPolicyRequestBuilder {
    bucket: Option<BucketName>,
    policy: Option<String>,
}

impl PutBucketPolicyRequestBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the bucket name.
    pub fn bucket(mut self, bucket: BucketName) -> Self {
        self.bucket = Some(bucket);
        self
    }

    /// Set the policy JSON string.
    pub fn policy(mut self, policy: impl Into<String>) -> Self {
        self.policy = Some(policy.into());
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<PutBucketPolicyRequest> {
        Ok(PutBucketPolicyRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            policy: self
                .policy
                .ok_or_else(|| OssError::MissingField("policy".into()))?,
        })
    }
}

/// Request to get the authorization policy of a bucket.
#[derive(Debug)]
pub struct GetBucketPolicyRequest {
    pub(crate) bucket: BucketName,
}

/// Builder for [`GetBucketPolicyRequest`].
#[derive(Debug, Default)]
pub struct GetBucketPolicyRequestBuilder {
    bucket: Option<BucketName>,
}

impl GetBucketPolicyRequestBuilder {
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
    pub fn build(self) -> Result<GetBucketPolicyRequest> {
        Ok(GetBucketPolicyRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
        })
    }
}

/// Request to delete the authorization policy of a bucket.
#[derive(Debug)]
pub struct DeleteBucketPolicyRequest {
    pub(crate) bucket: BucketName,
}

/// Builder for [`DeleteBucketPolicyRequest`].
#[derive(Debug, Default)]
pub struct DeleteBucketPolicyRequestBuilder {
    bucket: Option<BucketName>,
}

impl DeleteBucketPolicyRequestBuilder {
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
    pub fn build(self) -> Result<DeleteBucketPolicyRequest> {
        Ok(DeleteBucketPolicyRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
        })
    }
}

/// Request to set the versioning status of a bucket.
#[derive(Debug)]
pub struct PutBucketVersioningRequest {
    pub(crate) bucket: BucketName,
    pub(crate) status: crate::types::common::VersioningStatus,
}

/// Builder for [`PutBucketVersioningRequest`].
#[derive(Debug, Default)]
pub struct PutBucketVersioningRequestBuilder {
    bucket: Option<BucketName>,
    status: Option<crate::types::common::VersioningStatus>,
}

impl PutBucketVersioningRequestBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the bucket name.
    pub fn bucket(mut self, bucket: BucketName) -> Self {
        self.bucket = Some(bucket);
        self
    }

    /// Set the versioning status.
    pub fn status(mut self, status: crate::types::common::VersioningStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<PutBucketVersioningRequest> {
        Ok(PutBucketVersioningRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            status: self
                .status
                .ok_or_else(|| OssError::MissingField("status".into()))?,
        })
    }
}

/// Request to get the versioning status of a bucket.
#[derive(Debug)]
pub struct GetBucketVersioningRequest {
    pub(crate) bucket: BucketName,
}

/// Builder for [`GetBucketVersioningRequest`].
#[derive(Debug, Default)]
pub struct GetBucketVersioningRequestBuilder {
    bucket: Option<BucketName>,
}

impl GetBucketVersioningRequestBuilder {
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
    pub fn build(self) -> Result<GetBucketVersioningRequest> {
        Ok(GetBucketVersioningRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
        })
    }
}

/// Lifecycle rule for bucket lifecycle management.
///
/// Defines when objects should be expired or have their storage class transitioned.
#[derive(Debug, Clone, Default)]
pub struct LifecycleRule {
    /// Unique identifier for the rule.
    pub id: Option<String>,
    /// Object prefix that the rule applies to.
    pub prefix: Option<String>,
    /// Rule status (Enabled or Disabled).
    pub status: LifecycleRuleStatus,
    /// Expiration configuration.
    pub expiration: Option<LifecycleExpiration>,
    /// Storage class transition configurations.
    pub transitions: Vec<LifecycleTransition>,
}

impl LifecycleRule {
    /// Create a new lifecycle rule.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the rule ID.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the object prefix.
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Set the rule status.
    pub fn status(mut self, status: LifecycleRuleStatus) -> Self {
        self.status = status;
        self
    }

    /// Set the expiration configuration.
    pub fn expiration(mut self, expiration: LifecycleExpiration) -> Self {
        self.expiration = Some(expiration);
        self
    }

    /// Add a transition configuration.
    pub fn add_transition(mut self, transition: LifecycleTransition) -> Self {
        self.transitions.push(transition);
        self
    }

    /// Set multiple transition configurations.
    pub fn transitions(mut self, transitions: Vec<LifecycleTransition>) -> Self {
        self.transitions = transitions;
        self
    }
}

/// Lifecycle rule status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Deserialize, Serialize)]
pub enum LifecycleRuleStatus {
    /// Rule is enabled.
    #[default]
    #[serde(rename = "Enabled")]
    Enabled,
    /// Rule is disabled.
    #[serde(rename = "Disabled")]
    Disabled,
}

impl LifecycleRuleStatus {
    /// Convert to string for XML serialization.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Enabled => "Enabled",
            Self::Disabled => "Disabled",
        }
    }
}

/// Object expiration configuration.
#[derive(Debug, Clone)]
pub enum LifecycleExpiration {
    /// Expire after specified number of days.
    Days(u32),
    /// Expire on specified date (ISO 8601 format: YYYY-MM-DD).
    Date(String),
}

/// Storage class transition configuration.
#[derive(Debug, Clone)]
pub struct LifecycleTransition {
    /// Target storage class.
    pub storage_class: crate::types::common::StorageClass,
    /// Days after object creation when transition should occur.
    pub days: u32,
}

impl LifecycleTransition {
    /// Create a new transition configuration.
    pub fn new(storage_class: crate::types::common::StorageClass, days: u32) -> Self {
        Self {
            storage_class,
            days,
        }
    }
}

/// Request to set the lifecycle configuration of a bucket.
#[derive(Debug)]
pub struct PutBucketLifecycleRequest {
    pub(crate) bucket: BucketName,
    pub(crate) lifecycle_rules: Vec<LifecycleRule>,
}

/// Builder for [`PutBucketLifecycleRequest`].
#[derive(Debug, Default)]
pub struct PutBucketLifecycleRequestBuilder {
    bucket: Option<BucketName>,
    lifecycle_rules: Vec<LifecycleRule>,
}

impl PutBucketLifecycleRequestBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the bucket name.
    pub fn bucket(mut self, bucket: BucketName) -> Self {
        self.bucket = Some(bucket);
        self
    }

    /// Add a lifecycle rule.
    pub fn add_rule(mut self, rule: LifecycleRule) -> Self {
        self.lifecycle_rules.push(rule);
        self
    }

    /// Set multiple lifecycle rules.
    pub fn rules(mut self, rules: Vec<LifecycleRule>) -> Self {
        self.lifecycle_rules = rules;
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<PutBucketLifecycleRequest> {
        if self.lifecycle_rules.is_empty() {
            return Err(OssError::InvalidParameter {
                field: "lifecycle_rules".into(),
                reason: "at least one lifecycle rule is required".into(),
            });
        }
        Ok(PutBucketLifecycleRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            lifecycle_rules: self.lifecycle_rules,
        })
    }
}

/// Request to get the lifecycle configuration of a bucket.
#[derive(Debug)]
pub struct GetBucketLifecycleRequest {
    pub(crate) bucket: BucketName,
}

/// Builder for [`GetBucketLifecycleRequest`].
#[derive(Debug, Default)]
pub struct GetBucketLifecycleRequestBuilder {
    bucket: Option<BucketName>,
}

impl GetBucketLifecycleRequestBuilder {
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
    pub fn build(self) -> Result<GetBucketLifecycleRequest> {
        Ok(GetBucketLifecycleRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
        })
    }
}

/// Request to delete the lifecycle configuration of a bucket.
#[derive(Debug)]
pub struct DeleteBucketLifecycleRequest {
    pub(crate) bucket: BucketName,
}

/// Builder for [`DeleteBucketLifecycleRequest`].
#[derive(Debug, Default)]
pub struct DeleteBucketLifecycleRequestBuilder {
    bucket: Option<BucketName>,
}

impl DeleteBucketLifecycleRequestBuilder {
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
    pub fn build(self) -> Result<DeleteBucketLifecycleRequest> {
        Ok(DeleteBucketLifecycleRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
        })
    }
}

/// Request to set the encryption configuration of a bucket.
#[derive(Debug)]
pub struct PutBucketEncryptionRequest {
    pub(crate) bucket: BucketName,
    pub(crate) encryption: crate::types::common::ServerSideEncryption,
}

/// Builder for [`PutBucketEncryptionRequest`].
#[derive(Debug, Default)]
pub struct PutBucketEncryptionRequestBuilder {
    bucket: Option<BucketName>,
    encryption: Option<crate::types::common::ServerSideEncryption>,
}

impl PutBucketEncryptionRequestBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the bucket name.
    pub fn bucket(mut self, bucket: BucketName) -> Self {
        self.bucket = Some(bucket);
        self
    }

    /// Set the encryption algorithm.
    pub fn encryption(mut self, encryption: crate::types::common::ServerSideEncryption) -> Self {
        self.encryption = Some(encryption);
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<PutBucketEncryptionRequest> {
        Ok(PutBucketEncryptionRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            encryption: self
                .encryption
                .ok_or_else(|| OssError::MissingField("encryption".into()))?,
        })
    }
}

/// Request to get the encryption configuration of a bucket.
#[derive(Debug)]
pub struct GetBucketEncryptionRequest {
    pub(crate) bucket: BucketName,
}

/// Builder for [`GetBucketEncryptionRequest`].
#[derive(Debug, Default)]
pub struct GetBucketEncryptionRequestBuilder {
    bucket: Option<BucketName>,
}

impl GetBucketEncryptionRequestBuilder {
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
    pub fn build(self) -> Result<GetBucketEncryptionRequest> {
        Ok(GetBucketEncryptionRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
        })
    }
}

/// Request to delete the encryption configuration of a bucket.
#[derive(Debug)]
pub struct DeleteBucketEncryptionRequest {
    pub(crate) bucket: BucketName,
}

/// Builder for [`DeleteBucketEncryptionRequest`].
#[derive(Debug, Default)]
pub struct DeleteBucketEncryptionRequestBuilder {
    bucket: Option<BucketName>,
}

impl DeleteBucketEncryptionRequestBuilder {
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
    pub fn build(self) -> Result<DeleteBucketEncryptionRequest> {
        Ok(DeleteBucketEncryptionRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
        })
    }
}

/// Request to set the logging configuration of a bucket.
#[derive(Debug)]
pub struct PutBucketLoggingRequest {
    pub(crate) bucket: BucketName,
    pub(crate) target_bucket: BucketName,
    pub(crate) target_prefix: Option<String>,
}

/// Builder for [`PutBucketLoggingRequest`].
#[derive(Debug, Default)]
pub struct PutBucketLoggingRequestBuilder {
    bucket: Option<BucketName>,
    target_bucket: Option<BucketName>,
    target_prefix: Option<String>,
}

impl PutBucketLoggingRequestBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the bucket name.
    pub fn bucket(mut self, bucket: BucketName) -> Self {
        self.bucket = Some(bucket);
        self
    }

    /// Set the target bucket that receives the logs.
    pub fn target_bucket(mut self, target_bucket: BucketName) -> Self {
        self.target_bucket = Some(target_bucket);
        self
    }

    /// Set the prefix for log objects in the target bucket.
    pub fn target_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.target_prefix = Some(prefix.into());
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<PutBucketLoggingRequest> {
        Ok(PutBucketLoggingRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
            target_bucket: self
                .target_bucket
                .ok_or_else(|| OssError::MissingField("target_bucket".into()))?,
            target_prefix: self.target_prefix,
        })
    }
}

/// Request to get the logging configuration of a bucket.
#[derive(Debug)]
pub struct GetBucketLoggingRequest {
    pub(crate) bucket: BucketName,
}

/// Builder for [`GetBucketLoggingRequest`].
#[derive(Debug, Default)]
pub struct GetBucketLoggingRequestBuilder {
    bucket: Option<BucketName>,
}

impl GetBucketLoggingRequestBuilder {
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
    pub fn build(self) -> Result<GetBucketLoggingRequest> {
        Ok(GetBucketLoggingRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
        })
    }
}

/// Request to delete the logging configuration of a bucket.
#[derive(Debug)]
pub struct DeleteBucketLoggingRequest {
    pub(crate) bucket: BucketName,
}

/// Builder for [`DeleteBucketLoggingRequest`].
#[derive(Debug, Default)]
pub struct DeleteBucketLoggingRequestBuilder {
    bucket: Option<BucketName>,
}

impl DeleteBucketLoggingRequestBuilder {
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
    pub fn build(self) -> Result<DeleteBucketLoggingRequest> {
        Ok(DeleteBucketLoggingRequest {
            bucket: self
                .bucket
                .ok_or_else(|| OssError::MissingField("bucket".into()))?,
        })
    }
}

#[derive(Debug, Serialize)]
#[serde(rename = "CORSConfiguration")]
pub(crate) struct CorsConfigurationXml {
    #[serde(rename = "CORSRule")]
    pub cors_rules: Vec<CorsRuleXml>,
    #[serde(rename = "ResponseVary", skip_serializing_if = "Option::is_none")]
    pub response_vary: Option<bool>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CorsRuleXml {
    #[serde(rename = "AllowedOrigin")]
    pub allowed_origins: Vec<String>,
    #[serde(rename = "AllowedMethod")]
    pub allowed_methods: Vec<String>,
    #[serde(rename = "AllowedHeader", skip_serializing_if = "Vec::is_empty")]
    pub allowed_headers: Vec<String>,
    #[serde(rename = "ExposeHeader", skip_serializing_if = "Vec::is_empty")]
    pub expose_headers: Vec<String>,
    #[serde(rename = "MaxAgeSeconds", skip_serializing_if = "Option::is_none")]
    pub max_age_seconds: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename = "RefererConfiguration")]
pub(crate) struct RefererConfigurationXml {
    #[serde(rename = "AllowEmptyReferer")]
    pub allow_empty_referer: bool,
    #[serde(
        rename = "AllowTruncateQueryString",
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_truncate_query_string: Option<bool>,
    #[serde(rename = "TruncatePath", skip_serializing_if = "Option::is_none")]
    pub truncate_path: Option<bool>,
    #[serde(rename = "RefererList")]
    pub referer_list: RefererListXml,
    #[serde(rename = "RefererBlacklist", skip_serializing_if = "Option::is_none")]
    pub referer_blacklist: Option<RefererBlacklistXml>,
}

#[derive(Debug, Serialize)]
pub(crate) struct RefererListXml {
    #[serde(rename = "Referer", default)]
    pub referers: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct RefererBlacklistXml {
    #[serde(rename = "Referer", default)]
    pub referers: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename = "VersioningConfiguration")]
pub(crate) struct VersioningConfigurationXml {
    #[serde(rename = "Status")]
    pub status: crate::types::common::VersioningStatus,
}

#[derive(Debug, Serialize)]
#[serde(rename = "LifecycleConfiguration")]
pub(crate) struct LifecycleConfigurationXml {
    #[serde(rename = "Rule")]
    pub rules: Vec<LifecycleRuleXml>,
}

#[derive(Debug, Serialize)]
pub(crate) struct LifecycleRuleXml {
    #[serde(rename = "ID", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "Expiration", skip_serializing_if = "Option::is_none")]
    pub expiration: Option<LifecycleExpirationXml>,
    #[serde(rename = "Transition", skip_serializing_if = "Vec::is_empty")]
    pub transitions: Vec<LifecycleTransitionXml>,
}

#[derive(Debug, Serialize)]
pub(crate) struct LifecycleExpirationXml {
    #[serde(rename = "Days", skip_serializing_if = "Option::is_none")]
    pub days: Option<u32>,
    #[serde(rename = "Date", skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct LifecycleTransitionXml {
    #[serde(rename = "Days")]
    pub days: u32,
    #[serde(rename = "StorageClass")]
    pub storage_class: String,
}

#[derive(Debug, Serialize)]
#[serde(rename = "ServerSideEncryptionConfiguration")]
pub(crate) struct EncryptionConfigurationXml {
    #[serde(rename = "Rule")]
    pub rule: EncryptionRuleXml,
}

#[derive(Debug, Serialize)]
pub(crate) struct EncryptionRuleXml {
    #[serde(rename = "ApplyServerSideEncryptionByDefault")]
    pub apply_server_side_encryption_by_default: ApplyServerSideEncryptionByDefaultXml,
}

#[derive(Debug, Serialize)]
pub(crate) struct ApplyServerSideEncryptionByDefaultXml {
    #[serde(rename = "SSEAlgorithm")]
    pub sse_algorithm: ServerSideEncryption,
    #[serde(rename = "KMSMasterKeyID", skip_serializing_if = "Option::is_none")]
    pub kms_master_key_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename = "BucketLoggingStatus")]
pub(crate) struct LoggingConfigurationXml {
    #[serde(rename = "LoggingEnabled")]
    pub logging_enabled: LoggingEnabledXml,
}

#[derive(Debug, Serialize)]
pub(crate) struct LoggingEnabledXml {
    #[serde(rename = "TargetBucket")]
    pub target_bucket: String,
    #[serde(rename = "TargetPrefix", skip_serializing_if = "String::is_empty")]
    pub target_prefix: String,
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

    #[test]
    fn cors_rule_builder() {
        let rule = CorsRule::new()
            .add_allowed_origin("*")
            .add_allowed_method(crate::types::common::CorsHttpMethod::Get)
            .add_allowed_method(crate::types::common::CorsHttpMethod::Put)
            .allowed_headers(vec!["*".to_string()])
            .max_age_seconds(100);
        assert_eq!(rule.allowed_origins.len(), 1);
        assert_eq!(rule.allowed_methods.len(), 2);
        assert!(rule.allowed_headers.is_some());
        assert_eq!(rule.max_age_seconds, Some(100));
    }

    #[test]
    fn put_bucket_cors_request_builder() {
        use crate::types::common::CorsHttpMethod;

        let rule = CorsRule::new()
            .add_allowed_origin("https://example.com")
            .add_allowed_method(CorsHttpMethod::Get)
            .add_allowed_method(CorsHttpMethod::Put);

        let req = PutBucketCorsRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .add_rule(rule)
            .build();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.cors_rules.len(), 1);
    }

    #[test]
    fn put_bucket_cors_missing_bucket() {
        let rule = CorsRule::new()
            .add_allowed_origin("*")
            .add_allowed_method(crate::types::common::CorsHttpMethod::Get);

        let req = PutBucketCorsRequestBuilder::new().add_rule(rule).build();
        assert!(req.is_err());
    }

    #[test]
    fn get_bucket_cors_request_builder() {
        let req = GetBucketCorsRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn delete_bucket_cors_request_builder() {
        let req = DeleteBucketCorsRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn put_bucket_referer_request_builder() {
        let req = PutBucketRefererRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .allow_empty_referer(false)
            .allow_truncate_query_string(true)
            .add_referer("http://example.com")
            .add_referer("https://example.com")
            .referer_blacklist(vec!["http://refuse.com".to_string()])
            .build();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert!(!req.allow_empty_referer);
        assert_eq!(req.referer_list.len(), 2);
        assert!(req.referer_blacklist.is_some());
    }

    #[test]
    fn put_bucket_referer_missing_bucket() {
        let req = PutBucketRefererRequestBuilder::new()
            .allow_empty_referer(true)
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn get_bucket_referer_request_builder() {
        let req = GetBucketRefererRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn put_bucket_policy_request_builder() {
        let policy_json = r#"{"Version":"1","Statement":[]}"#;
        let req = PutBucketPolicyRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .policy(policy_json)
            .build();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.policy, policy_json);
    }

    #[test]
    fn put_bucket_policy_missing_bucket() {
        let req = PutBucketPolicyRequestBuilder::new()
            .policy(r#"{"Version":"1"}"#)
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn put_bucket_policy_missing_policy() {
        let req = PutBucketPolicyRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn get_bucket_policy_request_builder() {
        let req = GetBucketPolicyRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn delete_bucket_policy_request_builder() {
        let req = DeleteBucketPolicyRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn put_bucket_versioning_request_builder() {
        let req = PutBucketVersioningRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .status(crate::types::common::VersioningStatus::Enabled)
            .build();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.status, crate::types::common::VersioningStatus::Enabled);
    }

    #[test]
    fn put_bucket_versioning_missing_bucket() {
        let req = PutBucketVersioningRequestBuilder::new()
            .status(crate::types::common::VersioningStatus::Enabled)
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn put_bucket_versioning_missing_status() {
        let req = PutBucketVersioningRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn get_bucket_versioning_request_builder() {
        let req = GetBucketVersioningRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn lifecycle_rule_builder() {
        let rule = LifecycleRule::new()
            .id("rule1")
            .prefix("logs/")
            .status(LifecycleRuleStatus::Enabled)
            .expiration(LifecycleExpiration::Days(30))
            .add_transition(LifecycleTransition::new(
                crate::types::common::StorageClass::InfrequentAccess,
                7,
            ));
        assert_eq!(rule.id, Some("rule1".to_string()));
        assert_eq!(rule.prefix, Some("logs/".to_string()));
        assert_eq!(rule.transitions.len(), 1);
    }

    #[test]
    fn put_bucket_lifecycle_request_builder() {
        let rule = LifecycleRule::new()
            .id("delete-rule")
            .prefix("temp/")
            .status(LifecycleRuleStatus::Enabled)
            .expiration(LifecycleExpiration::Days(7));

        let req = PutBucketLifecycleRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .add_rule(rule)
            .build();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.lifecycle_rules.len(), 1);
    }

    #[test]
    fn put_bucket_lifecycle_missing_bucket() {
        let rule = LifecycleRule::new()
            .status(LifecycleRuleStatus::Enabled)
            .expiration(LifecycleExpiration::Days(30));

        let req = PutBucketLifecycleRequestBuilder::new()
            .add_rule(rule)
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn get_bucket_lifecycle_request_builder() {
        let req = GetBucketLifecycleRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn delete_bucket_lifecycle_request_builder() {
        let req = DeleteBucketLifecycleRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn lifecycle_rule_status_as_str() {
        assert_eq!(LifecycleRuleStatus::Enabled.as_str(), "Enabled");
        assert_eq!(LifecycleRuleStatus::Disabled.as_str(), "Disabled");
    }

    #[test]
    fn lifecycle_transition_new() {
        let transition = LifecycleTransition::new(crate::types::common::StorageClass::Archive, 90);
        assert_eq!(
            transition.storage_class,
            crate::types::common::StorageClass::Archive
        );
        assert_eq!(transition.days, 90);
    }

    #[test]
    fn lifecycle_rule_multiple_transitions() {
        let rule = LifecycleRule::new()
            .status(LifecycleRuleStatus::Enabled)
            .transitions(vec![
                LifecycleTransition::new(crate::types::common::StorageClass::InfrequentAccess, 30),
                LifecycleTransition::new(crate::types::common::StorageClass::Archive, 90),
            ]);
        assert_eq!(rule.transitions.len(), 2);
    }

    #[test]
    fn put_bucket_encryption_request_builder() {
        let req = PutBucketEncryptionRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .encryption(crate::types::common::ServerSideEncryption::AES256)
            .build();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(
            req.encryption,
            crate::types::common::ServerSideEncryption::AES256
        );
    }

    #[test]
    fn put_bucket_encryption_missing_bucket() {
        let req = PutBucketEncryptionRequestBuilder::new()
            .encryption(crate::types::common::ServerSideEncryption::KMS)
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn put_bucket_encryption_missing_encryption() {
        let req = PutBucketEncryptionRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn get_bucket_encryption_request_builder() {
        let req = GetBucketEncryptionRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn delete_bucket_encryption_request_builder() {
        let req = DeleteBucketEncryptionRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn put_bucket_logging_request_builder() {
        let req = PutBucketLoggingRequestBuilder::new()
            .bucket(BucketName::new("my-bucket").unwrap())
            .target_bucket(BucketName::new("log-bucket").unwrap())
            .target_prefix("logs/")
            .build();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.target_prefix, Some("logs/".to_string()));
    }

    #[test]
    fn put_bucket_logging_missing_bucket() {
        let req = PutBucketLoggingRequestBuilder::new()
            .target_bucket(BucketName::new("log-bucket").unwrap())
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn put_bucket_logging_missing_target_bucket() {
        let req = PutBucketLoggingRequestBuilder::new()
            .bucket(BucketName::new("my-bucket").unwrap())
            .build();
        assert!(req.is_err());
    }

    #[test]
    fn get_bucket_logging_request_builder() {
        let req = GetBucketLoggingRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn delete_bucket_logging_request_builder() {
        let req = DeleteBucketLoggingRequestBuilder::new()
            .bucket(BucketName::new("test-bucket").unwrap())
            .build();
        assert!(req.is_ok());
    }

    #[test]
    fn cors_rule_xml_serializes() {
        let rule = CorsRuleXml {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["GET".to_string(), "PUT".to_string()],
            allowed_headers: vec!["Authorization".to_string()],
            expose_headers: vec![],
            max_age_seconds: Some(100),
        };
        let config = CorsConfigurationXml {
            cors_rules: vec![rule],
            response_vary: Some(false),
        };
        let xml = quick_xml::se::to_string(&config).unwrap();
        assert!(xml.contains("<AllowedOrigin>*</AllowedOrigin>"));
        assert!(xml.contains("<AllowedMethod>GET</AllowedMethod>"));
        assert!(xml.contains("<AllowedMethod>PUT</AllowedMethod>"));
        assert!(xml.contains("<AllowedHeader>Authorization</AllowedHeader>"));
        assert!(xml.contains("<MaxAgeSeconds>100</MaxAgeSeconds>"));
    }

    #[test]
    fn referer_configuration_xml_serializes() {
        let config = RefererConfigurationXml {
            allow_empty_referer: true,
            allow_truncate_query_string: Some(true),
            truncate_path: Some(true),
            referer_list: RefererListXml {
                referers: vec!["http://example.com".to_string()],
            },
            referer_blacklist: None,
        };
        let xml = quick_xml::se::to_string(&config).unwrap();
        assert!(xml.contains("<AllowEmptyReferer>true</AllowEmptyReferer>"));
        assert!(xml.contains("<AllowTruncateQueryString>true</AllowTruncateQueryString>"));
        assert!(xml.contains("<TruncatePath>true</TruncatePath>"));
        assert!(xml.contains("<Referer>http://example.com</Referer>"));
    }

    #[test]
    fn versioning_configuration_xml_serializes() {
        let config = VersioningConfigurationXml {
            status: crate::types::common::VersioningStatus::Enabled,
        };
        let xml = quick_xml::se::to_string(&config).unwrap();
        assert!(xml.contains("<Status>Enabled</Status>"));
    }

    #[test]
    fn lifecycle_rule_xml_serializes() {
        let rule = LifecycleRuleXml {
            id: Some("rule1".to_string()),
            prefix: Some("logs/".to_string()),
            status: "Enabled".to_string(),
            expiration: Some(LifecycleExpirationXml {
                days: Some(30),
                date: None,
            }),
            transitions: vec![],
        };
        let xml = quick_xml::se::to_string(&rule).unwrap();
        assert!(xml.contains("<ID>rule1</ID>"));
        assert!(xml.contains("<Prefix>logs/</Prefix>"));
        assert!(xml.contains("<Status>Enabled</Status>"));
        assert!(xml.contains("<Days>30</Days>"));
    }

    #[test]
    fn lifecycle_transition_xml_serializes() {
        let transition = LifecycleTransitionXml {
            days: 90,
            storage_class: "Archive".to_string(),
        };
        let xml = quick_xml::se::to_string(&transition).unwrap();
        assert!(xml.contains("<Days>90</Days>"));
        assert!(xml.contains("<StorageClass>Archive</StorageClass>"));
    }

    #[test]
    fn encryption_configuration_xml_serializes() {
        let config = EncryptionConfigurationXml {
            rule: EncryptionRuleXml {
                apply_server_side_encryption_by_default: ApplyServerSideEncryptionByDefaultXml {
                    sse_algorithm: ServerSideEncryption::AES256,
                    kms_master_key_id: None,
                },
            },
        };
        let xml = quick_xml::se::to_string(&config).unwrap();
        assert!(xml.contains("<ApplyServerSideEncryptionByDefault>"));
        assert!(xml.contains("<SSEAlgorithm>AES256</SSEAlgorithm>"));
    }

    #[test]
    fn logging_configuration_xml_serializes() {
        let config = LoggingConfigurationXml {
            logging_enabled: LoggingEnabledXml {
                target_bucket: "log-bucket".to_string(),
                target_prefix: "logs/".to_string(),
            },
        };
        let xml = quick_xml::se::to_string(&config).unwrap();
        assert!(xml.contains("<TargetBucket>log-bucket</TargetBucket>"));
        assert!(xml.contains("<TargetPrefix>logs/</TargetPrefix>"));
    }

    #[test]
    fn logging_configuration_xml_serializes_no_prefix() {
        let config = LoggingConfigurationXml {
            logging_enabled: LoggingEnabledXml {
                target_bucket: "log-bucket".to_string(),
                target_prefix: String::new(),
            },
        };
        let xml = quick_xml::se::to_string(&config).unwrap();
        assert!(xml.contains("<TargetBucket>log-bucket</TargetBucket>"));
        // Empty prefix should be skipped
        assert!(!xml.contains("<TargetPrefix>"));
    }
}

//! Common newtypes and enums shared across OSS operations.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::{OssError, Result};

/// An OSS bucket name, validated on construction.
///
/// Bucket names must be 3-63 characters long, contain only lowercase letters,
/// digits, and hyphens, and must not start or end with a hyphen.
///
/// # Examples
/// ```
/// # use rs_ali_oss::types::BucketName;
/// let bucket = BucketName::new("my-bucket").unwrap();
/// assert_eq!(bucket.as_ref(), "my-bucket");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BucketName(String);

impl BucketName {
    /// Create a new validated bucket name.
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        if name.len() < 3 || name.len() > 63 {
            return Err(OssError::InvalidBucketName(format!(
                "must be 3-63 characters, got {}",
                name.len()
            )));
        }
        if !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(OssError::InvalidBucketName(
                "must contain only lowercase letters, digits, and hyphens".to_string(),
            ));
        }
        if name.starts_with('-') || name.ends_with('-') {
            return Err(OssError::InvalidBucketName(
                "must not start or end with a hyphen".to_string(),
            ));
        }
        Ok(Self(name))
    }
}

impl AsRef<str> for BucketName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BucketName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An OSS object key, validated on construction.
///
/// Object keys must be 1-1023 bytes long and non-empty.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectKey(String);

impl ObjectKey {
    /// Create a new validated object key.
    pub fn new(key: impl Into<String>) -> Result<Self> {
        let key = key.into();
        if key.is_empty() {
            return Err(OssError::InvalidObjectKey("must not be empty".to_string()));
        }
        if key.len() > 1023 {
            return Err(OssError::InvalidObjectKey(format!(
                "must be at most 1023 bytes, got {}",
                key.len()
            )));
        }
        Ok(Self(key))
    }
}

impl AsRef<str> for ObjectKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ObjectKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An OSS region identifier, validated on construction.
///
/// Region identifiers must be non-empty and contain only lowercase letters,
/// digits, and hyphens (e.g., "cn-hangzhou", "us-west-1").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Region(String);

impl Region {
    /// Create a new validated region.
    pub fn new(region: impl Into<String>) -> Result<Self> {
        let region = region.into();
        if region.is_empty() {
            return Err(OssError::InvalidRegion("must not be empty".to_string()));
        }
        if !region
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(OssError::InvalidRegion(
                "must contain only lowercase letters, digits, and hyphens".to_string(),
            ));
        }
        Ok(Self(region))
    }
}

impl AsRef<str> for Region {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// OSS storage class for objects and buckets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StorageClass {
    /// Standard storage (default).
    #[serde(rename = "Standard")]
    Standard,
    /// Infrequent access storage.
    #[serde(rename = "IA")]
    InfrequentAccess,
    /// Archive storage.
    #[serde(rename = "Archive")]
    Archive,
    /// Cold archive storage.
    #[serde(rename = "ColdArchive")]
    ColdArchive,
    /// Deep cold archive storage.
    #[serde(rename = "DeepColdArchive")]
    DeepColdArchive,
}

impl fmt::Display for StorageClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Standard => write!(f, "Standard"),
            Self::InfrequentAccess => write!(f, "IA"),
            Self::Archive => write!(f, "Archive"),
            Self::ColdArchive => write!(f, "ColdArchive"),
            Self::DeepColdArchive => write!(f, "DeepColdArchive"),
        }
    }
}

/// OSS object or bucket access control level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObjectAcl {
    /// Private (owner only).
    #[serde(rename = "private")]
    Private,
    /// Public read access.
    #[serde(rename = "public-read")]
    PublicRead,
    /// Public read-write access.
    #[serde(rename = "public-read-write")]
    PublicReadWrite,
    /// Inherit from bucket (default).
    #[serde(rename = "default")]
    Default,
}

impl fmt::Display for ObjectAcl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Private => write!(f, "private"),
            Self::PublicRead => write!(f, "public-read"),
            Self::PublicReadWrite => write!(f, "public-read-write"),
            Self::Default => write!(f, "default"),
        }
    }
}

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
/// # use rs_ali_oss::types::BucketAcl;
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

/// Metadata directive for CopyObject operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetadataDirective {
    /// Copy metadata from the source object (default).
    #[serde(rename = "COPY")]
    Copy,
    /// Replace metadata with values provided in the request.
    #[serde(rename = "REPLACE")]
    Replace,
}

impl fmt::Display for MetadataDirective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Copy => write!(f, "COPY"),
            Self::Replace => write!(f, "REPLACE"),
        }
    }
}

/// HTTP methods allowed in CORS rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorsHttpMethod {
    /// GET method.
    #[serde(rename = "GET")]
    Get,
    /// PUT method.
    #[serde(rename = "PUT")]
    Put,
    /// DELETE method.
    #[serde(rename = "DELETE")]
    Delete,
    /// POST method.
    #[serde(rename = "POST")]
    Post,
    /// HEAD method.
    #[serde(rename = "HEAD")]
    Head,
}

impl fmt::Display for CorsHttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Get => write!(f, "GET"),
            Self::Put => write!(f, "PUT"),
            Self::Delete => write!(f, "DELETE"),
            Self::Post => write!(f, "POST"),
            Self::Head => write!(f, "HEAD"),
        }
    }
}

/// Bucket versioning status.
///
/// Controls whether versioning is enabled for objects in a bucket.
/// Once enabled, versioning cannot be disabled, only suspended.
///
/// # Variants
///
/// * `Enabled` - Versioning is enabled for the bucket
/// * `Suspended` - Versioning is suspended (no new versions created)
///
/// # Examples
///
/// ```
/// # use rs_ali_oss::types::VersioningStatus;
/// let status = VersioningStatus::Enabled;
/// assert_eq!(status.to_string(), "Enabled");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VersioningStatus {
    /// Versioning is enabled.
    #[serde(rename = "Enabled")]
    Enabled,
    /// Versioning is suspended.
    #[serde(rename = "Suspended")]
    Suspended,
}

impl fmt::Display for VersioningStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Enabled => write!(f, "Enabled"),
            Self::Suspended => write!(f, "Suspended"),
        }
    }
}

/// Server-side encryption algorithm for bucket and objects.
///
/// Defines the encryption method used for server-side encryption.
///
/// # Variants
///
/// * `AES256` - AES-256 server-side encryption
/// * `KMS` - Key Management Service encryption
///
/// # Examples
///
/// ```
/// # use rs_ali_oss::types::ServerSideEncryption;
/// let sse = ServerSideEncryption::AES256;
/// assert_eq!(sse.to_string(), "AES256");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServerSideEncryption {
    /// AES-256 encryption.
    #[serde(rename = "AES256")]
    AES256,
    /// KMS encryption.
    #[serde(rename = "KMS")]
    KMS,
}

impl fmt::Display for ServerSideEncryption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AES256 => write!(f, "AES256"),
            Self::KMS => write!(f, "KMS"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_bucket_names() {
        assert!(BucketName::new("my-bucket").is_ok());
        assert!(BucketName::new("a1b2c3").is_ok());
        assert!(BucketName::new("abc").is_ok()); // minimum 3 chars
    }

    #[test]
    fn invalid_bucket_name_empty() {
        assert!(BucketName::new("").is_err());
    }

    #[test]
    fn invalid_bucket_name_uppercase() {
        assert!(BucketName::new("MyBucket").is_err());
    }

    #[test]
    fn invalid_bucket_name_starts_with_hyphen() {
        assert!(BucketName::new("-start").is_err());
    }

    #[test]
    fn invalid_bucket_name_ends_with_hyphen() {
        assert!(BucketName::new("end-").is_err());
    }

    #[test]
    fn invalid_bucket_name_too_short() {
        assert!(BucketName::new("ab").is_err());
    }

    #[test]
    fn invalid_bucket_name_too_long() {
        let long_name = "a".repeat(64);
        assert!(BucketName::new(long_name).is_err());
    }

    #[test]
    fn valid_object_keys() {
        assert!(ObjectKey::new("file.txt").is_ok());
        assert!(ObjectKey::new("path/to/file").is_ok());
        assert!(ObjectKey::new("中文/对象.txt").is_ok());
    }

    #[test]
    fn invalid_object_key_empty() {
        assert!(ObjectKey::new("").is_err());
    }

    #[test]
    fn valid_regions() {
        assert!(Region::new("cn-hangzhou").is_ok());
        assert!(Region::new("us-west-1").is_ok());
    }

    #[test]
    fn invalid_region_empty() {
        assert!(Region::new("").is_err());
    }

    #[test]
    fn invalid_region_uppercase() {
        assert!(Region::new("CN-Hangzhou").is_err());
    }

    #[test]
    fn storage_class_display() {
        assert_eq!(StorageClass::Standard.to_string(), "Standard");
        assert_eq!(StorageClass::InfrequentAccess.to_string(), "IA");
        assert_eq!(StorageClass::Archive.to_string(), "Archive");
        assert_eq!(StorageClass::ColdArchive.to_string(), "ColdArchive");
        assert_eq!(StorageClass::DeepColdArchive.to_string(), "DeepColdArchive");
    }

    #[test]
    fn object_acl_display() {
        assert_eq!(ObjectAcl::Private.to_string(), "private");
        assert_eq!(ObjectAcl::PublicRead.to_string(), "public-read");
        assert_eq!(ObjectAcl::PublicReadWrite.to_string(), "public-read-write");
        assert_eq!(ObjectAcl::Default.to_string(), "default");
    }

    #[test]
    fn storage_class_serde_round_trip() {
        let sc = StorageClass::InfrequentAccess;
        let json = serde_json::to_string(&sc).unwrap();
        let deserialized: StorageClass = serde_json::from_str(&json).unwrap();
        assert_eq!(sc, deserialized);
    }

    #[test]
    fn object_acl_serde_round_trip() {
        let acl = ObjectAcl::PublicRead;
        let json = serde_json::to_string(&acl).unwrap();
        let deserialized: ObjectAcl = serde_json::from_str(&json).unwrap();
        assert_eq!(acl, deserialized);
    }

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

    #[test]
    fn metadata_directive_display() {
        assert_eq!(MetadataDirective::Copy.to_string(), "COPY");
        assert_eq!(MetadataDirective::Replace.to_string(), "REPLACE");
    }

    #[test]
    fn metadata_directive_serde_round_trip() {
        let md = MetadataDirective::Replace;
        let json = serde_json::to_string(&md).unwrap();
        let deserialized: MetadataDirective = serde_json::from_str(&json).unwrap();
        assert_eq!(md, deserialized);
    }

    #[test]
    fn cors_http_method_display() {
        assert_eq!(CorsHttpMethod::Get.to_string(), "GET");
        assert_eq!(CorsHttpMethod::Put.to_string(), "PUT");
        assert_eq!(CorsHttpMethod::Delete.to_string(), "DELETE");
        assert_eq!(CorsHttpMethod::Post.to_string(), "POST");
        assert_eq!(CorsHttpMethod::Head.to_string(), "HEAD");
    }

    #[test]
    fn cors_http_method_serde_round_trip() {
        let method = CorsHttpMethod::Get;
        let json = serde_json::to_string(&method).unwrap();
        let deserialized: CorsHttpMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(method, deserialized);
    }

    #[test]
    fn versioning_status_display() {
        assert_eq!(VersioningStatus::Enabled.to_string(), "Enabled");
        assert_eq!(VersioningStatus::Suspended.to_string(), "Suspended");
    }

    #[test]
    fn versioning_status_serde_round_trip() {
        let status = VersioningStatus::Enabled;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: VersioningStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn server_side_encryption_display() {
        assert_eq!(ServerSideEncryption::AES256.to_string(), "AES256");
        assert_eq!(ServerSideEncryption::KMS.to_string(), "KMS");
    }

    #[test]
    fn server_side_encryption_serde_round_trip() {
        let sse = ServerSideEncryption::KMS;
        let json = serde_json::to_string(&sse).unwrap();
        let deserialized: ServerSideEncryption = serde_json::from_str(&json).unwrap();
        assert_eq!(sse, deserialized);
    }

    #[test]
    fn bucket_name_as_ref() {
        let bucket = BucketName::new("test-bucket").unwrap();
        let s: &str = bucket.as_ref();
        assert_eq!(s, "test-bucket");
    }

    #[test]
    fn object_key_display() {
        let key = ObjectKey::new("my/key.txt").unwrap();
        assert_eq!(key.to_string(), "my/key.txt");
    }

    #[test]
    fn bucket_name_exact_min_length() {
        let name = "abc";
        assert_eq!(name.len(), 3);
        assert!(BucketName::new(name).is_ok());
    }

    #[test]
    fn bucket_name_exact_max_length() {
        let name = "a".repeat(63);
        assert_eq!(name.len(), 63);
        assert!(BucketName::new(&name).is_ok());
    }

    #[test]
    fn bucket_name_one_over_max() {
        let name = "a".repeat(64);
        assert!(BucketName::new(&name).is_err());
    }

    #[test]
    fn bucket_name_one_under_min() {
        assert!(BucketName::new("ab").is_err());
    }

    #[test]
    fn bucket_name_rejects_underscore() {
        assert!(BucketName::new("my_bucket").is_err());
    }

    #[test]
    fn bucket_name_rejects_dot() {
        assert!(BucketName::new("my.bucket").is_err());
    }

    #[test]
    fn bucket_name_all_digits() {
        assert!(BucketName::new("123").is_ok());
    }

    #[test]
    fn bucket_name_hyphen_in_middle() {
        assert!(BucketName::new("a-b").is_ok());
    }

    #[test]
    fn object_key_exact_max_length() {
        let key = "k".repeat(1023);
        assert_eq!(key.len(), 1023);
        assert!(ObjectKey::new(&key).is_ok());
    }

    #[test]
    fn object_key_one_over_max() {
        let key = "k".repeat(1024);
        assert!(ObjectKey::new(&key).is_err());
    }

    #[test]
    fn object_key_single_byte() {
        assert!(ObjectKey::new("x").is_ok());
    }

    #[test]
    fn object_key_multibyte_chars_counted_as_bytes() {
        // 341 × 3-byte chars = 1023 bytes exactly
        let key = "文".repeat(341);
        assert_eq!(key.len(), 1023);
        assert!(ObjectKey::new(&key).is_ok());

        let key_over = "文".repeat(342);
        assert!(key_over.len() > 1023);
        assert!(ObjectKey::new(&key_over).is_err());
    }

    #[test]
    fn region_single_char() {
        assert!(Region::new("a").is_ok());
    }

    #[test]
    fn region_rejects_underscore() {
        assert!(Region::new("cn_hangzhou").is_err());
    }

    #[test]
    fn region_allows_digits_and_hyphens() {
        assert!(Region::new("us-east-1").is_ok());
        assert!(Region::new("ap-southeast-2").is_ok());
    }
}

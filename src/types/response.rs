//! Response types for OSS operations.

use std::collections::HashMap;
use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::common::StorageClass;

/// Response from a PutObject operation.
#[derive(Debug)]
pub struct PutObjectResponse {
    /// ETag of the uploaded object.
    pub etag: String,
    /// OSS request ID.
    pub request_id: Option<String>,
}

/// A wrapper around the response body that hides the underlying HTTP library.
///
/// Provides methods to consume the body as bytes, text, or a streaming byte stream.
pub struct ObjectBody(reqwest::Response);

impl ObjectBody {
    /// Create a new `ObjectBody` from a `reqwest::Response`.
    pub(crate) fn new(response: reqwest::Response) -> Self {
        Self(response)
    }

    /// Consume the body and return all bytes.
    pub async fn bytes(self) -> std::result::Result<bytes::Bytes, reqwest::Error> {
        self.0.bytes().await
    }

    /// Consume the body and return it as a UTF-8 string.
    pub async fn text(self) -> std::result::Result<String, reqwest::Error> {
        self.0.text().await
    }

    /// Return a streaming byte stream for incremental reading.
    pub fn bytes_stream(
        self,
    ) -> impl futures_util::Stream<Item = std::result::Result<bytes::Bytes, reqwest::Error>> {
        self.0.bytes_stream()
    }
}

impl fmt::Debug for ObjectBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<streaming body>")
    }
}

/// Response from a GetObject operation.
///
/// Use the [`ObjectBody`] methods (`.bytes()`, `.text()`, `.bytes_stream()`)
/// to consume the response data.
pub struct GetObjectResponse {
    /// The response body.
    pub body: ObjectBody,
    /// Content type of the object.
    pub content_type: Option<String>,
    /// Content length in bytes.
    pub content_length: Option<u64>,
    /// ETag of the object.
    pub etag: Option<String>,
    /// OSS request ID.
    pub request_id: Option<String>,
}

impl fmt::Debug for GetObjectResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GetObjectResponse")
            .field("content_type", &self.content_type)
            .field("content_length", &self.content_length)
            .field("etag", &self.etag)
            .field("request_id", &self.request_id)
            .field("body", &self.body)
            .finish()
    }
}

/// Response from a DeleteObject operation.
#[derive(Debug)]
pub struct DeleteObjectResponse {
    /// OSS request ID.
    pub request_id: Option<String>,
}

/// Response from a HeadObject operation.
#[derive(Debug)]
pub struct HeadObjectResponse {
    /// Content type of the object.
    pub content_type: Option<String>,
    /// Content length in bytes.
    pub content_length: Option<u64>,
    /// ETag of the object.
    pub etag: Option<String>,
    /// Last modified timestamp (parsed from HTTP header).
    pub last_modified: Option<DateTime<Utc>>,
    /// Custom metadata (x-oss-meta-* headers).
    pub metadata: HashMap<String, String>,
    /// OSS request ID.
    pub request_id: Option<String>,
}

/// Response from a ListObjectsV2 operation (XML-deserialized).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "ListBucketResult")]
pub struct ListObjectsV2Response {
    /// Bucket name.
    #[serde(rename = "Name")]
    pub name: String,
    /// The prefix used to filter results.
    #[serde(rename = "Prefix", default)]
    pub prefix: String,
    /// Maximum number of keys returned.
    #[serde(rename = "MaxKeys")]
    pub max_keys: u32,
    /// Number of keys returned in this response.
    #[serde(rename = "KeyCount")]
    pub key_count: u32,
    /// Whether the results are truncated (more pages available).
    #[serde(rename = "IsTruncated")]
    pub is_truncated: bool,
    /// Token to use for the next page of results.
    #[serde(rename = "NextContinuationToken", default)]
    pub next_continuation_token: Option<String>,
    /// Object entries in this page.
    #[serde(rename = "Contents", default)]
    pub contents: Vec<ObjectInfo>,
    /// Common prefix entries (when delimiter is used).
    #[serde(rename = "CommonPrefixes", default)]
    pub common_prefixes: Vec<CommonPrefix>,
}

/// Metadata for a single object in a listing.
#[derive(Debug, Clone, Deserialize)]
pub struct ObjectInfo {
    /// The object key.
    #[serde(rename = "Key")]
    pub key: String,
    /// Last modified timestamp.
    #[serde(rename = "LastModified")]
    pub last_modified: DateTime<Utc>,
    /// ETag of the object.
    #[serde(rename = "ETag")]
    pub etag: String,
    /// Size in bytes.
    #[serde(rename = "Size")]
    pub size: u64,
    /// Storage class of the object.
    #[serde(rename = "StorageClass")]
    pub storage_class: StorageClass,
}

/// A common prefix entry in a listing result (virtual directory).
#[derive(Debug, Clone, Deserialize)]
pub struct CommonPrefix {
    /// The prefix string.
    #[serde(rename = "Prefix")]
    pub prefix: String,
}

/// Response from a CreateBucket operation.
#[derive(Debug)]
pub struct CreateBucketResponse {
    /// OSS request ID.
    pub request_id: Option<String>,
}

/// Response from a DeleteBucket operation.
#[derive(Debug)]
pub struct DeleteBucketResponse {
    /// OSS request ID.
    pub request_id: Option<String>,
}

/// Response from a ListBuckets (GetService) operation (XML-deserialized).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "ListAllMyBucketsResult")]
pub struct ListBucketsResponse {
    /// The prefix used to filter results.
    #[serde(rename = "Prefix", default)]
    pub prefix: String,
    /// The marker used for pagination.
    #[serde(rename = "Marker", default)]
    pub marker: String,
    /// Maximum number of buckets returned.
    #[serde(rename = "MaxKeys")]
    pub max_keys: u32,
    /// Whether the results are truncated.
    #[serde(rename = "IsTruncated")]
    pub is_truncated: bool,
    /// Marker to use for the next page of results.
    #[serde(rename = "NextMarker", default)]
    pub next_marker: Option<String>,
    /// Container for the bucket list.
    #[serde(rename = "Buckets", default)]
    pub buckets: BucketsContainer,
}

/// Wrapper container for the bucket list in XML.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct BucketsContainer {
    /// The list of buckets.
    #[serde(rename = "Bucket", default)]
    pub bucket: Vec<BucketInfo>,
}

/// Metadata for a single bucket.
#[derive(Debug, Clone, Deserialize)]
pub struct BucketInfo {
    /// Bucket name.
    #[serde(rename = "Name")]
    pub name: String,
    /// Region/location of the bucket.
    #[serde(rename = "Location")]
    pub location: String,
    /// Creation date.
    #[serde(rename = "CreationDate")]
    pub creation_date: String,
    /// Storage class.
    #[serde(rename = "StorageClass")]
    pub storage_class: StorageClass,
    /// Extranet endpoint.
    #[serde(rename = "ExtranetEndpoint", default)]
    pub extranet_endpoint: String,
    /// Intranet endpoint.
    #[serde(rename = "IntranetEndpoint", default)]
    pub intranet_endpoint: String,
}

/// Response from a GetBucketInfo operation (XML-deserialized).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "BucketInfo")]
pub struct GetBucketInfoResponse {
    /// The bucket metadata.
    #[serde(rename = "Bucket")]
    pub bucket: BucketInfoDetail,
}

/// Detailed bucket metadata from GetBucketInfo.
#[derive(Debug, Clone, Deserialize)]
pub struct BucketInfoDetail {
    /// Bucket name.
    #[serde(rename = "Name")]
    pub name: String,
    /// Region/location.
    #[serde(rename = "Location")]
    pub location: String,
    /// Creation date.
    #[serde(rename = "CreationDate")]
    pub creation_date: String,
    /// Storage class.
    #[serde(rename = "StorageClass")]
    pub storage_class: StorageClass,
    /// Extranet endpoint.
    #[serde(rename = "ExtranetEndpoint", default)]
    pub extranet_endpoint: String,
    /// Intranet endpoint.
    #[serde(rename = "IntranetEndpoint", default)]
    pub intranet_endpoint: String,
    /// Access control list.
    #[serde(rename = "AccessControlList", default)]
    pub access_control_list: Option<AccessControlList>,
}

/// Access control list from GetBucketInfo.
#[derive(Debug, Clone, Deserialize)]
pub struct AccessControlList {
    /// The grant permission.
    #[serde(rename = "Grant")]
    pub grant: String,
}

/// Response from a CopyObject operation (XML-deserialized).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "CopyObjectResult")]
pub struct CopyObjectResponse {
    /// Last modified timestamp of the copied object.
    #[serde(rename = "LastModified")]
    pub last_modified: DateTime<Utc>,
    /// ETag of the copied object.
    #[serde(rename = "ETag")]
    pub etag: String,
}

/// Response from an InitiateMultipartUpload operation (XML-deserialized).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "InitiateMultipartUploadResult")]
pub struct InitiateMultipartUploadResponse {
    /// Bucket name.
    #[serde(rename = "Bucket")]
    pub bucket: String,
    /// Object key.
    #[serde(rename = "Key")]
    pub key: String,
    /// Upload ID to use for subsequent part uploads.
    #[serde(rename = "UploadId")]
    pub upload_id: String,
}

/// Response from an UploadPart operation.
#[derive(Debug)]
pub struct UploadPartResponse {
    /// ETag of the uploaded part.
    pub etag: String,
}

/// Response from a CompleteMultipartUpload operation (XML-deserialized).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "CompleteMultipartUploadResult")]
pub struct CompleteMultipartUploadResponse {
    /// URL location of the completed object.
    #[serde(rename = "Location")]
    pub location: String,
    /// Bucket name.
    #[serde(rename = "Bucket")]
    pub bucket: String,
    /// Object key.
    #[serde(rename = "Key")]
    pub key: String,
    /// ETag of the completed object.
    #[serde(rename = "ETag")]
    pub etag: String,
}

/// Response from an AbortMultipartUpload operation.
#[derive(Debug)]
pub struct AbortMultipartUploadResponse {
    /// OSS request ID.
    pub request_id: Option<String>,
}

/// Response from a ListParts operation (XML-deserialized).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "ListPartsResult")]
pub struct ListPartsResponse {
    /// Bucket name.
    #[serde(rename = "Bucket")]
    pub bucket: String,
    /// Object key.
    #[serde(rename = "Key")]
    pub key: String,
    /// Upload ID.
    #[serde(rename = "UploadId")]
    pub upload_id: String,
    /// Maximum number of parts returned.
    #[serde(rename = "MaxParts")]
    pub max_parts: u32,
    /// Whether the results are truncated.
    #[serde(rename = "IsTruncated")]
    pub is_truncated: bool,
    /// Marker for the next page of results.
    #[serde(rename = "NextPartNumberMarker", default)]
    pub next_part_number_marker: Option<u32>,
    /// Part entries.
    #[serde(rename = "Part", default)]
    pub parts: Vec<PartInfo>,
}

/// Metadata for a single part in a ListParts response.
#[derive(Debug, Clone, Deserialize)]
pub struct PartInfo {
    /// Part number.
    #[serde(rename = "PartNumber")]
    pub part_number: u32,
    /// Last modified timestamp.
    #[serde(rename = "LastModified")]
    pub last_modified: DateTime<Utc>,
    /// ETag of the part.
    #[serde(rename = "ETag")]
    pub etag: String,
    /// Size in bytes.
    #[serde(rename = "Size")]
    pub size: u64,
}

/// Response from a DeleteMultipleObjects operation (XML-deserialized).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "DeleteResult")]
pub struct DeleteMultipleObjectsResponse {
    /// Objects that were successfully deleted.
    #[serde(rename = "Deleted", default)]
    pub deleted: Vec<DeletedObject>,
}

/// A successfully deleted object in a batch delete response.
#[derive(Debug, Clone, Deserialize)]
pub struct DeletedObject {
    /// The key of the deleted object.
    #[serde(rename = "Key")]
    pub key: String,
}

/// Response from a ListMultipartUploads operation (XML-deserialized).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "ListMultipartUploadsResult")]
pub struct ListMultipartUploadsResponse {
    /// Bucket name.
    #[serde(rename = "Bucket")]
    pub bucket: String,
    /// The prefix used to filter results.
    #[serde(rename = "Prefix", default)]
    pub prefix: String,
    /// Maximum number of uploads returned.
    #[serde(rename = "MaxUploads")]
    pub max_uploads: u32,
    /// Whether the results are truncated.
    #[serde(rename = "IsTruncated")]
    pub is_truncated: bool,
    /// Key marker for the next page.
    #[serde(rename = "NextKeyMarker", default)]
    pub next_key_marker: Option<String>,
    /// Upload ID marker for the next page.
    #[serde(rename = "NextUploadIdMarker", default)]
    pub next_upload_id_marker: Option<String>,
    /// In-progress multipart uploads.
    #[serde(rename = "Upload", default)]
    pub uploads: Vec<MultipartUploadInfo>,
}

/// Metadata for a single in-progress multipart upload.
#[derive(Debug, Clone, Deserialize)]
pub struct MultipartUploadInfo {
    /// The object key.
    #[serde(rename = "Key")]
    pub key: String,
    /// The upload ID.
    #[serde(rename = "UploadId")]
    pub upload_id: String,
    /// When the upload was initiated.
    #[serde(rename = "Initiated")]
    pub initiated: DateTime<Utc>,
    /// Storage class of the upload.
    #[serde(rename = "StorageClass")]
    pub storage_class: StorageClass,
}

/// Response from a GetBucketLocation operation.
#[derive(Debug, Clone)]
pub struct GetBucketLocationResponse {
    /// The region/location string (e.g., "oss-cn-hangzhou").
    pub location: String,
}

/// Internal XML wrapper for deserializing `<LocationConstraint>`.
#[derive(Deserialize)]
#[serde(rename = "LocationConstraint")]
pub(crate) struct LocationConstraintXml {
    #[serde(rename = "$text")]
    pub location: String,
}

/// Response from a RestoreObject operation.
#[derive(Debug)]
pub struct RestoreObjectResponse {
    /// OSS request ID.
    pub request_id: Option<String>,
}

/// Response from an AppendObject operation.
#[derive(Debug)]
pub struct AppendObjectResponse {
    /// The position for the next append operation.
    pub next_append_position: u64,
    /// CRC64 checksum of the object.
    pub crc64: Option<String>,
    /// OSS request ID.
    pub request_id: Option<String>,
}

/// Response from a GetObjectAcl operation (XML-deserialized).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "AccessControlPolicy")]
pub struct GetObjectAclResponse {
    /// The access control list.
    #[serde(rename = "AccessControlList")]
    pub access_control_list: ObjectAccessControlList,
}

/// Access control list from GetObjectAcl.
#[derive(Debug, Clone, Deserialize)]
pub struct ObjectAccessControlList {
    /// The grant permission (e.g., "private", "public-read").
    #[serde(rename = "Grant")]
    pub grant: String,
}

/// Response from a PutObjectAcl operation.
#[derive(Debug)]
pub struct PutObjectAclResponse {
    /// OSS request ID.
    pub request_id: Option<String>,
}

/// Response from a GetObjectTagging operation (XML-deserialized).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "Tagging")]
pub struct GetObjectTaggingResponse {
    /// The tag set.
    #[serde(rename = "TagSet")]
    pub tag_set: TagSet,
}

/// A set of tags.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TagSet {
    /// The list of tags.
    #[serde(rename = "Tag", default)]
    pub tags: Vec<Tag>,
}

/// A single key-value tag.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tag {
    /// Tag key.
    #[serde(rename = "Key")]
    pub key: String,
    /// Tag value.
    #[serde(rename = "Value")]
    pub value: String,
}

/// XML wrapper for PutObjectTagging request body.
#[derive(Debug, Serialize)]
#[serde(rename = "Tagging")]
pub(crate) struct TaggingXml {
    #[serde(rename = "TagSet")]
    pub tag_set: TagSet,
}

/// Response from a PutObjectTagging operation.
#[derive(Debug)]
pub struct PutObjectTaggingResponse {
    /// OSS request ID.
    pub request_id: Option<String>,
}

/// Response from a DeleteObjectTagging operation.
#[derive(Debug)]
pub struct DeleteObjectTaggingResponse {
    /// OSS request ID.
    pub request_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_list_objects_v2_response() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult>
    <Name>my-bucket</Name>
    <Prefix>photos/</Prefix>
    <MaxKeys>100</MaxKeys>
    <KeyCount>2</KeyCount>
    <IsTruncated>false</IsTruncated>
    <Contents>
        <Key>photos/a.jpg</Key>
        <LastModified>2024-01-01T00:00:00.000Z</LastModified>
        <ETag>"abc123"</ETag>
        <Size>1024</Size>
        <StorageClass>Standard</StorageClass>
    </Contents>
    <Contents>
        <Key>photos/b.jpg</Key>
        <LastModified>2024-01-02T00:00:00.000Z</LastModified>
        <ETag>"def456"</ETag>
        <Size>2048</Size>
        <StorageClass>Standard</StorageClass>
    </Contents>
</ListBucketResult>"#;
        let resp: ListObjectsV2Response = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(resp.name, "my-bucket");
        assert_eq!(resp.prefix, "photos/");
        assert_eq!(resp.max_keys, 100);
        assert_eq!(resp.key_count, 2);
        assert!(!resp.is_truncated);
        assert_eq!(resp.contents.len(), 2);
        assert_eq!(resp.contents[0].key, "photos/a.jpg");
        assert_eq!(resp.contents[0].size, 1024);
        assert_eq!(resp.contents[1].key, "photos/b.jpg");
    }

    #[test]
    fn deserialize_list_objects_v2_with_common_prefixes() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult>
    <Name>my-bucket</Name>
    <Prefix></Prefix>
    <MaxKeys>100</MaxKeys>
    <KeyCount>0</KeyCount>
    <IsTruncated>false</IsTruncated>
    <CommonPrefixes>
        <Prefix>photos/</Prefix>
    </CommonPrefixes>
    <CommonPrefixes>
        <Prefix>videos/</Prefix>
    </CommonPrefixes>
</ListBucketResult>"#;
        let resp: ListObjectsV2Response = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(resp.common_prefixes.len(), 2);
        assert_eq!(resp.common_prefixes[0].prefix, "photos/");
        assert_eq!(resp.common_prefixes[1].prefix, "videos/");
        assert!(resp.contents.is_empty());
    }

    #[test]
    fn deserialize_list_objects_v2_empty() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult>
    <Name>my-bucket</Name>
    <Prefix></Prefix>
    <MaxKeys>100</MaxKeys>
    <KeyCount>0</KeyCount>
    <IsTruncated>false</IsTruncated>
</ListBucketResult>"#;
        let resp: ListObjectsV2Response = quick_xml::de::from_str(xml).unwrap();
        assert!(resp.contents.is_empty());
        assert!(resp.common_prefixes.is_empty());
    }

    #[test]
    fn deserialize_list_buckets_response() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListAllMyBucketsResult>
    <Prefix></Prefix>
    <Marker></Marker>
    <MaxKeys>100</MaxKeys>
    <IsTruncated>false</IsTruncated>
    <Buckets>
        <Bucket>
            <Name>bucket-one</Name>
            <Location>oss-cn-hangzhou</Location>
            <CreationDate>2024-01-01T00:00:00.000Z</CreationDate>
            <StorageClass>Standard</StorageClass>
            <ExtranetEndpoint>oss-cn-hangzhou.aliyuncs.com</ExtranetEndpoint>
            <IntranetEndpoint>oss-cn-hangzhou-internal.aliyuncs.com</IntranetEndpoint>
        </Bucket>
        <Bucket>
            <Name>bucket-two</Name>
            <Location>oss-us-west-1</Location>
            <CreationDate>2024-06-15T12:00:00.000Z</CreationDate>
            <StorageClass>IA</StorageClass>
            <ExtranetEndpoint>oss-us-west-1.aliyuncs.com</ExtranetEndpoint>
            <IntranetEndpoint>oss-us-west-1-internal.aliyuncs.com</IntranetEndpoint>
        </Bucket>
    </Buckets>
</ListAllMyBucketsResult>"#;
        let resp: ListBucketsResponse = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(resp.max_keys, 100);
        assert!(!resp.is_truncated);
        assert_eq!(resp.buckets.bucket.len(), 2);
        assert_eq!(resp.buckets.bucket[0].name, "bucket-one");
        assert_eq!(resp.buckets.bucket[0].location, "oss-cn-hangzhou");
        assert_eq!(resp.buckets.bucket[1].name, "bucket-two");
        assert_eq!(
            resp.buckets.bucket[1].storage_class,
            StorageClass::InfrequentAccess
        );
    }

    #[test]
    fn deserialize_list_buckets_empty() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListAllMyBucketsResult>
    <Prefix></Prefix>
    <Marker></Marker>
    <MaxKeys>100</MaxKeys>
    <IsTruncated>false</IsTruncated>
    <Buckets></Buckets>
</ListAllMyBucketsResult>"#;
        let resp: ListBucketsResponse = quick_xml::de::from_str(xml).unwrap();
        assert!(resp.buckets.bucket.is_empty());
    }

    #[test]
    fn deserialize_get_bucket_info_response() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<BucketInfo>
    <Bucket>
        <Name>my-bucket</Name>
        <Location>oss-cn-hangzhou</Location>
        <CreationDate>2024-01-01T00:00:00.000Z</CreationDate>
        <StorageClass>Standard</StorageClass>
        <ExtranetEndpoint>oss-cn-hangzhou.aliyuncs.com</ExtranetEndpoint>
        <IntranetEndpoint>oss-cn-hangzhou-internal.aliyuncs.com</IntranetEndpoint>
        <AccessControlList>
            <Grant>private</Grant>
        </AccessControlList>
    </Bucket>
</BucketInfo>"#;
        let resp: GetBucketInfoResponse = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(resp.bucket.name, "my-bucket");
        assert_eq!(resp.bucket.location, "oss-cn-hangzhou");
        assert_eq!(resp.bucket.storage_class, StorageClass::Standard);
        let acl = resp.bucket.access_control_list.unwrap();
        assert_eq!(acl.grant, "private");
    }

    #[test]
    fn deserialize_copy_object_response() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<CopyObjectResult>
    <LastModified>2024-03-15T10:30:00.000Z</LastModified>
    <ETag>"abc123def456"</ETag>
</CopyObjectResult>"#;
        let resp: CopyObjectResponse = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(
            resp.last_modified,
            "2024-03-15T10:30:00Z".parse::<DateTime<Utc>>().unwrap()
        );
        assert_eq!(resp.etag, "\"abc123def456\"");
    }

    #[test]
    fn deserialize_initiate_multipart_upload_response() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<InitiateMultipartUploadResult>
    <Bucket>test-bucket</Bucket>
    <Key>large-file.bin</Key>
    <UploadId>0004B9894A22E5B1-9C6D-1234-5678-ABCDEF012345</UploadId>
</InitiateMultipartUploadResult>"#;
        let resp: InitiateMultipartUploadResponse = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(resp.bucket, "test-bucket");
        assert_eq!(resp.key, "large-file.bin");
        assert_eq!(
            resp.upload_id,
            "0004B9894A22E5B1-9C6D-1234-5678-ABCDEF012345"
        );
    }

    #[test]
    fn deserialize_complete_multipart_upload_response() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<CompleteMultipartUploadResult>
    <Location>https://test-bucket.oss-cn-hangzhou.aliyuncs.com/large-file.bin</Location>
    <Bucket>test-bucket</Bucket>
    <Key>large-file.bin</Key>
    <ETag>"final-etag-123"</ETag>
</CompleteMultipartUploadResult>"#;
        let resp: CompleteMultipartUploadResponse = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(resp.bucket, "test-bucket");
        assert_eq!(resp.key, "large-file.bin");
        assert_eq!(resp.etag, "\"final-etag-123\"");
        assert!(resp.location.contains("large-file.bin"));
    }

    #[test]
    fn deserialize_list_parts_response() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListPartsResult>
    <Bucket>test-bucket</Bucket>
    <Key>large-file.bin</Key>
    <UploadId>upload-id-123</UploadId>
    <MaxParts>1000</MaxParts>
    <IsTruncated>false</IsTruncated>
    <Part>
        <PartNumber>1</PartNumber>
        <LastModified>2024-03-15T10:30:00.000Z</LastModified>
        <ETag>"part1-etag"</ETag>
        <Size>5242880</Size>
    </Part>
    <Part>
        <PartNumber>2</PartNumber>
        <LastModified>2024-03-15T10:31:00.000Z</LastModified>
        <ETag>"part2-etag"</ETag>
        <Size>3145728</Size>
    </Part>
</ListPartsResult>"#;
        let resp: ListPartsResponse = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(resp.bucket, "test-bucket");
        assert_eq!(resp.upload_id, "upload-id-123");
        assert_eq!(resp.max_parts, 1000);
        assert!(!resp.is_truncated);
        assert_eq!(resp.parts.len(), 2);
        assert_eq!(resp.parts[0].part_number, 1);
        assert_eq!(resp.parts[0].size, 5242880);
        assert_eq!(resp.parts[1].part_number, 2);
    }

    #[test]
    fn deserialize_delete_multiple_objects_response() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<DeleteResult>
    <Deleted><Key>file1.txt</Key></Deleted>
    <Deleted><Key>file2.txt</Key></Deleted>
</DeleteResult>"#;
        let resp: DeleteMultipleObjectsResponse = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(resp.deleted.len(), 2);
        assert_eq!(resp.deleted[0].key, "file1.txt");
        assert_eq!(resp.deleted[1].key, "file2.txt");
    }

    #[test]
    fn deserialize_list_multipart_uploads_response() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListMultipartUploadsResult>
    <Bucket>test-bucket</Bucket>
    <Prefix></Prefix>
    <MaxUploads>1000</MaxUploads>
    <IsTruncated>false</IsTruncated>
    <Upload>
        <Key>large-file.bin</Key>
        <UploadId>upload-id-001</UploadId>
        <Initiated>2024-03-15T10:30:00.000Z</Initiated>
        <StorageClass>Standard</StorageClass>
    </Upload>
    <Upload>
        <Key>another-file.zip</Key>
        <UploadId>upload-id-002</UploadId>
        <Initiated>2024-03-16T12:00:00.000Z</Initiated>
        <StorageClass>IA</StorageClass>
    </Upload>
</ListMultipartUploadsResult>"#;
        let resp: ListMultipartUploadsResponse = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(resp.bucket, "test-bucket");
        assert_eq!(resp.max_uploads, 1000);
        assert!(!resp.is_truncated);
        assert_eq!(resp.uploads.len(), 2);
        assert_eq!(resp.uploads[0].key, "large-file.bin");
        assert_eq!(resp.uploads[0].upload_id, "upload-id-001");
        assert_eq!(
            resp.uploads[1].storage_class,
            StorageClass::InfrequentAccess
        );
    }

    #[test]
    fn deserialize_list_multipart_uploads_empty() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListMultipartUploadsResult>
    <Bucket>test-bucket</Bucket>
    <Prefix></Prefix>
    <MaxUploads>1000</MaxUploads>
    <IsTruncated>false</IsTruncated>
</ListMultipartUploadsResult>"#;
        let resp: ListMultipartUploadsResponse = quick_xml::de::from_str(xml).unwrap();
        assert!(resp.uploads.is_empty());
    }

    #[test]
    fn deserialize_get_object_acl_response() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AccessControlPolicy>
    <AccessControlList>
        <Grant>public-read</Grant>
    </AccessControlList>
</AccessControlPolicy>"#;
        let resp: GetObjectAclResponse = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(resp.access_control_list.grant, "public-read");
    }

    #[test]
    fn deserialize_get_object_tagging_response() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Tagging>
    <TagSet>
        <Tag>
            <Key>env</Key>
            <Value>prod</Value>
        </Tag>
        <Tag>
            <Key>team</Key>
            <Value>backend</Value>
        </Tag>
    </TagSet>
</Tagging>"#;
        let resp: GetObjectTaggingResponse = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(resp.tag_set.tags.len(), 2);
        assert_eq!(resp.tag_set.tags[0].key, "env");
        assert_eq!(resp.tag_set.tags[0].value, "prod");
        assert_eq!(resp.tag_set.tags[1].key, "team");
    }

    #[test]
    fn deserialize_get_object_tagging_empty() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Tagging>
    <TagSet></TagSet>
</Tagging>"#;
        let resp: GetObjectTaggingResponse = quick_xml::de::from_str(xml).unwrap();
        assert!(resp.tag_set.tags.is_empty());
    }

    #[test]
    fn tagging_xml_serializes() {
        let wrapper = TaggingXml {
            tag_set: TagSet {
                tags: vec![Tag {
                    key: "env".to_string(),
                    value: "prod".to_string(),
                }],
            },
        };
        let xml = quick_xml::se::to_string(&wrapper).unwrap();
        assert!(xml.contains("<Key>env</Key>"));
        assert!(xml.contains("<Value>prod</Value>"));
    }
}

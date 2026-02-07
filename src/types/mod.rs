//! Request and response types for OSS operations.

pub mod common;
pub mod request;
pub mod response;

pub use common::{BucketName, MetadataDirective, ObjectAcl, ObjectKey, Region, StorageClass};
pub use request::{
    AbortMultipartUploadRequest, AbortMultipartUploadRequestBuilder, AppendObjectRequest,
    AppendObjectRequestBuilder, CompleteMultipartUploadRequest,
    CompleteMultipartUploadRequestBuilder, CompleteMultipartUploadXml, CompletedPart,
    CopyObjectRequest, CopyObjectRequestBuilder, CreateBucketRequest, CreateBucketRequestBuilder,
    DeleteBucketRequest, DeleteBucketRequestBuilder, DeleteMultipleObjectsRequest,
    DeleteMultipleObjectsRequestBuilder, DeleteObjectRequest, DeleteObjectRequestBuilder,
    DeleteObjectTaggingRequest, DeleteObjectTaggingRequestBuilder, GetBucketInfoRequest,
    GetBucketInfoRequestBuilder, GetBucketLocationRequest, GetBucketLocationRequestBuilder,
    GetObjectAclRequest, GetObjectAclRequestBuilder, GetObjectRequest, GetObjectRequestBuilder,
    GetObjectTaggingRequest, GetObjectTaggingRequestBuilder, HeadObjectRequest,
    HeadObjectRequestBuilder, InitiateMultipartUploadRequest,
    InitiateMultipartUploadRequestBuilder, ListBucketsRequest, ListBucketsRequestBuilder,
    ListMultipartUploadsRequest, ListMultipartUploadsRequestBuilder, ListObjectsV2Request,
    ListObjectsV2RequestBuilder, ListPartsRequest, ListPartsRequestBuilder, PresignedUrlRequest,
    PresignedUrlRequestBuilder, PutObjectAclRequest, PutObjectAclRequestBuilder, PutObjectRequest,
    PutObjectRequestBuilder, PutObjectTaggingRequest, PutObjectTaggingRequestBuilder,
    RestoreObjectRequest, RestoreObjectRequestBuilder, UploadPartRequest, UploadPartRequestBuilder,
};
pub use response::{
    AbortMultipartUploadResponse, AccessControlList, AppendObjectResponse, BucketInfo,
    BucketInfoDetail, BucketsContainer, CommonPrefix, CompleteMultipartUploadResponse,
    CopyObjectResponse, CreateBucketResponse, DeleteBucketResponse, DeleteMultipleObjectsResponse,
    DeleteObjectResponse, DeleteObjectTaggingResponse, DeletedObject, GetBucketInfoResponse,
    GetBucketLocationResponse, GetObjectAclResponse, GetObjectResponse, GetObjectTaggingResponse,
    HeadObjectResponse, InitiateMultipartUploadResponse, ListBucketsResponse,
    ListMultipartUploadsResponse, ListObjectsV2Response, ListPartsResponse, MultipartUploadInfo,
    ObjectAccessControlList, ObjectBody, ObjectInfo, PartInfo, PutObjectAclResponse,
    PutObjectResponse, PutObjectTaggingResponse, RestoreObjectResponse, Tag, TagSet,
    UploadPartResponse,
};

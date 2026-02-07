//! Integration tests for bucket operations using wiremock.

use rs_ali_oss::OssClient;
use rs_ali_oss::config::ClientBuilder;
use rs_ali_oss::types::common::BucketName;
use rs_ali_oss::types::request::{
    CreateBucketRequestBuilder, DeleteBucketRequestBuilder, GetBucketInfoRequestBuilder,
    ListBucketsRequestBuilder,
};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Create an `OssClient` that points at the given mock server.
fn mock_client(server: &MockServer) -> OssClient {
    OssClient::from_builder(
        ClientBuilder::new()
            .access_key_id("test-key-id")
            .access_key_secret("test-key-secret")
            .region("cn-hangzhou")
            .endpoint(server.uri())
            .allow_insecure(true)
            .max_retries(0),
    )
    .unwrap()
}

// ---- CreateBucket ----

#[tokio::test]
async fn create_bucket_returns_request_id() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).insert_header("x-oss-request-id", "CREATE-001"))
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = CreateBucketRequestBuilder::new()
        .bucket(BucketName::new("new-bucket").unwrap())
        .build()
        .unwrap();

    let response = client.create_bucket(request).await.unwrap();
    assert_eq!(response.request_id.as_deref(), Some("CREATE-001"));
}

#[tokio::test]
async fn create_bucket_with_storage_class_sends_xml_body() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).insert_header("x-oss-request-id", "CREATE-SC"))
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = CreateBucketRequestBuilder::new()
        .bucket(BucketName::new("ia-bucket").unwrap())
        .storage_class(rs_ali_oss::StorageClass::InfrequentAccess)
        .build()
        .unwrap();

    let response = client.create_bucket(request).await.unwrap();
    assert_eq!(response.request_id.as_deref(), Some("CREATE-SC"));
}

// ---- DeleteBucket ----

#[tokio::test]
async fn delete_bucket_returns_request_id() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(204).insert_header("x-oss-request-id", "DELETE-001"))
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = DeleteBucketRequestBuilder::new()
        .bucket(BucketName::new("old-bucket").unwrap())
        .build()
        .unwrap();

    let response = client.delete_bucket(request).await.unwrap();
    assert_eq!(response.request_id.as_deref(), Some("DELETE-001"));
}

// ---- ListBuckets ----

#[tokio::test]
async fn list_buckets_parses_xml_response() {
    let server = MockServer::start().await;

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
            <CreationDate>2025-01-01T00:00:00.000Z</CreationDate>
            <StorageClass>Standard</StorageClass>
            <ExtranetEndpoint>oss-cn-hangzhou.aliyuncs.com</ExtranetEndpoint>
            <IntranetEndpoint>oss-cn-hangzhou-internal.aliyuncs.com</IntranetEndpoint>
        </Bucket>
        <Bucket>
            <Name>bucket-two</Name>
            <Location>oss-us-west-1</Location>
            <CreationDate>2025-06-15T12:00:00.000Z</CreationDate>
            <StorageClass>IA</StorageClass>
            <ExtranetEndpoint>oss-us-west-1.aliyuncs.com</ExtranetEndpoint>
            <IntranetEndpoint>oss-us-west-1-internal.aliyuncs.com</IntranetEndpoint>
        </Bucket>
    </Buckets>
</ListAllMyBucketsResult>"#;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/xml")
                .set_body_string(xml),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = ListBucketsRequestBuilder::new().build().unwrap();

    let response = client.list_buckets(request).await.unwrap();
    assert_eq!(response.max_keys, 100);
    assert!(!response.is_truncated);
    assert_eq!(response.buckets.bucket.len(), 2);
    assert_eq!(response.buckets.bucket[0].name, "bucket-one");
    assert_eq!(response.buckets.bucket[0].location, "oss-cn-hangzhou");
    assert_eq!(response.buckets.bucket[1].name, "bucket-two");
    assert_eq!(
        response.buckets.bucket[1].storage_class,
        rs_ali_oss::StorageClass::InfrequentAccess
    );
}

#[tokio::test]
async fn list_buckets_with_prefix_and_max_keys() {
    let server = MockServer::start().await;

    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListAllMyBucketsResult>
    <Prefix>my-</Prefix>
    <Marker></Marker>
    <MaxKeys>5</MaxKeys>
    <IsTruncated>false</IsTruncated>
    <Buckets></Buckets>
</ListAllMyBucketsResult>"#;

    Mock::given(method("GET"))
        .and(path("/"))
        .and(query_param("prefix", "my-"))
        .and(query_param("max-keys", "5"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/xml")
                .set_body_string(xml),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = ListBucketsRequestBuilder::new()
        .prefix("my-")
        .max_keys(5)
        .build()
        .unwrap();

    let response = client.list_buckets(request).await.unwrap();
    assert_eq!(response.prefix, "my-");
    assert_eq!(response.max_keys, 5);
    assert!(response.buckets.bucket.is_empty());
}

// ---- GetBucketInfo ----

#[tokio::test]
async fn get_bucket_info_parses_xml_response() {
    let server = MockServer::start().await;

    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<BucketInfo>
    <Bucket>
        <Name>my-bucket</Name>
        <Location>oss-cn-hangzhou</Location>
        <CreationDate>2025-01-01T00:00:00.000Z</CreationDate>
        <StorageClass>Standard</StorageClass>
        <ExtranetEndpoint>oss-cn-hangzhou.aliyuncs.com</ExtranetEndpoint>
        <IntranetEndpoint>oss-cn-hangzhou-internal.aliyuncs.com</IntranetEndpoint>
        <AccessControlList>
            <Grant>private</Grant>
        </AccessControlList>
    </Bucket>
</BucketInfo>"#;

    Mock::given(method("GET"))
        .and(path("/"))
        .and(query_param("bucketInfo", ""))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/xml")
                .set_body_string(xml),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = GetBucketInfoRequestBuilder::new()
        .bucket(BucketName::new("my-bucket").unwrap())
        .build()
        .unwrap();

    let response = client.get_bucket_info(request).await.unwrap();
    assert_eq!(response.bucket.name, "my-bucket");
    assert_eq!(response.bucket.location, "oss-cn-hangzhou");
    assert_eq!(
        response.bucket.storage_class,
        rs_ali_oss::StorageClass::Standard
    );
    let acl = response.bucket.access_control_list.unwrap();
    assert_eq!(acl.grant, "private");
}

// ---- Error handling ----

#[tokio::test]
async fn create_bucket_conflict_returns_error() {
    let server = MockServer::start().await;

    let error_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Error>
    <Code>BucketAlreadyExists</Code>
    <Message>The requested bucket name is not available.</Message>
    <RequestId>ERR-409-REQ</RequestId>
    <HostId>oss-cn-hangzhou.aliyuncs.com</HostId>
</Error>"#;

    Mock::given(method("PUT"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(409)
                .insert_header("content-type", "application/xml")
                .set_body_string(error_xml),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = CreateBucketRequestBuilder::new()
        .bucket(BucketName::new("existing-bucket").unwrap())
        .build()
        .unwrap();

    let err = client.create_bucket(request).await.unwrap_err();
    let err_str = err.to_string();
    assert!(err_str.contains("BucketAlreadyExists"), "error: {err_str}");
}

#[tokio::test]
async fn delete_bucket_not_empty_returns_error() {
    let server = MockServer::start().await;

    let error_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Error>
    <Code>BucketNotEmpty</Code>
    <Message>The bucket you tried to delete is not empty.</Message>
    <RequestId>ERR-409-DEL</RequestId>
    <HostId>oss-cn-hangzhou.aliyuncs.com</HostId>
</Error>"#;

    Mock::given(method("DELETE"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(409)
                .insert_header("content-type", "application/xml")
                .set_body_string(error_xml),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = DeleteBucketRequestBuilder::new()
        .bucket(BucketName::new("non-empty-bucket").unwrap())
        .build()
        .unwrap();

    let err = client.delete_bucket(request).await.unwrap_err();
    let err_str = err.to_string();
    assert!(err_str.contains("BucketNotEmpty"), "error: {err_str}");
}

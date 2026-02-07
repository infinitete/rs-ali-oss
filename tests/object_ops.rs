//! Integration tests for object operations using wiremock.

use rs_ali_oss::OssClient;
use rs_ali_oss::config::ClientBuilder;
use rs_ali_oss::types::common::{BucketName, ObjectKey};
use rs_ali_oss::types::request::{
    CopyObjectRequestBuilder, DeleteMultipleObjectsRequestBuilder, DeleteObjectRequestBuilder,
    GetObjectRequestBuilder, HeadObjectRequestBuilder, ListObjectsV2RequestBuilder,
    PutObjectRequestBuilder,
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

// ---- PutObject ----

#[tokio::test]
async fn put_object_returns_etag_and_request_id() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/hello.txt"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("etag", "\"abc123\"")
                .insert_header("x-oss-request-id", "REQ-001"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = PutObjectRequestBuilder::new()
        .bucket(BucketName::new("my-bucket").unwrap())
        .key(ObjectKey::new("hello.txt").unwrap())
        .body(b"Hello, OSS!".to_vec())
        .content_type("text/plain")
        .build()
        .unwrap();

    let response = client.put_object(request).await.unwrap();
    assert_eq!(response.etag, "abc123");
    assert_eq!(response.request_id.as_deref(), Some("REQ-001"));
}

#[tokio::test]
async fn put_object_with_metadata_sends_request() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/doc.pdf"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("etag", "\"meta-etag\"")
                .insert_header("x-oss-request-id", "REQ-META"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = PutObjectRequestBuilder::new()
        .bucket(BucketName::new("my-bucket").unwrap())
        .key(ObjectKey::new("doc.pdf").unwrap())
        .body(b"PDF content".to_vec())
        .content_type("application/pdf")
        .metadata("author", "alice")
        .build()
        .unwrap();

    let response = client.put_object(request).await.unwrap();
    assert_eq!(response.etag, "meta-etag");
}

// ---- GetObject ----

#[tokio::test]
async fn get_object_returns_body_and_headers() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/hello.txt"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/plain")
                .insert_header("content-length", "11")
                .insert_header("etag", "\"get-etag\"")
                .set_body_bytes(b"Hello World"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = GetObjectRequestBuilder::new()
        .bucket(BucketName::new("my-bucket").unwrap())
        .key(ObjectKey::new("hello.txt").unwrap())
        .build()
        .unwrap();

    let response = client.get_object(request).await.unwrap();
    assert_eq!(response.content_type.as_deref(), Some("text/plain"));
    assert_eq!(response.content_length, Some(11));
    assert_eq!(response.etag.as_deref(), Some("get-etag"));

    let body = response.body.bytes().await.unwrap();
    assert_eq!(&body[..], b"Hello World");
}

#[tokio::test]
async fn get_object_with_range_sends_request() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/big-file.bin"))
        .respond_with(
            ResponseTemplate::new(206)
                .insert_header("content-type", "application/octet-stream")
                .insert_header("content-length", "100")
                .set_body_bytes(vec![0u8; 100]),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = GetObjectRequestBuilder::new()
        .bucket(BucketName::new("my-bucket").unwrap())
        .key(ObjectKey::new("big-file.bin").unwrap())
        .range("bytes=0-99")
        .build()
        .unwrap();

    let response = client.get_object(request).await.unwrap();
    assert_eq!(response.content_length, Some(100));
}

// ---- DeleteObject ----

#[tokio::test]
async fn delete_object_returns_request_id() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/old-file.txt"))
        .respond_with(ResponseTemplate::new(204).insert_header("x-oss-request-id", "DEL-001"))
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = DeleteObjectRequestBuilder::new()
        .bucket(BucketName::new("my-bucket").unwrap())
        .key(ObjectKey::new("old-file.txt").unwrap())
        .build()
        .unwrap();

    let response = client.delete_object(request).await.unwrap();
    assert_eq!(response.request_id.as_deref(), Some("DEL-001"));
}

// ---- HeadObject ----

#[tokio::test]
async fn head_object_returns_headers_and_metadata() {
    let server = MockServer::start().await;

    Mock::given(method("HEAD"))
        .and(path("/info.txt"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/plain")
                .insert_header("content-length", "42")
                .insert_header("etag", "\"head-etag\"")
                .insert_header("last-modified", "Sat, 01 Jan 2025 00:00:00 GMT")
                .insert_header("x-oss-meta-author", "bob")
                .insert_header("x-oss-meta-project", "demo"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = HeadObjectRequestBuilder::new()
        .bucket(BucketName::new("my-bucket").unwrap())
        .key(ObjectKey::new("info.txt").unwrap())
        .build()
        .unwrap();

    let response = client.head_object(request).await.unwrap();
    assert_eq!(response.content_type.as_deref(), Some("text/plain"));
    assert_eq!(response.content_length, Some(42));
    assert_eq!(response.etag.as_deref(), Some("head-etag"));
    assert!(response.last_modified.is_some());
    assert_eq!(
        response.metadata.get("author").map(|s| s.as_str()),
        Some("bob")
    );
    assert_eq!(
        response.metadata.get("project").map(|s| s.as_str()),
        Some("demo")
    );
}

// ---- ListObjectsV2 ----

#[tokio::test]
async fn list_objects_v2_parses_xml_response() {
    let server = MockServer::start().await;

    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult>
    <Name>my-bucket</Name>
    <Prefix></Prefix>
    <MaxKeys>100</MaxKeys>
    <KeyCount>2</KeyCount>
    <IsTruncated>false</IsTruncated>
    <Contents>
        <Key>file1.txt</Key>
        <LastModified>2025-01-01T00:00:00.000Z</LastModified>
        <ETag>"etag1"</ETag>
        <Size>1024</Size>
        <StorageClass>Standard</StorageClass>
    </Contents>
    <Contents>
        <Key>file2.txt</Key>
        <LastModified>2025-01-02T00:00:00.000Z</LastModified>
        <ETag>"etag2"</ETag>
        <Size>2048</Size>
        <StorageClass>Standard</StorageClass>
    </Contents>
</ListBucketResult>"#;

    Mock::given(method("GET"))
        .and(path("/"))
        .and(query_param("list-type", "2"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/xml")
                .set_body_string(xml),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = ListObjectsV2RequestBuilder::new()
        .bucket(BucketName::new("my-bucket").unwrap())
        .build()
        .unwrap();

    let response = client.list_objects_v2(request).await.unwrap();
    assert_eq!(response.name, "my-bucket");
    assert_eq!(response.max_keys, 100);
    assert_eq!(response.key_count, 2);
    assert!(!response.is_truncated);
    assert_eq!(response.contents.len(), 2);
    assert_eq!(response.contents[0].key, "file1.txt");
    assert_eq!(response.contents[0].size, 1024);
    assert_eq!(response.contents[1].key, "file2.txt");
}

#[tokio::test]
async fn list_objects_v2_with_prefix_and_delimiter() {
    let server = MockServer::start().await;

    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult>
    <Name>my-bucket</Name>
    <Prefix>photos/</Prefix>
    <MaxKeys>10</MaxKeys>
    <KeyCount>1</KeyCount>
    <IsTruncated>false</IsTruncated>
    <Contents>
        <Key>photos/a.jpg</Key>
        <LastModified>2025-01-01T00:00:00.000Z</LastModified>
        <ETag>"img-etag"</ETag>
        <Size>5000</Size>
        <StorageClass>Standard</StorageClass>
    </Contents>
    <CommonPrefixes>
        <Prefix>photos/2025/</Prefix>
    </CommonPrefixes>
</ListBucketResult>"#;

    Mock::given(method("GET"))
        .and(path("/"))
        .and(query_param("list-type", "2"))
        .and(query_param("prefix", "photos/"))
        .and(query_param("delimiter", "/"))
        .and(query_param("max-keys", "10"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/xml")
                .set_body_string(xml),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = ListObjectsV2RequestBuilder::new()
        .bucket(BucketName::new("my-bucket").unwrap())
        .prefix("photos/")
        .delimiter("/")
        .max_keys(10)
        .build()
        .unwrap();

    let response = client.list_objects_v2(request).await.unwrap();
    assert_eq!(response.prefix, "photos/");
    assert_eq!(response.contents.len(), 1);
    assert_eq!(response.common_prefixes.len(), 1);
    assert_eq!(response.common_prefixes[0].prefix, "photos/2025/");
}

// ---- CopyObject ----

#[tokio::test]
async fn copy_object_parses_xml_response() {
    let server = MockServer::start().await;

    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<CopyObjectResult>
    <LastModified>2025-02-01T12:00:00.000Z</LastModified>
    <ETag>"copy-etag-abc"</ETag>
</CopyObjectResult>"#;

    Mock::given(method("PUT"))
        .and(path("/dest-key.txt"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/xml")
                .set_body_string(xml),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = CopyObjectRequestBuilder::new()
        .bucket(BucketName::new("dest-bucket").unwrap())
        .key(ObjectKey::new("dest-key.txt").unwrap())
        .source_bucket(BucketName::new("src-bucket").unwrap())
        .source_key(ObjectKey::new("src-key.txt").unwrap())
        .build()
        .unwrap();

    let response = client.copy_object(request).await.unwrap();
    assert_eq!(
        response.last_modified,
        "2025-02-01T12:00:00Z"
            .parse::<chrono::DateTime<chrono::Utc>>()
            .unwrap()
    );
    assert_eq!(response.etag, "\"copy-etag-abc\"");
}

// ---- DeleteMultipleObjects ----

#[tokio::test]
async fn delete_multiple_objects_quiet_mode_empty_response() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .and(query_param("delete", ""))
        .respond_with(ResponseTemplate::new(200).set_body_string(""))
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = DeleteMultipleObjectsRequestBuilder::new()
        .bucket(BucketName::new("my-bucket").unwrap())
        .key(ObjectKey::new("file1.txt").unwrap())
        .key(ObjectKey::new("file2.txt").unwrap())
        .build()
        .unwrap();

    let response = client.delete_multiple_objects(request).await.unwrap();
    assert!(response.deleted.is_empty());
}

#[tokio::test]
async fn delete_multiple_objects_verbose_mode_returns_deleted() {
    let server = MockServer::start().await;

    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<DeleteResult>
    <Deleted><Key>a.txt</Key></Deleted>
    <Deleted><Key>b.txt</Key></Deleted>
    <Deleted><Key>c.txt</Key></Deleted>
</DeleteResult>"#;

    Mock::given(method("POST"))
        .and(path("/"))
        .and(query_param("delete", ""))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/xml")
                .set_body_string(xml),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = DeleteMultipleObjectsRequestBuilder::new()
        .bucket(BucketName::new("my-bucket").unwrap())
        .key(ObjectKey::new("a.txt").unwrap())
        .key(ObjectKey::new("b.txt").unwrap())
        .key(ObjectKey::new("c.txt").unwrap())
        .quiet(false)
        .build()
        .unwrap();

    let response = client.delete_multiple_objects(request).await.unwrap();
    assert_eq!(response.deleted.len(), 3);
    assert_eq!(response.deleted[0].key, "a.txt");
    assert_eq!(response.deleted[1].key, "b.txt");
    assert_eq!(response.deleted[2].key, "c.txt");
}

// ---- Error handling ----

#[tokio::test]
async fn server_error_404_returns_oss_error() {
    let server = MockServer::start().await;

    let error_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Error>
    <Code>NoSuchKey</Code>
    <Message>The specified key does not exist.</Message>
    <RequestId>ERR-404-REQ</RequestId>
    <HostId>my-bucket.oss-cn-hangzhou.aliyuncs.com</HostId>
</Error>"#;

    Mock::given(method("GET"))
        .and(path("/missing.txt"))
        .respond_with(
            ResponseTemplate::new(404)
                .insert_header("content-type", "application/xml")
                .set_body_string(error_xml),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = GetObjectRequestBuilder::new()
        .bucket(BucketName::new("my-bucket").unwrap())
        .key(ObjectKey::new("missing.txt").unwrap())
        .build()
        .unwrap();

    let err = client.get_object(request).await.unwrap_err();
    let err_str = err.to_string();
    assert!(err_str.contains("NoSuchKey"), "error: {err_str}");
    assert!(
        err_str.contains("The specified key does not exist"),
        "error: {err_str}"
    );
}

#[tokio::test]
async fn server_error_403_returns_access_denied() {
    let server = MockServer::start().await;

    let error_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Error>
    <Code>AccessDenied</Code>
    <Message>You have no right to access this object.</Message>
    <RequestId>ERR-403-REQ</RequestId>
    <HostId>bucket.oss-cn-hangzhou.aliyuncs.com</HostId>
</Error>"#;

    Mock::given(method("PUT"))
        .and(path("/protected.txt"))
        .respond_with(
            ResponseTemplate::new(403)
                .insert_header("content-type", "application/xml")
                .set_body_string(error_xml),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = PutObjectRequestBuilder::new()
        .bucket(BucketName::new("my-bucket").unwrap())
        .key(ObjectKey::new("protected.txt").unwrap())
        .body(b"data".to_vec())
        .build()
        .unwrap();

    let err = client.put_object(request).await.unwrap_err();
    let err_str = err.to_string();
    assert!(err_str.contains("AccessDenied"), "error: {err_str}");
}

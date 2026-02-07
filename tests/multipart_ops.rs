//! Integration tests for multipart upload operations using wiremock.

use rs_ali_oss::OssClient;
use rs_ali_oss::config::ClientBuilder;
use rs_ali_oss::types::common::{BucketName, ObjectKey};
use rs_ali_oss::types::request::{
    AbortMultipartUploadRequestBuilder, CompleteMultipartUploadRequestBuilder, CompletedPart,
    InitiateMultipartUploadRequestBuilder, ListPartsRequestBuilder, UploadPartRequestBuilder,
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

// ---- InitiateMultipartUpload ----

#[tokio::test]
async fn initiate_multipart_upload_parses_xml() {
    let server = MockServer::start().await;

    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<InitiateMultipartUploadResult>
    <Bucket>test-bucket</Bucket>
    <Key>large-file.bin</Key>
    <UploadId>UPLOAD-ID-12345</UploadId>
</InitiateMultipartUploadResult>"#;

    Mock::given(method("POST"))
        .and(path("/large-file.bin"))
        .and(query_param("uploads", ""))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/xml")
                .set_body_string(xml),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = InitiateMultipartUploadRequestBuilder::new()
        .bucket(BucketName::new("test-bucket").unwrap())
        .key(ObjectKey::new("large-file.bin").unwrap())
        .content_type("application/octet-stream")
        .build()
        .unwrap();

    let response = client.initiate_multipart_upload(request).await.unwrap();
    assert_eq!(response.bucket, "test-bucket");
    assert_eq!(response.key, "large-file.bin");
    assert_eq!(response.upload_id, "UPLOAD-ID-12345");
}

// ---- UploadPart ----

#[tokio::test]
async fn upload_part_returns_etag() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/large-file.bin"))
        .and(query_param("partNumber", "1"))
        .and(query_param("uploadId", "UPLOAD-ID-12345"))
        .respond_with(ResponseTemplate::new(200).insert_header("etag", "\"part1-etag-abc\""))
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = UploadPartRequestBuilder::new()
        .bucket(BucketName::new("test-bucket").unwrap())
        .key(ObjectKey::new("large-file.bin").unwrap())
        .upload_id("UPLOAD-ID-12345")
        .part_number(1)
        .body(vec![0u8; 1024])
        .build()
        .unwrap();

    let response = client.upload_part(request).await.unwrap();
    assert_eq!(response.etag, "part1-etag-abc");
}

#[tokio::test]
async fn upload_part_second_part() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/large-file.bin"))
        .and(query_param("partNumber", "2"))
        .and(query_param("uploadId", "UPLOAD-ID-12345"))
        .respond_with(ResponseTemplate::new(200).insert_header("etag", "\"part2-etag-def\""))
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = UploadPartRequestBuilder::new()
        .bucket(BucketName::new("test-bucket").unwrap())
        .key(ObjectKey::new("large-file.bin").unwrap())
        .upload_id("UPLOAD-ID-12345")
        .part_number(2)
        .body(vec![1u8; 512])
        .build()
        .unwrap();

    let response = client.upload_part(request).await.unwrap();
    assert_eq!(response.etag, "part2-etag-def");
}

// ---- CompleteMultipartUpload ----

#[tokio::test]
async fn complete_multipart_upload_parses_xml() {
    let server = MockServer::start().await;

    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<CompleteMultipartUploadResult>
    <Location>https://test-bucket.oss-cn-hangzhou.aliyuncs.com/large-file.bin</Location>
    <Bucket>test-bucket</Bucket>
    <Key>large-file.bin</Key>
    <ETag>"final-etag-xyz"</ETag>
</CompleteMultipartUploadResult>"#;

    Mock::given(method("POST"))
        .and(path("/large-file.bin"))
        .and(query_param("uploadId", "UPLOAD-ID-12345"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/xml")
                .set_body_string(xml),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = CompleteMultipartUploadRequestBuilder::new()
        .bucket(BucketName::new("test-bucket").unwrap())
        .key(ObjectKey::new("large-file.bin").unwrap())
        .upload_id("UPLOAD-ID-12345")
        .part(CompletedPart {
            part_number: 1,
            etag: "\"part1-etag\"".to_string(),
        })
        .part(CompletedPart {
            part_number: 2,
            etag: "\"part2-etag\"".to_string(),
        })
        .build()
        .unwrap();

    let response = client.complete_multipart_upload(request).await.unwrap();
    assert_eq!(response.bucket, "test-bucket");
    assert_eq!(response.key, "large-file.bin");
    assert_eq!(response.etag, "\"final-etag-xyz\"");
    assert!(response.location.contains("large-file.bin"));
}

// ---- AbortMultipartUpload ----

#[tokio::test]
async fn abort_multipart_upload_returns_request_id() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/large-file.bin"))
        .and(query_param("uploadId", "UPLOAD-ID-12345"))
        .respond_with(ResponseTemplate::new(204).insert_header("x-oss-request-id", "ABORT-001"))
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = AbortMultipartUploadRequestBuilder::new()
        .bucket(BucketName::new("test-bucket").unwrap())
        .key(ObjectKey::new("large-file.bin").unwrap())
        .upload_id("UPLOAD-ID-12345")
        .build()
        .unwrap();

    let response = client.abort_multipart_upload(request).await.unwrap();
    assert_eq!(response.request_id.as_deref(), Some("ABORT-001"));
}

// ---- ListParts ----

#[tokio::test]
async fn list_parts_parses_xml() {
    let server = MockServer::start().await;

    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListPartsResult>
    <Bucket>test-bucket</Bucket>
    <Key>large-file.bin</Key>
    <UploadId>UPLOAD-ID-12345</UploadId>
    <MaxParts>1000</MaxParts>
    <IsTruncated>false</IsTruncated>
    <Part>
        <PartNumber>1</PartNumber>
        <LastModified>2025-01-15T10:30:00.000Z</LastModified>
        <ETag>"part1-etag"</ETag>
        <Size>5242880</Size>
    </Part>
    <Part>
        <PartNumber>2</PartNumber>
        <LastModified>2025-01-15T10:31:00.000Z</LastModified>
        <ETag>"part2-etag"</ETag>
        <Size>3145728</Size>
    </Part>
</ListPartsResult>"#;

    Mock::given(method("GET"))
        .and(path("/large-file.bin"))
        .and(query_param("uploadId", "UPLOAD-ID-12345"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/xml")
                .set_body_string(xml),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = ListPartsRequestBuilder::new()
        .bucket(BucketName::new("test-bucket").unwrap())
        .key(ObjectKey::new("large-file.bin").unwrap())
        .upload_id("UPLOAD-ID-12345")
        .build()
        .unwrap();

    let response = client.list_parts(request).await.unwrap();
    assert_eq!(response.bucket, "test-bucket");
    assert_eq!(response.key, "large-file.bin");
    assert_eq!(response.upload_id, "UPLOAD-ID-12345");
    assert_eq!(response.max_parts, 1000);
    assert!(!response.is_truncated);
    assert_eq!(response.parts.len(), 2);
    assert_eq!(response.parts[0].part_number, 1);
    assert_eq!(response.parts[0].size, 5242880);
    assert_eq!(response.parts[1].part_number, 2);
    assert_eq!(response.parts[1].size, 3145728);
}

#[tokio::test]
async fn list_parts_with_pagination_params() {
    let server = MockServer::start().await;

    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListPartsResult>
    <Bucket>test-bucket</Bucket>
    <Key>large-file.bin</Key>
    <UploadId>UPLOAD-ID-12345</UploadId>
    <MaxParts>2</MaxParts>
    <IsTruncated>true</IsTruncated>
    <NextPartNumberMarker>4</NextPartNumberMarker>
    <Part>
        <PartNumber>3</PartNumber>
        <LastModified>2025-01-15T10:32:00.000Z</LastModified>
        <ETag>"part3-etag"</ETag>
        <Size>1048576</Size>
    </Part>
    <Part>
        <PartNumber>4</PartNumber>
        <LastModified>2025-01-15T10:33:00.000Z</LastModified>
        <ETag>"part4-etag"</ETag>
        <Size>2097152</Size>
    </Part>
</ListPartsResult>"#;

    Mock::given(method("GET"))
        .and(path("/large-file.bin"))
        .and(query_param("uploadId", "UPLOAD-ID-12345"))
        .and(query_param("max-parts", "2"))
        .and(query_param("part-number-marker", "2"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/xml")
                .set_body_string(xml),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = ListPartsRequestBuilder::new()
        .bucket(BucketName::new("test-bucket").unwrap())
        .key(ObjectKey::new("large-file.bin").unwrap())
        .upload_id("UPLOAD-ID-12345")
        .max_parts(2)
        .part_number_marker(2)
        .build()
        .unwrap();

    let response = client.list_parts(request).await.unwrap();
    assert!(response.is_truncated);
    assert_eq!(response.next_part_number_marker, Some(4));
    assert_eq!(response.parts.len(), 2);
    assert_eq!(response.parts[0].part_number, 3);
    assert_eq!(response.parts[1].part_number, 4);
}

// ---- Error handling ----

#[tokio::test]
async fn initiate_multipart_upload_error_returns_oss_error() {
    let server = MockServer::start().await;

    let error_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Error>
    <Code>InvalidArgument</Code>
    <Message>The specified argument is not valid.</Message>
    <RequestId>ERR-MP-001</RequestId>
    <HostId>bucket.oss-cn-hangzhou.aliyuncs.com</HostId>
</Error>"#;

    Mock::given(method("POST"))
        .and(path("/bad-key"))
        .and(query_param("uploads", ""))
        .respond_with(
            ResponseTemplate::new(400)
                .insert_header("content-type", "application/xml")
                .set_body_string(error_xml),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let request = InitiateMultipartUploadRequestBuilder::new()
        .bucket(BucketName::new("test-bucket").unwrap())
        .key(ObjectKey::new("bad-key").unwrap())
        .build()
        .unwrap();

    let err = client.initiate_multipart_upload(request).await.unwrap_err();
    let err_str = err.to_string();
    assert!(err_str.contains("InvalidArgument"), "error: {err_str}");
}

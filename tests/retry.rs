//! Integration tests for retry behavior using wiremock.

use rs_ali_oss::OssClient;
use rs_ali_oss::config::ClientBuilder;
use rs_ali_oss::error::OssError;
use rs_ali_oss::types::common::{BucketName, ObjectKey};
use rs_ali_oss::types::request::{GetObjectRequestBuilder, PutObjectRequestBuilder};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Respond, ResponseTemplate};

fn mock_client_with_retries(server: &MockServer, max_retries: u32) -> OssClient {
    OssClient::from_builder(
        ClientBuilder::new()
            .access_key_id("test-key-id")
            .access_key_secret("test-key-secret")
            .region("cn-hangzhou")
            .endpoint(server.uri())
            .allow_insecure(true)
            .max_retries(max_retries)
            .base_retry_delay(std::time::Duration::from_millis(1))
            .max_retry_delay(std::time::Duration::from_millis(10)),
    )
    .unwrap()
}

struct SequentialResponder {
    responses: std::sync::Mutex<Vec<ResponseTemplate>>,
}

impl SequentialResponder {
    fn new(responses: Vec<ResponseTemplate>) -> Self {
        let mut reversed = responses;
        reversed.reverse();
        Self {
            responses: std::sync::Mutex::new(reversed),
        }
    }
}

impl Respond for SequentialResponder {
    fn respond(&self, _request: &wiremock::Request) -> ResponseTemplate {
        let mut responses = self.responses.lock().unwrap();
        if let Some(resp) = responses.pop() {
            resp
        } else {
            ResponseTemplate::new(500).set_body_string("no more responses configured")
        }
    }
}

#[tokio::test]
async fn retry_500_500_then_200_succeeds() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/retry-test.txt"))
        .respond_with(SequentialResponder::new(vec![
            ResponseTemplate::new(500).set_body_string(
                r#"<Error><Code>InternalError</Code><Message>Internal</Message>
                <RequestId>R1</RequestId><HostId>H1</HostId></Error>"#,
            ),
            ResponseTemplate::new(500).set_body_string(
                r#"<Error><Code>InternalError</Code><Message>Internal</Message>
                <RequestId>R2</RequestId><HostId>H2</HostId></Error>"#,
            ),
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/plain")
                .set_body_bytes(b"success"),
        ]))
        .expect(3)
        .mount(&server)
        .await;

    let client = mock_client_with_retries(&server, 2);
    let request = GetObjectRequestBuilder::new()
        .bucket(BucketName::new("test-bucket").unwrap())
        .key(ObjectKey::new("retry-test.txt").unwrap())
        .build()
        .unwrap();

    let response = client.get_object(request).await.unwrap();
    let body = response.body.bytes().await.unwrap();
    assert_eq!(&body[..], b"success");
}

#[tokio::test]
async fn retry_exhausted_returns_retry_exhausted_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/always-fail.txt"))
        .respond_with(SequentialResponder::new(vec![
            ResponseTemplate::new(500).set_body_string(
                r#"<Error><Code>InternalError</Code><Message>fail1</Message>
                <RequestId>R1</RequestId><HostId>H1</HostId></Error>"#,
            ),
            ResponseTemplate::new(500).set_body_string(
                r#"<Error><Code>InternalError</Code><Message>fail2</Message>
                <RequestId>R2</RequestId><HostId>H2</HostId></Error>"#,
            ),
            ResponseTemplate::new(500).set_body_string(
                r#"<Error><Code>InternalError</Code><Message>fail3</Message>
                <RequestId>R3</RequestId><HostId>H3</HostId></Error>"#,
            ),
        ]))
        .expect(3)
        .mount(&server)
        .await;

    let client = mock_client_with_retries(&server, 2);
    let request = GetObjectRequestBuilder::new()
        .bucket(BucketName::new("test-bucket").unwrap())
        .key(ObjectKey::new("always-fail.txt").unwrap())
        .build()
        .unwrap();

    let err = client.get_object(request).await.unwrap_err();
    match err {
        OssError::ServerError { status, code, .. } => {
            assert_eq!(status, 500);
            assert_eq!(code, "InternalError");
        }
        other => panic!("expected ServerError on final attempt, got: {other:?}"),
    }
}

#[tokio::test]
async fn no_retry_on_4xx_errors() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/forbidden.txt"))
        .respond_with(ResponseTemplate::new(403).set_body_string(
            r#"<Error><Code>AccessDenied</Code><Message>Denied</Message>
            <RequestId>R1</RequestId><HostId>H1</HostId></Error>"#,
        ))
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client_with_retries(&server, 3);
    let request = GetObjectRequestBuilder::new()
        .bucket(BucketName::new("test-bucket").unwrap())
        .key(ObjectKey::new("forbidden.txt").unwrap())
        .build()
        .unwrap();

    let err = client.get_object(request).await.unwrap_err();
    match err {
        OssError::ServerError { status, code, .. } => {
            assert_eq!(status, 403);
            assert_eq!(code, "AccessDenied");
        }
        other => panic!("expected ServerError, got: {other:?}"),
    }
}

#[tokio::test]
async fn retry_with_buffered_body_succeeds() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/upload-retry.txt"))
        .respond_with(SequentialResponder::new(vec![
            ResponseTemplate::new(500).set_body_string(
                r#"<Error><Code>InternalError</Code><Message>Oops</Message>
                <RequestId>R1</RequestId><HostId>H1</HostId></Error>"#,
            ),
            ResponseTemplate::new(200)
                .insert_header("etag", "\"retry-etag\"")
                .insert_header("x-oss-request-id", "RETRY-OK"),
        ]))
        .expect(2)
        .mount(&server)
        .await;

    let client = mock_client_with_retries(&server, 1);
    let request = PutObjectRequestBuilder::new()
        .bucket(BucketName::new("test-bucket").unwrap())
        .key(ObjectKey::new("upload-retry.txt").unwrap())
        .body(b"hello retry".to_vec())
        .build()
        .unwrap();

    let response = client.put_object(request).await.unwrap();
    assert_eq!(response.etag, "retry-etag");
}

#[tokio::test]
async fn no_retry_when_max_retries_is_zero() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/no-retry.txt"))
        .respond_with(ResponseTemplate::new(500).set_body_string(
            r#"<Error><Code>InternalError</Code><Message>fail</Message>
            <RequestId>R1</RequestId><HostId>H1</HostId></Error>"#,
        ))
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client_with_retries(&server, 0);
    let request = GetObjectRequestBuilder::new()
        .bucket(BucketName::new("test-bucket").unwrap())
        .key(ObjectKey::new("no-retry.txt").unwrap())
        .build()
        .unwrap();

    let err = client.get_object(request).await.unwrap_err();
    assert!(matches!(err, OssError::ServerError { .. }));
}

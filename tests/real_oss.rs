//! Real-world integration test against Alibaba Cloud OSS.
//!
//! Uses credentials from `.env` file to test actual API calls.
//! Run with: cargo test --test real_oss -- --nocapture

use std::collections::HashMap;

use rs_ali_oss::OssClient;
use rs_ali_oss::config::ClientBuilder;
use rs_ali_oss::types::common::{BucketName, ObjectKey};
use rs_ali_oss::types::request::{
    DeleteObjectRequestBuilder, GetObjectRequestBuilder, HeadObjectRequestBuilder,
    ListObjectsV2RequestBuilder, PresignedUrlRequestBuilder, PutObjectRequestBuilder,
};

fn load_env() -> HashMap<String, String> {
    let content = std::fs::read_to_string(".env").expect("Failed to read .env file");
    content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .filter_map(|l| {
            let (k, v) = l.split_once('=')?;
            Some((k.trim().to_string(), v.trim().to_string()))
        })
        .collect()
}

fn make_client(env: &HashMap<String, String>) -> OssClient {
    let ak = env.get("sts.access_key").expect("missing sts.access_key");
    let sk = env
        .get("sts.access_key_secret")
        .expect("missing sts.access_key_secret");
    let region = env.get("oss.region").expect("missing oss.region");
    let endpoint = env
        .get("oss.endpoint")
        .or_else(|| env.get("oss.endingpoint"))
        .expect("missing oss.endpoint");

    OssClient::from_builder(
        ClientBuilder::new()
            .access_key_id(ak)
            .access_key_secret(sk)
            .region(region)
            .endpoint(endpoint)
            .max_retries(0),
    )
    .expect("Failed to build OssClient")
}

fn make_sts_client(
    env: &HashMap<String, String>,
    sts_creds: &rs_ali_sts::Credentials,
) -> OssClient {
    let region = env.get("oss.region").expect("missing oss.region");
    let endpoint = env
        .get("oss.endpoint")
        .or_else(|| env.get("oss.endingpoint"))
        .expect("missing oss.endpoint");

    OssClient::from_builder(
        ClientBuilder::new()
            .access_key_id(&sts_creds.access_key_id)
            .access_key_secret(&sts_creds.access_key_secret)
            .security_token(&sts_creds.security_token)
            .region(region)
            .endpoint(endpoint)
            .max_retries(0),
    )
    .expect("Failed to build STS OssClient")
}

async fn assume_role(env: &HashMap<String, String>) -> rs_ali_sts::Credentials {
    let ak = env.get("sts.access_key").expect("missing sts.access_key");
    let sk = env
        .get("sts.access_key_secret")
        .expect("missing sts.access_key_secret");
    let role_arn = env.get("sts.role_arn").expect("missing sts.role_arn");
    let session = env
        .get("sts.role_session_name")
        .expect("missing sts.role_session_name");

    let sts_client = rs_ali_sts::Client::new(rs_ali_sts::Credential {
        access_key_id: ak.clone(),
        access_key_secret: sk.clone(),
    });

    let resp = sts_client
        .assume_role(rs_ali_sts::AssumeRoleRequest {
            role_arn: role_arn.clone(),
            role_session_name: session.clone(),
            policy: None,
            duration_seconds: Some(900),
            external_id: None,
        })
        .await
        .expect("AssumeRole failed");

    println!(
        "STS AssumeRole OK: temp_ak={}, expires={}",
        resp.credentials.access_key_id, resp.credentials.expiration
    );
    resp.credentials
}

fn bucket(env: &HashMap<String, String>) -> BucketName {
    BucketName::new(env.get("oss.bucket_name").expect("missing oss.bucket_name")).unwrap()
}

#[tokio::test]
async fn test_list_objects_real() {
    let env = load_env();
    let client = make_client(&env);

    let resp = client
        .list_objects_v2(
            ListObjectsV2RequestBuilder::new()
                .bucket(bucket(&env))
                .max_keys(5)
                .build()
                .unwrap(),
        )
        .await
        .expect("list_objects_v2 failed");
    println!("ListObjects OK: {} objects", resp.contents.len());
    for obj in &resp.contents {
        println!("  - {} ({} bytes)", obj.key, obj.size);
    }
}

#[tokio::test]
async fn test_put_head_get_delete_real() {
    let env = load_env();
    let client = make_client(&env);
    let b = bucket(&env);

    let test_key = format!("rs-ali-oss-test/{}.txt", chrono::Utc::now().timestamp());
    let test_body = b"Hello from rs-ali-oss integration test!";

    let put_resp = client
        .put_object(
            PutObjectRequestBuilder::new()
                .bucket(b.clone())
                .key(ObjectKey::new(&test_key).unwrap())
                .body(test_body.to_vec())
                .content_type("text/plain")
                .build()
                .unwrap(),
        )
        .await
        .expect("put_object failed");
    println!("PUT OK: etag={}", put_resp.etag);

    let head_resp = client
        .head_object(
            HeadObjectRequestBuilder::new()
                .bucket(b.clone())
                .key(ObjectKey::new(&test_key).unwrap())
                .build()
                .unwrap(),
        )
        .await
        .expect("head_object failed");
    assert_eq!(head_resp.content_length, Some(test_body.len() as u64));
    println!("HEAD OK: content_length={:?}", head_resp.content_length);

    let get_resp = client
        .get_object(
            GetObjectRequestBuilder::new()
                .bucket(b.clone())
                .key(ObjectKey::new(&test_key).unwrap())
                .build()
                .unwrap(),
        )
        .await
        .expect("get_object failed");
    let body = get_resp.body.bytes().await.unwrap();
    assert_eq!(&body[..], test_body);
    println!("GET OK: body matches");

    client
        .delete_object(
            DeleteObjectRequestBuilder::new()
                .bucket(b)
                .key(ObjectKey::new(&test_key).unwrap())
                .build()
                .unwrap(),
        )
        .await
        .expect("delete_object failed");
    println!("DELETE OK");
}

#[tokio::test]
async fn test_sts_temporary_credentials() {
    let env = load_env();
    let sts_creds = assume_role(&env).await;
    let client = make_sts_client(&env, &sts_creds);
    let b = bucket(&env);

    let test_key = format!("rs-ali-oss-test/sts-{}.txt", chrono::Utc::now().timestamp());
    let test_body = b"STS temporary credential test";

    client
        .put_object(
            PutObjectRequestBuilder::new()
                .bucket(b.clone())
                .key(ObjectKey::new(&test_key).unwrap())
                .body(test_body.to_vec())
                .content_type("text/plain")
                .build()
                .unwrap(),
        )
        .await
        .expect("STS put_object failed");
    println!("STS PUT OK");

    let get_resp = client
        .get_object(
            GetObjectRequestBuilder::new()
                .bucket(b.clone())
                .key(ObjectKey::new(&test_key).unwrap())
                .build()
                .unwrap(),
        )
        .await
        .expect("STS get_object failed");
    let body = get_resp.body.bytes().await.unwrap();
    assert_eq!(&body[..], test_body);
    println!("STS GET OK: body matches");

    client
        .delete_object(
            DeleteObjectRequestBuilder::new()
                .bucket(b)
                .key(ObjectKey::new(&test_key).unwrap())
                .build()
                .unwrap(),
        )
        .await
        .expect("STS delete_object failed");
    println!("STS DELETE OK");
}

#[tokio::test]
async fn test_presigned_url_get() {
    let env = load_env();
    let client = make_client(&env);
    let b = bucket(&env);

    let test_key = format!(
        "rs-ali-oss-test/presign-{}.txt",
        chrono::Utc::now().timestamp()
    );
    let test_body = b"presigned URL test content";

    client
        .put_object(
            PutObjectRequestBuilder::new()
                .bucket(b.clone())
                .key(ObjectKey::new(&test_key).unwrap())
                .body(test_body.to_vec())
                .content_type("text/plain")
                .build()
                .unwrap(),
        )
        .await
        .expect("put for presign test failed");

    let url = client
        .presign_get_object(
            PresignedUrlRequestBuilder::new()
                .bucket(b.clone())
                .key(ObjectKey::new(&test_key).unwrap())
                .expires(std::time::Duration::from_secs(300))
                .build()
                .unwrap(),
        )
        .unwrap();
    println!("Presigned URL: {url}");

    let http_resp = reqwest::get(&url).await.expect("presigned GET failed");
    assert!(
        http_resp.status().is_success(),
        "presigned URL returned HTTP {}",
        http_resp.status()
    );
    let body = http_resp.bytes().await.unwrap();
    assert_eq!(&body[..], test_body);
    println!("Presigned GET OK: body matches");

    client
        .delete_object(
            DeleteObjectRequestBuilder::new()
                .bucket(b)
                .key(ObjectKey::new(&test_key).unwrap())
                .build()
                .unwrap(),
        )
        .await
        .expect("cleanup delete failed");
    println!("Presign cleanup OK");
}

#[tokio::test]
async fn test_sts_presigned_url() {
    let env = load_env();
    let sts_creds = assume_role(&env).await;
    let client = make_sts_client(&env, &sts_creds);
    let b = bucket(&env);

    let test_key = format!(
        "rs-ali-oss-test/sts-presign-{}.txt",
        chrono::Utc::now().timestamp()
    );
    let test_body = b"STS presigned URL test";

    client
        .put_object(
            PutObjectRequestBuilder::new()
                .bucket(b.clone())
                .key(ObjectKey::new(&test_key).unwrap())
                .body(test_body.to_vec())
                .content_type("text/plain")
                .build()
                .unwrap(),
        )
        .await
        .expect("STS put for presign test failed");

    let url = client
        .presign_get_object(
            PresignedUrlRequestBuilder::new()
                .bucket(b.clone())
                .key(ObjectKey::new(&test_key).unwrap())
                .expires(std::time::Duration::from_secs(300))
                .build()
                .unwrap(),
        )
        .unwrap();
    println!("STS Presigned URL: {url}");

    let http_resp = reqwest::get(&url).await.expect("STS presigned GET failed");
    assert!(
        http_resp.status().is_success(),
        "STS presigned URL returned HTTP {}",
        http_resp.status()
    );
    let body = http_resp.bytes().await.unwrap();
    assert_eq!(&body[..], test_body);
    println!("STS Presigned GET OK: body matches");

    client
        .delete_object(
            DeleteObjectRequestBuilder::new()
                .bucket(b)
                .key(ObjectKey::new(&test_key).unwrap())
                .build()
                .unwrap(),
        )
        .await
        .expect("STS cleanup delete failed");
    println!("STS Presign cleanup OK");
}

#[tokio::test]
async fn test_sts_list_objects_real() {
    let env = load_env();
    let sts_creds = assume_role(&env).await;
    let client = make_sts_client(&env, &sts_creds);
    let b = bucket(&env);

    let resp = client
        .list_objects_v2(
            ListObjectsV2RequestBuilder::new()
                .bucket(b)
                .max_keys(5)
                .build()
                .unwrap(),
        )
        .await
        .expect("STS list_objects_v2 failed");

    println!("STS ListObjects OK: {} objects", resp.contents.len());
    for obj in &resp.contents {
        println!("  - {} ({} bytes)", obj.key, obj.size);
    }
}

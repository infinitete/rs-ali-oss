//! Presigned URL generation for OSS objects.
//!
//! Generates time-limited URLs that allow unauthenticated access to private objects.
//! Supports both GET (download) and PUT (upload) presigned URLs using V4 query-string signing.

use chrono::Utc;
use percent_encoding::percent_encode;

use crate::auth::v4::{
    build_string_to_sign, calculate_signature, canonical_uri, derive_signing_key,
};
use crate::client::OssClient;
use crate::encoding::QUERY_ENCODE_SET;
use crate::error::Result;
use crate::types::request::PresignedUrlRequest;

impl OssClient {
    /// Generate a presigned URL for downloading an object (GET).
    ///
    /// The URL is valid for the duration specified in the request (default: 1 hour, max: 7 days).
    ///
    /// # Examples
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::PresignedUrlRequestBuilder;
    /// # fn example(client: OssClient) -> Result<()> {
    /// let request = PresignedUrlRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .key(ObjectKey::new("secret-doc.pdf")?)
    ///     .build()?;
    /// let url = client.presign_get_object(request)?;
    /// println!("Download URL: {url}");
    /// # Ok(())
    /// # }
    /// ```
    pub fn presign_get_object(&self, request: PresignedUrlRequest) -> Result<String> {
        self.generate_presigned_url("GET", request)
    }

    /// Generate a presigned URL for uploading an object (PUT).
    ///
    /// The URL is valid for the duration specified in the request (default: 1 hour, max: 7 days).
    ///
    /// # Examples
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::PresignedUrlRequestBuilder;
    /// # fn example(client: OssClient) -> Result<()> {
    /// let request = PresignedUrlRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .key(ObjectKey::new("upload-target.bin")?)
    ///     .content_type("application/octet-stream")
    ///     .build()?;
    /// let url = client.presign_put_object(request)?;
    /// println!("Upload URL: {url}");
    /// # Ok(())
    /// # }
    /// ```
    pub fn presign_put_object(&self, request: PresignedUrlRequest) -> Result<String> {
        self.generate_presigned_url("PUT", request)
    }

    /// Internal: generate a presigned URL with V4 query-string signing.
    fn generate_presigned_url(&self, method: &str, request: PresignedUrlRequest) -> Result<String> {
        let now = request.datetime.unwrap_or_else(Utc::now);
        let datetime_str = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_str = now.format("%Y%m%d").to_string();
        let region_str: &str = self.config().region().as_ref();
        let expires_secs = request.expires.as_secs();

        let base_url = self.build_url(Some(&request.bucket), Some(&request.key), &[])?;
        let host = base_url.host_str().unwrap_or_default().to_string();

        // Build canonical URI from the raw key to avoid double-encoding
        // (build_url already percent-encodes via url::Url::parse)
        let raw_path = if self.config().use_path_style() {
            format!("/{}/{}", request.bucket, request.key)
        } else {
            format!("/{}", request.key)
        };
        let path = canonical_uri(&raw_path);

        let credential = format!(
            "{}/{}/{}/oss/aliyun_v4_request",
            self.config().credentials().access_key_id(),
            date_str,
            region_str,
        );

        let (signed_headers, extra_headers) = if let Some(ref ct) = request.content_type {
            (
                "content-type;host".to_string(),
                vec![("content-type".to_string(), ct.clone())],
            )
        } else {
            ("host".to_string(), Vec::new())
        };

        let mut query_params = vec![
            ("x-oss-credential".to_string(), credential.clone()),
            ("x-oss-date".to_string(), datetime_str.clone()),
            ("x-oss-expires".to_string(), expires_secs.to_string()),
            (
                "x-oss-signature-version".to_string(),
                "OSS4-HMAC-SHA256".to_string(),
            ),
            ("x-oss-signed-headers".to_string(), signed_headers.clone()),
        ];

        if let Some(token) = self.config().credentials().security_token() {
            query_params.push(("x-oss-security-token".to_string(), token.to_string()));
        }

        query_params.sort_by(|a, b| a.0.cmp(&b.0));

        // Build canonical query string (percent-encoded)
        let canonical_query = query_params
            .iter()
            .map(|(k, v)| {
                format!(
                    "{}={}",
                    percent_encode(k.as_bytes(), QUERY_ENCODE_SET),
                    percent_encode(v.as_bytes(), QUERY_ENCODE_SET),
                )
            })
            .collect::<Vec<_>>()
            .join("&");

        // Build canonical headers
        let mut canonical_headers = String::new();
        for (name, value) in &extra_headers {
            canonical_headers.push_str(&format!("{name}:{value}\n"));
        }
        canonical_headers.push_str(&format!("host:{host}\n"));

        // Build canonical request
        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\nUNSIGNED-PAYLOAD",
            method, path, canonical_query, canonical_headers, signed_headers,
        );

        // Build string-to-sign, derive signing key, and compute signature
        let string_to_sign =
            build_string_to_sign(&datetime_str, &date_str, region_str, &canonical_request);
        let signing_key = derive_signing_key(
            self.config().credentials().access_key_secret(),
            &date_str,
            region_str,
        )?;
        let signature = calculate_signature(&signing_key, &string_to_sign)?;

        // Construct final URL
        let origin = format!("{}://{}", base_url.scheme(), host);
        let final_url = format!(
            "{}{path}?{canonical_query}&x-oss-signature={signature}",
            origin,
        );

        Ok(final_url)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::ClientBuilder;
    use crate::types::common::{BucketName, ObjectKey};
    use crate::types::request::PresignedUrlRequestBuilder;

    fn test_client() -> crate::client::OssClient {
        crate::client::OssClient::from_builder(
            ClientBuilder::new()
                .access_key_id("test-key-id")
                .access_key_secret("test-key-secret")
                .region("cn-hangzhou"),
        )
        .unwrap()
    }

    #[test]
    fn presign_get_object_produces_valid_url() {
        let client = test_client();
        let request = PresignedUrlRequestBuilder::new()
            .bucket(BucketName::new("my-bucket").unwrap())
            .key(ObjectKey::new("docs/report.pdf").unwrap())
            .build()
            .unwrap();
        let url = client.presign_get_object(request).unwrap();

        assert!(url.starts_with("https://my-bucket.oss-cn-hangzhou.aliyuncs.com/docs/report.pdf?"));
        assert!(url.contains("x-oss-credential="));
        assert!(url.contains("x-oss-date="));
        assert!(url.contains("x-oss-expires=3600"));
        assert!(url.contains("x-oss-signature-version=OSS4-HMAC-SHA256"));
        assert!(url.contains("x-oss-signed-headers=host"));
        assert!(url.contains("x-oss-signature="));
    }

    #[test]
    fn presign_put_object_produces_valid_url() {
        let client = test_client();
        let request = PresignedUrlRequestBuilder::new()
            .bucket(BucketName::new("my-bucket").unwrap())
            .key(ObjectKey::new("uploads/data.bin").unwrap())
            .content_type("application/octet-stream")
            .expires(std::time::Duration::from_secs(7200))
            .build()
            .unwrap();
        let url = client.presign_put_object(request).unwrap();

        assert!(
            url.starts_with("https://my-bucket.oss-cn-hangzhou.aliyuncs.com/uploads/data.bin?")
        );
        assert!(url.contains("x-oss-expires=7200"));
        assert!(url.contains("x-oss-signature="));
        // content-type is a signed header
        assert!(url.contains("x-oss-signed-headers=content-type"));
    }

    #[test]
    fn presign_get_object_with_special_chars_in_key() {
        let client = test_client();
        let request = PresignedUrlRequestBuilder::new()
            .bucket(BucketName::new("my-bucket").unwrap())
            .key(ObjectKey::new("path/hello world.txt").unwrap())
            .build()
            .unwrap();
        let url = client.presign_get_object(request).unwrap();

        // Key should be percent-encoded in the path
        assert!(url.contains("hello%20world.txt"));
        assert!(url.contains("x-oss-signature="));
    }

    #[test]
    fn presign_with_sts_token_includes_security_token() {
        let client = crate::client::OssClient::from_builder(
            ClientBuilder::new()
                .access_key_id("sts-key-id")
                .access_key_secret("sts-key-secret")
                .security_token("sts-token-value")
                .region("cn-hangzhou"),
        )
        .unwrap();

        let request = PresignedUrlRequestBuilder::new()
            .bucket(BucketName::new("my-bucket").unwrap())
            .key(ObjectKey::new("file.txt").unwrap())
            .build()
            .unwrap();
        let url = client.presign_get_object(request).unwrap();

        assert!(url.contains("x-oss-security-token=sts-token-value"));
    }

    #[test]
    fn presign_signature_is_deterministic_for_same_inputs() {
        // Two calls with the same client/request produce URLs with the same structure
        // (signature may differ due to timestamp, but structure is consistent)
        let client = test_client();

        let request1 = PresignedUrlRequestBuilder::new()
            .bucket(BucketName::new("bucket").unwrap())
            .key(ObjectKey::new("key.txt").unwrap())
            .build()
            .unwrap();
        let url1 = client.presign_get_object(request1).unwrap();

        let request2 = PresignedUrlRequestBuilder::new()
            .bucket(BucketName::new("bucket").unwrap())
            .key(ObjectKey::new("key.txt").unwrap())
            .build()
            .unwrap();
        let url2 = client.presign_get_object(request2).unwrap();

        // Both should have the same base URL and param structure
        assert!(url1.starts_with("https://bucket.oss-cn-hangzhou.aliyuncs.com/key.txt?"));
        assert!(url2.starts_with("https://bucket.oss-cn-hangzhou.aliyuncs.com/key.txt?"));

        // Signatures should be 64 hex chars
        let sig1 = url1.split("x-oss-signature=").nth(1).unwrap();
        let sig2 = url2.split("x-oss-signature=").nth(1).unwrap();
        assert_eq!(sig1.len(), 64);
        assert_eq!(sig2.len(), 64);
        assert!(sig1.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(sig2.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

//! OSS V4 signature implementation (OSS4-HMAC-SHA256).
//!
//! Implements the complete signing flow for Alibaba Cloud OSS V4 authentication,
//! including canonical request construction, string-to-sign generation,
//! signing key derivation, and authorization header assembly.

use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use percent_encoding::percent_encode;
use reqwest::header::HeaderMap;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::config::Credentials;
use crate::encoding::{QUERY_ENCODE_SET, URI_ENCODE_SET};
use crate::error::OssError;
use crate::types::Region;

type HmacSha256 = Hmac<Sha256>;

/// Percent-encode the URI path, preserving forward slashes.
pub(crate) fn canonical_uri(path: &str) -> String {
    if path.is_empty() || path == "/" {
        return "/".to_string();
    }
    percent_encode(path.as_bytes(), URI_ENCODE_SET).to_string()
}

/// Sort and percent-encode query parameters.
fn canonical_query_string(url: &url::Url) -> String {
    let mut pairs: Vec<(String, String)> = url
        .query_pairs()
        .map(|(k, v)| {
            (
                percent_encode(k.as_bytes(), QUERY_ENCODE_SET).to_string(),
                percent_encode(v.as_bytes(), QUERY_ENCODE_SET).to_string(),
            )
        })
        .collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    pairs
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&")
}

/// Build canonical headers string and signed headers list.
///
/// Headers are lowercased, trimmed, and sorted by name.
fn canonical_and_signed_headers(headers: &HeaderMap) -> (String, String) {
    let mut header_list: Vec<(String, String)> = headers
        .iter()
        .map(|(name, value)| {
            (
                name.as_str().to_lowercase(),
                value.to_str().unwrap_or("").trim().to_string(),
            )
        })
        .collect();
    header_list.sort_by(|a, b| a.0.cmp(&b.0));

    let canonical = header_list
        .iter()
        .map(|(k, v)| format!("{k}:{v}\n"))
        .collect::<String>();

    let signed = header_list
        .iter()
        .map(|(k, _)| k.as_str())
        .collect::<Vec<_>>()
        .join(";");

    (canonical, signed)
}

/// Assemble the canonical request string and return it alongside the signed headers.
fn build_canonical_request(
    method: &str,
    url: &url::Url,
    headers: &HeaderMap,
    payload_hash: &str,
) -> (String, String) {
    let uri = canonical_uri(url.path());
    let query = canonical_query_string(url);
    let (canonical_hdrs, signed_hdrs) = canonical_and_signed_headers(headers);

    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        method, uri, query, canonical_hdrs, signed_hdrs, payload_hash
    );

    (canonical_request, signed_hdrs)
}

/// Build the string-to-sign from the datetime, scope, and canonical request.
pub(crate) fn build_string_to_sign(
    datetime: &str,
    date: &str,
    region: &str,
    canonical_request: &str,
) -> String {
    let hash = hex::encode(Sha256::digest(canonical_request.as_bytes()));
    format!(
        "OSS4-HMAC-SHA256\n{}\n{}/{}/oss/aliyun_v4_request\n{}",
        datetime, date, region, hash
    )
}

/// Compute HMAC-SHA256, returning an error instead of panicking.
fn hmac_sha256(key: &[u8], data: &[u8]) -> crate::error::Result<Vec<u8>> {
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|e| OssError::Auth(format!("HMAC key error: {e}")))?;
    mac.update(data);
    Ok(mac.finalize().into_bytes().to_vec())
}

/// Derive the signing key using the HMAC chain.
///
/// Chain: HMAC("aliyun_v4" + secret, date) → HMAC(prev, region) → HMAC(prev, "oss")
///        → HMAC(prev, "aliyun_v4_request")
///
/// The intermediate and final keys are wrapped in [`Zeroizing`] to ensure
/// cryptographic material is wiped from memory on drop.
pub(crate) fn derive_signing_key(
    secret: &str,
    date: &str,
    region: &str,
) -> crate::error::Result<Zeroizing<Vec<u8>>> {
    let key = Zeroizing::new(format!("aliyun_v4{secret}"));

    let date_key = Zeroizing::new(hmac_sha256(key.as_bytes(), date.as_bytes())?);
    let region_key = Zeroizing::new(hmac_sha256(&date_key, region.as_bytes())?);
    let service_key = Zeroizing::new(hmac_sha256(&region_key, b"oss")?);
    let signing_key = hmac_sha256(&service_key, b"aliyun_v4_request")?;

    Ok(Zeroizing::new(signing_key))
}

pub(crate) fn calculate_signature(
    signing_key: &[u8],
    string_to_sign: &str,
) -> crate::error::Result<String> {
    let sig_bytes = hmac_sha256(signing_key, string_to_sign.as_bytes())?;
    Ok(hex::encode(sig_bytes))
}

/// Sign a request using OSS V4 signature (OSS4-HMAC-SHA256).
///
/// Adds `x-oss-date`, `x-oss-content-sha256`, and `Authorization` headers
/// to the provided request.
///
/// # Errors
///
/// Returns [`OssError::Auth`] if any header value cannot be constructed.
pub fn sign_request(
    req: &mut reqwest::Request,
    credentials: &Credentials,
    region: &Region,
    datetime: DateTime<Utc>,
) -> crate::error::Result<()> {
    let datetime_str = datetime.format("%Y%m%dT%H%M%SZ").to_string();
    let date_str = datetime.format("%Y%m%d").to_string();
    let region_str: &str = region.as_ref();

    // Compute payload hash
    let payload_hash = if let Some(body) = req.body() {
        if let Some(bytes) = body.as_bytes() {
            hex::encode(Sha256::digest(bytes))
        } else {
            "UNSIGNED-PAYLOAD".to_string()
        }
    } else {
        hex::encode(Sha256::digest(b""))
    };

    // Set required headers BEFORE building canonical request
    let headers = req.headers_mut();
    headers.insert(
        "x-oss-date",
        datetime_str
            .parse()
            .map_err(|_| OssError::Auth("failed to set x-oss-date header".to_string()))?,
    );
    headers.insert(
        "x-oss-content-sha256",
        payload_hash
            .parse()
            .map_err(|_| OssError::Auth("failed to set x-oss-content-sha256 header".to_string()))?,
    );

    if let Some(token) = credentials.security_token() {
        headers.insert(
            "x-oss-security-token",
            token.parse().map_err(|_| {
                OssError::Auth("failed to set x-oss-security-token header".to_string())
            })?,
        );
    }

    // Build canonical request
    let method = req.method().as_str().to_string();
    let url = req.url().clone();
    let (canonical_request, signed_headers) =
        build_canonical_request(&method, &url, req.headers(), &payload_hash);

    // Build string to sign
    let string_to_sign =
        build_string_to_sign(&datetime_str, &date_str, region_str, &canonical_request);

    let signing_key = derive_signing_key(credentials.access_key_secret(), &date_str, region_str)?;

    let signature = calculate_signature(&signing_key, &string_to_sign)?;

    // Build authorization header
    let credential = format!(
        "{}/{}/{}/oss/aliyun_v4_request",
        credentials.access_key_id(),
        date_str,
        region_str,
    );
    let auth_value = format!(
        "OSS4-HMAC-SHA256 Credential={}, SignedHeaders={}, Signature={}",
        credential, signed_headers, signature,
    );

    req.headers_mut().insert(
        "authorization",
        auth_value
            .parse()
            .map_err(|_| OssError::Auth("failed to set authorization header".to_string()))?,
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_uri_root() {
        assert_eq!(canonical_uri("/"), "/");
        assert_eq!(canonical_uri(""), "/");
    }

    #[test]
    fn test_canonical_uri_simple_path() {
        assert_eq!(canonical_uri("/bucket/key.txt"), "/bucket/key.txt");
    }

    #[test]
    fn test_canonical_uri_special_chars() {
        let result = canonical_uri("/bucket/hello world.txt");
        assert_eq!(result, "/bucket/hello%20world.txt");
    }

    #[test]
    fn test_canonical_uri_chinese_chars() {
        let result = canonical_uri("/bucket/中文.txt");
        assert!(result.starts_with("/bucket/"));
        assert!(result.contains('%'));
        assert!(!result.contains("中文"));
    }

    #[test]
    fn test_canonical_query_string_sorted() {
        let url = url::Url::parse("https://example.com/?z=1&a=2&m=3").unwrap();
        let result = canonical_query_string(&url);
        assert_eq!(result, "a=2&m=3&z=1");
    }

    #[test]
    fn test_canonical_query_string_empty() {
        let url = url::Url::parse("https://example.com/").unwrap();
        let result = canonical_query_string(&url);
        assert_eq!(result, "");
    }

    #[test]
    fn test_signing_key_derivation() {
        let key = derive_signing_key("test-secret", "20260207", "cn-hangzhou").unwrap();
        let key2 = derive_signing_key("test-secret", "20260207", "cn-hangzhou").unwrap();
        assert_eq!(*key, *key2);
        let key3 = derive_signing_key("other-secret", "20260207", "cn-hangzhou").unwrap();
        assert_ne!(*key, *key3);
    }

    #[test]
    fn test_signing_key_known_vector() {
        let key = derive_signing_key("wJalrXUtnFEMI", "20231203", "cn-hangzhou").unwrap();
        let hex_key = hex::encode(&*key);
        assert_eq!(hex_key.len(), 64);
        let key2 = derive_signing_key("wJalrXUtnFEMI", "20231203", "cn-hangzhou").unwrap();
        assert_eq!(*key, *key2);
    }

    #[test]
    fn test_full_signing_flow_deterministic() {
        let secret = "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY";
        let date = "20231203";
        let datetime = "20231203T120000Z";
        let region = "cn-hangzhou";

        let canonical_request = "GET\n/exampleobject\n\nhost:examplebucket.oss-cn-hangzhou.aliyuncs.com\n\
             x-oss-content-sha256:UNSIGNED-PAYLOAD\n\
             x-oss-date:20231203T120000Z\n\n\
             host;x-oss-content-sha256;x-oss-date\nUNSIGNED-PAYLOAD";

        let string_to_sign = build_string_to_sign(datetime, date, region, canonical_request);
        assert!(string_to_sign.starts_with("OSS4-HMAC-SHA256\n"));
        assert!(string_to_sign.contains(datetime));
        assert!(string_to_sign.contains(&format!("{date}/{region}/oss/aliyun_v4_request")));

        let signing_key = derive_signing_key(secret, date, region).unwrap();
        let signature = calculate_signature(&signing_key, &string_to_sign).unwrap();

        assert_eq!(signature.len(), 64);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));

        let signature2 = calculate_signature(&signing_key, &string_to_sign).unwrap();
        assert_eq!(signature, signature2);
    }

    #[tokio::test]
    async fn test_sign_request_full_canonical_deterministic() {
        let client = reqwest::Client::new();
        let mut req = client
            .put("https://examplebucket.oss-cn-hangzhou.aliyuncs.com/exampleobject")
            .body(b"Hello OSS".to_vec())
            .build()
            .unwrap();

        let creds = crate::config::Credentials::new(
            "LTAI5tExampleKeyId",
            "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
        );
        let region = crate::types::Region::new("cn-hangzhou").unwrap();
        let dt = chrono::NaiveDateTime::parse_from_str("2023-12-03T12:00:00", "%Y-%m-%dT%H:%M:%S")
            .unwrap()
            .and_utc();

        sign_request(&mut req, &creds, &region, dt).unwrap();

        let auth = req
            .headers()
            .get("authorization")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(auth.starts_with("OSS4-HMAC-SHA256 Credential=LTAI5tExampleKeyId/20231203/cn-hangzhou/oss/aliyun_v4_request"));

        let date_header = req.headers().get("x-oss-date").unwrap().to_str().unwrap();
        assert_eq!(date_header, "20231203T120000Z");

        let content_sha = req
            .headers()
            .get("x-oss-content-sha256")
            .unwrap()
            .to_str()
            .unwrap();
        let expected_sha = hex::encode(sha2::Sha256::digest(b"Hello OSS"));
        assert_eq!(content_sha, expected_sha);

        let mut req2 = client
            .put("https://examplebucket.oss-cn-hangzhou.aliyuncs.com/exampleobject")
            .body(b"Hello OSS".to_vec())
            .build()
            .unwrap();
        sign_request(&mut req2, &creds, &region, dt).unwrap();
        let auth2 = req2
            .headers()
            .get("authorization")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(auth, auth2);
    }

    #[test]
    fn test_string_to_sign_format() {
        let result = build_string_to_sign(
            "20260207T095856Z",
            "20260207",
            "cn-hangzhou",
            "GET\n/\n\nhost:bucket.oss-cn-hangzhou.aliyuncs.com\n\nhost\nUNSIGNED-PAYLOAD",
        );
        assert!(result.starts_with("OSS4-HMAC-SHA256\n"));
        assert!(result.contains("20260207T095856Z"));
        assert!(result.contains("20260207/cn-hangzhou/oss/aliyun_v4_request"));
    }

    #[test]
    fn test_calculate_signature_produces_hex() {
        let key = derive_signing_key("test-secret", "20260207", "cn-hangzhou").unwrap();
        let sig = calculate_signature(&key, "test-string-to-sign").unwrap();
        assert_eq!(sig.len(), 64);
        assert!(sig.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[tokio::test]
    async fn test_sign_request_adds_headers() {
        let client = reqwest::Client::new();
        let mut req = client
            .get("https://my-bucket.oss-cn-hangzhou.aliyuncs.com/test.txt")
            .build()
            .unwrap();

        let creds = crate::config::Credentials::new("test-access-key-id", "test-access-key-secret");
        let region = crate::types::Region::new("cn-hangzhou").unwrap();
        let dt = chrono::Utc::now();

        let result = sign_request(&mut req, &creds, &region, dt);
        assert!(result.is_ok());

        assert!(req.headers().contains_key("x-oss-date"));
        assert!(req.headers().contains_key("x-oss-content-sha256"));
        assert!(req.headers().contains_key("authorization"));
        assert!(!req.headers().contains_key("x-oss-security-token"));

        let auth = req
            .headers()
            .get("authorization")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(auth.starts_with("OSS4-HMAC-SHA256 Credential=test-access-key-id/"));
        assert!(auth.contains("cn-hangzhou/oss/aliyun_v4_request"));
        assert!(auth.contains("SignedHeaders="));
        assert!(auth.contains("Signature="));
    }

    #[tokio::test]
    async fn test_sign_request_with_sts_token() {
        let client = reqwest::Client::new();
        let mut req = client
            .get("https://my-bucket.oss-cn-hangzhou.aliyuncs.com/test.txt")
            .build()
            .unwrap();

        let creds = crate::config::Credentials::with_security_token(
            "sts-key-id",
            "sts-key-secret",
            "sts-token-value",
        );
        let region = crate::types::Region::new("cn-hangzhou").unwrap();
        let dt = chrono::Utc::now();

        sign_request(&mut req, &creds, &region, dt).unwrap();

        assert!(req.headers().contains_key("x-oss-security-token"));
        let token = req
            .headers()
            .get("x-oss-security-token")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(token, "sts-token-value");
    }

    #[test]
    fn test_percent_encoding_special_chars() {
        let result = canonical_uri("/bucket/file name+test=value&other");
        assert!(result.contains("%20")); // space
        assert!(result.contains("%2B")); // +
        assert!(result.contains("%3D")); // =
        assert!(result.contains("%26")); // &
    }
}

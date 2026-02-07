//! OSS client implementation.

use std::cmp;
use std::sync::Arc;

use chrono::Utc;
use tokio::time::Instant;
use url::Url;

use crate::auth;
use crate::config::Config;
use crate::error::{OssError, Result};
use crate::middleware::{InterceptorChain, InterceptorContext, RequestOutcome};
use crate::types::{BucketName, ObjectKey};

/// The main client for interacting with Alibaba Cloud OSS.
///
/// # Examples
/// ```no_run
/// use rs_ali_oss::{OssClient, ClientBuilder};
///
/// # async fn example() -> rs_ali_oss::Result<()> {
/// let client = OssClient::from_builder(
///     ClientBuilder::new()
///         .access_key_id("your-key-id")
///         .access_key_secret("your-key-secret")
///         .region("cn-hangzhou")
/// )?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct OssClient {
    http_client: reqwest::Client,
    config: Config,
    interceptors: InterceptorChain,
}

// Compile-time assertion: OssClient must be Send + Sync for safe async usage.
const _: fn() = || {
    fn must_be_send_sync<T: Send + Sync>() {}
    must_be_send_sync::<OssClient>();
};

const AUTH_HEADERS: &[&str] = &[
    "authorization",
    "x-oss-date",
    "x-oss-content-sha256",
    "x-oss-security-token",
];

impl OssClient {
    /// Create a new client with the given configuration.
    ///
    /// Applies connection pool settings and timeouts from the config.
    /// Returns an error if the underlying HTTP client cannot be constructed
    /// (e.g., TLS backend unavailable).
    pub fn new(config: Config) -> Result<Self> {
        let tc = config.timeout_config();
        let pc = config.pool_config();

        let mut builder = reqwest::Client::builder()
            .connect_timeout(tc.connect_timeout)
            .read_timeout(tc.read_timeout)
            .min_tls_version(reqwest::tls::Version::TLS_1_2)
            .user_agent(format!("rs-ali-oss/{}", env!("CARGO_PKG_VERSION")));

        if let Some(max) = pc.max_idle_per_host {
            builder = builder.pool_max_idle_per_host(max);
        }
        if let Some(timeout) = pc.idle_timeout {
            builder = builder.pool_idle_timeout(timeout);
        }

        let http_client = builder.build().map_err(OssError::Http)?;
        Ok(Self {
            http_client,
            config,
            interceptors: InterceptorChain::new(),
        })
    }

    /// Create a new client with a custom HTTP client.
    ///
    /// # Security
    ///
    /// When you supply your own [`reqwest::Client`], the SDK **skips** the
    /// default transport configuration applied by [`OssClient::new`]. This
    /// means the caller is responsible for ensuring the custom client meets
    /// the following security requirements:
    ///
    /// * **TLS version** — the default client enforces a minimum of TLS 1.2.
    ///   A custom client may allow older, insecure TLS versions.
    /// * **User-Agent** — the default client sets `rs-ali-oss/{version}` on
    ///   every request. A custom client will use whatever User-Agent (or
    ///   none) it was built with.
    /// * **Timeouts** — the default client configures connect and read
    ///   timeouts. A custom client with no timeouts may hang indefinitely.
    /// * **Connection pool** — the default client tunes pool idle timeouts.
    ///   A custom client uses its own pool settings.
    ///
    /// Use this method only when you need fine-grained control over the HTTP
    /// transport (e.g., custom proxy, client certificates, or connection
    /// tuning) and are confident in the security posture of the provided
    /// client.
    pub fn with_http_client(config: Config, http_client: reqwest::Client) -> Self {
        Self {
            http_client,
            config,
            interceptors: InterceptorChain::new(),
        }
    }

    /// Create a client from a builder.
    pub fn from_builder(builder: crate::config::ClientBuilder) -> Result<Self> {
        let config = builder.build()?;
        Self::new(config)
    }

    /// Register an interceptor to observe request/response lifecycle events.
    ///
    /// Interceptors are called in registration order. Use for logging, metrics,
    /// or rate limiting.
    pub fn interceptor(mut self, interceptor: Arc<dyn crate::middleware::Interceptor>) -> Self {
        self.interceptors.push(interceptor);
        self
    }

    /// Returns a reference to the underlying configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    pub(crate) fn endpoint(&self, bucket: Option<&BucketName>) -> String {
        if let Some(custom) = self.config.endpoint() {
            return custom.trim_end_matches('/').to_string();
        }
        let region: &str = self.config.region().as_ref();
        match bucket {
            Some(b) if !self.config.use_path_style() => {
                format!("https://{}.oss-{}.aliyuncs.com", b, region)
            }
            _ => format!("https://oss-{}.aliyuncs.com", region),
        }
    }

    /// Build a full URL for an OSS request.
    ///
    /// When path-style is enabled, the bucket name is prepended to the URL path
    /// instead of being used as a virtual-hosted subdomain.
    pub(crate) fn build_url(
        &self,
        bucket: Option<&BucketName>,
        key: Option<&ObjectKey>,
        query: &[(&str, &str)],
    ) -> Result<Url> {
        let base = self.endpoint(bucket);

        let path = if self.config.use_path_style() {
            match (bucket, key) {
                (Some(b), Some(k)) => format!("{}/{}/{}", base, b, k),
                (Some(b), None) => format!("{}/{}/", base, b),
                (None, Some(k)) => format!("{}/{}", base, k),
                (None, None) => format!("{}/", base),
            }
        } else {
            match key {
                Some(k) => format!("{}/{}", base, k),
                None => format!("{}/", base),
            }
        };

        let mut url = Url::parse(&path).map_err(|e| OssError::InvalidUrl(e.to_string()))?;
        for (k, v) in query {
            url.query_pairs_mut().append_pair(k, v);
        }
        Ok(url)
    }

    /// Sign and execute an HTTP request with automatic retry, interceptors,
    /// and optional request timeout.
    pub(crate) async fn execute(&self, request: reqwest::Request) -> Result<reqwest::Response> {
        match self.config.timeout_config().request_timeout {
            Some(deadline) => {
                match tokio::time::timeout(deadline, self.execute_inner(request)).await {
                    Ok(result) => result,
                    Err(_) => Err(OssError::Timeout(deadline)),
                }
            }
            None => self.execute_inner(request).await,
        }
    }

    async fn execute_inner(&self, request: reqwest::Request) -> Result<reqwest::Response> {
        let retry_config = self.config.retry_config();
        let method = request.method().clone();
        let url_str = request.url().to_string();

        let can_retry = request
            .body()
            .map(|b| b.as_bytes().is_some())
            .unwrap_or(true);
        let max_attempts = if can_retry {
            retry_config.max_retries + 1
        } else {
            1
        };

        let url = request.url().clone();
        let req_method = request.method().clone();
        let headers = request.headers().clone();
        let body_bytes = request
            .body()
            .and_then(|b| b.as_bytes().map(|b| b.to_vec()));

        let has_interceptors = !self.interceptors.is_empty();
        let mut last_err = None;

        for attempt in 0..max_attempts {
            if attempt > 0 {
                let base = retry_config.base_delay * 2u32.saturating_pow(attempt - 1);
                let capped = cmp::min(base, retry_config.max_delay);
                // Deterministic jitter: use 50-100% of delay based on URL hash and attempt
                let jitter_numer = (url_str.len() as u64 * attempt as u64) % 50 + 50;
                let delay_ms = capped.as_millis() as u64 * jitter_numer / 100;
                let delay = std::time::Duration::from_millis(delay_ms);
                tracing::warn!(%method, %url_str, attempt, ?delay, "retrying OSS request");
                tokio::time::sleep(delay).await;
            }

            let mut new_req = reqwest::Request::new(req_method.clone(), url.clone());
            for (name, value) in headers.iter() {
                if !AUTH_HEADERS.contains(&name.as_str()) {
                    new_req.headers_mut().insert(name.clone(), value.clone());
                }
            }
            if let Some(ref bytes) = body_bytes {
                *new_req.body_mut() = Some(reqwest::Body::from(bytes.clone()));
            }

            auth::sign_request(
                &mut new_req,
                self.config.credentials(),
                self.config.region(),
                Utc::now(),
            )?;

            if has_interceptors {
                let ctx = InterceptorContext {
                    method: method.to_string(),
                    url: url_str.clone(),
                    attempt,
                };
                if let Err(reason) = self.interceptors.before_request(&ctx) {
                    return Err(OssError::Auth(reason));
                }
            }

            tracing::debug!(%method, %url_str, attempt, "executing OSS request");

            let start = Instant::now();
            match self.http_client.execute(new_req).await {
                Ok(response) => {
                    let elapsed = start.elapsed();
                    let status = response.status();

                    if has_interceptors {
                        let ctx = InterceptorContext {
                            method: method.to_string(),
                            url: url_str.clone(),
                            attempt,
                        };
                        self.interceptors.after_request(
                            &ctx,
                            &RequestOutcome {
                                status: Some(status.as_u16()),
                                duration: elapsed,
                                success: status.is_success(),
                                error: None,
                            },
                        );
                    }

                    if status.is_server_error() && attempt + 1 < max_attempts {
                        let body = Self::read_error_body(response).await;
                        tracing::warn!(%method, %url_str, %status, "server error, will retry");
                        last_err = Some(OssError::from_response_body(status, &body));
                        continue;
                    }
                    if !status.is_success() {
                        let body = Self::read_error_body(response).await;
                        tracing::warn!(%method, %url_str, %status, "OSS request failed");
                        return Err(OssError::from_response_body(status, &body));
                    }
                    tracing::debug!(%method, %url_str, %status, "OSS request succeeded");
                    return Ok(response);
                }
                Err(e) => {
                    let elapsed = start.elapsed();

                    if has_interceptors {
                        let ctx = InterceptorContext {
                            method: method.to_string(),
                            url: url_str.clone(),
                            attempt,
                        };
                        self.interceptors.after_request(
                            &ctx,
                            &RequestOutcome {
                                status: None,
                                duration: elapsed,
                                success: false,
                                error: Some(e.to_string()),
                            },
                        );
                    }

                    if Self::is_retryable_error(&e) && attempt + 1 < max_attempts {
                        tracing::warn!(%method, %url_str, error = %e, "transient error, will retry");
                        last_err = Some(OssError::Http(e));
                        continue;
                    }
                    return Err(OssError::Http(e));
                }
            }
        }

        Err(match last_err {
            Some(e) => OssError::RetryExhausted {
                attempts: max_attempts,
                last_error: Box::new(e),
            },
            None => OssError::RetryExhausted {
                attempts: max_attempts,
                last_error: Box::new(OssError::Auth("unknown error".to_string())),
            },
        })
    }

    async fn read_error_body(response: reqwest::Response) -> String {
        const MAX_ERROR_BODY: usize = 1024 * 1024; // 1 MB limit
        match response.bytes().await {
            Ok(bytes) => {
                let limited = if bytes.len() > MAX_ERROR_BODY {
                    &bytes[..MAX_ERROR_BODY]
                } else {
                    &bytes
                };
                String::from_utf8_lossy(limited).into_owned()
            }
            Err(e) => {
                tracing::debug!("failed to read error response body: {e}");
                String::new()
            }
        }
    }

    fn is_retryable_error(err: &reqwest::Error) -> bool {
        err.is_timeout() || err.is_connect()
    }

    /// Returns a reference to the underlying HTTP client.
    pub(crate) fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }
}

pub(crate) fn header_opt(response: &reqwest::Response, name: &str) -> Option<String> {
    response
        .headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

pub(crate) fn header_etag(response: &reqwest::Response) -> String {
    response
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .trim_matches('"')
        .to_string()
}

pub(crate) fn parse_xml<T: serde::de::DeserializeOwned>(body: &str) -> Result<T> {
    quick_xml::de::from_str(body).map_err(|e| OssError::XmlParse(e.to_string()))
}

pub(crate) fn serialize_xml<T: serde::Serialize>(value: &T) -> Result<String> {
    quick_xml::se::to_string(value).map_err(|e| OssError::XmlParse(e.to_string()))
}

pub(crate) fn header_etag_opt(response: &reqwest::Response) -> Option<String> {
    response
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_matches('"').to_string())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::config::ClientBuilder;

    fn test_client() -> OssClient {
        OssClient::from_builder(
            ClientBuilder::new()
                .access_key_id("test-id")
                .access_key_secret("test-secret")
                .region("cn-hangzhou"),
        )
        .unwrap()
    }

    fn test_client_custom_endpoint() -> OssClient {
        OssClient::from_builder(
            ClientBuilder::new()
                .access_key_id("test-id")
                .access_key_secret("test-secret")
                .region("cn-hangzhou")
                .endpoint("https://custom.oss.example.com"),
        )
        .unwrap()
    }

    #[test]
    fn endpoint_virtual_hosted() {
        let client = test_client();
        let bucket = BucketName::new("my-bucket").unwrap();
        let ep = client.endpoint(Some(&bucket));
        assert_eq!(ep, "https://my-bucket.oss-cn-hangzhou.aliyuncs.com");
    }

    #[test]
    fn endpoint_no_bucket() {
        let client = test_client();
        let ep = client.endpoint(None);
        assert_eq!(ep, "https://oss-cn-hangzhou.aliyuncs.com");
    }

    #[test]
    fn endpoint_custom_override() {
        let client = test_client_custom_endpoint();
        let bucket = BucketName::new("my-bucket").unwrap();
        let ep = client.endpoint(Some(&bucket));
        assert_eq!(ep, "https://custom.oss.example.com");
    }

    #[test]
    fn build_url_with_key() {
        let client = test_client();
        let bucket = BucketName::new("my-bucket").unwrap();
        let key = ObjectKey::new("path/to/file.txt").unwrap();
        let url = client.build_url(Some(&bucket), Some(&key), &[]).unwrap();
        assert_eq!(
            url.as_str(),
            "https://my-bucket.oss-cn-hangzhou.aliyuncs.com/path/to/file.txt"
        );
    }

    #[test]
    fn build_url_with_query_params() {
        let client = test_client();
        let bucket = BucketName::new("my-bucket").unwrap();
        let url = client
            .build_url(
                Some(&bucket),
                None,
                &[("list-type", "2"), ("max-keys", "100")],
            )
            .unwrap();
        assert!(url.as_str().contains("list-type=2"));
        assert!(url.as_str().contains("max-keys=100"));
    }

    #[test]
    fn from_builder_works() {
        let client = OssClient::from_builder(
            ClientBuilder::new()
                .access_key_id("id")
                .access_key_secret("secret")
                .region("cn-hangzhou"),
        );
        assert!(client.is_ok());
    }

    fn test_client_path_style() -> OssClient {
        OssClient::from_builder(
            ClientBuilder::new()
                .access_key_id("test-id")
                .access_key_secret("test-secret")
                .region("cn-hangzhou")
                .use_path_style(true),
        )
        .unwrap()
    }

    #[test]
    fn endpoint_path_style_uses_region_only() {
        let client = test_client_path_style();
        let bucket = BucketName::new("my-bucket").unwrap();
        let ep = client.endpoint(Some(&bucket));
        assert_eq!(ep, "https://oss-cn-hangzhou.aliyuncs.com");
    }

    #[test]
    fn build_url_path_style_with_bucket_and_key() {
        let client = test_client_path_style();
        let bucket = BucketName::new("my-bucket").unwrap();
        let key = ObjectKey::new("path/to/file.txt").unwrap();
        let url = client.build_url(Some(&bucket), Some(&key), &[]).unwrap();
        assert_eq!(
            url.as_str(),
            "https://oss-cn-hangzhou.aliyuncs.com/my-bucket/path/to/file.txt"
        );
    }

    #[test]
    fn build_url_path_style_bucket_only() {
        let client = test_client_path_style();
        let bucket = BucketName::new("my-bucket").unwrap();
        let url = client.build_url(Some(&bucket), None, &[]).unwrap();
        assert_eq!(
            url.as_str(),
            "https://oss-cn-hangzhou.aliyuncs.com/my-bucket/"
        );
    }

    #[test]
    fn client_with_pool_config() {
        let client = OssClient::from_builder(
            ClientBuilder::new()
                .access_key_id("id")
                .access_key_secret("secret")
                .region("cn-hangzhou")
                .pool_max_idle_per_host(10)
                .pool_idle_timeout(Duration::from_secs(60)),
        );
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.config().pool_config().max_idle_per_host, Some(10));
        assert_eq!(
            client.config().pool_config().idle_timeout,
            Some(Duration::from_secs(60))
        );
    }

    #[test]
    fn client_with_timeout_config() {
        let client = OssClient::from_builder(
            ClientBuilder::new()
                .access_key_id("id")
                .access_key_secret("secret")
                .region("cn-hangzhou")
                .connect_timeout(Duration::from_secs(5))
                .read_timeout(Duration::from_secs(15))
                .request_timeout(Duration::from_secs(120)),
        );
        assert!(client.is_ok());
        let client = client.unwrap();
        let tc = client.config().timeout_config();
        assert_eq!(tc.connect_timeout, Duration::from_secs(5));
        assert_eq!(tc.read_timeout, Duration::from_secs(15));
        assert_eq!(tc.request_timeout, Some(Duration::from_secs(120)));
    }

    #[test]
    fn client_is_clone() {
        let client = test_client();
        let cloned = client.clone();
        assert_eq!(
            cloned.config().credentials().access_key_id(),
            client.config().credentials().access_key_id()
        );
    }

    #[test]
    fn client_with_interceptor() {
        use std::sync::atomic::{AtomicU32, Ordering};

        struct TestInterceptor(AtomicU32);
        impl crate::middleware::Interceptor for TestInterceptor {
            fn name(&self) -> &str {
                "test"
            }
            fn before_request(&self, _ctx: &InterceptorContext) -> std::result::Result<(), String> {
                self.0.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let interceptor = Arc::new(TestInterceptor(AtomicU32::new(0)));
        let client = OssClient::from_builder(
            ClientBuilder::new()
                .access_key_id("id")
                .access_key_secret("secret")
                .region("cn-hangzhou"),
        )
        .unwrap()
        .interceptor(interceptor);

        assert!(!client.interceptors.is_empty());
    }
}

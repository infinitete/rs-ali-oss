//! Configuration types for the Alibaba Cloud OSS client.

use std::fmt;
use std::time::Duration;

use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use crate::error::{OssError, Result};
use crate::types::Region;

/// OSS access credentials.
///
/// The `access_key_secret` and `security_token` are protected with `Zeroize` and
/// redacted in `Debug` output.
/// `Display` is intentionally not implemented to prevent accidental credential exposure.
///
/// # STS Support
///
/// For temporary credentials obtained via STS (Security Token Service), use
/// [`Credentials::with_security_token`] or construct from an STS response via
/// [`Credentials::from_sts`] (requires the `sts` feature).
#[derive(Clone)]
pub struct Credentials {
    access_key_id: String,
    access_key_secret: SecretString,
    security_token: Option<SecretString>,
}

/// A string that is zeroed on drop and redacted in Debug.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
struct SecretString(String);

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("****")
    }
}

impl Credentials {
    /// Create new credentials with a permanent access key pair.
    pub fn new(access_key_id: impl Into<String>, access_key_secret: impl Into<String>) -> Self {
        Self {
            access_key_id: access_key_id.into(),
            access_key_secret: SecretString(access_key_secret.into()),
            security_token: None,
        }
    }

    /// Create new credentials with a temporary STS security token.
    pub fn with_security_token(
        access_key_id: impl Into<String>,
        access_key_secret: impl Into<String>,
        security_token: impl Into<String>,
    ) -> Self {
        Self {
            access_key_id: access_key_id.into(),
            access_key_secret: SecretString(access_key_secret.into()),
            security_token: Some(SecretString(security_token.into())),
        }
    }

    /// Construct credentials from an STS `Credentials` response.
    #[cfg(feature = "sts")]
    pub fn from_sts(sts_creds: &rs_ali_sts::Credentials) -> Self {
        Self::with_security_token(
            &sts_creds.access_key_id,
            &sts_creds.access_key_secret,
            &sts_creds.security_token,
        )
    }

    /// Returns the access key ID.
    pub fn access_key_id(&self) -> &str {
        &self.access_key_id
    }

    /// Returns the access key secret.
    pub fn access_key_secret(&self) -> &str {
        &self.access_key_secret.0
    }

    /// Returns the STS security token, if set.
    pub fn security_token(&self) -> Option<&str> {
        self.security_token.as_ref().map(|s| s.0.as_str())
    }
}

impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Credentials")
            .field("access_key_id", &self.access_key_id)
            .field("access_key_secret", &"****")
            .field(
                "security_token",
                &self.security_token.as_ref().map(|_| "****"),
            )
            .finish()
    }
}

/// Configuration for automatic request retry with exponential backoff.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (default: 3).
    pub max_retries: u32,
    /// Base delay between retries (default: 200ms).
    pub base_delay: Duration,
    /// Maximum delay between retries (default: 30s).
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(30),
        }
    }
}

/// Configuration for HTTP connection pooling.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum idle connections per host (default: no limit).
    pub max_idle_per_host: Option<usize>,
    /// Idle connection timeout (default: 90 seconds).
    pub idle_timeout: Option<Duration>,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_idle_per_host: None,
            idle_timeout: Some(Duration::from_secs(90)),
        }
    }
}

/// Configuration for HTTP timeouts.
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// TCP connect timeout (default: 10 seconds).
    pub connect_timeout: Duration,
    /// Per-response read timeout (default: 30 seconds).
    pub read_timeout: Duration,
    /// Overall request timeout including retries (default: none).
    pub request_timeout: Option<Duration>,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            read_timeout: Duration::from_secs(30),
            request_timeout: None,
        }
    }
}

/// Configuration for the OSS client.
#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) credentials: Credentials,
    pub(crate) region: Region,
    pub(crate) endpoint: Option<String>,
    pub(crate) use_path_style: bool,
    pub(crate) retry_config: RetryConfig,
    pub(crate) pool_config: PoolConfig,
    pub(crate) timeout_config: TimeoutConfig,
}

impl Config {
    /// Returns the credentials.
    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }

    /// Returns the region.
    pub fn region(&self) -> &Region {
        &self.region
    }

    /// Returns the custom endpoint, if set.
    pub fn endpoint(&self) -> Option<&str> {
        self.endpoint.as_deref()
    }

    /// Returns whether path-style URLs are used.
    pub fn use_path_style(&self) -> bool {
        self.use_path_style
    }

    /// Returns the retry configuration.
    pub fn retry_config(&self) -> &RetryConfig {
        &self.retry_config
    }

    /// Returns the connection pool configuration.
    pub fn pool_config(&self) -> &PoolConfig {
        &self.pool_config
    }

    /// Returns the timeout configuration.
    pub fn timeout_config(&self) -> &TimeoutConfig {
        &self.timeout_config
    }
}

/// Builder for constructing an OSS [`Config`].
///
/// # Examples
/// ```
/// # use rs_ali_oss::config::ClientBuilder;
/// let config = ClientBuilder::new()
///     .access_key_id("LTAI5tXXXX")
///     .access_key_secret("your-secret")
///     .region("cn-hangzhou")
///     .build();
/// ```
#[derive(Default)]
pub struct ClientBuilder {
    access_key_id: Option<String>,
    access_key_secret: Option<Zeroizing<String>>,
    security_token: Option<Zeroizing<String>>,
    region: Option<String>,
    endpoint: Option<String>,
    use_path_style: bool,
    max_retries: Option<u32>,
    base_retry_delay: Option<Duration>,
    max_retry_delay: Option<Duration>,
    pool_max_idle_per_host: Option<usize>,
    pool_idle_timeout: Option<Duration>,
    connect_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
    request_timeout: Option<Duration>,
    allow_insecure: bool,
}

impl ClientBuilder {
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the access key ID.
    pub fn access_key_id(mut self, id: impl Into<String>) -> Self {
        self.access_key_id = Some(id.into());
        self
    }

    /// Set the access key secret.
    pub fn access_key_secret(mut self, secret: impl Into<String>) -> Self {
        self.access_key_secret = Some(Zeroizing::new(secret.into()));
        self
    }

    /// Set the STS security token for temporary credentials.
    pub fn security_token(mut self, token: impl Into<String>) -> Self {
        self.security_token = Some(Zeroizing::new(token.into()));
        self
    }

    /// Set the region (e.g., "cn-hangzhou").
    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    /// Set a custom endpoint URL override.
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Use path-style URLs instead of virtual-hosted style.
    pub fn use_path_style(mut self, use_path_style: bool) -> Self {
        self.use_path_style = use_path_style;
        self
    }

    /// Set the maximum number of retry attempts for transient errors.
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Set the base delay between retries.
    pub fn base_retry_delay(mut self, delay: Duration) -> Self {
        self.base_retry_delay = Some(delay);
        self
    }

    /// Set the maximum delay between retries.
    pub fn max_retry_delay(mut self, delay: Duration) -> Self {
        self.max_retry_delay = Some(delay);
        self
    }

    /// Set the maximum number of idle connections kept alive per host.
    pub fn pool_max_idle_per_host(mut self, max: usize) -> Self {
        self.pool_max_idle_per_host = Some(max);
        self
    }

    /// Set how long idle connections remain in the pool before being closed.
    pub fn pool_idle_timeout(mut self, timeout: Duration) -> Self {
        self.pool_idle_timeout = Some(timeout);
        self
    }

    /// Set the TCP connect timeout (default: 10 seconds).
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    /// Set the per-response read timeout (default: 30 seconds).
    pub fn read_timeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = Some(timeout);
        self
    }

    /// Set an overall request timeout that caps the entire operation including retries.
    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = Some(timeout);
        self
    }

    /// Allow insecure HTTP endpoints (default: false).
    ///
    /// By default, custom endpoints must use HTTPS. Enable this for local development
    /// or testing with HTTP endpoints.
    pub fn allow_insecure(mut self, allow: bool) -> Self {
        self.allow_insecure = allow;
        self
    }

    /// Build the [`Config`], validating all required fields.
    pub fn build(self) -> Result<Config> {
        let access_key_id = self
            .access_key_id
            .ok_or_else(|| OssError::MissingField("access_key_id".to_string()))?;

        if access_key_id.trim().is_empty() {
            return Err(OssError::InvalidParameter {
                field: "access_key_id".into(),
                reason: "must not be empty or whitespace-only".into(),
            });
        }

        let mut access_key_secret = self
            .access_key_secret
            .ok_or_else(|| OssError::MissingField("access_key_secret".to_string()))?;
        let region_str = self
            .region
            .ok_or_else(|| OssError::MissingField("region".to_string()))?;
        let region = Region::new(region_str)?;

        if self
            .endpoint
            .as_ref()
            .is_some_and(|e| e.starts_with("http://") && !self.allow_insecure)
        {
            return Err(OssError::InvalidParameter {
                field: "endpoint".into(),
                reason: "HTTP endpoints are insecure; use HTTPS or call .allow_insecure(true)"
                    .into(),
            });
        }

        let mut retry_config = RetryConfig::default();
        if let Some(max_retries) = self.max_retries {
            retry_config.max_retries = max_retries;
        }
        if let Some(base_delay) = self.base_retry_delay {
            retry_config.base_delay = base_delay;
        }
        if let Some(max_delay) = self.max_retry_delay {
            retry_config.max_delay = max_delay;
        }

        let mut pool_config = PoolConfig::default();
        if let Some(max) = self.pool_max_idle_per_host {
            pool_config.max_idle_per_host = Some(max);
        }
        if let Some(timeout) = self.pool_idle_timeout {
            pool_config.idle_timeout = Some(timeout);
        }

        let mut timeout_config = TimeoutConfig::default();
        if let Some(t) = self.connect_timeout {
            timeout_config.connect_timeout = t;
        }
        if let Some(t) = self.read_timeout {
            timeout_config.read_timeout = t;
        }
        timeout_config.request_timeout = self.request_timeout;

        // Move the inner String out of Zeroizing to avoid creating an
        // intermediate, unzeroized copy on the heap.
        let secret_str = Zeroizing::new(std::mem::take(&mut *access_key_secret));

        Ok(Config {
            credentials: match self.security_token {
                Some(mut token) => {
                    let token_str = Zeroizing::new(std::mem::take(&mut *token));
                    Credentials::with_security_token(access_key_id, &*secret_str, &*token_str)
                }
                None => Credentials::new(access_key_id, &*secret_str),
            },
            region,
            endpoint: self.endpoint,
            use_path_style: self.use_path_style,
            retry_config,
            pool_config,
            timeout_config,
        })
    }
}

impl fmt::Debug for ClientBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientBuilder")
            .field("access_key_id", &self.access_key_id)
            .field(
                "access_key_secret",
                &self.access_key_secret.as_ref().map(|_| "****"),
            )
            .field(
                "security_token",
                &self.security_token.as_ref().map(|_| "****"),
            )
            .field("region", &self.region)
            .field("endpoint", &self.endpoint)
            .field("use_path_style", &self.use_path_style)
            .field("max_retries", &self.max_retries)
            .field("base_retry_delay", &self.base_retry_delay)
            .field("max_retry_delay", &self.max_retry_delay)
            .field("pool_max_idle_per_host", &self.pool_max_idle_per_host)
            .field("pool_idle_timeout", &self.pool_idle_timeout)
            .field("connect_timeout", &self.connect_timeout)
            .field("read_timeout", &self.read_timeout)
            .field("request_timeout", &self.request_timeout)
            .field("allow_insecure", &self.allow_insecure)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_with_all_fields() {
        let config = ClientBuilder::new()
            .access_key_id("test-id")
            .access_key_secret("test-secret")
            .region("cn-hangzhou")
            .build();
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.credentials().access_key_id(), "test-id");
        assert_eq!(config.credentials().access_key_secret(), "test-secret");
        assert_eq!(config.region().as_ref(), "cn-hangzhou");
    }

    #[test]
    fn builder_missing_access_key_id() {
        let result = ClientBuilder::new()
            .access_key_secret("secret")
            .region("cn-hangzhou")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_missing_access_key_secret() {
        let result = ClientBuilder::new()
            .access_key_id("id")
            .region("cn-hangzhou")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_missing_region() {
        let result = ClientBuilder::new()
            .access_key_id("id")
            .access_key_secret("secret")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_invalid_region() {
        let result = ClientBuilder::new()
            .access_key_id("id")
            .access_key_secret("secret")
            .region("INVALID")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn credentials_debug_redacts_secret() {
        let creds = Credentials::new("LTAI5tXXXX", "my-super-secret-key");
        let debug_output = format!("{creds:?}");
        assert!(debug_output.contains("LTAI5tXXXX"));
        assert!(!debug_output.contains("my-super-secret-key"));
        assert!(debug_output.contains("****"));
    }

    #[test]
    fn config_with_custom_endpoint() {
        let config = ClientBuilder::new()
            .access_key_id("id")
            .access_key_secret("secret")
            .region("cn-hangzhou")
            .endpoint("https://custom.oss.example.com")
            .build()
            .unwrap();
        assert_eq!(config.endpoint(), Some("https://custom.oss.example.com"));
    }

    #[test]
    fn config_default_no_path_style() {
        let config = ClientBuilder::new()
            .access_key_id("id")
            .access_key_secret("secret")
            .region("cn-hangzhou")
            .build()
            .unwrap();
        assert!(!config.use_path_style());
    }

    #[test]
    fn credentials_without_security_token() {
        let creds = Credentials::new("id", "secret");
        assert_eq!(creds.access_key_id(), "id");
        assert_eq!(creds.access_key_secret(), "secret");
        assert!(creds.security_token().is_none());
    }

    #[test]
    fn credentials_with_security_token() {
        let creds = Credentials::with_security_token("sts-id", "sts-secret", "sts-token");
        assert_eq!(creds.access_key_id(), "sts-id");
        assert_eq!(creds.access_key_secret(), "sts-secret");
        assert_eq!(creds.security_token(), Some("sts-token"));
    }

    #[test]
    fn credentials_debug_redacts_security_token() {
        let creds = Credentials::with_security_token("id", "my-secret-value", "my-token-value");
        let debug_output = format!("{creds:?}");
        assert!(!debug_output.contains("my-secret-value"));
        assert!(!debug_output.contains("my-token-value"));
        assert!(debug_output.contains("****"));
    }

    #[test]
    fn builder_with_security_token() {
        let config = ClientBuilder::new()
            .access_key_id("sts-id")
            .access_key_secret("sts-secret")
            .security_token("sts-token")
            .region("cn-hangzhou")
            .build()
            .unwrap();
        assert_eq!(config.credentials().access_key_id(), "sts-id");
        assert_eq!(config.credentials().security_token(), Some("sts-token"));
    }

    #[test]
    fn builder_debug_redacts_secrets() {
        let builder = ClientBuilder::new()
            .access_key_id("LTAI5tXXXX")
            .access_key_secret("super-secret-key-12345")
            .security_token("sensitive-token-value-67890")
            .region("cn-hangzhou");
        let debug_output = format!("{builder:?}");
        assert!(debug_output.contains("LTAI5tXXXX"));
        assert!(debug_output.contains("****"));
        assert!(!debug_output.contains("super-secret-key-12345"));
        assert!(!debug_output.contains("sensitive-token-value-67890"));
        assert!(debug_output.contains("cn-hangzhou"));
    }

    #[test]
    fn builder_debug_without_secrets_shows_none() {
        let builder = ClientBuilder::new().access_key_id("LTAI5tXXXX");
        let debug_output = format!("{builder:?}");
        assert!(debug_output.contains("LTAI5tXXXX"));
        assert!(debug_output.contains("access_key_secret: None"));
        assert!(debug_output.contains("security_token: None"));
    }

    #[test]
    fn builder_empty_access_key_id() {
        let result = ClientBuilder::new()
            .access_key_id("")
            .access_key_secret("secret")
            .region("cn-hangzhou")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_whitespace_access_key_id() {
        let result = ClientBuilder::new()
            .access_key_id("   ")
            .access_key_secret("secret")
            .region("cn-hangzhou")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_rejects_http_endpoint() {
        let result = ClientBuilder::new()
            .access_key_id("id")
            .access_key_secret("secret")
            .region("cn-hangzhou")
            .endpoint("http://localhost:8080")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_allows_http_with_insecure_flag() {
        let result = ClientBuilder::new()
            .access_key_id("id")
            .access_key_secret("secret")
            .region("cn-hangzhou")
            .endpoint("http://localhost:8080")
            .allow_insecure(true)
            .build();
        assert!(result.is_ok());
    }

    #[test]
    fn builder_allows_https_endpoint() {
        let result = ClientBuilder::new()
            .access_key_id("id")
            .access_key_secret("secret")
            .region("cn-hangzhou")
            .endpoint("https://custom.oss.example.com")
            .build();
        assert!(result.is_ok());
    }
}

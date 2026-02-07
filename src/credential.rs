//! Credential providers for flexible authentication.
//!
//! Provides a [`CredentialProvider`] trait and built-in implementations for
//! loading credentials from various sources.

use std::fmt;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use crate::config::Credentials;
use crate::error::{OssError, Result};

/// A source of OSS credentials.
///
/// Implement this trait to provide credentials from custom sources such as
/// vaults, metadata services, or credential files.
pub trait CredentialProvider: Send + Sync {
    /// Resolve credentials from this provider.
    fn resolve(&self) -> Result<Credentials>;

    /// Provider name for diagnostics.
    fn provider_name(&self) -> &str;
}

impl fmt::Debug for dyn CredentialProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CredentialProvider({})", self.provider_name())
    }
}

/// Provides fixed credentials that never change.
#[derive(Clone)]
pub struct StaticProvider {
    credentials: Credentials,
}

impl StaticProvider {
    /// Create a provider with permanent access keys.
    pub fn new(access_key_id: impl Into<String>, access_key_secret: impl Into<String>) -> Self {
        Self {
            credentials: Credentials::new(access_key_id, access_key_secret),
        }
    }

    /// Create a provider with temporary STS credentials.
    pub fn with_security_token(
        access_key_id: impl Into<String>,
        access_key_secret: impl Into<String>,
        security_token: impl Into<String>,
    ) -> Self {
        Self {
            credentials: Credentials::with_security_token(
                access_key_id,
                access_key_secret,
                security_token,
            ),
        }
    }

    /// Create a provider from existing credentials.
    pub fn from_credentials(credentials: Credentials) -> Self {
        Self { credentials }
    }
}

impl fmt::Debug for StaticProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StaticProvider")
            .field("credentials", &self.credentials)
            .finish()
    }
}

impl CredentialProvider for StaticProvider {
    fn resolve(&self) -> Result<Credentials> {
        Ok(self.credentials.clone())
    }

    fn provider_name(&self) -> &str {
        "static"
    }
}

/// Loads credentials from environment variables.
///
/// Reads the following variables:
/// - `ALIBABA_CLOUD_ACCESS_KEY_ID` (required)
/// - `ALIBABA_CLOUD_ACCESS_KEY_SECRET` (required)
/// - `ALIBABA_CLOUD_SECURITY_TOKEN` (optional, for STS)
#[derive(Debug, Clone, Default)]
pub struct EnvironmentProvider;

impl EnvironmentProvider {
    /// Create a new environment credential provider.
    pub fn new() -> Self {
        Self
    }
}

impl CredentialProvider for EnvironmentProvider {
    fn resolve(&self) -> Result<Credentials> {
        let access_key_id = std::env::var("ALIBABA_CLOUD_ACCESS_KEY_ID").map_err(|_| {
            OssError::MissingField(
                "ALIBABA_CLOUD_ACCESS_KEY_ID environment variable not set".to_string(),
            )
        })?;

        let access_key_secret = std::env::var("ALIBABA_CLOUD_ACCESS_KEY_SECRET").map_err(|_| {
            OssError::MissingField(
                "ALIBABA_CLOUD_ACCESS_KEY_SECRET environment variable not set".to_string(),
            )
        })?;

        if access_key_id.trim().is_empty() {
            return Err(OssError::InvalidParameter {
                field: "ALIBABA_CLOUD_ACCESS_KEY_ID".into(),
                reason: "must not be empty".into(),
            });
        }

        match std::env::var("ALIBABA_CLOUD_SECURITY_TOKEN") {
            Ok(token) if !token.is_empty() => Ok(Credentials::with_security_token(
                access_key_id,
                access_key_secret,
                token,
            )),
            _ => Ok(Credentials::new(access_key_id, access_key_secret)),
        }
    }

    fn provider_name(&self) -> &str {
        "environment"
    }
}

/// Tries multiple providers in order, returning the first successful result.
pub struct ProviderChain {
    providers: Vec<Box<dyn CredentialProvider>>,
}

impl ProviderChain {
    /// Create an empty provider chain.
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Create a chain with the default provider order:
    /// 1. Environment variables
    pub fn default_chain() -> Self {
        let mut chain = Self::new();
        chain.push(EnvironmentProvider::new());
        chain
    }

    /// Append a provider to the chain.
    pub fn push(&mut self, provider: impl CredentialProvider + 'static) {
        self.providers.push(Box::new(provider));
    }

    /// Append a provider to the chain (builder pattern).
    pub fn with(mut self, provider: impl CredentialProvider + 'static) -> Self {
        self.push(provider);
        self
    }
}

impl Default for ProviderChain {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ProviderChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<&str> = self.providers.iter().map(|p| p.provider_name()).collect();
        f.debug_struct("ProviderChain")
            .field("providers", &names)
            .finish()
    }
}

impl CredentialProvider for ProviderChain {
    fn resolve(&self) -> Result<Credentials> {
        let mut last_err = None;
        for provider in &self.providers {
            match provider.resolve() {
                Ok(creds) => return Ok(creds),
                Err(e) => last_err = Some(e),
            }
        }
        Err(last_err
            .unwrap_or_else(|| OssError::Auth("no credential providers configured".to_string())))
    }

    fn provider_name(&self) -> &str {
        "chain"
    }
}

/// Wraps another provider and caches its credentials for a configurable TTL.
///
/// When [`resolve`](CredentialProvider::resolve) is called, returns the cached
/// credentials if they are still valid. Otherwise, calls the inner provider to
/// obtain fresh credentials and caches the result.
///
/// This is especially useful for STS temporary credentials that are expensive
/// to obtain and have a limited lifetime.
///
/// # Examples
/// ```
/// use rs_ali_oss::credential::{CachingProvider, EnvironmentProvider};
/// use std::time::Duration;
///
/// let provider = CachingProvider::new(EnvironmentProvider::new(), Duration::from_secs(900));
/// ```
pub struct CachingProvider {
    inner: Box<dyn CredentialProvider>,
    ttl: Duration,
    cache: RwLock<Option<CachedEntry>>,
}

struct CachedEntry {
    credentials: Credentials,
    fetched_at: Instant,
}

impl CachingProvider {
    /// Create a caching wrapper around `inner` that refreshes every `ttl`.
    pub fn new(inner: impl CredentialProvider + 'static, ttl: Duration) -> Self {
        Self {
            inner: Box::new(inner),
            ttl,
            cache: RwLock::new(None),
        }
    }

    /// Force-clear the cached credentials so the next `resolve` fetches fresh ones.
    pub fn invalidate(&self) {
        let mut guard = self.cache.write().unwrap_or_else(|e| e.into_inner());
        *guard = None;
    }
}

impl CredentialProvider for CachingProvider {
    fn resolve(&self) -> Result<Credentials> {
        // Fast path: read lock, return cached if still valid
        {
            let guard = self.cache.read().unwrap_or_else(|e| e.into_inner());
            if let Some(ref entry) = *guard
                && entry.fetched_at.elapsed() < self.ttl
            {
                return Ok(entry.credentials.clone());
            }
        }

        // Slow path: write lock, double-check, then refresh
        let mut guard = self.cache.write().unwrap_or_else(|e| e.into_inner());
        if let Some(ref entry) = *guard
            && entry.fetched_at.elapsed() < self.ttl
        {
            return Ok(entry.credentials.clone());
        }

        let credentials = self.inner.resolve()?;
        *guard = Some(CachedEntry {
            credentials: credentials.clone(),
            fetched_at: Instant::now(),
        });
        Ok(credentials)
    }

    fn provider_name(&self) -> &str {
        "caching"
    }
}

impl fmt::Debug for CachingProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CachingProvider")
            .field("inner", &self.inner.provider_name())
            .field("ttl", &self.ttl)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_provider_resolves() {
        let provider = StaticProvider::new("ak-id", "ak-secret");
        let creds = provider.resolve().unwrap();
        assert_eq!(creds.access_key_id(), "ak-id");
        assert_eq!(creds.access_key_secret(), "ak-secret");
        assert!(creds.security_token().is_none());
    }

    #[test]
    fn static_provider_with_sts() {
        let provider = StaticProvider::with_security_token("ak-id", "ak-secret", "token");
        let creds = provider.resolve().unwrap();
        assert_eq!(creds.security_token(), Some("token"));
    }

    #[test]
    fn static_provider_from_credentials() {
        let creds = Credentials::new("id", "secret");
        let provider = StaticProvider::from_credentials(creds);
        let resolved = provider.resolve().unwrap();
        assert_eq!(resolved.access_key_id(), "id");
    }

    #[test]
    fn static_provider_debug_redacts() {
        let provider = StaticProvider::new("LTAI5tXXXX", "super-secret");
        let debug = format!("{provider:?}");
        assert!(debug.contains("LTAI5tXXXX"));
        assert!(!debug.contains("super-secret"));
    }

    #[test]
    fn chain_empty_fails() {
        let chain = ProviderChain::new();
        assert!(chain.resolve().is_err());
    }

    #[test]
    fn chain_returns_first_success() {
        let chain = ProviderChain::new()
            .with(StaticProvider::new("first-id", "first-secret"))
            .with(StaticProvider::new("second-id", "second-secret"));
        let creds = chain.resolve().unwrap();
        assert_eq!(creds.access_key_id(), "first-id");
    }

    #[test]
    fn chain_skips_failures() {
        let chain = ProviderChain::new()
            .with(EnvironmentProvider::new())
            .with(StaticProvider::new("fallback-id", "fallback-secret"));
        let creds = chain.resolve().unwrap();
        assert_eq!(creds.access_key_id(), "fallback-id");
    }

    #[test]
    fn chain_debug_shows_provider_names() {
        let chain = ProviderChain::new()
            .with(EnvironmentProvider::new())
            .with(StaticProvider::new("id", "secret"));
        let debug = format!("{chain:?}");
        assert!(debug.contains("environment"));
        assert!(debug.contains("static"));
    }

    #[test]
    fn environment_provider_missing_vars_fails() {
        temp_env::with_vars_unset(
            [
                "ALIBABA_CLOUD_ACCESS_KEY_ID",
                "ALIBABA_CLOUD_ACCESS_KEY_SECRET",
                "ALIBABA_CLOUD_SECURITY_TOKEN",
            ],
            || {
                let provider = EnvironmentProvider::new();
                assert!(provider.resolve().is_err());
            },
        );
    }

    #[test]
    fn provider_name_correct() {
        assert_eq!(StaticProvider::new("a", "b").provider_name(), "static");
        assert_eq!(EnvironmentProvider::new().provider_name(), "environment");
        assert_eq!(ProviderChain::new().provider_name(), "chain");
        assert_eq!(
            CachingProvider::new(StaticProvider::new("a", "b"), Duration::from_secs(60))
                .provider_name(),
            "caching"
        );
    }

    #[test]
    fn caching_provider_returns_credentials() {
        let provider = CachingProvider::new(
            StaticProvider::new("cached-id", "cached-secret"),
            Duration::from_secs(300),
        );
        let creds = provider.resolve().unwrap();
        assert_eq!(creds.access_key_id(), "cached-id");
        assert_eq!(creds.access_key_secret(), "cached-secret");
    }

    #[test]
    fn caching_provider_returns_same_credentials_within_ttl() {
        let provider = CachingProvider::new(
            StaticProvider::new("id", "secret"),
            Duration::from_secs(300),
        );
        let creds1 = provider.resolve().unwrap();
        let creds2 = provider.resolve().unwrap();
        assert_eq!(creds1.access_key_id(), creds2.access_key_id());
    }

    #[test]
    fn caching_provider_refreshes_after_ttl() {
        let provider = CachingProvider::new(
            StaticProvider::new("id", "secret"),
            Duration::from_millis(0),
        );
        let creds = provider.resolve().unwrap();
        assert_eq!(creds.access_key_id(), "id");
        std::thread::sleep(Duration::from_millis(1));
        let creds2 = provider.resolve().unwrap();
        assert_eq!(creds2.access_key_id(), "id");
    }

    #[test]
    fn caching_provider_invalidate_clears_cache() {
        let provider = CachingProvider::new(
            StaticProvider::new("id", "secret"),
            Duration::from_secs(300),
        );
        provider.resolve().unwrap();
        provider.invalidate();
        let creds = provider.resolve().unwrap();
        assert_eq!(creds.access_key_id(), "id");
    }

    #[test]
    fn caching_provider_debug_shows_inner_name() {
        let provider =
            CachingProvider::new(StaticProvider::new("id", "secret"), Duration::from_secs(60));
        let debug = format!("{provider:?}");
        assert!(debug.contains("CachingProvider"));
        assert!(debug.contains("static"));
        assert!(debug.contains("60"));
    }

    #[test]
    fn caching_provider_propagates_inner_error() {
        let provider = CachingProvider::new(ProviderChain::new(), Duration::from_secs(300));
        assert!(provider.resolve().is_err());
    }
}

//! HTTP middleware/interceptor abstraction for request lifecycle hooks.
//!
//! Interceptors can observe and optionally modify requests before they are sent
//! and responses after they are received. Use cases include logging, metrics,
//! rate limiting, and custom header injection.

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

/// Context passed to interceptors for each request attempt.
#[derive(Debug)]
pub struct InterceptorContext {
    /// HTTP method (e.g., "GET", "PUT").
    pub method: String,
    /// Full request URL.
    pub url: String,
    /// Zero-based retry attempt number.
    pub attempt: u32,
}

/// Result of a completed HTTP request, passed to `after_request`.
#[derive(Debug)]
pub struct RequestOutcome {
    /// HTTP status code, if a response was received.
    pub status: Option<u16>,
    /// Wall-clock duration of the HTTP round-trip.
    pub duration: Duration,
    /// Whether the request succeeded (2xx status).
    pub success: bool,
    /// Error message, if the request failed.
    pub error: Option<String>,
}

/// Trait for intercepting HTTP request/response lifecycle events.
///
/// Implement this trait to add cross-cutting concerns such as logging,
/// metrics collection, rate limiting, or custom header injection.
///
/// # Examples
/// ```
/// use rs_ali_oss::middleware::{Interceptor, InterceptorContext, RequestOutcome};
///
/// struct MetricsInterceptor;
///
/// impl Interceptor for MetricsInterceptor {
///     fn name(&self) -> &str { "metrics" }
///
///     fn before_request(&self, ctx: &InterceptorContext) -> Result<(), String> {
///         println!("[{}] {} {}", ctx.attempt, ctx.method, ctx.url);
///         Ok(())
///     }
///
///     fn after_request(&self, ctx: &InterceptorContext, outcome: &RequestOutcome) {
///         println!(
///             "[{}] {} {} -> {:?} in {:?}",
///             ctx.attempt, ctx.method, ctx.url, outcome.status, outcome.duration
///         );
///     }
/// }
/// ```
pub trait Interceptor: Send + Sync {
    /// Human-readable name for this interceptor (used in logging).
    fn name(&self) -> &str;

    /// Called before each HTTP request attempt (including retries).
    ///
    /// Return `Err(reason)` to abort the request with an authentication error.
    /// Return `Ok(())` to proceed normally.
    fn before_request(&self, _ctx: &InterceptorContext) -> Result<(), String> {
        Ok(())
    }

    /// Called after each HTTP request attempt completes (success or failure).
    fn after_request(&self, _ctx: &InterceptorContext, _outcome: &RequestOutcome) {}
}

impl fmt::Debug for dyn Interceptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Interceptor({})", self.name())
    }
}

/// An ordered collection of interceptors applied to every request.
#[derive(Clone, Default)]
pub(crate) struct InterceptorChain {
    interceptors: Vec<Arc<dyn Interceptor>>,
}

impl fmt::Debug for InterceptorChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.interceptors.iter().map(|i| i.name()))
            .finish()
    }
}

impl InterceptorChain {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn push(&mut self, interceptor: Arc<dyn Interceptor>) {
        self.interceptors.push(interceptor);
    }

    pub(crate) fn before_request(&self, ctx: &InterceptorContext) -> Result<(), String> {
        for interceptor in &self.interceptors {
            interceptor.before_request(ctx)?;
        }
        Ok(())
    }

    pub(crate) fn after_request(&self, ctx: &InterceptorContext, outcome: &RequestOutcome) {
        for interceptor in &self.interceptors {
            interceptor.after_request(ctx, outcome);
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.interceptors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::*;

    struct CountingInterceptor {
        before_count: AtomicU32,
        after_count: AtomicU32,
    }

    impl CountingInterceptor {
        fn new() -> Self {
            Self {
                before_count: AtomicU32::new(0),
                after_count: AtomicU32::new(0),
            }
        }
    }

    impl Interceptor for CountingInterceptor {
        fn name(&self) -> &str {
            "counting"
        }

        fn before_request(&self, _ctx: &InterceptorContext) -> Result<(), String> {
            self.before_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn after_request(&self, _ctx: &InterceptorContext, _outcome: &RequestOutcome) {
            self.after_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    struct BlockingInterceptor;

    impl Interceptor for BlockingInterceptor {
        fn name(&self) -> &str {
            "blocking"
        }

        fn before_request(&self, _ctx: &InterceptorContext) -> Result<(), String> {
            Err("rate limited".to_string())
        }
    }

    fn sample_context() -> InterceptorContext {
        InterceptorContext {
            method: "GET".to_string(),
            url: "https://example.com/test".to_string(),
            attempt: 0,
        }
    }

    fn sample_outcome() -> RequestOutcome {
        RequestOutcome {
            status: Some(200),
            duration: Duration::from_millis(42),
            success: true,
            error: None,
        }
    }

    #[test]
    fn chain_calls_interceptors_in_order() {
        let counter = Arc::new(CountingInterceptor::new());
        let mut chain = InterceptorChain::new();
        chain.push(counter.clone());

        let ctx = sample_context();
        chain.before_request(&ctx).unwrap();
        chain.after_request(&ctx, &sample_outcome());

        assert_eq!(counter.before_count.load(Ordering::SeqCst), 1);
        assert_eq!(counter.after_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn chain_propagates_before_error() {
        let mut chain = InterceptorChain::new();
        chain.push(Arc::new(BlockingInterceptor));

        let ctx = sample_context();
        let result = chain.before_request(&ctx);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "rate limited");
    }

    #[test]
    fn chain_stops_on_first_error() {
        let counter = Arc::new(CountingInterceptor::new());
        let mut chain = InterceptorChain::new();
        chain.push(Arc::new(BlockingInterceptor));
        chain.push(counter.clone());

        let ctx = sample_context();
        let _ = chain.before_request(&ctx);

        assert_eq!(counter.before_count.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn empty_chain_succeeds() {
        let chain = InterceptorChain::new();
        let ctx = sample_context();
        assert!(chain.before_request(&ctx).is_ok());
        assert!(chain.is_empty());
    }

    #[test]
    fn interceptor_debug_shows_name() {
        let interceptor: Arc<dyn Interceptor> = Arc::new(CountingInterceptor::new());
        let debug = format!("{interceptor:?}");
        assert!(debug.contains("counting"));
    }

    #[test]
    fn chain_debug_shows_names() {
        let mut chain = InterceptorChain::new();
        chain.push(Arc::new(CountingInterceptor::new()));
        chain.push(Arc::new(BlockingInterceptor));
        let debug = format!("{chain:?}");
        assert!(debug.contains("counting"));
        assert!(debug.contains("blocking"));
    }
}

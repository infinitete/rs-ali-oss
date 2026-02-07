//! Progress tracking for upload and download operations.

use std::sync::Arc;

/// Describes the type of transfer being tracked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferKind {
    /// Upload (PUT) operation.
    Upload,
    /// Download (GET) operation.
    Download,
}

/// Snapshot of transfer progress at a point in time.
#[derive(Debug, Clone)]
pub struct TransferProgress {
    /// Bytes transferred so far.
    pub bytes_transferred: u64,
    /// Total bytes expected (if known).
    pub total_bytes: Option<u64>,
    /// Transfer direction.
    pub kind: TransferKind,
}

impl TransferProgress {
    /// Returns the completion ratio (0.0 to 1.0), or None if total is unknown.
    pub fn fraction(&self) -> Option<f64> {
        self.total_bytes
            .map(|total| self.bytes_transferred as f64 / total as f64)
    }
}

/// Receives progress updates during upload/download operations.
///
/// Implement this trait to display progress bars, log throughput, or
/// enforce transfer timeouts.
pub trait ProgressListener: Send + Sync {
    /// Called periodically as bytes are transferred.
    fn on_progress(&self, progress: &TransferProgress);
}

impl<F> ProgressListener for F
where
    F: Fn(&TransferProgress) + Send + Sync,
{
    fn on_progress(&self, progress: &TransferProgress) {
        self(progress);
    }
}

/// A no-op listener that discards all progress events.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopProgressListener;

impl ProgressListener for NoopProgressListener {
    fn on_progress(&self, _progress: &TransferProgress) {}
}

/// Wraps a [`ProgressListener`] in an [`Arc`] for shared ownership.
pub fn shared_listener(listener: impl ProgressListener + 'static) -> Arc<dyn ProgressListener> {
    Arc::new(listener)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    #[test]
    fn closure_as_listener() {
        let counter = Arc::new(AtomicU64::new(0));
        let counter_clone = counter.clone();
        let listener = move |p: &TransferProgress| {
            counter_clone.store(p.bytes_transferred, Ordering::SeqCst);
        };

        let progress = TransferProgress {
            bytes_transferred: 42,
            total_bytes: Some(100),
            kind: TransferKind::Upload,
        };
        listener.on_progress(&progress);
        assert_eq!(counter.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn fraction_with_total() {
        let p = TransferProgress {
            bytes_transferred: 50,
            total_bytes: Some(100),
            kind: TransferKind::Download,
        };
        assert!((p.fraction().unwrap() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn fraction_without_total() {
        let p = TransferProgress {
            bytes_transferred: 50,
            total_bytes: None,
            kind: TransferKind::Upload,
        };
        assert!(p.fraction().is_none());
    }

    #[test]
    fn noop_listener_compiles() {
        let listener = NoopProgressListener;
        let progress = TransferProgress {
            bytes_transferred: 0,
            total_bytes: None,
            kind: TransferKind::Upload,
        };
        listener.on_progress(&progress);
    }

    #[test]
    fn shared_listener_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Arc<dyn ProgressListener>>();
    }

    #[test]
    fn transfer_kind_eq() {
        assert_eq!(TransferKind::Upload, TransferKind::Upload);
        assert_ne!(TransferKind::Upload, TransferKind::Download);
    }
}

//! Host context trait for integrating logging, metrics, and cancellation.

use std::fmt;

/// Log levels for host logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Error level
    Error,
    /// Warning level
    Warn,
    /// Info level
    Info,
    /// Debug level
    Debug,
    /// Trace level
    Trace,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Trace => write!(f, "TRACE"),
        }
    }
}

/// Trait for host-provided context services.
///
/// This trait allows embedders to integrate their own logging, metrics,
/// and cancellation mechanisms with fusabi-host.
///
/// # Examples
///
/// ```rust
/// use fusabi_host::{HostContext, LogLevel};
///
/// struct MyContext {
///     request_id: String,
/// }
///
/// impl HostContext for MyContext {
///     fn log(&self, level: LogLevel, message: &str) {
///         println!("[{}] [{}] {}", self.request_id, level, message);
///     }
///
///     fn record_metric(&self, name: &str, value: f64, _tags: &[(&str, &str)]) {
///         println!("METRIC: {} = {}", name, value);
///     }
///
///     fn should_cancel(&self) -> bool {
///         false
///     }
/// }
/// ```
pub trait HostContext: Send + Sync {
    /// Log a message at the specified level.
    ///
    /// Embedders can integrate with their logging infrastructure (tracing,
    /// log, slog, etc.) by implementing this method.
    ///
    /// # Arguments
    ///
    /// * `level` - The log level
    /// * `message` - The message to log
    fn log(&self, level: LogLevel, message: &str);

    /// Record a metric value.
    ///
    /// Embedders can integrate with their metrics infrastructure (Prometheus,
    /// StatsD, OpenTelemetry, etc.) by implementing this method.
    ///
    /// # Arguments
    ///
    /// * `name` - The metric name
    /// * `value` - The metric value
    /// * `tags` - Key-value pairs for metric tags/labels
    fn record_metric(&self, name: &str, value: f64, tags: &[(&str, &str)]);

    /// Check if execution should be cancelled.
    ///
    /// This allows external cancellation of script execution, for example:
    /// - Request timeout in a web server
    /// - User cancellation in a UI
    /// - Shutdown signal in a daemon
    ///
    /// The engine will check this periodically during execution and stop
    /// if it returns `true`.
    fn should_cancel(&self) -> bool;

    /// Log at ERROR level (convenience method).
    fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }

    /// Log at WARN level (convenience method).
    fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message);
    }

    /// Log at INFO level (convenience method).
    fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }

    /// Log at DEBUG level (convenience method).
    fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }

    /// Log at TRACE level (convenience method).
    fn trace(&self, message: &str) {
        self.log(LogLevel::Trace, message);
    }

    /// Record a counter metric (convenience method).
    fn counter(&self, name: &str, value: u64, tags: &[(&str, &str)]) {
        self.record_metric(name, value as f64, tags);
    }

    /// Record a gauge metric (convenience method).
    fn gauge(&self, name: &str, value: f64, tags: &[(&str, &str)]) {
        self.record_metric(name, value, tags);
    }

    /// Record a histogram/timing metric (convenience method).
    fn histogram(&self, name: &str, value: f64, tags: &[(&str, &str)]) {
        self.record_metric(name, value, tags);
    }
}

/// Default host context that uses tracing and does not support cancellation.
///
/// This is a simple implementation suitable for development and testing.
#[derive(Debug, Clone, Default)]
pub struct DefaultHostContext;

impl HostContext for DefaultHostContext {
    fn log(&self, level: LogLevel, message: &str) {
        match level {
            LogLevel::Error => tracing::error!("{}", message),
            LogLevel::Warn => tracing::warn!("{}", message),
            LogLevel::Info => tracing::info!("{}", message),
            LogLevel::Debug => tracing::debug!("{}", message),
            LogLevel::Trace => tracing::trace!("{}", message),
        }
    }

    fn record_metric(&self, name: &str, value: f64, tags: &[(&str, &str)]) {
        let tags_str = tags
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(",");

        if tags_str.is_empty() {
            tracing::debug!(metric = name, value = value);
        } else {
            tracing::debug!(metric = name, value = value, tags = tags_str);
        }
    }

    fn should_cancel(&self) -> bool {
        false
    }
}

/// No-op host context for environments without logging/metrics.
///
/// This is useful for embedded systems or when you want to disable all
/// logging and metrics overhead.
#[derive(Debug, Clone, Default)]
pub struct NoopHostContext;

impl HostContext for NoopHostContext {
    fn log(&self, _level: LogLevel, _message: &str) {
        // No-op
    }

    fn record_metric(&self, _name: &str, _value: f64, _tags: &[(&str, &str)]) {
        // No-op
    }

    fn should_cancel(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestContext {
        logs: std::sync::Arc<std::sync::Mutex<Vec<(LogLevel, String)>>>,
        metrics: std::sync::Arc<std::sync::Mutex<Vec<(String, f64)>>>,
        cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    impl TestContext {
        fn new() -> Self {
            Self {
                logs: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
                metrics: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
                cancel_flag: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            }
        }

        fn get_logs(&self) -> Vec<(LogLevel, String)> {
            self.logs.lock().unwrap().clone()
        }

        fn get_metrics(&self) -> Vec<(String, f64)> {
            self.metrics.lock().unwrap().clone()
        }

        fn set_cancel(&self, cancel: bool) {
            self.cancel_flag
                .store(cancel, std::sync::atomic::Ordering::Relaxed);
        }
    }

    impl HostContext for TestContext {
        fn log(&self, level: LogLevel, message: &str) {
            self.logs
                .lock()
                .unwrap()
                .push((level, message.to_string()));
        }

        fn record_metric(&self, name: &str, value: f64, _tags: &[(&str, &str)]) {
            self.metrics
                .lock()
                .unwrap()
                .push((name.to_string(), value));
        }

        fn should_cancel(&self) -> bool {
            self.cancel_flag
                .load(std::sync::atomic::Ordering::Relaxed)
        }
    }

    #[test]
    fn test_log_levels() {
        let ctx = TestContext::new();

        ctx.error("error message");
        ctx.warn("warn message");
        ctx.info("info message");
        ctx.debug("debug message");
        ctx.trace("trace message");

        let logs = ctx.get_logs();
        assert_eq!(logs.len(), 5);
        assert_eq!(logs[0], (LogLevel::Error, "error message".to_string()));
        assert_eq!(logs[1], (LogLevel::Warn, "warn message".to_string()));
        assert_eq!(logs[2], (LogLevel::Info, "info message".to_string()));
        assert_eq!(logs[3], (LogLevel::Debug, "debug message".to_string()));
        assert_eq!(logs[4], (LogLevel::Trace, "trace message".to_string()));
    }

    #[test]
    fn test_metrics() {
        let ctx = TestContext::new();

        ctx.counter("requests", 42, &[]);
        ctx.gauge("memory_mb", 128.5, &[("region", "us-west")]);
        ctx.histogram("duration_ms", 250.0, &[]);

        let metrics = ctx.get_metrics();
        assert_eq!(metrics.len(), 3);
        assert_eq!(metrics[0], ("requests".to_string(), 42.0));
        assert_eq!(metrics[1], ("memory_mb".to_string(), 128.5));
        assert_eq!(metrics[2], ("duration_ms".to_string(), 250.0));
    }

    #[test]
    fn test_cancellation() {
        let ctx = TestContext::new();

        assert!(!ctx.should_cancel());

        ctx.set_cancel(true);
        assert!(ctx.should_cancel());

        ctx.set_cancel(false);
        assert!(!ctx.should_cancel());
    }

    #[test]
    fn test_default_context() {
        let ctx = DefaultHostContext;
        ctx.info("test message");
        ctx.record_metric("test", 1.0, &[]);
        assert!(!ctx.should_cancel());
    }

    #[test]
    fn test_noop_context() {
        let ctx = NoopHostContext;
        ctx.info("test message");
        ctx.record_metric("test", 1.0, &[]);
        assert!(!ctx.should_cancel());
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Error.to_string(), "ERROR");
        assert_eq!(LogLevel::Warn.to_string(), "WARN");
        assert_eq!(LogLevel::Info.to_string(), "INFO");
        assert_eq!(LogLevel::Debug.to_string(), "DEBUG");
        assert_eq!(LogLevel::Trace.to_string(), "TRACE");
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Error < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Trace);
    }
}

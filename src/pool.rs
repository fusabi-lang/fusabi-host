//! Engine pool for concurrent Fusabi execution.

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, Receiver, Sender, TryRecvError};
use parking_lot::Mutex;

use crate::capabilities::Capabilities;
use crate::engine::{Engine, EngineConfig};
use crate::error::{Error, Result};
use crate::limits::Limits;
use crate::sandbox::SandboxConfig;
use crate::value::Value;

/// Configuration for an engine pool.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Number of engines in the pool.
    pub size: usize,
    /// Engine configuration template.
    pub engine_config: EngineConfig,
    /// Maximum time to wait for an engine.
    pub acquire_timeout: Duration,
    /// Whether to create engines lazily.
    pub lazy_init: bool,
    /// Maximum idle time before an engine is recycled.
    pub max_idle_time: Option<Duration>,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            size: num_cpus::get().max(2),
            engine_config: EngineConfig::default(),
            acquire_timeout: Duration::from_secs(30),
            lazy_init: false,
            max_idle_time: Some(Duration::from_secs(300)),
        }
    }
}

impl PoolConfig {
    /// Create a new pool configuration with the specified size.
    pub fn new(size: usize) -> Self {
        Self {
            size: size.max(1),
            ..Default::default()
        }
    }

    /// Set the engine configuration.
    pub fn with_engine_config(mut self, config: EngineConfig) -> Self {
        self.engine_config = config;
        self
    }

    /// Set resource limits for all engines.
    pub fn with_limits(mut self, limits: Limits) -> Self {
        self.engine_config.limits = limits;
        self
    }

    /// Set capabilities for all engines.
    pub fn with_capabilities(mut self, capabilities: Capabilities) -> Self {
        self.engine_config.capabilities = capabilities;
        self
    }

    /// Set sandbox configuration for all engines.
    pub fn with_sandbox(mut self, sandbox: SandboxConfig) -> Self {
        self.engine_config.sandbox = sandbox;
        self
    }

    /// Set the acquire timeout.
    pub fn with_acquire_timeout(mut self, timeout: Duration) -> Self {
        self.acquire_timeout = timeout;
        self
    }

    /// Enable lazy initialization.
    pub fn with_lazy_init(mut self, lazy: bool) -> Self {
        self.lazy_init = lazy;
        self
    }

    /// Set the maximum idle time.
    pub fn with_max_idle_time(mut self, time: Option<Duration>) -> Self {
        self.max_idle_time = time;
        self
    }
}

/// Statistics about pool usage.
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Total number of engines.
    pub total: usize,
    /// Number of available engines.
    pub available: usize,
    /// Number of engines currently in use.
    pub in_use: usize,
    /// Total number of acquisitions.
    pub acquisitions: u64,
    /// Total number of releases.
    pub releases: u64,
    /// Total number of timeouts.
    pub timeouts: u64,
    /// Total execution count.
    pub executions: u64,
    /// Total execution time.
    pub total_execution_time: Duration,
}

impl PoolStats {
    /// Calculate average execution time.
    pub fn avg_execution_time(&self) -> Duration {
        if self.executions == 0 {
            Duration::ZERO
        } else {
            self.total_execution_time / self.executions as u32
        }
    }
}

/// Internal wrapper for pooled engines.
struct PooledEngine {
    engine: Engine,
    created_at: Instant,
    last_used: Instant,
    use_count: u64,
}

impl PooledEngine {
    fn new(engine: Engine) -> Self {
        let now = Instant::now();
        Self {
            engine,
            created_at: now,
            last_used: now,
            use_count: 0,
        }
    }

    fn mark_used(&mut self) {
        self.last_used = Instant::now();
        self.use_count += 1;
    }

    fn idle_time(&self) -> Duration {
        self.last_used.elapsed()
    }
}

/// A handle to a pooled engine.
///
/// When dropped, the engine is returned to the pool.
pub struct PoolHandle {
    engine: Option<PooledEngine>,
    return_tx: Sender<PooledEngine>,
    stats: Arc<PoolStatsInner>,
    start_time: Instant,
}

impl PoolHandle {
    /// Execute source code with the pooled engine.
    pub fn execute(&self, source: &str) -> Result<Value> {
        let engine = self.engine.as_ref().ok_or(Error::Internal(
            "pool handle has no engine".into(),
        ))?;
        engine.engine.execute(source)
    }

    /// Execute bytecode with the pooled engine.
    pub fn execute_bytecode(&self, bytecode: &[u8]) -> Result<Value> {
        let engine = self.engine.as_ref().ok_or(Error::Internal(
            "pool handle has no engine".into(),
        ))?;
        engine.engine.execute_bytecode(bytecode)
    }

    /// Get a reference to the underlying engine.
    pub fn engine(&self) -> &Engine {
        &self.engine.as_ref().unwrap().engine
    }

    /// Cancel the current execution.
    pub fn cancel(&self) {
        if let Some(ref e) = self.engine {
            e.engine.cancel();
        }
    }
}

impl Drop for PoolHandle {
    fn drop(&mut self) {
        if let Some(mut engine) = self.engine.take() {
            // Update stats
            let elapsed = self.start_time.elapsed();
            self.stats.releases.fetch_add(1, Ordering::Relaxed);
            self.stats.add_execution_time(elapsed);

            engine.mark_used();

            // Return engine to pool
            let _ = self.return_tx.try_send(engine);
        }
    }
}

/// Internal stats tracking.
struct PoolStatsInner {
    acquisitions: AtomicU64,
    releases: AtomicU64,
    timeouts: AtomicU64,
    executions: AtomicU64,
    execution_time_nanos: AtomicU64,
}

impl PoolStatsInner {
    fn new() -> Self {
        Self {
            acquisitions: AtomicU64::new(0),
            releases: AtomicU64::new(0),
            timeouts: AtomicU64::new(0),
            executions: AtomicU64::new(0),
            execution_time_nanos: AtomicU64::new(0),
        }
    }

    fn add_execution_time(&self, duration: Duration) {
        self.executions.fetch_add(1, Ordering::Relaxed);
        self.execution_time_nanos
            .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }
}

/// A pool of Fusabi engines for concurrent execution.
///
/// The pool manages a fixed number of engines and provides thread-safe
/// access to them for parallel script execution.
pub struct EnginePool {
    config: PoolConfig,
    engine_rx: Receiver<PooledEngine>,
    engine_tx: Sender<PooledEngine>,
    stats: Arc<PoolStatsInner>,
    shutdown: AtomicBool,
    created: AtomicUsize,
}

impl EnginePool {
    /// Create a new engine pool with the given configuration.
    pub fn new(config: PoolConfig) -> Result<Self> {
        let (tx, rx) = bounded(config.size);

        let pool = Self {
            config: config.clone(),
            engine_rx: rx,
            engine_tx: tx.clone(),
            stats: Arc::new(PoolStatsInner::new()),
            shutdown: AtomicBool::new(false),
            created: AtomicUsize::new(0),
        };

        // Pre-create engines if not lazy
        if !config.lazy_init {
            for _ in 0..config.size {
                let engine = Engine::new(config.engine_config.clone())?;
                tx.send(PooledEngine::new(engine))
                    .map_err(|_| Error::Internal("failed to initialize pool".into()))?;
                pool.created.fetch_add(1, Ordering::Relaxed);
            }
        }

        Ok(pool)
    }

    /// Acquire an engine from the pool.
    ///
    /// Blocks until an engine is available or the timeout expires.
    pub fn acquire(&self) -> Result<PoolHandle> {
        if self.shutdown.load(Ordering::Relaxed) {
            return Err(Error::PoolShutdown);
        }

        self.stats.acquisitions.fetch_add(1, Ordering::Relaxed);

        // Try to get an existing engine
        match self.engine_rx.recv_timeout(self.config.acquire_timeout) {
            Ok(engine) => Ok(PoolHandle {
                engine: Some(engine),
                return_tx: self.engine_tx.clone(),
                stats: self.stats.clone(),
                start_time: Instant::now(),
            }),
            Err(_) => {
                // Try lazy creation if we haven't reached capacity
                if self.config.lazy_init {
                    let created = self.created.load(Ordering::Relaxed);
                    if created < self.config.size {
                        if self
                            .created
                            .compare_exchange(
                                created,
                                created + 1,
                                Ordering::SeqCst,
                                Ordering::Relaxed,
                            )
                            .is_ok()
                        {
                            let engine = Engine::new(self.config.engine_config.clone())?;
                            return Ok(PoolHandle {
                                engine: Some(PooledEngine::new(engine)),
                                return_tx: self.engine_tx.clone(),
                                stats: self.stats.clone(),
                                start_time: Instant::now(),
                            });
                        }
                    }
                }

                self.stats.timeouts.fetch_add(1, Ordering::Relaxed);
                Err(Error::PoolTimeout)
            }
        }
    }

    /// Try to acquire an engine without blocking.
    pub fn try_acquire(&self) -> Result<PoolHandle> {
        if self.shutdown.load(Ordering::Relaxed) {
            return Err(Error::PoolShutdown);
        }

        self.stats.acquisitions.fetch_add(1, Ordering::Relaxed);

        match self.engine_rx.try_recv() {
            Ok(engine) => Ok(PoolHandle {
                engine: Some(engine),
                return_tx: self.engine_tx.clone(),
                stats: self.stats.clone(),
                start_time: Instant::now(),
            }),
            Err(TryRecvError::Empty) => {
                // Try lazy creation
                if self.config.lazy_init {
                    let created = self.created.load(Ordering::Relaxed);
                    if created < self.config.size {
                        if self
                            .created
                            .compare_exchange(
                                created,
                                created + 1,
                                Ordering::SeqCst,
                                Ordering::Relaxed,
                            )
                            .is_ok()
                        {
                            let engine = Engine::new(self.config.engine_config.clone())?;
                            return Ok(PoolHandle {
                                engine: Some(PooledEngine::new(engine)),
                                return_tx: self.engine_tx.clone(),
                                stats: self.stats.clone(),
                                start_time: Instant::now(),
                            });
                        }
                    }
                }
                Err(Error::PoolExhausted {
                    count: self.config.size,
                })
            }
            Err(TryRecvError::Disconnected) => Err(Error::PoolShutdown),
        }
    }

    /// Execute source code using a pooled engine.
    ///
    /// Convenience method that acquires an engine, executes, and returns it.
    pub fn execute(&self, source: &str) -> Result<Value> {
        let handle = self.acquire()?;
        handle.execute(source)
    }

    /// Execute bytecode using a pooled engine.
    pub fn execute_bytecode(&self, bytecode: &[u8]) -> Result<Value> {
        let handle = self.acquire()?;
        handle.execute_bytecode(bytecode)
    }

    /// Get current pool statistics.
    pub fn stats(&self) -> PoolStats {
        let available = self.engine_rx.len();
        let created = self.created.load(Ordering::Relaxed);
        let in_use = created.saturating_sub(available);

        let execution_nanos = self.stats.execution_time_nanos.load(Ordering::Relaxed);

        PoolStats {
            total: self.config.size,
            available,
            in_use,
            acquisitions: self.stats.acquisitions.load(Ordering::Relaxed),
            releases: self.stats.releases.load(Ordering::Relaxed),
            timeouts: self.stats.timeouts.load(Ordering::Relaxed),
            executions: self.stats.executions.load(Ordering::Relaxed),
            total_execution_time: Duration::from_nanos(execution_nanos),
        }
    }

    /// Get the pool configuration.
    pub fn config(&self) -> &PoolConfig {
        &self.config
    }

    /// Check if the pool is healthy.
    pub fn is_healthy(&self) -> bool {
        !self.shutdown.load(Ordering::Relaxed) && self.engine_rx.len() > 0
    }

    /// Shut down the pool, preventing new acquisitions.
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }

    /// Check if the pool has been shut down.
    pub fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Relaxed)
    }
}

impl std::fmt::Debug for EnginePool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stats = self.stats();
        f.debug_struct("EnginePool")
            .field("size", &self.config.size)
            .field("available", &stats.available)
            .field("in_use", &stats.in_use)
            .field("shutdown", &self.is_shutdown())
            .finish()
    }
}

// Async support when tokio is enabled
#[cfg(feature = "async-runtime-tokio")]
mod async_support {
    use super::*;
    use tokio::sync::Semaphore;
    use std::sync::Arc;

    /// Async wrapper for the engine pool.
    pub struct AsyncEnginePool {
        inner: Arc<EnginePool>,
        semaphore: Arc<Semaphore>,
    }

    impl AsyncEnginePool {
        /// Create a new async pool wrapper.
        pub fn new(pool: EnginePool) -> Self {
            let permits = pool.config.size;
            Self {
                inner: Arc::new(pool),
                semaphore: Arc::new(Semaphore::new(permits)),
            }
        }

        /// Acquire an engine asynchronously.
        pub async fn acquire(&self) -> Result<PoolHandle> {
            let _permit = self
                .semaphore
                .acquire()
                .await
                .map_err(|_| Error::PoolShutdown)?;

            self.inner.try_acquire()
        }

        /// Execute source code asynchronously.
        pub async fn execute(&self, source: &str) -> Result<Value> {
            let handle = self.acquire().await?;

            // Run execution in blocking task to avoid blocking the runtime
            let source = source.to_string();
            tokio::task::spawn_blocking(move || handle.execute(&source))
                .await
                .map_err(|e| Error::Internal(e.to_string()))?
        }

        /// Get pool statistics.
        pub fn stats(&self) -> PoolStats {
            self.inner.stats()
        }

        /// Shutdown the pool.
        pub fn shutdown(&self) {
            self.inner.shutdown();
        }
    }
}

#[cfg(feature = "async-runtime-tokio")]
pub use async_support::AsyncEnginePool;

#[cfg(test)]
mod tests {
    use super::*;

    fn num_cpus_get() -> usize {
        4 // Mock for testing
    }

    #[test]
    fn test_pool_creation() {
        let pool = EnginePool::new(PoolConfig::new(4)).unwrap();
        assert_eq!(pool.config().size, 4);

        let stats = pool.stats();
        assert_eq!(stats.total, 4);
        assert_eq!(stats.available, 4);
        assert_eq!(stats.in_use, 0);
    }

    #[test]
    fn test_pool_acquire_release() {
        let pool = EnginePool::new(PoolConfig::new(2)).unwrap();

        let handle1 = pool.acquire().unwrap();
        assert_eq!(pool.stats().in_use, 1);

        let handle2 = pool.acquire().unwrap();
        assert_eq!(pool.stats().in_use, 2);

        drop(handle1);
        assert_eq!(pool.stats().in_use, 1);

        drop(handle2);
        assert_eq!(pool.stats().in_use, 0);
    }

    #[test]
    fn test_pool_execute() {
        let pool = EnginePool::new(PoolConfig::new(2)).unwrap();

        let result = pool.execute("42").unwrap();
        assert_eq!(result, Value::Int(42));

        let result = pool.execute("1 + 2").unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_pool_exhausted() {
        let config = PoolConfig::new(1).with_acquire_timeout(Duration::from_millis(10));
        let pool = EnginePool::new(config).unwrap();

        let _handle = pool.acquire().unwrap();

        // Second acquire should timeout
        let result = pool.acquire();
        assert!(matches!(result, Err(Error::PoolTimeout)));
    }

    #[test]
    fn test_pool_try_acquire() {
        let pool = EnginePool::new(PoolConfig::new(1)).unwrap();

        let handle = pool.try_acquire().unwrap();

        let result = pool.try_acquire();
        assert!(matches!(result, Err(Error::PoolExhausted { .. })));

        drop(handle);

        let _handle2 = pool.try_acquire().unwrap();
    }

    #[test]
    fn test_pool_lazy_init() {
        let config = PoolConfig::new(4).with_lazy_init(true);
        let pool = EnginePool::new(config).unwrap();

        // No engines created yet
        assert_eq!(pool.created.load(Ordering::Relaxed), 0);

        // Acquire creates one
        let _handle = pool.try_acquire().unwrap();
        assert_eq!(pool.created.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_pool_shutdown() {
        let pool = EnginePool::new(PoolConfig::new(2)).unwrap();

        assert!(!pool.is_shutdown());
        pool.shutdown();
        assert!(pool.is_shutdown());

        let result = pool.acquire();
        assert!(matches!(result, Err(Error::PoolShutdown)));
    }

    #[test]
    fn test_pool_stats() {
        let pool = EnginePool::new(PoolConfig::new(2)).unwrap();

        let handle = pool.acquire().unwrap();
        let _ = handle.execute("42");
        drop(handle);

        let stats = pool.stats();
        assert_eq!(stats.acquisitions, 1);
        assert_eq!(stats.releases, 1);
        assert_eq!(stats.executions, 1);
        assert!(stats.total_execution_time > Duration::ZERO);
    }

    #[test]
    fn test_pool_config_builder() {
        let config = PoolConfig::new(8)
            .with_limits(Limits::strict())
            .with_capabilities(Capabilities::none())
            .with_acquire_timeout(Duration::from_secs(5))
            .with_lazy_init(true);

        assert_eq!(config.size, 8);
        assert_eq!(config.acquire_timeout, Duration::from_secs(5));
        assert!(config.lazy_init);
    }

    #[test]
    #[ignore]
    fn test_handle_cancel() {
        let pool = EnginePool::new(PoolConfig::new(1)).unwrap();
        let handle = pool.acquire().unwrap();

        handle.cancel();
        let result = handle.execute("42");
        assert!(matches!(result, Err(Error::Cancelled)));
    }
}

// Mock num_cpus for the default
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
}

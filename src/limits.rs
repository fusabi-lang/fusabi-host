//! Resource limits for script execution.

use std::time::Duration;
use thiserror::Error;

/// A violation of resource limits.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum LimitViolation {
    /// Execution time exceeded.
    #[error("execution time limit exceeded: {limit:?} (actual: {actual:?})")]
    TimeExceeded {
        /// The configured limit.
        limit: Duration,
        /// The actual time taken.
        actual: Duration,
    },

    /// Memory limit exceeded.
    #[error("memory limit exceeded: {limit} bytes (actual: {actual} bytes)")]
    MemoryExceeded {
        /// The configured limit in bytes.
        limit: usize,
        /// The actual memory used in bytes.
        actual: usize,
    },

    /// CPU instruction limit exceeded.
    #[error("instruction limit exceeded: {limit} (actual: {actual})")]
    InstructionsExceeded {
        /// The configured limit.
        limit: u64,
        /// The actual instruction count.
        actual: u64,
    },

    /// Call stack depth exceeded.
    #[error("stack depth limit exceeded: {limit} (actual: {actual})")]
    StackDepthExceeded {
        /// The configured limit.
        limit: usize,
        /// The actual depth.
        actual: usize,
    },

    /// Output size limit exceeded.
    #[error("output size limit exceeded: {limit} bytes (actual: {actual} bytes)")]
    OutputSizeExceeded {
        /// The configured limit in bytes.
        limit: usize,
        /// The actual size in bytes.
        actual: usize,
    },

    /// File system operation limit exceeded.
    #[error("filesystem operation limit exceeded: {limit} operations")]
    FsOpsExceeded {
        /// The configured limit.
        limit: usize,
    },

    /// Network operation limit exceeded.
    #[error("network operation limit exceeded: {limit} operations")]
    NetOpsExceeded {
        /// The configured limit.
        limit: usize,
    },
}

/// Resource limits for script execution.
///
/// These limits control how much resources a script can consume.
/// All limits are optional - `None` means unlimited.
#[derive(Debug, Clone, PartialEq)]
pub struct Limits {
    /// Maximum execution time.
    pub timeout: Option<Duration>,

    /// Maximum memory usage in bytes.
    pub memory_bytes: Option<usize>,

    /// Maximum number of VM instructions.
    pub max_instructions: Option<u64>,

    /// Maximum call stack depth.
    pub max_stack_depth: Option<usize>,

    /// Maximum output size in bytes.
    pub max_output_bytes: Option<usize>,

    /// Maximum filesystem operations.
    pub max_fs_ops: Option<usize>,

    /// Maximum network operations.
    pub max_net_ops: Option<usize>,

    /// Maximum concurrent tasks/coroutines.
    pub max_concurrent_tasks: Option<usize>,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            timeout: Some(Duration::from_secs(30)),
            memory_bytes: Some(64 * 1024 * 1024), // 64 MB
            max_instructions: Some(10_000_000),    // 10M instructions
            max_stack_depth: Some(1000),
            max_output_bytes: Some(1024 * 1024), // 1 MB
            max_fs_ops: Some(100),
            max_net_ops: Some(10),
            max_concurrent_tasks: Some(16),
        }
    }
}

impl Limits {
    /// Create limits with no restrictions (unlimited).
    pub fn unlimited() -> Self {
        Self {
            timeout: None,
            memory_bytes: None,
            max_instructions: None,
            max_stack_depth: None,
            max_output_bytes: None,
            max_fs_ops: None,
            max_net_ops: None,
            max_concurrent_tasks: None,
        }
    }

    /// Create strict limits suitable for untrusted code.
    pub fn strict() -> Self {
        Self {
            timeout: Some(Duration::from_secs(5)),
            memory_bytes: Some(16 * 1024 * 1024), // 16 MB
            max_instructions: Some(1_000_000),     // 1M instructions
            max_stack_depth: Some(100),
            max_output_bytes: Some(64 * 1024), // 64 KB
            max_fs_ops: Some(0),                // No FS access
            max_net_ops: Some(0),               // No network access
            max_concurrent_tasks: Some(4),
        }
    }

    /// Set the timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the memory limit in bytes.
    pub fn with_memory_bytes(mut self, bytes: usize) -> Self {
        self.memory_bytes = Some(bytes);
        self
    }

    /// Set the memory limit in megabytes.
    pub fn with_memory_mb(mut self, mb: usize) -> Self {
        self.memory_bytes = Some(mb * 1024 * 1024);
        self
    }

    /// Set the instruction limit.
    pub fn with_max_instructions(mut self, count: u64) -> Self {
        self.max_instructions = Some(count);
        self
    }

    /// Set the stack depth limit.
    pub fn with_max_stack_depth(mut self, depth: usize) -> Self {
        self.max_stack_depth = Some(depth);
        self
    }

    /// Set the output size limit in bytes.
    pub fn with_max_output_bytes(mut self, bytes: usize) -> Self {
        self.max_output_bytes = Some(bytes);
        self
    }

    /// Set the filesystem operations limit.
    pub fn with_max_fs_ops(mut self, ops: usize) -> Self {
        self.max_fs_ops = Some(ops);
        self
    }

    /// Set the network operations limit.
    pub fn with_max_net_ops(mut self, ops: usize) -> Self {
        self.max_net_ops = Some(ops);
        self
    }

    /// Set the concurrent tasks limit.
    pub fn with_max_concurrent_tasks(mut self, tasks: usize) -> Self {
        self.max_concurrent_tasks = Some(tasks);
        self
    }

    /// Remove the timeout limit.
    pub fn no_timeout(mut self) -> Self {
        self.timeout = None;
        self
    }

    /// Check if time limit is exceeded.
    pub fn check_time(&self, elapsed: Duration) -> Result<(), LimitViolation> {
        if let Some(limit) = self.timeout {
            if elapsed > limit {
                return Err(LimitViolation::TimeExceeded {
                    limit,
                    actual: elapsed,
                });
            }
        }
        Ok(())
    }

    /// Check if memory limit is exceeded.
    pub fn check_memory(&self, used: usize) -> Result<(), LimitViolation> {
        if let Some(limit) = self.memory_bytes {
            if used > limit {
                return Err(LimitViolation::MemoryExceeded {
                    limit,
                    actual: used,
                });
            }
        }
        Ok(())
    }

    /// Check if instruction limit is exceeded.
    pub fn check_instructions(&self, count: u64) -> Result<(), LimitViolation> {
        if let Some(limit) = self.max_instructions {
            if count > limit {
                return Err(LimitViolation::InstructionsExceeded {
                    limit,
                    actual: count,
                });
            }
        }
        Ok(())
    }

    /// Check if stack depth limit is exceeded.
    pub fn check_stack_depth(&self, depth: usize) -> Result<(), LimitViolation> {
        if let Some(limit) = self.max_stack_depth {
            if depth > limit {
                return Err(LimitViolation::StackDepthExceeded {
                    limit,
                    actual: depth,
                });
            }
        }
        Ok(())
    }
}

/// Runtime limit tracker for monitoring during execution.
#[derive(Debug, Clone)]
pub struct LimitTracker {
    limits: Limits,
    start_time: std::time::Instant,
    instructions_executed: u64,
    memory_used: usize,
    current_stack_depth: usize,
    output_bytes: usize,
    fs_ops: usize,
    net_ops: usize,
}

impl LimitTracker {
    /// Create a new tracker with the given limits.
    pub fn new(limits: Limits) -> Self {
        Self {
            limits,
            start_time: std::time::Instant::now(),
            instructions_executed: 0,
            memory_used: 0,
            current_stack_depth: 0,
            output_bytes: 0,
            fs_ops: 0,
            net_ops: 0,
        }
    }

    /// Reset the tracker for a new execution.
    pub fn reset(&mut self) {
        self.start_time = std::time::Instant::now();
        self.instructions_executed = 0;
        self.memory_used = 0;
        self.current_stack_depth = 0;
        self.output_bytes = 0;
        self.fs_ops = 0;
        self.net_ops = 0;
    }

    /// Check timeout limit.
    pub fn check_timeout(&self) -> Result<(), LimitViolation> {
        self.limits.check_time(self.start_time.elapsed())
    }

    /// Record instruction execution and check limit.
    pub fn record_instructions(&mut self, count: u64) -> Result<(), LimitViolation> {
        self.instructions_executed += count;
        self.limits.check_instructions(self.instructions_executed)
    }

    /// Record memory allocation and check limit.
    pub fn record_memory(&mut self, bytes: usize) -> Result<(), LimitViolation> {
        self.memory_used = bytes;
        self.limits.check_memory(self.memory_used)
    }

    /// Record stack push and check limit.
    pub fn push_stack(&mut self) -> Result<(), LimitViolation> {
        self.current_stack_depth += 1;
        self.limits.check_stack_depth(self.current_stack_depth)
    }

    /// Record stack pop.
    pub fn pop_stack(&mut self) {
        self.current_stack_depth = self.current_stack_depth.saturating_sub(1);
    }

    /// Record output bytes and check limit.
    pub fn record_output(&mut self, bytes: usize) -> Result<(), LimitViolation> {
        self.output_bytes += bytes;
        if let Some(limit) = self.limits.max_output_bytes {
            if self.output_bytes > limit {
                return Err(LimitViolation::OutputSizeExceeded {
                    limit,
                    actual: self.output_bytes,
                });
            }
        }
        Ok(())
    }

    /// Record filesystem operation and check limit.
    pub fn record_fs_op(&mut self) -> Result<(), LimitViolation> {
        self.fs_ops += 1;
        if let Some(limit) = self.limits.max_fs_ops {
            if self.fs_ops > limit {
                return Err(LimitViolation::FsOpsExceeded { limit });
            }
        }
        Ok(())
    }

    /// Record network operation and check limit.
    pub fn record_net_op(&mut self) -> Result<(), LimitViolation> {
        self.net_ops += 1;
        if let Some(limit) = self.limits.max_net_ops {
            if self.net_ops > limit {
                return Err(LimitViolation::NetOpsExceeded { limit });
            }
        }
        Ok(())
    }

    /// Get elapsed time since start.
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get current memory usage.
    pub fn memory_used(&self) -> usize {
        self.memory_used
    }

    /// Get instruction count.
    pub fn instructions_executed(&self) -> u64 {
        self.instructions_executed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_limits() {
        let limits = Limits::default();
        assert!(limits.timeout.is_some());
        assert!(limits.memory_bytes.is_some());
        assert!(limits.max_instructions.is_some());
    }

    #[test]
    fn test_unlimited() {
        let limits = Limits::unlimited();
        assert!(limits.timeout.is_none());
        assert!(limits.memory_bytes.is_none());
    }

    #[test]
    fn test_strict_limits() {
        let limits = Limits::strict();
        assert_eq!(limits.max_fs_ops, Some(0));
        assert_eq!(limits.max_net_ops, Some(0));
    }

    #[test]
    fn test_builder_pattern() {
        let limits = Limits::default()
            .with_timeout(Duration::from_secs(10))
            .with_memory_mb(32);

        assert_eq!(limits.timeout, Some(Duration::from_secs(10)));
        assert_eq!(limits.memory_bytes, Some(32 * 1024 * 1024));
    }

    #[test]
    fn test_limit_checks() {
        let limits = Limits::default().with_memory_bytes(1000);

        assert!(limits.check_memory(500).is_ok());
        assert!(limits.check_memory(1500).is_err());
    }

    #[test]
    fn test_limit_tracker() {
        let limits = Limits::default().with_max_instructions(100);
        let mut tracker = LimitTracker::new(limits);

        assert!(tracker.record_instructions(50).is_ok());
        assert!(tracker.record_instructions(60).is_err());
    }
}

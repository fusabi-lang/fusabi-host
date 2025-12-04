//! Error types for fusabi-host operations.

use std::fmt;
use thiserror::Error;

use crate::limits::LimitViolation;
use crate::convert::ValueConversionError;

/// Result type alias using [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during Fusabi host operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Compilation failed with the given message.
    #[error("compilation error: {0}")]
    Compilation(String),

    /// Runtime execution failed.
    #[error("runtime error: {0}")]
    Runtime(String),

    /// A resource limit was violated.
    #[error("limit violation: {0}")]
    LimitViolation(#[from] LimitViolation),

    /// Value conversion failed.
    #[error("value conversion error: {0}")]
    ValueConversion(#[from] ValueConversionError),

    /// Capability was denied.
    #[error("capability denied: {capability}")]
    CapabilityDenied {
        /// The capability that was denied.
        capability: String,
    },

    /// Sandbox policy violation.
    #[error("sandbox violation: {0}")]
    SandboxViolation(String),

    /// Engine pool exhausted.
    #[error("engine pool exhausted, all {count} engines busy")]
    PoolExhausted {
        /// Number of engines in the pool.
        count: usize,
    },

    /// Pool acquire timeout.
    #[error("timeout waiting for engine from pool")]
    PoolTimeout,

    /// Pool was shut down.
    #[error("engine pool has been shut down")]
    PoolShutdown,

    /// Engine was poisoned (panicked during execution).
    #[error("engine poisoned: {0}")]
    EnginePoisoned(String),

    /// IO error occurred.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid configuration.
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    /// Version incompatibility.
    #[error("version incompatibility: expected {expected}, got {actual}")]
    VersionMismatch {
        /// Expected version range.
        expected: String,
        /// Actual version.
        actual: String,
    },

    /// Host function registration error.
    #[error("host function error: {0}")]
    HostFunction(String),

    /// Bytecode validation failed.
    #[error("invalid bytecode: {0}")]
    InvalidBytecode(String),

    /// Timeout during execution.
    #[error("execution timeout after {0:?}")]
    Timeout(std::time::Duration),

    /// Cancelled by user.
    #[error("execution cancelled")]
    Cancelled,

    /// Internal error (should not happen).
    #[error("internal error: {0}")]
    Internal(String),
}

impl Error {
    /// Create a compilation error.
    pub fn compilation(msg: impl Into<String>) -> Self {
        Self::Compilation(msg.into())
    }

    /// Create a runtime error.
    pub fn runtime(msg: impl Into<String>) -> Self {
        Self::Runtime(msg.into())
    }

    /// Create a capability denied error.
    pub fn capability_denied(capability: impl Into<String>) -> Self {
        Self::CapabilityDenied {
            capability: capability.into(),
        }
    }

    /// Create a sandbox violation error.
    pub fn sandbox_violation(msg: impl Into<String>) -> Self {
        Self::SandboxViolation(msg.into())
    }

    /// Create an invalid config error.
    pub fn invalid_config(msg: impl Into<String>) -> Self {
        Self::InvalidConfig(msg.into())
    }

    /// Create a version mismatch error.
    pub fn version_mismatch(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::VersionMismatch {
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// Create a host function error.
    pub fn host_function(msg: impl Into<String>) -> Self {
        Self::HostFunction(msg.into())
    }

    /// Create an invalid bytecode error.
    pub fn invalid_bytecode(msg: impl Into<String>) -> Self {
        Self::InvalidBytecode(msg.into())
    }

    /// Returns true if this is a transient error that may succeed on retry.
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            Self::PoolExhausted { .. } | Self::PoolTimeout | Self::Timeout(_)
        )
    }

    /// Returns true if this error indicates the engine is unusable.
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            Self::EnginePoisoned(_) | Self::PoolShutdown | Self::Internal(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::compilation("syntax error at line 5");
        assert_eq!(err.to_string(), "compilation error: syntax error at line 5");

        let err = Error::PoolExhausted { count: 4 };
        assert_eq!(err.to_string(), "engine pool exhausted, all 4 engines busy");
    }

    #[test]
    fn test_error_classification() {
        assert!(Error::PoolTimeout.is_transient());
        assert!(Error::PoolExhausted { count: 4 }.is_transient());
        assert!(!Error::Compilation("test".into()).is_transient());

        assert!(Error::EnginePoisoned("panic".into()).is_fatal());
        assert!(Error::PoolShutdown.is_fatal());
        assert!(!Error::PoolTimeout.is_fatal());
    }
}

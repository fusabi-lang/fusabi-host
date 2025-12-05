//! # fusabi-host
//!
//! Shared host/runtime utilities for Fusabi across Scarab, Tolaria, Hibana, and Phage.
//!
//! This crate provides:
//! - **Engine pools** for concurrent Fusabi execution with thread-safe access
//! - **Value conversion helpers** for seamless Value↔Serde transformations
//! - **Typed host function macros** for ergonomic host function registration
//! - **Sandbox and capability configuration** for secure script execution
//! - **Stable compile/run APIs** for consistent host integration
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use fusabi_host::{EnginePool, PoolConfig, Capabilities, Limits};
//!
//! // Create a pool with 4 engines
//! let config = PoolConfig::new(4)
//!     .with_limits(Limits::default())
//!     .with_capabilities(Capabilities::none());
//!
//! let pool = EnginePool::new(config)?;
//!
//! // Execute a script
//! let result = pool.execute("1 + 2")?;
//! ```
//!
//! ## Feature Flags
//!
//! - `serde-support` (default): Enable Value↔Serde conversion helpers
//! - `async-runtime-tokio`: Async execution support via Tokio
//! - `async-runtime-async-std`: Async execution support via async-std
//! - `metrics-prometheus`: Prometheus metrics integration

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod capabilities;
mod compile;
mod convert;
mod engine;
mod error;
mod limits;
pub mod macros;
mod pool;
mod sandbox;
mod value;

pub use capabilities::{Capabilities, Capability};
pub use compile::{
    compile_source, compile_file, validate_bytecode, extract_bytecode_metadata, CompileOptions,
    CompileResult, Metadata,
};
pub use convert::{FromValue, IntoValue, ValueConversionError};

#[cfg(feature = "serde-support")]
pub use convert::{from_value_serde, to_value_serde};
pub use engine::{Engine, EngineConfig, ExecutionContext, HostRegistry};
pub use error::{Error, Result};
pub use limits::{Limits, LimitViolation};
pub use pool::{EnginePool, PoolConfig, PoolHandle, PoolStats};
pub use sandbox::{Sandbox, SandboxConfig, PathPolicy, NetPolicy};
pub use value::{Value, ValueType};

/// Crate version for compatibility checks
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Minimum supported Fusabi version (LTS alignment)
pub const MIN_FUSABI_VERSION: &str = "0.18.0";

/// Maximum supported Fusabi version
pub const MAX_FUSABI_VERSION: &str = "0.19.0";

/// Check if a Fusabi version is compatible with this host runtime
pub fn is_compatible_version(version: &str) -> bool {
    // Simple semver check - in production would use semver crate
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 2 {
        return false;
    }

    let Ok(major) = parts[0].parse::<u32>() else {
        return false;
    };
    let Ok(minor) = parts[1].parse::<u32>() else {
        return false;
    };

    // Compatible with 0.18.x and 0.19.x
    major == 0 && (minor == 18 || minor == 19)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compatibility() {
        assert!(is_compatible_version("0.18.0"));
        assert!(is_compatible_version("0.18.5"));
        assert!(is_compatible_version("0.19.0"));
        assert!(!is_compatible_version("0.17.0"));
        assert!(!is_compatible_version("0.20.0"));
        assert!(!is_compatible_version("1.0.0"));
        assert!(!is_compatible_version("invalid"));
    }
}

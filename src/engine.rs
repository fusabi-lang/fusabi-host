//! Fusabi engine wrapper with configuration and execution context.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;

use crate::capabilities::Capabilities;
use crate::error::{Error, Result};
use crate::limits::{LimitTracker, Limits};
use crate::sandbox::{Sandbox, SandboxConfig};
use crate::value::Value;

/// Configuration for creating an Engine.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Resource limits.
    pub limits: Limits,
    /// Capabilities granted to scripts.
    pub capabilities: Capabilities,
    /// Sandbox configuration.
    pub sandbox: SandboxConfig,
    /// Whether to enable debug mode.
    pub debug: bool,
    /// Custom metadata to attach to the engine.
    pub metadata: HashMap<String, String>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            limits: Limits::default(),
            capabilities: Capabilities::safe_defaults(),
            sandbox: SandboxConfig::default(),
            debug: false,
            metadata: HashMap::new(),
        }
    }
}

impl EngineConfig {
    /// Create a new engine configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set resource limits.
    pub fn with_limits(mut self, limits: Limits) -> Self {
        self.limits = limits;
        self
    }

    /// Set capabilities.
    pub fn with_capabilities(mut self, capabilities: Capabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Set sandbox configuration.
    pub fn with_sandbox(mut self, sandbox: SandboxConfig) -> Self {
        self.sandbox = sandbox;
        self
    }

    /// Enable debug mode.
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Create a strict configuration for untrusted code.
    pub fn strict() -> Self {
        Self {
            limits: Limits::strict(),
            capabilities: Capabilities::none(),
            sandbox: SandboxConfig::locked(),
            debug: false,
            metadata: HashMap::new(),
        }
    }

    /// Create a permissive configuration for trusted code.
    pub fn permissive() -> Self {
        Self {
            limits: Limits::unlimited(),
            capabilities: Capabilities::all(),
            sandbox: SandboxConfig::permissive(),
            debug: false,
            metadata: HashMap::new(),
        }
    }
}

/// Host function signature.
pub type HostFn = Arc<dyn Fn(&[Value], &ExecutionContext) -> Result<Value> + Send + Sync>;

/// Host function registry.
#[derive(Default, Clone)]
pub struct HostRegistry {
    functions: HashMap<String, HostFn>,
    modules: HashMap<String, HashMap<String, HostFn>>,
}

impl HostRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a global host function.
    pub fn register<S, F>(&mut self, name: S, f: F)
    where
        S: Into<String>,
        F: Fn(&[Value], &ExecutionContext) -> Result<Value> + Send + Sync + 'static,
    {
        self.functions.insert(name.into(), Arc::new(f));
    }

    /// Register a host function in a module namespace.
    pub fn register_module<M, N, F>(&mut self, module: M, name: N, f: F)
    where
        M: Into<String>,
        N: Into<String>,
        F: Fn(&[Value], &ExecutionContext) -> Result<Value> + Send + Sync + 'static,
    {
        self.modules
            .entry(module.into())
            .or_default()
            .insert(name.into(), Arc::new(f));
    }

    /// Look up a global function.
    pub fn get(&self, name: &str) -> Option<&HostFn> {
        self.functions.get(name)
    }

    /// Look up a module function.
    pub fn get_module(&self, module: &str, name: &str) -> Option<&HostFn> {
        self.modules.get(module).and_then(|m| m.get(name))
    }

    /// Get all registered function names.
    pub fn function_names(&self) -> impl Iterator<Item = &String> {
        self.functions.keys()
    }

    /// Get all registered module names.
    pub fn module_names(&self) -> impl Iterator<Item = &String> {
        self.modules.keys()
    }

    /// Merge another registry into this one.
    pub fn merge(&mut self, other: HostRegistry) {
        self.functions.extend(other.functions);
        for (module, funcs) in other.modules {
            self.modules.entry(module).or_default().extend(funcs);
        }
    }
}

impl std::fmt::Debug for HostRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HostRegistry")
            .field("functions", &self.functions.keys().collect::<Vec<_>>())
            .field("modules", &self.modules.keys().collect::<Vec<_>>())
            .finish()
    }
}

/// Execution context passed to host functions.
#[derive(Debug)]
pub struct ExecutionContext {
    /// Engine ID for tracking.
    pub engine_id: u64,
    /// Capabilities available to the script.
    pub capabilities: Capabilities,
    /// Current limit tracker.
    limit_tracker: Mutex<LimitTracker>,
    /// Sandbox instance.
    sandbox: Sandbox,
    /// Custom context data.
    custom: Mutex<HashMap<String, Value>>,
    /// Start time of current execution.
    start_time: Instant,
    /// Whether execution has been cancelled.
    cancelled: std::sync::atomic::AtomicBool,
}

impl ExecutionContext {
    /// Create a new execution context.
    pub fn new(
        engine_id: u64,
        capabilities: Capabilities,
        limits: Limits,
        sandbox: Sandbox,
    ) -> Self {
        Self {
            engine_id,
            capabilities,
            limit_tracker: Mutex::new(LimitTracker::new(limits)),
            sandbox,
            custom: Mutex::new(HashMap::new()),
            start_time: Instant::now(),
            cancelled: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Check if a capability is granted.
    pub fn has_capability(&self, cap: crate::Capability) -> bool {
        self.capabilities.has(cap)
    }

    /// Require a capability, returning an error if not granted.
    pub fn require_capability(&self, cap: crate::Capability) -> Result<()> {
        self.capabilities.require(cap)
    }

    /// Get the sandbox for permission checks.
    pub fn sandbox(&self) -> &Sandbox {
        &self.sandbox
    }

    /// Record instruction execution and check limits.
    pub fn record_instructions(&self, count: u64) -> Result<()> {
        self.limit_tracker.lock().record_instructions(count)?;
        Ok(())
    }

    /// Record memory usage and check limits.
    pub fn record_memory(&self, bytes: usize) -> Result<()> {
        self.limit_tracker.lock().record_memory(bytes)?;
        Ok(())
    }

    /// Record output and check limits.
    pub fn record_output(&self, bytes: usize) -> Result<()> {
        self.limit_tracker.lock().record_output(bytes)?;
        Ok(())
    }

    /// Record filesystem operation and check limits.
    pub fn record_fs_op(&self) -> Result<()> {
        self.limit_tracker.lock().record_fs_op()?;
        Ok(())
    }

    /// Record network operation and check limits.
    pub fn record_net_op(&self) -> Result<()> {
        self.limit_tracker.lock().record_net_op()?;
        Ok(())
    }

    /// Check timeout and return error if exceeded.
    pub fn check_timeout(&self) -> Result<()> {
        self.limit_tracker.lock().check_timeout()?;
        Ok(())
    }

    /// Get elapsed time since execution started.
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Check if execution has been cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Cancel execution.
    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Set custom context data.
    pub fn set_custom(&self, key: impl Into<String>, value: Value) {
        self.custom.lock().insert(key.into(), value);
    }

    /// Get custom context data.
    pub fn get_custom(&self, key: &str) -> Option<Value> {
        self.custom.lock().get(key).cloned()
    }

    /// Reset the context for a new execution.
    pub fn reset(&self, limits: Limits) {
        *self.limit_tracker.lock() = LimitTracker::new(limits);
        self.cancelled
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.custom.lock().clear();
    }
}

/// A Fusabi execution engine.
///
/// The engine provides a sandboxed environment for executing Fusabi scripts
/// with configurable limits and capabilities.
pub struct Engine {
    id: u64,
    config: EngineConfig,
    registry: HostRegistry,
    context: ExecutionContext,
    /// Bytecode cache for compiled scripts.
    bytecode_cache: Mutex<HashMap<String, Vec<u8>>>,
}

impl Engine {
    /// Create a new engine with the given configuration.
    pub fn new(config: EngineConfig) -> Result<Self> {
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let sandbox = Sandbox::new(config.sandbox.clone())?;
        let context = ExecutionContext::new(
            id,
            config.capabilities.clone(),
            config.limits.clone(),
            sandbox,
        );

        Ok(Self {
            id,
            config,
            registry: HostRegistry::new(),
            context,
            bytecode_cache: Mutex::new(HashMap::new()),
        })
    }

    /// Get the engine ID.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get the engine configuration.
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    /// Get mutable access to the host registry.
    pub fn registry_mut(&mut self) -> &mut HostRegistry {
        &mut self.registry
    }

    /// Get the host registry.
    pub fn registry(&self) -> &HostRegistry {
        &self.registry
    }

    /// Get the execution context.
    pub fn context(&self) -> &ExecutionContext {
        &self.context
    }

    /// Execute a source string and return the result.
    pub fn execute(&self, source: &str) -> Result<Value> {
        // Check for cancellation before starting (before reset clears it)
        if self.context.is_cancelled() {
            return Err(Error::Cancelled);
        }

        self.context.reset(self.config.limits.clone());

        // Simulate compilation and execution
        // In a real implementation, this would call the actual Fusabi VM
        self.simulate_execution(source)
    }

    /// Execute compiled bytecode.
    pub fn execute_bytecode(&self, bytecode: &[u8]) -> Result<Value> {
        // Check for cancellation before starting (before reset clears it)
        if self.context.is_cancelled() {
            return Err(Error::Cancelled);
        }

        self.context.reset(self.config.limits.clone());

        // Validate bytecode header
        if bytecode.len() < 8 || &bytecode[0..4] != b"FZB\x00" {
            return Err(Error::invalid_bytecode("invalid bytecode header"));
        }

        // Simulate bytecode execution
        self.simulate_bytecode_execution(bytecode)
    }

    /// Cancel any ongoing execution.
    pub fn cancel(&self) {
        self.context.cancel();
    }

    /// Check if the engine is healthy.
    pub fn is_healthy(&self) -> bool {
        !self.context.is_cancelled()
    }

    // Internal simulation methods - would be replaced with actual VM calls

    fn simulate_execution(&self, source: &str) -> Result<Value> {
        // Check timeout periodically during "execution"
        self.context.check_timeout()?;

        // Record some instructions
        self.context.record_instructions(source.len() as u64 * 10)?;

        // Simple expression evaluation simulation
        let trimmed = source.trim();

        // Handle simple numeric expressions
        if let Ok(n) = trimmed.parse::<i64>() {
            return Ok(Value::Int(n));
        }

        if let Ok(f) = trimmed.parse::<f64>() {
            return Ok(Value::Float(f));
        }

        // Handle simple string literals
        if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() > 1 {
            return Ok(Value::String(trimmed[1..trimmed.len() - 1].to_string()));
        }

        // Handle simple addition
        if let Some(pos) = trimmed.find('+') {
            let left = trimmed[..pos].trim();
            let right = trimmed[pos + 1..].trim();
            if let (Ok(l), Ok(r)) = (left.parse::<i64>(), right.parse::<i64>()) {
                return Ok(Value::Int(l + r));
            }
        }

        // Handle boolean literals
        match trimmed {
            "true" => return Ok(Value::Bool(true)),
            "false" => return Ok(Value::Bool(false)),
            "null" | "nil" => return Ok(Value::Null),
            _ => {}
        }

        // Default: return null for unrecognized input
        Ok(Value::Null)
    }

    fn simulate_bytecode_execution(&self, _bytecode: &[u8]) -> Result<Value> {
        self.context.check_timeout()?;
        self.context.record_instructions(100)?;
        Ok(Value::Null)
    }
}

impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("id", &self.id)
            .field("config", &self.config)
            .field("registry", &self.registry)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = Engine::new(EngineConfig::default()).unwrap();
        assert!(engine.id() > 0);
        assert!(engine.is_healthy());
    }

    #[test]
    fn test_engine_execute_numbers() {
        let engine = Engine::new(EngineConfig::default()).unwrap();

        let result = engine.execute("42").unwrap();
        assert_eq!(result, Value::Int(42));

        let result = engine.execute("3.14").unwrap();
        assert_eq!(result, Value::Float(3.14));
    }

    #[test]
    fn test_engine_execute_addition() {
        let engine = Engine::new(EngineConfig::default()).unwrap();

        let result = engine.execute("1 + 2").unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_engine_execute_string() {
        let engine = Engine::new(EngineConfig::default()).unwrap();

        let result = engine.execute("\"hello\"").unwrap();
        assert_eq!(result, Value::String("hello".into()));
    }

    #[test]
    fn test_engine_execute_booleans() {
        let engine = Engine::new(EngineConfig::default()).unwrap();

        assert_eq!(engine.execute("true").unwrap(), Value::Bool(true));
        assert_eq!(engine.execute("false").unwrap(), Value::Bool(false));
        assert_eq!(engine.execute("null").unwrap(), Value::Null);
    }

    #[test]
    #[ignore]
    fn test_engine_cancel() {
        let engine = Engine::new(EngineConfig::default()).unwrap();
        engine.cancel();

        let result = engine.execute("42");
        assert!(matches!(result, Err(Error::Cancelled)));
    }

    #[test]
    fn test_host_registry() {
        let mut registry = HostRegistry::new();

        registry.register("test_fn", |args, _ctx| {
            if args.is_empty() {
                Ok(Value::Null)
            } else {
                Ok(args[0].clone())
            }
        });

        registry.register_module("math", "add", |args, _ctx| {
            let a = args.get(0).and_then(|v| v.as_int()).unwrap_or(0);
            let b = args.get(1).and_then(|v| v.as_int()).unwrap_or(0);
            Ok(Value::Int(a + b))
        });

        assert!(registry.get("test_fn").is_some());
        assert!(registry.get_module("math", "add").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_execution_context_capabilities() {
        use crate::Capability;

        let caps = Capabilities::safe_defaults();
        let sandbox = Sandbox::new(SandboxConfig::default()).unwrap();
        let ctx = ExecutionContext::new(1, caps, Limits::default(), sandbox);

        assert!(ctx.has_capability(Capability::TimeRead));
        assert!(!ctx.has_capability(Capability::FsWrite));
    }

    #[test]
    fn test_execution_context_custom_data() {
        let sandbox = Sandbox::new(SandboxConfig::default()).unwrap();
        let ctx = ExecutionContext::new(
            1,
            Capabilities::none(),
            Limits::default(),
            sandbox,
        );

        ctx.set_custom("key", Value::Int(42));
        assert_eq!(ctx.get_custom("key"), Some(Value::Int(42)));
        assert_eq!(ctx.get_custom("nonexistent"), None);
    }

    #[test]
    fn test_engine_config_builder() {
        let config = EngineConfig::new()
            .with_limits(Limits::strict())
            .with_capabilities(Capabilities::none())
            .with_debug(true)
            .with_metadata("name", "test-engine");

        assert!(config.debug);
        assert_eq!(config.metadata.get("name"), Some(&"test-engine".to_string()));
    }
}

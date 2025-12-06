# Embedding fusabi-host

This guide describes how to embed fusabi-host in different types of applications.

## Embedding Patterns

### Pattern 1: Long-Running Daemon

For services that run continuously and execute scripts frequently:

```rust
use fusabi_host::{EnginePool, PoolConfig, Capabilities, Limits};
use std::sync::Arc;

struct ScriptService {
    pool: Arc<EnginePool>,
}

impl ScriptService {
    pub fn new(worker_count: usize) -> Result<Self, fusabi_host::Error> {
        let config = PoolConfig::new(worker_count)
            .with_limits(Limits::default())
            .with_capabilities(Capabilities::safe_defaults());

        let pool = EnginePool::new(config)?;

        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    pub async fn execute_script(&self, source: &str) -> Result<fusabi_host::Value, fusabi_host::Error> {
        // Pool automatically manages engine lifecycle
        self.pool.execute(source)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = ScriptService::new(4)?;

    // Service runs indefinitely, reusing engines from pool
    loop {
        let script = receive_script_from_queue().await?;
        let result = service.execute_script(&script).await?;
        send_result_to_queue(result).await?;
    }
}
```

**Best Practices for Daemons:**
- Use `EnginePool` with worker count = CPU cores
- Set reasonable `Limits` to prevent resource exhaustion
- Monitor pool statistics via `PoolStats`
- Implement health checks using `is_healthy()`
- Handle cancellation for graceful shutdown

### Pattern 2: Short-Lived CLI Tool

For command-line tools that execute a script and exit:

```rust
use fusabi_host::{Engine, EngineConfig, Capabilities, Limits};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <script.fsx>", args[0]);
        std::process::exit(1);
    }

    let source = fs::read_to_string(&args[1])?;

    // Create a single engine for one-time execution
    let config = EngineConfig::new()
        .with_limits(Limits::unlimited())
        .with_capabilities(Capabilities::all());

    let engine = Engine::new(config)?;

    let result = engine.execute(&source)?;
    println!("{:?}", result);

    Ok(())
}
```

**Best Practices for CLI:**
- Use single `Engine` instance (not `EnginePool`)
- Consider more permissive `Capabilities` for trusted scripts
- Set timeout via `Limits::with_timeout()` for long-running scripts
- Use `compile_file()` for better error messages with file paths

### Pattern 3: Request-Scoped Execution

For web services where each request executes a script:

```rust
use fusabi_host::{EnginePool, PoolConfig, HostContext};
use axum::{Router, Json, extract::State};
use std::sync::Arc;

struct AppState {
    pool: EnginePool,
}

async fn execute_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ExecuteRequest>,
) -> Result<Json<ExecuteResponse>, ApiError> {
    // Each request borrows an engine from the pool
    let result = state.pool.execute(&payload.script)?;

    Ok(Json(ExecuteResponse {
        value: result,
    }))
}

#[tokio::main]
async fn main() {
    let config = PoolConfig::new(8)
        .with_limits(Limits::strict())
        .with_capabilities(Capabilities::safe_defaults());

    let state = Arc::new(AppState {
        pool: EnginePool::new(config).unwrap(),
    });

    let app = Router::new()
        .route("/execute", axum::routing::post(execute_handler))
        .with_state(state);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

**Best Practices for Request-Scoped:**
- Use `EnginePool` sized for concurrent request load
- Set strict `Limits` to prevent DoS attacks
- Use `safe_defaults()` capabilities for untrusted user scripts
- Implement request timeout at both HTTP and engine level
- Consider per-user/tenant resource quotas

## HostContext Integration

The `HostContext` trait allows you to integrate logging, metrics, and cancellation:

```rust
use fusabi_host::{HostContext, Engine, EngineConfig};
use tracing::{info, warn};

struct MyHostContext {
    request_id: String,
}

impl HostContext for MyHostContext {
    fn log(&self, level: fusabi_host::LogLevel, message: &str) {
        match level {
            fusabi_host::LogLevel::Error => tracing::error!(request_id = %self.request_id, "{}", message),
            fusabi_host::LogLevel::Warn => warn!(request_id = %self.request_id, "{}", message),
            fusabi_host::LogLevel::Info => info!(request_id = %self.request_id, "{}", message),
            fusabi_host::LogLevel::Debug => tracing::debug!(request_id = %self.request_id, "{}", message),
            fusabi_host::LogLevel::Trace => tracing::trace!(request_id = %self.request_id, "{}", message),
        }
    }

    fn record_metric(&self, name: &str, value: f64, tags: &[(&str, &str)]) {
        // Send to your metrics backend (Prometheus, StatsD, etc.)
        metrics::gauge!(name, value, tags.iter().copied());
    }

    fn should_cancel(&self) -> bool {
        // Check external cancellation source
        // e.g., tokio::task::is_cancelled() or check a channel
        false
    }
}

fn execute_with_context(source: &str, request_id: String) -> fusabi_host::Result<fusabi_host::Value> {
    let ctx = MyHostContext { request_id };
    let engine = Engine::with_context(EngineConfig::default(), ctx)?;
    engine.execute(source)
}
```

## Resource Management

### Memory Limits

```rust
use fusabi_host::Limits;
use std::time::Duration;

let limits = Limits::default()
    .with_memory_mb(64)           // Limit to 64MB heap
    .with_timeout(Duration::from_secs(10))  // 10 second timeout
    .with_max_instructions(10_000_000);      // Instruction limit
```

### Capability Sandboxing

```rust
use fusabi_host::{Capabilities, Capability};

// Minimal capabilities for untrusted code
let minimal = Capabilities::none()
    .with(Capability::TimeRead)
    .with(Capability::Random);

// Safe defaults (time, random, stdout, logging)
let safe = Capabilities::safe_defaults();

// Full access for trusted code
let trusted = Capabilities::all();
```

### Filesystem Isolation

```rust
use fusabi_host::sandbox::{SandboxConfig, PathPolicy};

let sandbox = SandboxConfig::locked()
    .with_read_paths(["/app/data", "/etc/config"])
    .with_write_paths(["/app/data/output"])
    .with_deny_paths(["/etc/passwd", "/etc/shadow"]);
```

## Error Handling

```rust
use fusabi_host::Error;

match engine.execute(source) {
    Ok(value) => println!("Result: {:?}", value),
    Err(Error::Timeout) => eprintln!("Script exceeded timeout"),
    Err(Error::MemoryLimit) => eprintln!("Script exceeded memory limit"),
    Err(Error::CapabilityDenied { capability }) => {
        eprintln!("Script attempted to use denied capability: {:?}", capability)
    }
    Err(Error::Cancelled) => eprintln!("Execution was cancelled"),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Performance Tuning

### Pool Sizing

```rust
// For CPU-bound scripts
let cpu_bound_size = num_cpus::get();

// For I/O-bound scripts (can over-subscribe)
let io_bound_size = num_cpus::get() * 2;

// For mixed workload
let mixed_size = num_cpus::get() + 2;
```

### Bytecode Caching

```rust
use fusabi_host::compile::{compile_source, CompileOptions};

// Compile once, execute many times
let compiled = compile_source(source, &CompileOptions::production())?;

// Cache bytecode for reuse
let cache = std::collections::HashMap::new();
cache.insert("script_id", compiled.bytecode);

// Execute from cache
let result = engine.execute_bytecode(&cache["script_id"])?;
```

## Testing Embedded Applications

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_execution() {
        let config = EngineConfig::default();
        let engine = Engine::new(config).unwrap();

        let result = engine.execute("1 + 2").unwrap();
        assert_eq!(result, fusabi_host::Value::Int(3));
    }

    #[tokio::test]
    async fn test_pool_concurrent_execution() {
        let pool = EnginePool::new(PoolConfig::new(2)).unwrap();

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let pool = pool.clone();
                tokio::spawn(async move {
                    pool.execute(&format!("{}", i)).unwrap()
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }
    }
}
```

## Production Checklist

- [ ] Set appropriate `Limits` for your workload
- [ ] Configure `Capabilities` based on trust level
- [ ] Enable `SandboxConfig` for filesystem/network isolation
- [ ] Implement `HostContext` for logging and metrics
- [ ] Size `EnginePool` based on load testing
- [ ] Add health checks and monitoring
- [ ] Configure graceful shutdown with cancellation
- [ ] Test error handling paths
- [ ] Enable MSRV checks in CI
- [ ] Document minimum Rust version requirement

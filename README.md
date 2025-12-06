# fusabi-host

Shared host/runtime utilities for Fusabi across Scarab, Tolaria, Hibana, and Phage.

## Features

- **Engine Pools** - Thread-safe engine pooling for concurrent script execution
- **Value Conversion** - Seamless Value↔Serde transformation helpers
- **Typed Host Functions** - Ergonomic macros for host function registration
- **Sandbox & Capabilities** - Fine-grained security controls for untrusted code
- **Stable Compile/Run APIs** - Consistent interfaces for host integration

## Quick Start

```rust
use fusabi_host::{EnginePool, PoolConfig, Capabilities, Limits};

fn main() -> fusabi_host::Result<()> {
    // Create a pool with 4 engines
    let config = PoolConfig::new(4)
        .with_limits(Limits::default())
        .with_capabilities(Capabilities::safe_defaults());

    let pool = EnginePool::new(config)?;

    // Execute a script
    let result = pool.execute("1 + 2")?;
    println!("Result: {}", result);

    Ok(())
}
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `serde-support` (default) | Enable Value↔Serde conversion helpers |
| `async-runtime-tokio` | Async execution support via Tokio |
| `async-runtime-async-std` | Async execution support via async-std |
| `metrics-prometheus` | Prometheus metrics integration |

## Safety Model

### Capabilities

Scripts must be granted explicit capabilities for privileged operations:

```rust
use fusabi_host::{Capabilities, Capability};

// No capabilities (fully sandboxed)
let none = Capabilities::none();

// Safe defaults (time, random, stdout, logging)
let safe = Capabilities::safe_defaults();

// Full access (use with trusted code only)
let all = Capabilities::all();

// Custom capability set
let custom = Capabilities::none()
    .with(Capability::FsRead)
    .with(Capability::NetRequest);
```

### Resource Limits

Control resource consumption with configurable limits:

```rust
use fusabi_host::Limits;
use std::time::Duration;

let limits = Limits::default()
    .with_timeout(Duration::from_secs(5))
    .with_memory_mb(32)
    .with_max_instructions(1_000_000);

// Strict limits for untrusted code
let strict = Limits::strict();

// No limits (trusted code only)
let unlimited = Limits::unlimited();
```

### Sandbox Configuration

Control filesystem and network access:

```rust
use fusabi_host::sandbox::{SandboxConfig, PathPolicy, NetPolicy};

let sandbox = SandboxConfig::locked()
    .with_read_paths(["/app/data"])
    .with_allowed_hosts(["api.example.com", "*.trusted.org"]);
```

## Typed Host Functions

Register host functions with automatic argument conversion:

```rust
use fusabi_host::{host_fn, Engine, EngineConfig};

let mut engine = Engine::new(EngineConfig::default())?;

// Register a simple function
engine.registry_mut().register("add", host_fn!(add(a: i64, b: i64) -> i64 {
    a + b
}));

// Register with context access
engine.registry_mut().register("log", host_fn!(ctx, log(msg: String) -> () {
    ctx.record_output(msg.len())?;
    println!("{}", msg);
    Ok(())
}));
```

## Compilation API

Compile Fusabi source to bytecode:

```rust
use fusabi_host::compile::{compile_source, compile_file, CompileOptions};

// Compile source string
let result = compile_source("fn main() { 42 }", &CompileOptions::default())?;
println!("Bytecode size: {} bytes", result.bytecode.len());

// Compile from file
let result = compile_file("script.fsx".as_ref(), &CompileOptions::production())?;

// Access metadata
for export in &result.metadata.exports {
    println!("Export: {}", export.name);
}
```

## Documentation

For detailed documentation, see:

- [Embedding Guide](docs/versions/vNEXT/embedding.md) - How to integrate fusabi-host in different application types
- [Feature Flags & Platform Support](docs/versions/vNEXT/features.md) - Platform support, MSRV, and future directions
- [Compatibility](docs/compat.md) - Version compatibility and migration guides
- [Release Process](docs/RELEASE.md) - For maintainers
- [API Documentation](https://docs.rs/fusabi-host) - Full API reference on docs.rs

## Version Compatibility

This crate is aligned with Fusabi LTS releases:

| fusabi-host | Fusabi |
|-------------|--------|
| 0.1.x | 0.18.x - 0.19.x |

See [docs/compat.md](docs/compat.md) for detailed compatibility information and [docs/versions/](docs/versions/) for version-specific documentation.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

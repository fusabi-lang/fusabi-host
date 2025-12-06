# Feature Flags and Platform Support

## no_std Support Investigation

### Current Status

As of version 0.1.x, fusabi-host does **not** support `no_std` environments.

### Dependencies Requiring std

The following dependencies currently require the standard library:

1. **parking_lot** - Uses OS primitives for mutex/rwlock
2. **crossbeam-channel** - Requires threading support
3. **tracing** - Uses std::io for output
4. **tokio/async-std** - Async runtimes require OS threads

### Potential Path to no_std

To support `no_std` in the future, the following changes would be needed:

```toml
[features]
default = ["std", "serde-support"]
std = ["dep:parking_lot", "dep:crossbeam-channel"]
alloc = []  # Requires alloc but not std
```

**Required changes:**
- Replace `parking_lot` with spin locks for `no_std` targets
- Replace `crossbeam-channel` with lock-free queues or remove pooling
- Make `tracing` optional or provide no-op implementation
- Remove async runtime features for `no_std`
- Use `core::` and `alloc::` instead of `std::`

**Estimated effort:** 2-3 weeks of development + testing

### Use Cases for no_std

Potential applications:
- Embedded systems (ARM Cortex-M)
- Bare metal environments
- Custom OS development
- Resource-constrained devices

## WASM Support Investigation

### Current Status

As of version 0.1.x, fusabi-host does **not** support WebAssembly targets.

### Challenges for WASM

1. **Threading** - EnginePool requires threads, which are experimental in WASM
2. **Filesystem** - Sandbox assumes POSIX filesystem
3. **Async Runtime** - Tokio/async-std have limited WASM support
4. **System Calls** - Many capabilities require OS-level access

### Potential WASM Strategy

#### Option 1: Single-Threaded WASM

Remove pooling, focus on single-engine execution:

```rust
#[cfg(target_arch = "wasm32")]
pub struct WasmEngine {
    // Single-threaded, no pooling
    inner: Engine,
}

#[cfg(target_arch = "wasm32")]
impl WasmEngine {
    pub async fn execute(&self, source: &str) -> Result<Value> {
        // Use wasm_bindgen_futures for async
        self.inner.execute(source)
    }
}
```

#### Option 2: WASI Support

Target `wasm32-wasi` for fuller OS integration:

```toml
[target.'cfg(target_os = "wasi")'.dependencies]
wasi = "0.11"
```

**WASI provides:**
- Filesystem access (via capability-based security)
- Environment variables
- Random number generation
- Clock/time functions

### Feature Flag Approach

```toml
[features]
default = ["std", "serde-support"]
std = ["dep:parking_lot", "dep:crossbeam-channel", "dep:tracing"]
wasm = ["wasm-bindgen", "js-sys", "web-sys"]
wasm-single-threaded = ["wasm"]
```

### WASM Compatibility Table

| Feature | WASM Support | Notes |
|---------|--------------|-------|
| Engine | Partial | Single-threaded only |
| EnginePool | No | Requires threads |
| Capabilities (safe) | Yes | Limited by WASM sandbox |
| Capabilities (fs/net) | WASI only | Requires WASI runtime |
| Sandbox | Limited | Redundant in WASM (already sandboxed) |
| Async runtime (tokio) | No | Use wasm-bindgen-futures |
| Async runtime (async-std) | No | Use wasm-bindgen-futures |
| Metrics | Partial | Console-based only |

### Roadmap for WASM

**Phase 1** (v0.2.x):
- Investigate `wasm32-unknown-unknown` single-threaded support
- Identify minimal feature set for WASM
- Create proof-of-concept example

**Phase 2** (v0.3.x):
- Implement `wasm` feature flag
- Support `wasm32-wasi` target
- Document WASM limitations

**Phase 3** (v0.4.x):
- Optimize for WASM binary size
- Add browser-based examples
- Support SharedArrayBuffer for threading

## Platform Support Matrix

| Platform | Architecture | Status | Notes |
|----------|--------------|--------|-------|
| Linux | x86_64 | Fully supported | Primary development platform |
| Linux | aarch64 | Fully supported | Tested in CI |
| macOS | x86_64 | Fully supported | Intel Macs |
| macOS | aarch64 | Fully supported | Apple Silicon |
| Windows | x86_64 | Supported | Some sandbox features limited |
| Windows | aarch64 | Untested | Should work but not tested |
| FreeBSD | x86_64 | Untested | Should work with std |
| WASM | wasm32 | Not supported | See investigation above |
| iOS | aarch64 | Untested | Requires investigation |
| Android | aarch64 | Untested | Requires investigation |

## MSRV (Minimum Supported Rust Version)

**Current MSRV: 1.75**

This version is chosen for:
- `let-else` statements
- `Option::unzip()`
- Improved type inference
- `#[diagnostic::on_unimplemented]` attribute

### MSRV Policy

- MSRV is tested in CI
- MSRV bumps are considered breaking changes
- We aim to support last 4 stable Rust releases (~6 months)
- MSRV can be increased in minor versions if needed for critical features

### Testing MSRV Locally

```bash
# Install specific Rust version
rustup install 1.75

# Test with MSRV
cargo +1.75 check --all-features
cargo +1.75 test --all-features
```

## Feature Flag Matrix

| Feature | Default | Dependencies | MSRV Impact |
|---------|---------|--------------|-------------|
| `serde-support` | Yes | serde, serde_json | None |
| `async-runtime-tokio` | No | tokio | None |
| `async-runtime-async-std` | No | async-std | None |
| `metrics-prometheus` | No | prometheus | None |

### Feature Compatibility

- `async-runtime-tokio` and `async-runtime-async-std` are **mutually exclusive**
- All features are tested in CI via feature matrix
- Minimal build (no default features) is tested

## Future Feature Flags

Planned for future versions:

```toml
[features]
# v0.2.x
wasm = ["wasm-bindgen", "js-sys"]
no_std = ["spin", "heapless"]
metrics-statsd = ["cadence"]
metrics-opentelemetry = ["opentelemetry"]

# v0.3.x
jit = ["cranelift"]
aot = ["wasmtime"]
scripting-python = ["pyo3"]
scripting-lua = ["mlua"]
```

## Testing Feature Combinations

Our CI tests the following combinations:

1. No features (minimal build)
2. Default features only
3. `serde-support`
4. `async-runtime-tokio`
5. `serde-support,async-runtime-tokio`
6. All features

This ensures compatibility across common configurations.

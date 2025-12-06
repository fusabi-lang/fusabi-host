# Compatibility

This document describes version compatibility between `fusabi-host` and the Fusabi language runtime.

## Version Matrix

| fusabi-host | Fusabi Runtime | Status |
|-------------|----------------|--------|
| 0.1.x | 0.18.x - 0.19.x | Active |

## Fusabi LTS Alignment

The `fusabi-host` crate aligns with Fusabi LTS (Long Term Support) releases:

- **0.18.x**: Current LTS release
- **0.19.x**: Next LTS release (preview support)

## API Stability

### Stable APIs

The following APIs are considered stable and follow semver:

- `EnginePool` and `PoolConfig`
- `Engine` and `EngineConfig`
- `Value` and conversion traits
- `Capabilities` and `Capability`
- `Limits` and `LimitViolation`
- `Sandbox` and `SandboxConfig`
- `compile_source` and `compile_file`
- `Error` and `Result` types

### Unstable APIs

The following APIs may change between minor versions:

- Internal bytecode format details
- Metrics integration (behind feature flag)
- Async runtime specifics

## Migration Notes

### From fusabi 0.17.x

If migrating from direct fusabi 0.17.x usage:

1. Replace direct `Engine` creation with `fusabi_host::Engine`
2. Replace manual `Value` parsing with `FromValue`/`IntoValue` traits
3. Use `Capabilities` instead of manual permission checks
4. Use `Limits` for resource control instead of custom implementations

### Breaking Changes

#### 0.1.0

- Initial release, no breaking changes

## Feature Flag Compatibility

| Feature | Requires |
|---------|----------|
| `serde-support` | serde 1.0, serde_json 1.0 |
| `async-runtime-tokio` | tokio 1.0 |
| `async-runtime-async-std` | async-std 1.12 |
| `metrics-prometheus` | prometheus 0.13 |

## Platform Support

| Platform | Status |
|----------|--------|
| Linux x86_64 | Fully supported |
| Linux aarch64 | Fully supported |
| macOS x86_64 | Fully supported |
| macOS aarch64 | Fully supported |
| Windows x86_64 | Supported |
| WASM | Not supported |

## Reporting Compatibility Issues

If you encounter compatibility issues:

1. Check this document for known limitations
2. Verify you're using compatible versions
3. Open an issue with:
   - `fusabi-host` version
   - Fusabi runtime version
   - Minimal reproduction case
   - Error messages

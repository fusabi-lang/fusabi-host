//! Integration tests for fusabi-host.

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use fusabi_host::{
    compile_source, CompileOptions,
    Engine, EngineConfig,
    EnginePool, PoolConfig,
    SandboxConfig,
    Capabilities, Capability, Error, FromValue, Limits, Result, Value,
};

#[test]
fn test_engine_lifecycle() {
    let engine = Engine::new(EngineConfig::default()).unwrap();

    assert!(engine.id() > 0);
    assert!(engine.is_healthy());

    let result = engine.execute("42").unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_engine_with_strict_config() {
    let config = EngineConfig::strict();
    let engine = Engine::new(config).unwrap();

    // Should still execute basic expressions
    let result = engine.execute("1 + 1").unwrap();
    assert_eq!(result, Value::Int(2));
}

#[test]
fn test_pool_concurrent_execution() {
    let pool = Arc::new(EnginePool::new(PoolConfig::new(4)).unwrap());

    let handles: Vec<thread::JoinHandle<i64>> = (1..=8)
        .map(|i| {
            let pool = Arc::clone(&pool);
            thread::spawn(move || {
                let result = pool.execute(&format!("{}", i * 10)).unwrap();
                result.as_int().unwrap()
            })
        })
        .collect();

    let results: Vec<i64> = handles.into_iter().map(|h: thread::JoinHandle<i64>| h.join().unwrap()).collect();

    // All results should be multiples of 10
    for result in &results {
        assert_eq!(result % 10, 0);
    }

    let stats = pool.stats();
    assert_eq!(stats.executions, 8);
}

#[test]
fn test_pool_exhaustion() {
    let config = PoolConfig::new(2).with_acquire_timeout(Duration::from_millis(50));

    let pool = EnginePool::new(config).unwrap();

    // Acquire all engines
    let _h1 = pool.acquire().unwrap();
    let _h2 = pool.acquire().unwrap();

    // Third should timeout
    let result = pool.acquire();
    assert!(matches!(result, Err(Error::PoolTimeout)));
}

#[test]
fn test_compile_and_execute() {
    let source = "42";
    let result = compile_source(source, &CompileOptions::default()).unwrap();

    assert!(!result.bytecode.is_empty());
    assert!(result.bytecode.starts_with(b"FZB\x00"));

    let engine = Engine::new(EngineConfig::default()).unwrap();
    let exec_result = engine.execute_bytecode(&result.bytecode).unwrap();

    // Bytecode execution returns null in simulation
    assert!(exec_result.is_null());
}

#[test]
fn test_capabilities_enforcement() {
    let caps = Capabilities::none().with(Capability::FsRead);

    assert!(caps.require(Capability::FsRead).is_ok());
    assert!(caps.require(Capability::FsWrite).is_err());
    assert!(caps.require(Capability::ProcessExec).is_err());
}

#[test]
fn test_limits_enforcement() {
    let limits = Limits::default().with_max_instructions(100);

    // Check passes for small count
    assert!(limits.check_instructions(50).is_ok());

    // Check fails for large count
    assert!(limits.check_instructions(150).is_err());
}

#[test]
fn test_sandbox_path_policy() {
    use fusabi_host::PathPolicy;
    use std::path::Path;

    let policy = PathPolicy::DenyAll;
    assert!(!policy.is_allowed(Path::new("/any/path")));

    let policy = PathPolicy::AllowAll;
    assert!(policy.is_allowed(Path::new("/any/path")));
}

#[test]
fn test_sandbox_net_policy() {
    use fusabi_host::NetPolicy;

    let policy = NetPolicy::allow(["api.example.com", "*.trusted.org"]);

    assert!(policy.is_allowed("api.example.com"));
    assert!(policy.is_allowed("sub.trusted.org"));
    assert!(!policy.is_allowed("evil.com"));
}

#[test]
fn test_value_conversions() {
    use fusabi_host::FromValue;

    // Test basic conversions
    assert_eq!(i64::from_value(Value::Int(42)).unwrap(), 42);
    assert_eq!(String::from_value(Value::String("hello".into())).unwrap(), "hello");
    assert_eq!(bool::from_value(Value::Bool(true)).unwrap(), true);

    // Test optional
    let opt: Option<i64> = Option::from_value(Value::Null).unwrap();
    assert_eq!(opt, None);

    let opt: Option<i64> = Option::from_value(Value::Int(42)).unwrap();
    assert_eq!(opt, Some(42));

    // Test type mismatch
    let result = i64::from_value(Value::String("not a number".into()));
    assert!(result.is_err());
}

#[test]
fn test_engine_cancellation() {
    let engine = Engine::new(EngineConfig::default()).unwrap();

    engine.cancel();
    assert!(!engine.is_healthy());

    let result = engine.execute("42");
    assert!(matches!(result, Err(Error::Cancelled)));
}

#[test]
fn test_pool_shutdown() {
    let pool = EnginePool::new(PoolConfig::new(2)).unwrap();

    pool.shutdown();
    assert!(pool.is_shutdown());

    let result = pool.acquire();
    assert!(matches!(result, Err(Error::PoolShutdown)));
}

#[test]
fn test_metadata_extraction() {
    let source = r#"
@require fs:read
@require net:request
import json
import http

export fn main() { }
export fn helper() { }
"#;

    let result = compile_source(source, &CompileOptions::default()).unwrap();
    let metadata = &result.metadata;

    assert!(metadata.requires_capability("fs:read"));
    assert!(metadata.requires_capability("net:request"));
    assert!(metadata.imports_module("json"));
    assert!(metadata.imports_module("http"));
    assert!(metadata.get_export("main").is_some());
    assert!(metadata.get_export("helper").is_some());
}

#[test]
fn test_compile_options_presets() {
    let dev = CompileOptions::development();
    assert_eq!(dev.opt_level, 0);
    assert!(dev.debug_info);
    assert!(!dev.strip);

    let prod = CompileOptions::production();
    assert_eq!(prod.opt_level, 2);
    assert!(!prod.debug_info);
    assert!(prod.strip);
}

#[test]
fn test_host_registry() {
    use fusabi_host::HostRegistry;

    let mut registry = HostRegistry::new();

    registry.register("test_fn", |_args, _ctx| Ok(Value::Int(42)));
    registry.register_module("math", "pi", |_args, _ctx| Ok(Value::Float(3.14159)));

    assert!(registry.get("test_fn").is_some());
    assert!(registry.get_module("math", "pi").is_some());
    assert!(registry.get("nonexistent").is_none());
}

#[test]
fn test_typed_host_functions() {
    use fusabi_host::typed_host_fn_2;
    use fusabi_host::Sandbox;

    let add = typed_host_fn_2(|a: i64, b: i64| -> i64 { a + b });

    let sandbox = Sandbox::new(SandboxConfig::default()).unwrap();
    let ctx = fusabi_host::ExecutionContext::new(
        1,
        Capabilities::none(),
        Limits::default(),
        sandbox,
    );

    let result = add(&[Value::Int(3), Value::Int(4)], &ctx).unwrap();
    assert_eq!(result, Value::Int(7));
}

#[test]
fn test_lazy_pool_init() {
    let config = PoolConfig::new(4).with_lazy_init(true);
    let pool = EnginePool::new(config).unwrap();

    // Initially no engines created
    let stats = pool.stats();
    assert_eq!(stats.available, 0);

    // Acquire creates one
    let handle = pool.try_acquire().unwrap();
    let result = handle.execute("1").unwrap();
    assert_eq!(result, Value::Int(1));
}

#[cfg(feature = "serde-support")]
mod serde_tests {
    use super::*;
    use fusabi_host::{from_value_serde, to_value_serde};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestStruct {
        name: String,
        value: i32,
        tags: Vec<String>,
    }

    #[test]
    fn test_serde_roundtrip() {
        let original = TestStruct {
            name: "test".into(),
            value: 42,
            tags: vec!["a".into(), "b".into()],
        };

        let value = to_value_serde(&original).unwrap();
        let restored: TestStruct = from_value_serde(value).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn test_json_conversion() {
        let value = Value::Map({
            let mut m = std::collections::HashMap::new();
            m.insert("key".into(), Value::String("value".into()));
            m
        });

        let json = value.to_json_string();
        let parsed = Value::from_json_str(&json).unwrap();

        let map = parsed.as_map().unwrap();
        assert_eq!(map.get("key"), Some(&Value::String("value".into())));
    }
}

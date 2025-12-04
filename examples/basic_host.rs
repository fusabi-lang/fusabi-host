//! Basic example of using fusabi-host to execute scripts.

use fusabi_host::{
    compile::{compile_source, CompileOptions},
    engine::{Engine, EngineConfig, HostRegistry},
    host_fn, Capabilities, Limits, Result, Value,
};

fn main() -> Result<()> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Create a basic engine with default configuration
    let config = EngineConfig::default()
        .with_limits(Limits::default())
        .with_capabilities(Capabilities::safe_defaults());

    let mut engine = Engine::new(config)?;

    // Register some host functions
    register_host_functions(engine.registry_mut());

    // Execute simple expressions
    println!("=== Simple Expressions ===");

    let result = engine.execute("42")?;
    println!("42 = {}", result);

    let result = engine.execute("1 + 2")?;
    println!("1 + 2 = {}", result);

    let result = engine.execute("true")?;
    println!("true = {}", result);

    let result = engine.execute("\"hello world\"")?;
    println!("string = {}", result);

    // Compile source to bytecode
    println!("\n=== Compilation ===");

    let source = r#"
        @require fs:read
        import json

        export fn main() {
            let x = 42
            x * 2
        }
    "#;

    let compile_result = compile_source(source, &CompileOptions::development())?;
    println!("Compiled {} bytes of source to {} bytes of bytecode",
        compile_result.stats.source_bytes,
        compile_result.stats.bytecode_bytes);

    println!("Language version: {}", compile_result.metadata.language_version);
    println!("Required capabilities: {:?}", compile_result.metadata.required_capabilities);
    println!("Exports: {:?}", compile_result.metadata.exports.iter()
        .map(|e| &e.name).collect::<Vec<_>>());

    if !compile_result.warnings.is_empty() {
        println!("Warnings:");
        for warning in &compile_result.warnings {
            println!("  - {}", warning.message);
        }
    }

    // Execute bytecode
    println!("\n=== Bytecode Execution ===");
    let result = engine.execute_bytecode(&compile_result.bytecode)?;
    println!("Bytecode result: {}", result);

    Ok(())
}

fn register_host_functions(registry: &mut HostRegistry) {
    // Register a simple add function
    registry.register("add", |args, _ctx| {
        let a = args.get(0).and_then(|v| v.as_int()).unwrap_or(0);
        let b = args.get(1).and_then(|v| v.as_int()).unwrap_or(0);
        Ok(Value::Int(a + b))
    });

    // Register a string concatenation function
    registry.register("concat", |args, _ctx| {
        let parts: Vec<String> = args
            .iter()
            .map(|v| match v {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .collect();
        Ok(Value::String(parts.join("")))
    });

    // Register functions in a module namespace
    registry.register_module("math", "pi", |_args, _ctx| {
        Ok(Value::Float(std::f64::consts::PI))
    });

    registry.register_module("math", "sqrt", |args, _ctx| {
        let n = args.get(0).and_then(|v| v.as_float()).unwrap_or(0.0);
        Ok(Value::Float(n.sqrt()))
    });

    registry.register_module("math", "pow", |args, _ctx| {
        let base = args.get(0).and_then(|v| v.as_float()).unwrap_or(0.0);
        let exp = args.get(1).and_then(|v| v.as_float()).unwrap_or(1.0);
        Ok(Value::Float(base.powf(exp)))
    });

    println!("Registered host functions:");
    for name in registry.function_names() {
        println!("  - {}", name);
    }
    for module in registry.module_names() {
        println!("  - {}.*", module);
    }
}

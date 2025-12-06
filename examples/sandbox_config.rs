//! Example demonstrating sandbox and capability configuration.

use std::path::Path;

use fusabi_host::{
    Engine, EngineConfig,
    NetPolicy, PathPolicy, Sandbox, SandboxConfig,
    Capabilities, Capability, Limits, Result,
};

fn main() -> Result<()> {
    println!("=== Sandbox Configuration Example ===\n");

    // Demonstrate different capability configurations
    demonstrate_capabilities()?;

    // Demonstrate sandbox policies
    demonstrate_sandbox_policies()?;

    // Demonstrate complete secure configuration
    demonstrate_secure_config()?;

    Ok(())
}

fn demonstrate_capabilities() -> Result<()> {
    println!("=== Capabilities ===\n");

    // No capabilities
    let none = Capabilities::none();
    println!("None: {} capabilities", none.len());

    // Safe defaults
    let safe = Capabilities::safe_defaults();
    println!("Safe defaults:");
    for cap in safe.granted() {
        println!("  - {}", cap.name());
    }

    // Full access
    let all = Capabilities::all();
    println!("\nAll: {} capabilities", all.len());
    println!("Has dangerous: {}", all.has_dangerous());

    // Custom set
    let custom = Capabilities::none()
        .with(Capability::FsRead)
        .with(Capability::NetRequest)
        .with(Capability::TimeRead);

    println!("\nCustom set:");
    for cap in custom.granted() {
        println!("  - {}", cap.name());
    }

    // Check capabilities
    println!("\nCapability checks:");
    println!("  Has FsRead: {}", custom.has(Capability::FsRead));
    println!("  Has FsWrite: {}", custom.has(Capability::FsWrite));
    println!("  Has dangerous: {}", custom.has_dangerous());

    // Require capability
    match custom.require(Capability::FsRead) {
        Ok(()) => println!("  Require FsRead: granted"),
        Err(e) => println!("  Require FsRead: {}", e),
    }

    match custom.require(Capability::ProcessExec) {
        Ok(()) => println!("  Require ProcessExec: granted"),
        Err(e) => println!("  Require ProcessExec: denied"),
    }

    // Parse from names
    let from_names = Capabilities::from_names(["fs:read", "net:request", "invalid"]);
    println!("\nFrom names ['fs:read', 'net:request', 'invalid']:");
    println!("  Parsed {} capabilities", from_names.len());

    // Merge and intersect
    let a = Capabilities::none()
        .with(Capability::FsRead)
        .with(Capability::TimeRead);
    let b = Capabilities::none()
        .with(Capability::FsRead)
        .with(Capability::NetRequest);

    let merged = a.merge(&b);
    let intersected = a.intersect(&b);

    println!("\nMerge/Intersect:");
    println!("  A: FsRead, TimeRead");
    println!("  B: FsRead, NetRequest");
    println!("  Merged: {} caps", merged.len());
    println!("  Intersected: {} caps", intersected.len());

    Ok(())
}

fn demonstrate_sandbox_policies() -> Result<()> {
    println!("\n=== Sandbox Policies ===\n");

    // Path policies
    println!("Path Policies:");

    let deny_all = PathPolicy::DenyAll;
    let allow_all = PathPolicy::AllowAll;
    let allowlist = PathPolicy::allow(["/tmp", "/home/user/data"]);
    let denylist = PathPolicy::deny(["/etc", "/root"]);

    println!("  DenyAll allows /tmp: {}", deny_all.is_allowed(Path::new("/tmp")));
    println!("  AllowAll allows /etc: {}", allow_all.is_allowed(Path::new("/etc")));

    // Network policies
    println!("\nNetwork Policies:");

    let net_deny = NetPolicy::DenyAll;
    let net_allow = NetPolicy::AllowAll;
    let net_allowlist = NetPolicy::allow(["api.example.com", "*.trusted.org"]);
    let net_denylist = NetPolicy::deny(["evil.com", "*.malware.net"]);

    println!("  DenyAll allows example.com: {}", net_deny.is_allowed("example.com"));
    println!("  AllowAll allows anything: {}", net_allow.is_allowed("anything.com"));

    println!("\n  Allowlist tests:");
    println!("    api.example.com: {}", net_allowlist.is_allowed("api.example.com"));
    println!("    sub.trusted.org: {}", net_allowlist.is_allowed("sub.trusted.org"));
    println!("    trusted.org: {}", net_allowlist.is_allowed("trusted.org"));
    println!("    other.com: {}", net_allowlist.is_allowed("other.com"));

    println!("\n  Denylist tests:");
    println!("    good.com: {}", net_denylist.is_allowed("good.com"));
    println!("    evil.com: {}", net_denylist.is_allowed("evil.com"));
    println!("    download.malware.net: {}", net_denylist.is_allowed("download.malware.net"));

    Ok(())
}

fn demonstrate_secure_config() -> Result<()> {
    println!("\n=== Secure Engine Configuration ===\n");

    // Locked sandbox config
    let sandbox_config = SandboxConfig::locked()
        .with_read_paths(["/app/data", "/app/config"])
        .with_allowed_hosts(["api.myapp.com", "*.internal.local"])
        .with_env_vars(["APP_ENV", "APP_VERSION"])
        .with_temp_isolation();

    println!("Sandbox configuration:");
    println!("  Can read /app/data: {}", sandbox_config.can_read(Path::new("/app/data")));
    println!("  Can read /etc/passwd: {}", sandbox_config.can_read(Path::new("/etc/passwd")));
    println!("  Can write /app/data: {}", sandbox_config.can_write(Path::new("/app/data")));
    println!("  Can connect to api.myapp.com: {}", sandbox_config.can_connect("api.myapp.com"));
    println!("  Can connect to evil.com: {}", sandbox_config.can_connect("evil.com"));
    println!("  Can access APP_ENV: {}", sandbox_config.can_access_env("APP_ENV"));
    println!("  Can access SECRET_KEY: {}", sandbox_config.can_access_env("SECRET_KEY"));

    // Create sandbox instance
    let sandbox = Sandbox::new(sandbox_config)?;
    println!("\nSandbox created");
    if let Some(temp) = sandbox.temp_dir() {
        println!("  Isolated temp dir: {}", temp.display());
    }

    // Permission checks
    println!("\nPermission checks:");
    match sandbox.check_read(Path::new("/app/data/file.txt")) {
        Ok(()) => println!("  Read /app/data/file.txt: allowed"),
        Err(e) => println!("  Read /app/data/file.txt: {}", e),
    }

    match sandbox.check_read(Path::new("/etc/passwd")) {
        Ok(()) => println!("  Read /etc/passwd: allowed"),
        Err(e) => println!("  Read /etc/passwd: denied"),
    }

    match sandbox.check_connect("api.myapp.com") {
        Ok(()) => println!("  Connect api.myapp.com: allowed"),
        Err(e) => println!("  Connect api.myapp.com: {}", e),
    }

    match sandbox.check_connect("hacker.com") {
        Ok(()) => println!("  Connect hacker.com: allowed"),
        Err(e) => println!("  Connect hacker.com: denied"),
    }

    // Full engine configuration
    println!("\n=== Creating Secure Engine ===\n");

    let engine_config = EngineConfig::new()
        .with_limits(
            Limits::strict()
                .with_timeout(std::time::Duration::from_secs(5))
                .with_memory_mb(16),
        )
        .with_capabilities(
            Capabilities::none()
                .with(Capability::TimeRead)
                .with(Capability::Logging)
                .with(Capability::Serialize),
        )
        .with_sandbox(SandboxConfig::locked())
        .with_debug(false)
        .with_metadata("purpose", "untrusted-plugin");

    let engine = Engine::new(engine_config)?;

    println!("Engine created with ID: {}", engine.id());
    println!("Engine config:");
    println!("  Timeout: {:?}", engine.config().limits.timeout);
    println!("  Memory limit: {:?} bytes", engine.config().limits.memory_bytes);
    println!("  Max instructions: {:?}", engine.config().limits.max_instructions);
    println!("  Debug mode: {}", engine.config().debug);

    // Test execution
    let result = engine.execute("1 + 1")?;
    println!("\nExecution test: 1 + 1 = {}", result);

    // Check context capabilities
    let ctx = engine.context();
    println!("\nContext capability checks:");
    println!("  TimeRead: {}", ctx.has_capability(Capability::TimeRead));
    println!("  FsWrite: {}", ctx.has_capability(Capability::FsWrite));
    println!("  ProcessExec: {}", ctx.has_capability(Capability::ProcessExec));

    Ok(())
}

//! Example demonstrating engine pooling for concurrent execution.

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use fusabi_host::{
    EnginePool, PoolConfig,
    Capabilities, Limits, Result,
};

fn main() -> Result<()> {
    println!("=== Engine Pool Example ===\n");

    // Create a pool with 4 engines
    let config = PoolConfig::new(4)
        .with_limits(Limits::default().with_timeout(Duration::from_secs(5)))
        .with_capabilities(Capabilities::safe_defaults())
        .with_acquire_timeout(Duration::from_secs(10));

    let pool = Arc::new(EnginePool::new(config)?);

    println!("Created pool with {} engines", pool.config().size);
    println!("Initial stats: {:?}\n", pool.stats());

    // Demonstrate sequential execution
    println!("=== Sequential Execution ===");
    for i in 1..=5 {
        let result = pool.execute(&format!("{} * 2", i))?;
        println!("  {} * 2 = {}", i, result);
    }

    // Demonstrate parallel execution
    println!("\n=== Parallel Execution ===");

    let handles: Vec<_> = (1..=8)
        .map(|i| {
            let pool = Arc::clone(&pool);
            thread::spawn(move || {
                let expr = format!("{} + {}", i, i * 10);
                let tid = thread::current().id();
                println!("  Thread {:?}: evaluating '{}'", tid, expr);

                match pool.execute(&expr) {
                    Ok(result) => {
                        println!("  Thread {:?}: {} = {}", tid, expr, result);
                        Ok(result)
                    }
                    Err(e) => {
                        println!("  Thread {:?}: error - {}", tid, e);
                        Err(e)
                    }
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        let _ = handle.join();
    }

    println!("\nFinal stats: {:?}", pool.stats());

    // Demonstrate pool handle lifecycle
    println!("\n=== Handle Lifecycle ===");

    {
        let handle1 = pool.acquire()?;
        println!("Acquired handle 1, in_use: {}", pool.stats().in_use);

        let handle2 = pool.acquire()?;
        println!("Acquired handle 2, in_use: {}", pool.stats().in_use);

        // Execute with handles
        let r1 = handle1.execute("100")?;
        let r2 = handle2.execute("200")?;
        println!("Results: {}, {}", r1, r2);

        // Handles automatically returned when dropped
    }

    println!("After handles dropped, in_use: {}", pool.stats().in_use);

    // Demonstrate try_acquire
    println!("\n=== Try Acquire ===");

    // Acquire all engines
    let mut handles = Vec::new();
    for i in 0..4 {
        match pool.try_acquire() {
            Ok(h) => {
                println!("  try_acquire {}: success", i);
                handles.push(h);
            }
            Err(e) => {
                println!("  try_acquire {}: {}", i, e);
            }
        }
    }

    // Fifth should fail (pool exhausted)
    match pool.try_acquire() {
        Ok(_) => println!("  try_acquire 4: success (unexpected!)"),
        Err(e) => println!("  try_acquire 4: {} (expected)", e),
    }

    // Release and try again
    handles.pop();
    match pool.try_acquire() {
        Ok(_) => println!("  After release, try_acquire: success"),
        Err(e) => println!("  After release, try_acquire: {}", e),
    }

    drop(handles);

    // Demonstrate cancellation
    println!("\n=== Cancellation ===");

    let handle = pool.acquire()?;
    handle.cancel();
    match handle.execute("42") {
        Ok(v) => println!("  Unexpected success: {}", v),
        Err(e) => println!("  After cancel: {} (expected)", e),
    }

    // Demonstrate shutdown
    println!("\n=== Shutdown ===");

    pool.shutdown();
    println!("Pool shutdown: {}", pool.is_shutdown());

    match pool.acquire() {
        Ok(_) => println!("  Unexpected acquire success"),
        Err(e) => println!("  Acquire after shutdown: {} (expected)", e),
    }

    println!("\nFinal stats:");
    let stats = pool.stats();
    println!("  Total engines: {}", stats.total);
    println!("  Total acquisitions: {}", stats.acquisitions);
    println!("  Total releases: {}", stats.releases);
    println!("  Total executions: {}", stats.executions);
    println!("  Total timeouts: {}", stats.timeouts);
    println!("  Avg execution time: {:?}", stats.avg_execution_time());

    Ok(())
}

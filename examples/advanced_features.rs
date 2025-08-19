use rust_test_harness::{
    before_all, before_each, after_each, after_all, 
    test, test_with_tags, test_with_docker, DockerRunOptions, 
    Readiness, TestRunner, TestConfig
};
use std::time::Duration;

fn main() {
    // Initialize logger
    env_logger::init();

    // Register hooks
    before_all(|_| {
        println!("🚀 Starting advanced test suite...");
        Ok(())
    });

    after_all(|_| {
        println!("✨ Test suite completed!");
        Ok(())
    });

    before_each(|_| {
        println!("  📋 Setting up test...");
        Ok(())
    });

    after_each(|_| {
        println!("  🧹 Cleaning up test...");
        Ok(())
    });

    // Tests with different characteristics
    test("fast unit test", |_| {
        // This should complete quickly
        Ok(())
    });

    test("medium complexity test", |_| {
        // Simulate some work
        let mut sum = 0;
        for i in 0..1000 {
            sum += i;
        }
        assert_eq!(sum, 499500);
        Ok(())
    });

    test_with_tags("database integration", vec!["integration", "database", "slow"], |_| {
        // Simulate database operations
        std::thread::sleep(Duration::from_millis(200));
        Ok(())
    });

    test_with_tags("api integration", vec!["integration", "api", "slow"], |_| {
        // Simulate API calls
        std::thread::sleep(Duration::from_millis(150));
        Ok(())
    });

    test_with_tags("performance test", vec!["performance", "benchmark"], |_| {
        // Simulate performance testing
        std::thread::sleep(Duration::from_millis(100));
        Ok(())
    });

    // Docker tests with different readiness strategies
    let nginx_opts = DockerRunOptions::new("nginx:alpine")
        .port(8080, 80)
        .ready_timeout(Duration::from_secs(15))
        .readiness(Readiness::PortOpen(80));

    test_with_docker("nginx http test", nginx_opts, |ctx| {
        if ctx.docker.is_some() {
            println!("    🌐 Nginx container is ready!");
            Ok(())
        } else {
            Err("Nginx container not available".into())
        }
    });

    let postgres_opts = DockerRunOptions::new("postgres:13-alpine")
        .port(5432, 5432)
        .env("POSTGRES_PASSWORD", "testpass")
        .env("POSTGRES_DB", "testdb")
        .ready_timeout(Duration::from_secs(20))
        .readiness(Readiness::PortOpen(5432));

    test_with_docker("postgres database test", postgres_opts, |ctx| {
        if ctx.docker.is_some() {
            println!("    🗄️  Postgres container is ready!");
            Ok(())
        } else {
            Err("Postgres container not available".into())
        }
    });

    // Run with different configurations
    println!("\n=== Running with default configuration ===");
    let default_config = TestConfig::default();
    let default_runner = TestRunner::with_config(default_config);
    let _ = default_runner.run();

    println!("\n=== Running with custom configuration ===");
    let custom_config = TestConfig {
        filter: Some("integration".to_string()),
        skip_tags: vec!["slow".to_string()],
        max_concurrency: Some(2),
        shuffle_seed: Some(42),
        color: Some(true),
        junit_xml: Some("test-results.xml".to_string()),
    };

    let custom_runner = TestRunner::with_config(custom_config.clone());
    let exit_code = custom_runner.run();

    println!("\n=== Configuration used ===");
    println!("Filter: {:?}", custom_config.filter);
    println!("Skip tags: {:?}", custom_config.skip_tags);
    println!("Max concurrency: {:?}", custom_config.max_concurrency);
    println!("Shuffle seed: {:?}", custom_config.shuffle_seed);
    println!("Color: {:?}", custom_config.color);
    println!("JUnit XML: {:?}", custom_config.junit_xml);

    std::process::exit(exit_code);
} 
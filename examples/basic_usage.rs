use rust_test_harness::{
    before_all, before_each, after_each, after_all, 
    test, test_with_tags, test_with_docker, DockerRunOptions, Readiness
};

fn main() {
    // Initialize logger
    env_logger::init();

    // Register global hooks
    before_all(|_| {
        println!("Setting up test environment...");
        Ok(())
    });

    after_all(|_| {
        println!("Cleaning up test environment...");
        Ok(())
    });

    before_each(|_| {
        println!("  Preparing test...");
        Ok(())
    });

    after_each(|_| {
        println!("  Cleaning up test...");
        Ok(())
    });

    // Basic tests
    test("simple arithmetic", |_| {
        assert_eq!(2 + 2, 4);
        assert_eq!(10 - 5, 5);
        Ok(())
    });

    test("string operations", |_| {
        let text = "hello world";
        assert!(text.contains("hello"));
        assert_eq!(text.len(), 11);
        Ok(())
    });

    // Tagged tests
    test_with_tags("integration test", vec!["integration", "slow"], |_| {
        // Simulate some integration work
        std::thread::sleep(std::time::Duration::from_millis(100));
        Ok(())
    });

    test_with_tags("unit test", vec!["unit", "fast"], |_| {
        // Fast unit test
        Ok(())
    });

    // Docker test with custom options
    let redis_opts = DockerRunOptions::new("redis:alpine")
        .port(6379, 6379)
        .env("REDIS_PASSWORD", "testpass")
        .ready_timeout(std::time::Duration::from_secs(10))
        .readiness(Readiness::PortOpen(6379));

    test_with_docker("redis container test", redis_opts, |ctx| {
        if ctx.docker.is_some() {
            println!("    Redis container is running!");
            Ok(())
        } else {
            Err("Docker container not available".into())
        }
    });

    // Run all tests
    let exit_code = rust_test_harness::run_all();
    std::process::exit(exit_code);
} 
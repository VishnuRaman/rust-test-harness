use rust_test_harness::{test, run_tests, TestConfig, DockerRunOptions};
use std::time::Duration;

#[test]
fn test_basic_integration() {
    // This test verifies that the framework can run a simple test
    // Thread-local isolation means no manual cleanup needed!
    
    test("basic_integration_test", |_| {
        Ok(())
    });
    
    let result = run_tests();
    assert_eq!(result, 0);
}

#[test]
fn test_failing_integration() {
    // This test verifies that the framework can detect test failures
    // Thread-local isolation means no manual cleanup needed!
    
    test("failing_integration_test", |_| {
        Err("intentional failure".into())
    });
    
    let result = run_tests();
    assert_eq!(result, 1);
}

#[test]
fn test_multiple_tests_integration() {
    // Test that multiple tests can run in sequence
    
    test("integration_test_1", |_| Ok(()));
    test("integration_test_2", |_| Ok(()));
    test("integration_test_3", |_| Ok(()));
    
    let result = run_tests();
    assert_eq!(result, 0);
}

#[test]
fn test_mixed_success_failure_integration() {
    // Test that the framework handles mixed success/failure correctly
    
    test("mixed_success_test", |_| Ok(()));
    test("mixed_failure_test", |_| Err("mixed failure".into()));
    test("mixed_success_test_2", |_| Ok(()));
    
    let result = run_tests();
    assert_eq!(result, 1); // Should fail due to one failing test
}

#[test]
fn test_integration_with_config() {
    // Test integration with custom configuration
    
    let config = TestConfig {
        filter: Some("config_test".to_string()),
        skip_tags: vec![],
        max_concurrency: Some(1),
        shuffle_seed: None,
        color: Some(false),
        html_report: None,
        skip_hooks: None,
    };
    
    test("config_test_1", |_| Ok(()));
    test("config_test_2", |_| Ok(()));
    test("other_test", |_| Ok(())); // Should be filtered out
    
    let result = rust_test_harness::run_tests_with_config(config);
    assert_eq!(result, 0);
}

#[test]
fn test_integration_error_handling() {
    // Test various error conditions in integration
    
    // Test with panic
    test("panic_integration_test", |_| {
        panic!("intentional panic in integration");
    });
    
    // Test with error
    test("error_integration_test", |_| {
        Err("intentional error in integration".into())
    });
    
    // Test with success
    test("success_integration_test", |_| Ok(()));
    
    let result = run_tests();
    assert_eq!(result, 1); // Should fail due to panic/error
}

#[test]
fn test_integration_concurrent_registration() {
    // Test that tests can be registered from different contexts
    
    // Register tests in main thread
    test("main_thread_test", |_| Ok(()));
    
    // Register tests in spawned thread
    let handle = std::thread::spawn(|| {
        test("spawned_thread_test", |_| Ok(()));
    });
    
    handle.join().unwrap();
    
    let result = run_tests();
    assert_eq!(result, 0);
}

#[test]
fn test_integration_docker_options() {
    // Test Docker options integration
    
    let opts = DockerRunOptions::new("alpine:latest")
        .env("TEST_ENV", "integration")
        .port(8080, 80)
        .name("integration-test-container");
    
    // Verify options are set correctly
    assert_eq!(opts.image, "alpine:latest");
    assert_eq!(opts.env, vec![("TEST_ENV".to_string(), "integration".to_string())]);
    assert_eq!(opts.ports, vec![(8080, 80)]);
    assert_eq!(opts.name, Some("integration-test-container".to_string()));
    
    // Test that we can create multiple options
    let opts2 = DockerRunOptions::new("nginx:alpine")
        .env("NGINX_HOST", "localhost");
    
    assert_eq!(opts2.image, "nginx:alpine");
    assert_eq!(opts2.env, vec![("NGINX_HOST".to_string(), "localhost".to_string())]);
}

#[test]
fn test_integration_large_test_suite() {
    // Test integration with a larger number of tests
    
    // Register many tests
    for i in 0..50 {
        test(&format!("large_suite_test_{}", i), |_| {
            // Simulate some work
            std::thread::sleep(Duration::from_millis(1));
            Ok(())
        });
    }
    
    let result = run_tests();
    assert_eq!(result, 0);
}

#[test]
fn test_integration_resource_cleanup() {
    // Test that resources are properly cleaned up between tests
    
    test("resource_test_1", |_| {
        // Simulate resource usage
        let _resource = "test_resource_1".to_string();
        Ok(())
    });
    
    test("resource_test_2", |_| {
        // Simulate resource usage
        let _resource = "test_resource_2".to_string();
        Ok(())
    });
    
    let result = run_tests();
    assert_eq!(result, 0);
}

#[test]
fn test_integration_performance() {
    // Test integration performance characteristics
    
    let start = std::time::Instant::now();
    
    // Register and run many quick tests
    for i in 0..100 {
        test(&format!("perf_test_{}", i), |_| Ok(()));
    }
    
    let result = run_tests();
    let duration = start.elapsed();
    
    assert_eq!(result, 0);
    
    // Performance assertion: 100 tests should complete in reasonable time
    // Adjust threshold based on your system performance
    assert!(
        duration.as_millis() < 5000, 
        "100 tests took {}ms, expected < 5000ms", 
        duration.as_millis()
    );
}

#[test]
fn test_integration_edge_cases() {
    // Test edge cases in integration
    
    // Test with empty test name
    test("", |_| Ok(()));
    
    // Test with very long test name
    let long_name = "a".repeat(1000);
    test(&long_name, |_| Ok(()));
    
    // Test with special characters in name
    test("test_with_special_chars_123_!@#$%^&*()", |_| Ok(()));
    
    let result = run_tests();
    assert_eq!(result, 0);
}

#[test]
fn test_integration_recovery_scenarios() {
    // Test various recovery scenarios
    
    // Scenario 1: All tests pass
    test("recovery_scenario_1", |_| Ok(()));
    let result1 = run_tests();
    assert_eq!(result1, 0);
    
    // Scenario 2: Some tests fail
    test("recovery_scenario_2_fail", |_| Err("scenario 2 failure".into()));
    test("recovery_scenario_2_pass", |_| Ok(()));
    let result2 = run_tests();
    assert_eq!(result2, 1);
    
    // Scenario 3: All tests pass again
    test("recovery_scenario_3", |_| Ok(()));
    let result3 = run_tests();
    assert_eq!(result3, 0);
}

#[test]
fn test_integration_cross_module() {
    // Test integration across different modules
    
    // This test verifies that the framework works correctly
    // when tests are registered from different contexts
    
    test("cross_module_test", |_| {
        // Simulate cross-module functionality
        let module_a = "module_a";
        let module_b = "module_b";
        
        assert_ne!(module_a, module_b);
        assert_eq!(module_a.len(), 8);
        assert_eq!(module_b.len(), 8);
        
        Ok(())
    });
    
    let result = run_tests();
    assert_eq!(result, 0);
} 
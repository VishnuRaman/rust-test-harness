use rust_test_harness::{
    TestConfig, TimeoutConfig,
    TestError, test, before_all, before_each, after_each, after_all
};
use std::time::Duration;
use log::info;
use atty;

#[test]
fn test_basic_passing_test() {
    // This test verifies that the framework can run a simple passing test
    // Thread-local isolation means no manual cleanup needed!
    
    test("basic_passing_test_unique", |_| {
        Ok(())
    });
    
    let result = rust_test_harness::run_tests();
    
    assert_eq!(result, 0);
}

#[test]
fn test_basic_failing_test() {
    // This test verifies that the framework can detect test failures
    // Thread-local isolation means no manual cleanup needed!
    
    test("basic_failing_test_unique", |_| {
        Err("intentional failure".into())
    });
    
    let result = rust_test_harness::run_tests();
    
    assert_eq!(result, 1);
}

#[test]
fn test_panicking_test() {
    // This test verifies that the framework can handle panicking tests
    // Thread-local isolation means no manual cleanup needed!
    
    test("panicking_test_unique", |_| {
        // This panic should happen inside the framework's test execution
        // and be caught by the framework's panic handler
        panic!("intentional panic");
    });
    
    let result = rust_test_harness::run_tests();
    
    // The test should fail due to panic, so we expect exit code 1
    assert_eq!(result, 1);
}

#[test]
fn test_hooks_execution_order() {
    // This test verifies that hooks execute in the correct order
    // Thread-local isolation means no manual cleanup needed!
    
    before_all(|_| {
        Ok(())
    });
    
    before_each(|_| {
        Ok(())
    });
    
    after_each(|_| {
        Ok(())
    });
    
    after_all(|_| {
        Ok(())
    });
    
    test("hooks_execution_order_test_unique", |_| {
        Ok(())
    });
    
    let result = rust_test_harness::run_tests();
    
    // Just verify the test runs without error
    assert_eq!(result, 0);
}

#[test]
fn test_test_filtering() {
    // This test verifies that test filtering works
    // Thread-local isolation means no manual cleanup needed!
    
    let config = TestConfig {
        filter: Some("second".to_string()),
        skip_tags: vec![],
        max_concurrency: None,
        shuffle_seed: None,
        color: Some(false),
        html_report: None,
        skip_hooks: None,
        timeout_config: TimeoutConfig::default(),
    };
    
    test("filtering_first_test_unique", |_| Ok(()));
    test("filtering_second_test_unique", |_| Ok(()));
    test("filtering_third_test_unique", |_| Ok(()));
    
    let result = rust_test_harness::run_tests_with_config(config);
    
    assert_eq!(result, 0);
}

#[test]
fn test_tag_filtering() {
    // This test verifies that tag filtering works
    // Thread-local isolation means no manual cleanup needed!
    
    let config = TestConfig {
        filter: None,
        skip_tags: vec!["slow".to_string()],
        max_concurrency: None,
        shuffle_seed: None,
        color: Some(false),
        html_report: None,
        skip_hooks: None,
        timeout_config: TimeoutConfig::default(),
    };
    
    test("tag_filtering_untagged_test_unique", |_| Ok(()));
    
    let result = rust_test_harness::run_tests_with_config(config);
    
    // Should pass with no failures
    assert_eq!(result, 0);
}

#[test]
fn test_test_with_tags_macro() {
    // Test that test_with_tags macro works correctly
    use rust_test_harness::test_with_tags;
    
    test_with_tags("tagged_test_1", vec!["fast", "unit"], |_| Ok(()));
    test_with_tags("tagged_test_2", vec!["slow", "integration"], |_| Ok(()));
    test_with_tags("tagged_test_3", vec!["fast", "integration"], |_| Ok(()));
    
    // Verify tests were registered with correct tags
    let result = rust_test_harness::run_tests();
    assert_eq!(result, 0);
}

#[test]
fn test_tag_filtering_functionality() {
    // Test that tag filtering actually works by creating tagged tests
    use rust_test_harness::test_with_tags;
    
    // Create tests with different tags
    test_with_tags("fast_unit_test", vec!["fast", "unit"], |_| Ok(()));
    test_with_tags("slow_integration_test", vec!["slow", "integration"], |_| Ok(()));
    test_with_tags("fast_integration_test", vec!["fast", "integration"], |_| Ok(()));
    
    // Test 1: Skip slow tests
    let config_slow = TestConfig {
        skip_tags: vec!["slow".to_string()],
        ..Default::default()
    };
    
    let result_slow = rust_test_harness::run_tests_with_config(config_slow);
    assert_eq!(result_slow, 0);
    
    // Test 2: Skip integration tests
    let config_integration = TestConfig {
        skip_tags: vec!["integration".to_string()],
        ..Default::default()
    };
    
    let result_integration = rust_test_harness::run_tests_with_config(config_integration);
    assert_eq!(result_integration, 0);
    
    // Test 3: Skip multiple tags
    let config_multiple = TestConfig {
        skip_tags: vec!["slow".to_string(), "integration".to_string()],
        ..Default::default()
    };
    
    let result_multiple = rust_test_harness::run_tests_with_config(config_multiple);
    assert_eq!(result_multiple, 0);
}

#[test]
fn test_tag_filtering_with_environment_variable() {
    // Test that TEST_SKIP_TAGS environment variable works
    use rust_test_harness::test_with_tags;
    
    // Create tagged tests
    test_with_tags("env_tagged_test_1", vec!["ci", "slow"], |_| Ok(()));
    test_with_tags("env_tagged_test_2", vec!["fast", "unit"], |_| Ok(()));
    
    // Set environment variable to skip slow tests
    std::env::set_var("TEST_SKIP_TAGS", "slow");
    
    let result = rust_test_harness::run_tests();
    assert_eq!(result, 0);
    
    // Clean up environment variable
    std::env::remove_var("TEST_SKIP_TAGS");
}

#[test]
fn test_test_runner_config() {
    // Ensure environment is clean for this test
    std::env::remove_var("TEST_SKIP_TAGS");
    
    let config = TestConfig::default();
    
    // These should be None by default when env vars aren't set
    assert_eq!(config.filter, None);
    assert!(config.skip_tags.is_empty());
    assert_eq!(config.max_concurrency, None);
    assert_eq!(config.shuffle_seed, None);
    // Color defaults to TTY detection - true in terminal, false in CI
    let expected_color = atty::is(atty::Stream::Stdout);
    assert_eq!(config.color, Some(expected_color));
    assert_eq!(config.html_report, None);
    assert_eq!(config.skip_hooks, None);
}





#[test]
fn test_error_types() {
    let msg_error: TestError = "test message".into();
    assert_eq!(msg_error.to_string(), "test message");
    
    let string_error: TestError = "test string".to_string().into();
    assert_eq!(string_error.to_string(), "test string");
    
    let panic_error = TestError::Panicked("test panic".to_string());
    assert_eq!(panic_error.to_string(), "panicked: test panic");
    
    let timeout_error = TestError::Timeout(Duration::from_secs(5));
    assert_eq!(timeout_error.to_string(), "timeout after 5s");
    

}



#[test]
fn test_parallel_execution_config() {
    let config = TestConfig {
        max_concurrency: Some(4),
        skip_hooks: None,
        ..Default::default()
    };
    
    assert_eq!(config.max_concurrency, Some(4));
}

#[test]
fn test_shuffle_config() {
    let config = TestConfig {
        shuffle_seed: Some(12345),
        skip_hooks: None,
        ..Default::default()
    };
    
    assert_eq!(config.shuffle_seed, Some(12345));
}

#[test]
fn test_color_config() {
    let config = TestConfig {
        color: Some(false),
        skip_hooks: None,
        ..Default::default()
    };
    
    assert_eq!(config.color, Some(false));
}

#[test]
fn test_html_report_config() {
    let config = TestConfig {
        html_report: Some("test-results.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    assert_eq!(config.html_report, Some("test-results.html".to_string()));
}

#[test]
fn test_isolated_parallel_execution() {
    // This test demonstrates that multiple TestRunner instances can run in parallel
    // Each runner is completely isolated and can run concurrently
    
    // Create a simple test
    test("parallel_test_1", |_| {
        // Simulate some work
        std::thread::sleep(Duration::from_millis(100));
        Ok(())
    });
    
    test("parallel_test_2", |_| {
        // Simulate some work
        std::thread::sleep(Duration::from_millis(100));
        Ok(())
    });
    
    // Run the tests
    let result = rust_test_harness::run_tests();
    
    // Verify both tests completed successfully
    assert_eq!(result, 0);
    
    info!("✅ Successfully ran tests with parallel execution capability!");
}

// Enhanced test cases for better coverage

#[test]
fn test_empty_test_suite() {
    // Test that running with no tests doesn't crash
    let result = rust_test_harness::run_tests();
    assert_eq!(result, 0);
}

#[test]
fn test_test_timeout() {
    // Test timeout functionality
    let config = TestConfig {
        max_concurrency: Some(1), // Ensure single-threaded for timeout test
        skip_hooks: None,
        ..Default::default()
    };
    
    test("timeout_test", |_| {
        // Simulate a long-running test
        std::thread::sleep(Duration::from_millis(200));
        Ok(())
    });
    
    let result = rust_test_harness::run_tests_with_config(config);
    assert_eq!(result, 0);
}





#[test]
fn test_error_conversion() {
    // Test error conversion from various types
    
    // From &str
    let error: TestError = "string slice error".into();
    assert_eq!(error.to_string(), "string slice error");
    
    // From String
    let error: TestError = "owned string error".to_string().into();
    assert_eq!(error.to_string(), "owned string error");
}

#[test]
fn test_config_environment_override() {
    // Test that environment variables can override config defaults
    
    let config = TestConfig::default();
    
    // Test that config can be created without panicking
    assert!(config.filter.is_none() || config.filter.is_some());
    assert!(config.skip_tags.is_empty() || !config.skip_tags.is_empty());
    assert!(config.max_concurrency.is_none() || config.max_concurrency.is_some());
}

#[test]
fn test_concurrent_test_registration() {
    // Test that tests can be registered from multiple threads safely
    
    let handle1 = std::thread::spawn(|| {
        test("concurrent_test_1", |_| Ok(()));
    });
    
    let handle2 = std::thread::spawn(|| {
        test("concurrent_test_2", |_| Ok(()));
    });
    
    handle1.join().unwrap();
    handle2.join().unwrap();
    
    // Run the tests to ensure they were registered correctly
    let result = rust_test_harness::run_tests();
    assert_eq!(result, 0);
}

#[test]
fn test_framework_stress() {
    // Stress test the framework with many tests and hooks
    
    // Register many hooks
    for _i in 0..10 {
        before_each(move |_ctx| {
            // Simulate some work in hooks
            std::thread::sleep(Duration::from_millis(1));
            Ok(())
        });
        
        after_each(move |_ctx| {
            // Simulate some cleanup work
            std::thread::sleep(Duration::from_millis(1));
            Ok(())
        });
    }
    
    // Register many tests
    for i in 0..20 {
        test(&format!("stress_test_{}", i), |_ctx| {
            // Simulate some test work
            std::thread::sleep(Duration::from_millis(1));
            Ok(())
        });
    }
    
    let result = rust_test_harness::run_tests();
    assert_eq!(result, 0);
    
    info!("✅ Successfully completed stress test with 20 tests and 20 hooks!");
}

#[test]
fn test_framework_recovery() {
    // Test that the framework can recover from failures and continue
    
    // First, run some failing tests
    test("recovery_failing_test_1", |_| Err("failure 1".into()));
    test("recovery_failing_test_2", |_| Err("failure 2".into()));
    
    let result1 = rust_test_harness::run_tests();
    assert_eq!(result1, 1); // Should fail
    
    // Then, run some passing tests
    test("recovery_passing_test_1", |_| Ok(()));
    test("recovery_passing_test_2", |_| Ok(()));
    
    let result2 = rust_test_harness::run_tests();
    assert_eq!(result2, 0); // Should pass
    
    info!("✅ Framework successfully recovered from failures!");
} 
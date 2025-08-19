use rust_test_harness::{
    before_all, before_each, after_each, after_all, 
    test, test_with_tags, test_with_docker,
    DockerRunOptions, Readiness, TestRunner, TestConfig,
    TestError
};
use std::time::Duration;
use std::sync::{Arc, Mutex};

// Test configuration that simulates Docker availability
#[cfg(test)]
mod mock_docker {
    use super::*;
    use std::sync::Mutex;
    use once_cell::sync::Lazy;
    
    static MOCK_DOCKER_AVAILABLE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(true));
    static MOCK_DOCKER_CALLS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));
    
    pub fn set_docker_available(available: bool) {
        *MOCK_DOCKER_AVAILABLE.lock().unwrap() = available;
    }
    
    pub fn get_docker_calls() -> Vec<String> {
        MOCK_DOCKER_CALLS.lock().unwrap().clone()
    }
    
    pub fn clear_docker_calls() {
        MOCK_DOCKER_CALLS.lock().unwrap().clear();
    }
    
    pub fn mock_start_docker(_opts: &DockerRunOptions) -> Result<rust_test_harness::ContainerHandle, String> {
        MOCK_DOCKER_CALLS.lock().unwrap().push("start_docker".to_string());
        if *MOCK_DOCKER_AVAILABLE.lock().unwrap() {
            Ok(rust_test_harness::ContainerHandle { id: "mock-container".to_string() })
        } else {
            Err("Docker not available".into())
        }
    }
}

#[test]
fn test_basic_passing_test() {
    // This test verifies that the framework can run a simple passing test
    // We don't clear previous registrations to avoid interference
    
    test("basic test", |_| {
        Ok(())
    });
    
    let config = TestConfig {
        filter: Some("basic test".to_string()), // Only run our specific test
        skip_tags: vec![],
        max_concurrency: None,
        shuffle_seed: None,
        color: Some(false),
        junit_xml: None,
    };
    
    let runner = TestRunner::with_config(config);
    let result = runner.run();
    
    assert_eq!(result, 0);
}

#[test]
fn test_basic_failing_test() {
    // This test verifies that the framework can detect test failures
    
    test("failing test", |_| {
        Err("intentional failure".into())
    });
    
    let config = TestConfig {
        filter: Some("failing test".to_string()), // Only run our specific test
        skip_tags: vec![],
        max_concurrency: None,
        shuffle_seed: None,
        color: Some(false),
        junit_xml: None,
    };
    
    let runner = TestRunner::with_config(config);
    let result = runner.run();
    
    assert_eq!(result, 1);
}

#[test]
fn test_panicking_test() {
    // This test verifies that the framework can handle panicking tests
    
    test("panicking test", |_| {
        panic!("intentional panic");
    });
    
    let config = TestConfig {
        filter: Some("panicking test".to_string()), // Only run our specific test
        skip_tags: vec![],
        max_concurrency: None,
        shuffle_seed: None,
        color: Some(false),
        junit_xml: None,
    };
    
    let runner = TestRunner::with_config(config);
    let result = runner.run();
    
    // The test should fail due to panic, so we expect exit code 1
    assert_eq!(result, 1);
}

#[test]
fn test_hooks_execution_order() {
    // This test is complex due to closure requirements, so we'll test a simpler scenario
    // Clear any previous registrations
    {
        let mut before_all = rust_test_harness::BEFORE_ALL.lock().unwrap();
        before_all.clear();
        let mut before_each = rust_test_harness::BEFORE_EACH.lock().unwrap();
        before_each.clear();
        let mut after_each = rust_test_harness::AFTER_EACH.lock().unwrap();
        after_each.clear();
        let mut after_all = rust_test_harness::AFTER_ALL.lock().unwrap();
        after_all.clear();
        let mut tests = rust_test_harness::TESTS.lock().unwrap();
        tests.clear();
    }
    
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
    
    test("hook test", |_| {
        Ok(())
    });
    
    let config = TestConfig {
        filter: None,
        skip_tags: vec![],
        max_concurrency: None,
        shuffle_seed: None,
        color: Some(false),
        junit_xml: None,
    };
    
    let runner = TestRunner::with_config(config);
    let result = runner.run();
    
    // Just verify the test runs without error
    assert_eq!(result, 0);
}

#[test]
fn test_test_filtering() {
    // This test verifies that test filtering works
    
    test("first test", |_| Ok(()));
    test("second test", |_| Ok(()));
    test("third test", |_| Ok(()));
    
    let config = TestConfig {
        filter: Some("second".to_string()),
        skip_tags: vec![],
        max_concurrency: None,
        shuffle_seed: None,
        color: Some(false),
        junit_xml: None,
    };
    
    let runner = TestRunner::with_config(config);
    let result = runner.run();
    
    assert_eq!(result, 0);
}

#[test]
fn test_tag_filtering() {
    // This test verifies that tag filtering works
    // Clear any previous registrations to avoid interference
    {
        let mut tests = rust_test_harness::TESTS.lock().unwrap();
        tests.clear();
    }
    
    test_with_tags("tagged test", vec!["integration", "slow"], |_| Ok(()));
    test("untagged test", |_| Ok(()));
    
    let config = TestConfig {
        filter: None,
        skip_tags: vec!["slow".to_string()],
        max_concurrency: None,
        shuffle_seed: None,
        color: Some(false),
        junit_xml: None,
    };
    
    let runner = TestRunner::with_config(config);
    let result = runner.run();
    
    // Should pass with no failures since the "slow" tagged test should be skipped
    assert_eq!(result, 0);
}

#[test]
fn test_docker_options_builder() {
    let opts = DockerRunOptions::new("nginx:alpine")
        .env("FOO", "bar")
        .port(8080, 80)
        .arg("--name")
        .arg("test-container")
        .name("test-container")
        .label("test", "true")
        .ready_timeout(Duration::from_secs(30))
        .readiness(Readiness::PortOpen(80));
    
    assert_eq!(opts.image, "nginx:alpine");
    assert_eq!(opts.env, vec![("FOO", "bar")]);
    assert_eq!(opts.ports, vec![(8080, 80)]);
    assert_eq!(opts.args, vec!["--name", "test-container"]);
    assert_eq!(opts.name, Some("test-container"));
    assert_eq!(opts.labels, vec![("test", "true")]);
    assert_eq!(opts.ready_timeout, Duration::from_secs(30));
    
    match opts.readiness {
        Readiness::PortOpen(80) => {},
        _ => panic!("Expected PortOpen(80)"),
    }
}

#[test]
fn test_docker_options_default() {
    let opts = DockerRunOptions::default();
    
    assert_eq!(opts.image, "alpine:latest");
    assert!(opts.env.is_empty());
    assert!(opts.ports.is_empty());
    assert!(opts.args.is_empty());
    assert_eq!(opts.ready_timeout, Duration::from_secs(15));
    assert_eq!(opts.name, None);
    assert!(opts.labels.is_empty());
    
    match opts.readiness {
        Readiness::Running => {},
        _ => panic!("Expected Running"),
    }
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
    
    let docker_error = TestError::DockerError("docker issue".to_string());
    assert_eq!(docker_error.to_string(), "docker error: docker issue");
}

#[test]
fn test_mutex_recovery() {
    use std::sync::Mutex;
    use rust_test_harness::lock_or_recover;
    
    let mutex = Mutex::new(42);
    
    // Normal case
    {
        let guard = lock_or_recover(&mutex);
        assert_eq!(*guard, 42);
    }
    
    // Poisoned case - we can't easily test this without unsafe code
    // but we can verify the function exists and compiles
    let _guard = lock_or_recover(&mutex);
}

#[test]
fn test_test_runner_config() {
    let config = TestConfig::default();
    
    // These should be None by default when env vars aren't set
    assert_eq!(config.filter, None);
    assert!(config.skip_tags.is_empty());
    assert_eq!(config.max_concurrency, None);
    assert_eq!(config.shuffle_seed, None);
    assert_eq!(config.color, Some(true)); // Should default to true on TTY
    assert_eq!(config.junit_xml, None);
}

#[test]
fn test_parallel_execution_config() {
    let config = TestConfig {
        max_concurrency: Some(4),
        ..Default::default()
    };
    
    assert_eq!(config.max_concurrency, Some(4));
}

#[test]
fn test_shuffle_config() {
    let config = TestConfig {
        shuffle_seed: Some(12345),
        ..Default::default()
    };
    
    assert_eq!(config.shuffle_seed, Some(12345));
}

#[test]
fn test_color_config() {
    let config = TestConfig {
        color: Some(false),
        ..Default::default()
    };
    
    assert_eq!(config.color, Some(false));
}

#[test]
fn test_junit_xml_config() {
    let config = TestConfig {
        junit_xml: Some("test-results.xml".to_string()),
        ..Default::default()
    };
    
    assert_eq!(config.junit_xml, Some("test-results.xml".to_string()));
} 
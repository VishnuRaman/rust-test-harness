//! HTML Reporting Example
//! 
//! This example demonstrates how to generate beautiful, interactive HTML reports
//! for your test results using the rust-test-harness framework.
//! 
//! Features demonstrated:
//! - Basic HTML report generation
//! - Environment variable configuration
//! - Custom configuration with HTML reporting
//! - Different test scenarios (pass, fail, skip)
//! - Test metadata (tags, timeouts, Docker)

use rust_test_harness::{
    test, run_tests_with_config, TestConfig, DockerRunOptions
};
use std::time::Duration;

fn main() {
    println!("🧪 HTML Reporting Example");
    println!("=========================");
    println!();
    
    // Example 1: Basic HTML Report Generation
    println!("📊 Example 1: Basic HTML Report");
    println!("Generating basic HTML report...");
    
    // Register some tests
    test("basic_passing_test", |_| Ok(()));
    test("another_passing_test", |_| Ok(()));
    
    let config = TestConfig {
        html_report: Some("basic_report.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    println!("✅ Basic report generated with exit code: {}", result);
    println!();
    
    // Example 2: HTML Report with Mixed Results
    println!("📊 Example 2: Mixed Results Report");
    println!("Generating report with pass/fail/skip results...");
    
    // Clear previous tests and register new ones
    test("successful_test", |_| Ok(()));
    test("failing_test", |_| Err("intentional failure".into()));
    test("skipped_test", |_| Ok(()));
    
    let config = TestConfig {
        html_report: Some("mixed_results_report.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    println!("✅ Mixed results report generated with exit code: {}", result);
    println!();
    
    // Example 3: HTML Report with Rich Metadata
    println!("📊 Example 3: Rich Metadata Report");
    println!("Generating report with tags, timeouts, and Docker...");
    
    // Clear previous tests and register new ones with realistic scenarios
    test("tagged_test", |_| Ok(()));
    test("timeout_test", |_| Ok(()));
    test("docker_integration_test", |_| Ok(()));
    test("database_connection_test", |_| Ok(()));
    test("api_endpoint_test", |_| Ok(()));
    
    // Note: In a real scenario, you'd use test_with_tags and test_with_docker
    // For this example, we'll simulate the metadata by showing different test types
    
    let config = TestConfig {
        html_report: Some("rich_metadata_report.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    println!("✅ Rich metadata report generated with exit code: {}", result);
    println!("   📊 5 tests with different types (tags, timeouts, Docker, DB, API)");
    println!();
    
    // Example 4: Large Test Suite Report
    println!("📊 Example 4: Large Test Suite Report");
    println!("Generating report for many tests with realistic mixed results...");
    
    // Clear previous tests and register many tests with realistic scenarios
    for i in 0..25 {
        match i {
            // Some tests pass normally
            0..=15 => {
                test(&format!("large_suite_test_{}", i), |_| Ok(()));
            },
            // Some tests fail with different error types
            16..=19 => {
                test(&format!("large_suite_test_{}", i), move |_| {
                    Err(format!("Test {} failed due to assertion error", i).into())
                });
            },
            // Some tests have timeouts
            20..=22 => {
                test(&format!("large_suite_test_{}", i), move |_| {
                    // Simulate a test that takes too long and fails due to timeout
                    std::thread::sleep(Duration::from_millis(50));
                    Err(format!("Test {} failed due to timeout (exceeded 30ms limit)", i).into())
                });
            },
            // Some tests panic
            23 => {
                test(&format!("large_suite_test_{}", i), move |_| {
                    panic!("Test {} panicked due to unexpected condition", i);
                });
            },
            // Some tests are skipped (simulated by returning error)
            24 => {
                test(&format!("large_suite_test_{}", i), |_| {
                    Err("Test skipped due to missing dependencies".into())
                });
            },
            _ => unreachable!(),
        }
    }
    
    let config = TestConfig {
        html_report: Some("large_suite_report.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    println!("✅ Large suite report generated with exit code: {}", result);
    println!("   📊 16 tests passed, 7 failed (4 errors + 3 timeouts), 1 panic, 1 skipped");
    println!();
    
    // Example 5: Environment Variable Configuration
    println!("📊 Example 5: Environment Variable Configuration");
    println!("Setting TEST_HTML_REPORT environment variable...");
    
    // Set environment variable
    std::env::set_var("TEST_HTML_REPORT", "env_var_report.html");
    
    // Clear previous tests and register new ones
    test("env_test", |_| Ok(()));
    
    let config = TestConfig::default();
    println!("📝 Config HTML report path: {:?}", config.html_report);
    
    let result = run_tests_with_config(config);
    println!("✅ Environment variable report generated with exit code: {}", result);
    println!();
    
    // Example 6: Performance Testing Report
    println!("📊 Example 6: Performance Testing Report");
    println!("Generating report for performance tests with realistic scenarios...");
    
    // Clear previous tests and register performance tests with mixed results
    for i in 0..15 {
        match i {
            // Fast tests that pass
            0..=8 => {
                test(&format!("perf_test_{}", i), |_| {
                    // Simulate some work
                    std::thread::sleep(Duration::from_millis(5));
                    Ok(())
                });
            },
            // Medium tests that pass
            9..=11 => {
                test(&format!("perf_test_{}", i), |_| {
                    // Simulate medium work
                    std::thread::sleep(Duration::from_millis(20));
                    Ok(())
                });
            },
            // Slow tests that pass
            12..=13 => {
                test(&format!("perf_test_{}", i), |_| {
                    // Simulate slow but successful tests
                    std::thread::sleep(Duration::from_millis(50));
                    Ok(())
                });
            },
            // One test that fails due to timeout
            14 => {
                test(&format!("perf_test_{}", i), |_| {
                    // Simulate a test that takes too long and fails due to timeout
                    std::thread::sleep(Duration::from_millis(100));
                    Err("Performance test exceeded expected time limit (50ms)".into())
                });
            },
            _ => unreachable!(),
        }
    }
    
    let config = TestConfig {
        html_report: Some("performance_report.html".to_string()),
        max_concurrency: Some(4),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    println!("✅ Performance report generated with exit code: {}", result);
    println!("   📊 14 tests passed, 1 failed (timeout)");
    println!();
    
    // Summary
    println!("🎉 HTML Reporting Examples Complete!");
    println!("=====================================");
    println!();
    println!("Generated HTML reports:");
    println!("  📄 basic_report.html - Basic functionality");
    println!("  📄 mixed_results_report.html - Pass/fail/skip results");
    println!("  📄 rich_metadata_report.html - Rich test metadata");
    println!("  📄 large_suite_report.html - Large test suite");
    println!("  📄 env_var_report.html - Environment variable config");
    println!("  📄 performance_report.html - Performance testing");
    println!();
    println!("📖 HTML Report Features:");
    println!("  🔽 Expandable test details - Click any test to expand");
    println!("  🔍 Real-time search - Search by name, status, or tags");
    println!("  ⌨️  Keyboard shortcuts - Ctrl+F (search), Ctrl+A (expand all)");
    println!("  🚨 Auto-expand failed - Failed tests automatically expand");
    println!("  📱 Responsive design - Works on all devices");
    println!();
    println!("💡 Usage Tips:");
    println!("  • Open any .html file in your web browser");
    println!("  • Use Ctrl+F to search for specific tests");
    println!("  • Click test headers to expand/collapse details");
    println!("  • Failed tests are automatically expanded for visibility");
    println!("  • Reports work great in CI/CD pipelines and team sharing");
    println!();
    println!("🔧 Configuration Options:");
    println!("  • Set TEST_HTML_REPORT environment variable");
    println!("  • Use TestConfig.html_report for programmatic control");
    println!("  • Combine with other config options (filtering, concurrency)");
} 
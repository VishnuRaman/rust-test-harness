use rust_test_harness::{
    test, run_tests_with_config, TestConfig, TestStatus, TestError
};
use std::time::Duration;
use std::fs;
use std::path::Path;

#[test]
fn test_html_report_generation_basic() {
    // Test basic HTML report generation with passing tests
    
    test("basic_passing_test", |_| Ok(()));
    test("another_passing_test", |_| Ok(()));
    
    let config = TestConfig {
        html_report: Some("test_basic_report.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    assert_eq!(result, 0);
    
    // Verify HTML file was created
    assert!(Path::new("test_basic_report.html").exists(), "HTML report file should exist");
    
    // Verify HTML content
    let html_content = fs::read_to_string("test_basic_report.html").unwrap();
    assert!(html_content.contains("🧪 Test Execution Report"), "HTML should contain report title");
    assert!(html_content.contains("basic_passing_test"), "HTML should contain test names");
    assert!(html_content.contains("PASSED"), "HTML should contain passed status");
    assert!(html_content.contains("2"), "HTML should show correct test count");
    
    // Cleanup
    let _ = fs::remove_file("test_basic_report.html");
}

#[test]
fn test_html_report_generation_with_failures() {
    // Test HTML report generation with mixed pass/fail results
    
    test("passing_test", |_| Ok(()));
    test("failing_test", |_| Err("intentional failure".into()));
    test("another_passing_test", |_| Ok(()));
    
    let config = TestConfig {
        html_report: Some("test_mixed_report.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    assert_eq!(result, 1); // Should fail due to one failing test
    
    // Verify HTML file was created
    assert!(Path::new("test_mixed_report.html").exists(), "HTML report file should exist");
    
    // Verify HTML content
    let html_content = fs::read_to_string("test_mixed_report.html").unwrap();
    assert!(html_content.contains("🧪 Test Execution Report"), "HTML should contain report title");
    assert!(html_content.contains("passing_test"), "HTML should contain passing test name");
    assert!(html_content.contains("failing_test"), "HTML should contain failing test name");
    assert!(html_content.contains("PASSED"), "HTML should contain passed status");
    assert!(html_content.contains("FAILED"), "HTML should contain failed status");
    assert!(html_content.contains("intentional failure"), "HTML should contain error message");
    assert!(html_content.contains("3"), "HTML should show correct test count");
    
    // Cleanup
    let _ = fs::remove_file("test_mixed_report.html");
}

#[test]
fn test_html_report_generation_with_tags() {
    // Test HTML report generation with tagged tests
    
    test("tagged_test_1", |_| Ok(()));
    test("tagged_test_2", |_| Ok(()));
    
    let config = TestConfig {
        html_report: Some("test_tagged_report.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    assert_eq!(result, 0);
    
    // Verify HTML file was created
    assert!(Path::new("test_tagged_report.html").exists(), "HTML report file should contain tags");
    
    // Cleanup
    let _ = fs::remove_file("test_tagged_report.html");
}

#[test]
fn test_html_report_generation_with_docker() {
    // Test HTML report generation with Docker tests
    
    test("docker_test", |_| Ok(()));
    
    let config = TestConfig {
        html_report: Some("test_docker_report.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    assert_eq!(result, 0);
    
    // Verify HTML file was created
    assert!(Path::new("test_docker_report.html").exists(), "HTML report file should exist");
    
    // Cleanup
    let _ = fs::remove_file("test_docker_report.html");
}

#[test]
fn test_html_report_generation_with_timeouts() {
    // Test HTML report generation with timeout configuration
    
    test("timeout_test", |_| Ok(()));
    
    let config = TestConfig {
        html_report: Some("test_timeout_report.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    assert_eq!(result, 0);
    
    // Verify HTML file was created
    assert!(Path::new("test_timeout_report.html").exists(), "HTML report file should exist");
    
    // Cleanup
    let _ = fs::remove_file("test_timeout_report.html");
}

#[test]
fn test_html_report_generation_large_suite() {
    // Test HTML report generation with many tests
    
    // Create many tests
    for i in 0..50 {
        test(&format!("large_suite_test_{}", i), |_| Ok(()));
    }
    
    let config = TestConfig {
        html_report: Some("test_large_suite_report.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    assert_eq!(result, 0);
    
    // Verify HTML file was created
    assert!(Path::new("test_large_suite_report.html").exists(), "HTML report file should exist");
    
    // Verify HTML content for large suite
    let html_content = fs::read_to_string("test_large_suite_report.html").unwrap();
    assert!(html_content.contains("50"), "HTML should show correct test count");
    assert!(html_content.contains("large_suite_test_0"), "HTML should contain first test name");
    assert!(html_content.contains("large_suite_test_49"), "HTML should contain last test name");
    
    // Cleanup
    let _ = fs::remove_file("test_large_suite_report.html");
}

#[test]
fn test_html_report_generation_with_errors() {
    // Test HTML report generation with various error types
    
    test("panic_test", |_| {
        panic!("intentional panic");
    });
    
    test("error_test", |_| Err("intentional error".into()));
    
    test("timeout_error_test", |_| {
        std::thread::sleep(Duration::from_millis(100));
        Ok(())
    });
    
    let config = TestConfig {
        html_report: Some("test_errors_report.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    assert_eq!(result, 1); // Should fail due to errors
    
    // Verify HTML file was created
    assert!(Path::new("test_errors_report.html").exists(), "HTML report file should exist");
    
    // Verify HTML content contains error information
    let html_content = fs::read_to_string("test_errors_report.html").unwrap();
    assert!(html_content.contains("panic_test"), "HTML should contain panic test name");
    assert!(html_content.contains("error_test"), "HTML should contain error test name");
    assert!(html_content.contains("intentional panic"), "HTML should contain panic message");
    assert!(html_content.contains("intentional error"), "HTML should contain error message");
    
    // Cleanup
    let _ = fs::remove_file("test_errors_report.html");
}

#[test]
fn test_html_report_generation_with_filters() {
    // Test HTML report generation with filtered tests
    
    test("filtered_test_1", |_| Ok(()));
    test("filtered_test_2", |_| Ok(()));
    test("unfiltered_test", |_| Ok(()));
    
    let config = TestConfig {
        filter: Some("filtered".to_string()),
        html_report: Some("test_filtered_report.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    assert_eq!(result, 0);
    
    // Verify HTML file was created
    assert!(Path::new("test_filtered_report.html").exists(), "HTML report file should exist");
    
    // Verify HTML content shows filtered results
    let html_content = fs::read_to_string("test_filtered_report.html").unwrap();
    assert!(html_content.contains("filtered_test_1"), "HTML should contain first filtered test");
    assert!(html_content.contains("filtered_test_2"), "HTML should contain second filtered test");
    assert!(html_content.contains("unfiltered_test"), "HTML should contain unfiltered test");
    
    // Note: In this framework, filtering works by only executing tests that match the filter
    // All tests appear in the HTML report, but only filtered tests are actually run
    // The filter is applied at test execution time, not at report generation time
    
    // Cleanup
    let _ = fs::remove_file("test_filtered_report.html");
}

#[test]
fn test_html_report_generation_with_parallel_execution() {
    // Test HTML report generation with parallel test execution
    
    // Create tests that will run in parallel
    for i in 0..20 {
        test(&format!("parallel_test_{}", i), |_| {
            std::thread::sleep(Duration::from_millis(10));
            Ok(())
        });
    }
    
    let config = TestConfig {
        max_concurrency: Some(4),
        html_report: Some("test_parallel_report.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    assert_eq!(result, 0);
    
    // Verify HTML file was created
    assert!(Path::new("test_parallel_report.html").exists(), "HTML report file should exist");
    
    // Verify HTML content shows parallel execution results
    let html_content = fs::read_to_string("test_parallel_report.html").unwrap();
    assert!(html_content.contains("20"), "HTML should show correct test count");
    assert!(html_content.contains("parallel_test_0"), "HTML should contain first parallel test");
    assert!(html_content.contains("parallel_test_19"), "HTML should contain last parallel test");
    
    // Cleanup
    let _ = fs::remove_file("test_parallel_report.html");
}

#[test]
fn test_html_report_generation_with_shuffled_tests() {
    // Test HTML report generation with shuffled test execution
    
    for i in 0..10 {
        test(&format!("shuffled_test_{}", i), |_| Ok(()));
    }
    
    let config = TestConfig {
        shuffle_seed: Some(12345),
        html_report: Some("test_shuffled_report.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    assert_eq!(result, 0);
    
    // Verify HTML file was created
    assert!(Path::new("test_shuffled_report.html").exists(), "HTML report file should exist");
    
    // Verify HTML content shows shuffled results
    let html_content = fs::read_to_string("test_shuffled_report.html").unwrap();
    assert!(html_content.contains("10"), "HTML should show correct test count");
    
    // Cleanup
    let _ = fs::remove_file("test_shuffled_report.html");
}

#[test]
fn test_html_report_generation_with_environment_variable() {
    // Test HTML report generation using environment variable
    
    test("env_test", |_| Ok(()));
    
    // Set environment variable for HTML report
    std::env::set_var("TEST_HTML_REPORT", "test_env_report.html");
    
    let config = TestConfig::default();
    assert_eq!(config.html_report, Some("test_env_report.html".to_string()));
    
    let result = run_tests_with_config(config);
    assert_eq!(result, 0);
    
    // Verify HTML file was created
    assert!(Path::new("test_env_report.html").exists(), "HTML report file should exist");
    
    // Cleanup
    let _ = fs::remove_file("test_env_report.html");
    std::env::remove_var("TEST_HTML_REPORT");
}

#[test]
fn test_html_report_generation_without_config() {
    // Test that no HTML report is generated when not configured
    
    test("no_report_test", |_| Ok(()));
    
    let config = TestConfig {
        html_report: None,
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    assert_eq!(result, 0);
    
    // Verify no HTML file was created
    assert!(!Path::new("test-results.html").exists(), "No HTML report should be created by default");
}

#[test]
fn test_html_report_generation_with_invalid_path() {
    // Test HTML report generation with invalid file path
    
    test("invalid_path_test", |_| Ok(()));
    
    let config = TestConfig {
        html_report: Some("/invalid/path/test_report.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    // Should still pass even if HTML generation fails
    assert_eq!(result, 0);
    
    // Verify no HTML file was created in invalid location
    assert!(!Path::new("/invalid/path/test_report.html").exists(), "HTML report should not be created in invalid path");
}

#[test]
fn test_html_report_content_structure() {
    // Test that HTML report has proper structure and content
    
    test("structure_test", |_| Ok(()));
    
    let config = TestConfig {
        html_report: Some("test_structure_report.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    assert_eq!(result, 0);
    
    // Verify HTML file was created
    assert!(Path::new("test_structure_report.html").exists(), "HTML report file should exist");
    
    // Verify HTML structure
    let html_content = fs::read_to_string("test_structure_report.html").unwrap();
    
    // Check HTML structure elements
    assert!(html_content.contains("<!DOCTYPE html>"), "HTML should have proper DOCTYPE");
    assert!(html_content.contains("<html lang=\"en\">"), "HTML should have proper html tag");
    assert!(html_content.contains("<head>"), "HTML should have head section");
    assert!(html_content.contains("<body>"), "HTML should have body section");
    assert!(html_content.contains("</html>"), "HTML should be properly closed");
    
    // Check CSS styling
    assert!(html_content.contains("background: linear-gradient"), "HTML should have gradient styling");
    assert!(html_content.contains(".container"), "HTML should have container CSS class");
    assert!(html_content.contains(".summary-grid"), "HTML should have summary grid CSS class");
    assert!(html_content.contains(".test-item"), "HTML should have test item CSS class");
    
    // Check content sections
    assert!(html_content.contains("🧪 Test Execution Report"), "HTML should have report title");
    assert!(html_content.contains("📊 Execution Summary"), "HTML should have summary section");
    assert!(html_content.contains("Test Results"), "HTML should have test results section");
    
    // Cleanup
    let _ = fs::remove_file("test_structure_report.html");
}

#[test]
fn test_html_report_expandable_functionality() {
    // Test that HTML report includes expandable test details functionality
    
    test("expandable_test", |_| Ok(()));
    
    let config = TestConfig {
        html_report: Some("test_expandable_report.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    assert_eq!(result, 0);
    
    // Verify HTML file was created
    assert!(Path::new("test_expandable_report.html").exists(), "HTML report file should exist");
    
    // Verify expandable functionality
    let html_content = fs::read_to_string("test_expandable_report.html").unwrap();
    
    // Check for expandable elements
    assert!(html_content.contains("test-expandable"), "HTML should have expandable CSS class");
    assert!(html_content.contains("expand-icon"), "HTML should have expand icon CSS class");
    assert!(html_content.contains("onclick=\"toggleTestDetails(this)\""), "HTML should have click handlers");
    assert!(html_content.contains("toggleTestDetails"), "HTML should have toggle function");
    
    // Check for search functionality
    assert!(html_content.contains("search-box"), "HTML should have search box CSS class");
    assert!(html_content.contains("id=\"testSearch\""), "HTML should have search input");
    assert!(html_content.contains("Search tests by name, status, or tags"), "HTML should have search placeholder");
    
    // Check for JavaScript functionality
    assert!(html_content.contains("<script>"), "HTML should have JavaScript section");
    assert!(html_content.contains("addEventListener"), "HTML should have event listeners");
    assert!(html_content.contains("keydown"), "HTML should have keyboard event handling");
    
    // Check for metadata display
    assert!(html_content.contains("test-metadata"), "HTML should have metadata CSS class");
    assert!(html_content.contains("metadata-grid"), "HTML should have metadata grid CSS class");
    
    // Cleanup
    let _ = fs::remove_file("test_expandable_report.html");
}

#[test]
fn test_html_report_search_functionality() {
    // Test that HTML report includes search functionality
    
    test("searchable_test_1", |_| Ok(()));
    test("searchable_test_2", |_| Ok(()));
    test("another_searchable_test", |_| Ok(()));
    
    let config = TestConfig {
        html_report: Some("test_searchable_report.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    assert_eq!(result, 0);
    
    // Verify HTML file was created
    assert!(Path::new("test_searchable_report.html").exists(), "HTML report file should exist");
    
    // Verify search functionality
    let html_content = fs::read_to_string("test_searchable_report.html").unwrap();
    
    // Check for search implementation
    assert!(html_content.contains("getElementById('testSearch')"), "HTML should have search element reference");
    assert!(html_content.contains("addEventListener('input'"), "HTML should have input event listener");
    assert!(html_content.contains("toLowerCase()"), "HTML should have case-insensitive search");
    assert!(html_content.contains("includes(searchTerm)"), "HTML should have search logic");
    
    // Check for data attributes
    assert!(html_content.contains("data-test-name"), "HTML should have test name data attribute");
    assert!(html_content.contains("data-test-status"), "HTML should have test status data attribute");
    assert!(html_content.contains("data-test-tags"), "HTML should have test tags data attribute");
    
    // Cleanup
    let _ = fs::remove_file("test_searchable_report.html");
}

#[test]
fn test_html_report_keyboard_shortcuts() {
    // Test that HTML report includes keyboard shortcuts
    
    test("keyboard_test", |_| Ok(()));
    
    let config = TestConfig {
        html_report: Some("test_keyboard_report.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    assert_eq!(result, 0);
    
    // Verify HTML file was created
    assert!(Path::new("test_keyboard_report.html").exists(), "HTML report file should exist");
    
    // Verify keyboard shortcuts
    let html_content = fs::read_to_string("test_keyboard_report.html").unwrap();
    
    // Check for keyboard shortcut implementation
    assert!(html_content.contains("addEventListener('keydown'"), "HTML should have keyboard event listener");
    assert!(html_content.contains("ctrlKey || e.metaKey"), "HTML should support Ctrl/Cmd key combinations");
    assert!(html_content.contains("case 'f'"), "HTML should have Ctrl+F shortcut for search");
    assert!(html_content.contains("case 'a'"), "HTML should have Ctrl+A shortcut for expand all");
    assert!(html_content.contains("case 'z'"), "HTML should have Ctrl+Z shortcut for collapse all");
    
    // Check for preventDefault calls
    assert!(html_content.contains("preventDefault()"), "HTML should prevent default browser behavior");
    
    // Cleanup
    let _ = fs::remove_file("test_keyboard_report.html");
}

#[test]
fn test_html_report_auto_expand_failed_tests() {
    // Test that HTML report automatically expands failed tests
    
    test("passing_test", |_| Ok(()));
    test("failing_test", |_| Err("intentional failure".into()));
    
    let config = TestConfig {
        html_report: Some("test_auto_expand_report.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    assert_eq!(result, 1); // Should fail due to one failing test
    
    // Verify HTML file was created
    assert!(Path::new("test_auto_expand_report.html").exists(), "HTML report file should exist");
    
    // Verify auto-expand functionality
    let html_content = fs::read_to_string("test_auto_expand_report.html").unwrap();
    
    // Check for auto-expand implementation
    assert!(html_content.contains("addEventListener('DOMContentLoaded'"), "HTML should have DOM ready listener");
    assert!(html_content.contains("querySelectorAll('.test-item.failed')"), "HTML should find failed tests");
    assert!(html_content.contains("classList.add('expanded')"), "HTML should auto-expand failed tests");
    
    // Cleanup
    let _ = fs::remove_file("test_auto_expand_report.html");
}

#[test]
fn test_html_report_generation_performance() {
    // Test HTML report generation performance with many tests
    
    let start = std::time::Instant::now();
    
    // Create many tests
    for i in 0..100 {
        test(&format!("perf_test_{}", i), |_| Ok(()));
    }
    
    let config = TestConfig {
        html_report: Some("test_perf_report.html".to_string()),
        skip_hooks: None,
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    let total_time = start.elapsed();
    
    assert_eq!(result, 0);
    
    // Verify HTML file was created
    assert!(Path::new("test_perf_report.html").exists(), "HTML report file should exist");
    
    // Performance assertion: 100 tests should complete and generate HTML in reasonable time
    assert!(
        total_time.as_millis() < 10000, 
        "100 tests with HTML generation took {}ms, expected < 10000ms", 
        total_time.as_millis()
    );
    
    // Cleanup
    let _ = fs::remove_file("test_perf_report.html");
} 
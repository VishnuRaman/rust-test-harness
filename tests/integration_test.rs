use rust_test_harness::{before_all, before_each, after_each, after_all, test, run_all};

#[test]
fn test_framework_integration() {
    // This test verifies that the framework can be used and runs without crashing
    // We'll just check that the basic functionality works
    
    // Clear any previous state
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
    
    // Register a simple test
    test("integration test", |_| {
        Ok(())
    });
    
    // Run the framework
    let result = run_all();
    
    // Should pass with no failures
    assert_eq!(result, 0);
} 
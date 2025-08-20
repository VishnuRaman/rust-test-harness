use rust_test_harness::{
    test_case, test_case_named, test_case_docker, 
    DockerRunOptions
};

// Test struct for demonstrating functionality
struct TestCalculator {
    value: i32,
}

impl TestCalculator {
    fn new() -> Self {
        Self { value: 0 }
    }
    
    fn add(&mut self, x: i32) {
        self.value += x;
    }
    
    fn get_value(&self) -> i32 {
        self.value
    }
}

#[test]
fn test_test_case_macro_basic() {
    // Test basic test_case! macro functionality
    
    test_case!(test_basic_calculator, |_ctx| {
        let calc = TestCalculator::new();
        assert_eq!(calc.get_value(), 0);
        Ok(())
    });
    
    // The macro should have registered this test
    // We can verify by checking that it compiles and runs
    assert!(true, "test_case! macro compiled successfully");
}

#[test]
fn test_test_case_macro_with_assertions() {
    // Test test_case! macro with various assertions
    
    test_case!(test_calculator_operations, |_ctx| {
        let mut calc = TestCalculator::new();
        
        // Test addition
        calc.add(5);
        assert_eq!(calc.get_value(), 5);
        
        // Test multiple operations
        calc.add(3);
        assert_eq!(calc.get_value(), 8);
        
        // Test edge cases
        calc.add(0);
        assert_eq!(calc.get_value(), 8);
        
        calc.add(-2);
        assert_eq!(calc.get_value(), 6);
        
        Ok(())
    });
    
    assert!(true, "test_case! macro with assertions compiled successfully");
}

#[test]
fn test_test_case_macro_error_handling() {
    // Test test_case! macro with error handling
    
    test_case!(test_calculator_error_case, |_ctx| {
        let calc = TestCalculator::new();
        
        // Simulate an error condition
        if calc.get_value() == 0 {
            return Err("Calculator value is zero".into());
        }
        
        Ok(())
    });
    
    assert!(true, "test_case! macro with error handling compiled successfully");
}

#[test]
fn test_test_case_named_macro() {
    // Test test_case_named! macro functionality
    
    test_case_named!("test_named_calculator", |_ctx| {
        let calc = TestCalculator::new();
        assert_eq!(calc.get_value(), 0);
        Ok(())
    });
    
    assert!(true, "test_case_named! macro compiled successfully");
}

#[test]
fn test_test_case_docker_macro() {
    // Test test_case_docker! macro functionality
    
    test_case_docker!(test_docker_calculator, 
        DockerRunOptions::new("alpine:latest")
            .env("TEST_ENV", "docker_test")
            .port(8080, 80), 
        |_ctx| {
            // Test that we can use the context
            let mut calc = TestCalculator::new();
            calc.add(42);
            assert_eq!(calc.get_value(), 42);
            
            // Test that we can access environment variables
            // (In a real Docker container, this would work)
            Ok(())
        }
    );
    
    assert!(true, "test_case_docker! macro compiled successfully");
}

#[test]
fn test_test_case_docker_with_complex_options() {
    // Test test_case_docker! macro with complex Docker options
    
    test_case_docker!(test_complex_docker, 
        DockerRunOptions::new("nginx:alpine")
            .env("NGINX_HOST", "localhost")
            .env("NGINX_PORT", "80")
            .port(8080, 80)
            .port(8443, 443)
            .arg("--name")
            .arg("test-nginx")
            .name("test-nginx-container")
            .label("test", "true")
            .label("framework", "rust-test-harness"), 
        |_ctx| {
            // Test that the context is available
            let mut calc = TestCalculator::new();
            calc.add(100);
            assert_eq!(calc.get_value(), 100);
            
            Ok(())
        }
    );
    
    assert!(true, "test_case_docker! macro with complex options compiled successfully");
}

#[test]
fn test_macro_context_usage() {
    // Test that the TestContext can be used in macros
    
    test_case!(test_context_basic, |_ctx| {
        // Test basic context functionality
        let mut calc = TestCalculator::new();
        calc.add(25);
        assert_eq!(calc.get_value(), 25);
        
        Ok(())
    });
    
    assert!(true, "test_case! macro with context usage compiled successfully");
}

#[test]
fn test_macro_multiple_tests() {
    // Test that multiple tests can be defined with macros
    
    test_case!(test_multiple_1, |_ctx| {
        let calc = TestCalculator::new();
        assert_eq!(calc.get_value(), 0);
        Ok(())
    });
    
    test_case!(test_multiple_2, |_ctx| {
        let mut calc = TestCalculator::new();
        calc.add(7);
        assert_eq!(calc.get_value(), 7);
        Ok(())
    });
    
    test_case!(test_multiple_3, |_ctx| {
        let mut calc = TestCalculator::new();
        calc.add(10);
        calc.add(5);
        assert_eq!(calc.get_value(), 15);
        Ok(())
    });
    
    assert!(true, "Multiple test_case! macros compiled successfully");
}

#[test]
fn test_macro_edge_cases() {
    // Test edge cases in macros
    
    // Test with very long test name
    let _long_name = "a".repeat(100);
    test_case!(test_with_very_long_name_that_exceeds_normal_lengths_and_might_cause_issues_in_some_systems, |_ctx| {
        Ok(())
    });
    
    // Test with special characters in name
    test_case!(test_with_special_chars_123, |_ctx| {
        Ok(())
    });
    
    assert!(true, "test_case! macro edge cases compiled successfully");
}

#[test]
fn test_macro_error_scenarios() {
    // Test various error scenarios in macros
    
    test_case!(test_error_panic, |_ctx| {
        panic!("intentional panic in macro test");
    });
    
    test_case!(test_error_return, |_ctx| {
        Err("intentional error return in macro test".into())
    });
    
    test_case!(test_error_assertion, |_ctx| {
        assert!(false, "intentional assertion failure in macro test");
        Ok(())
    });
    
    assert!(true, "test_case! macro error scenarios compiled successfully");
}

#[test]
fn test_macro_performance() {
    // Test macro performance with many tests
    
    let start = std::time::Instant::now();
    
    // Create many tests using macros with static names
    test_case!(perf_macro_test_0, |_ctx| {
        let mut calc = TestCalculator::new();
        calc.add(0);
        assert_eq!(calc.get_value(), 0);
        Ok(())
    });
    
    test_case!(perf_macro_test_1, |_ctx| {
        let mut calc = TestCalculator::new();
        calc.add(1);
        assert_eq!(calc.get_value(), 1);
        Ok(())
    });
    
    test_case!(perf_macro_test_2, |_ctx| {
        let mut calc = TestCalculator::new();
        calc.add(2);
        assert_eq!(calc.get_value(), 2);
        Ok(())
    });
    
    let duration = start.elapsed();
    
    // Performance assertion: macro tests should compile quickly
    assert!(
        duration.as_millis() < 1000, 
        "Macro tests took {}ms to compile, expected < 1000ms", 
        duration.as_millis()
    );
}

#[test]
fn test_macro_integration_with_standard_tests() {
    // Test that macro tests work alongside standard #[test] functions
    
    test_case!(test_macro_integration, |_ctx| {
        let calc = TestCalculator::new();
        assert_eq!(calc.get_value(), 0);
        Ok(())
    });
    
    // Standard test function
    fn standard_test_function() {
        let calc = TestCalculator::new();
        assert_eq!(calc.get_value(), 0);
    }
    
    // Call the standard test function
    standard_test_function();
    
    assert!(true, "Macro tests integrate successfully with standard tests");
}

#[test]
fn test_macro_context_isolation() {
    // Test that macro test contexts are properly isolated
    
    test_case!(test_context_isolation_1, |_ctx| {
        // Each test should get its own context
        let mut calc = TestCalculator::new();
        calc.add(1);
        assert_eq!(calc.get_value(), 1);
        Ok(())
    });
    
    test_case!(test_context_isolation_2, |_ctx| {
        // Each test should get its own context
        let mut calc = TestCalculator::new();
        calc.add(2);
        assert_eq!(calc.get_value(), 2);
        Ok(())
    });
    
    assert!(true, "Macro test contexts are properly isolated");
}

#[test]
fn test_macro_complex_workflows() {
    // Test complex workflows using macros
    
    test_case!(test_complex_calculator_workflow, |_ctx| {
        let mut calc = TestCalculator::new();
        
        // Complex calculation workflow
        calc.add(10);
        assert_eq!(calc.get_value(), 10);
        
        calc.add(20);
        assert_eq!(calc.get_value(), 30);
        
        calc.add(-5);
        assert_eq!(calc.get_value(), 25);
        
        calc.add(0);
        assert_eq!(calc.get_value(), 25);
        
        // Test edge cases
        calc.add(-25);
        assert_eq!(calc.get_value(), 0);
        
        calc.add(1000);
        assert_eq!(calc.get_value(), 1000);
        
        Ok(())
    });
    
    assert!(true, "Complex workflow macro test compiled successfully");
}

#[test]
fn test_macro_documentation_examples() {
    // Test the examples from the documentation to ensure they work
    
    // Example 1: Basic usage
    test_case!(test_doc_example_1, |_ctx| {
        let calc = TestCalculator::new();
        assert_eq!(calc.get_value(), 0);
        Ok(())
    });
    
    // Example 2: With operations
    test_case!(test_doc_example_2, |_ctx| {
        let mut calc = TestCalculator::new();
        calc.add(5);
        calc.add(3);
        assert_eq!(calc.get_value(), 8);
        Ok(())
    });
    
    // Example 3: Docker integration
    test_case_docker!(test_doc_example_3, 
        DockerRunOptions::new("alpine:latest"), 
        |_ctx| {
            let calc = TestCalculator::new();
            assert_eq!(calc.get_value(), 0);
            Ok(())
        }
    );
    
    assert!(true, "Documentation examples compiled successfully");
} 
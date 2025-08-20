//! Example demonstrating how to use rust-test-harness exactly like Rust's built-in testing framework
//! 
//! This shows how you can:
//! 1. Use `mod tests { ... }` blocks
//! 2. Define tests outside of main() functions
//! 3. Use standard Rust test patterns with #[test] attributes
//! 4. Leverage the framework's enhanced features (hooks, Docker, etc.)

use rust_test_harness::{
    test_case, test_case_named, test_case_docker, 
    before_all, before_each, after_each, after_all,
    DockerRunOptions, TestContext
};
use std::time::Duration;

// This is your main application code - tests are completely separate
pub struct Calculator {
    value: i32,
}

impl Calculator {
    pub fn new() -> Self {
        Self { value: 0 }
    }
    
    pub fn add(&mut self, x: i32) {
        self.value += x;
    }
    
    pub fn subtract(&mut self, x: i32) {
        self.value -= x;
    }
    
    pub fn get_value(&self) -> i32 {
        self.value
    }
}

// Tests are defined in a separate module, just like Rust's standard testing
#[cfg(test)]
mod tests {
    use super::*;
    
    // Global setup that runs once before all tests
    fn setup_hooks() {
        before_all(|_ctx| {
            println!("🚀 Setting up test environment...");
            Ok(())
        });
        
        // Setup that runs before each individual test
        before_each(|_ctx| {
            println!("📝 Preparing test...");
            Ok(())
        });
        
        // Cleanup that runs after each individual test
        after_each(|_ctx| {
            println!("🧹 Cleaning up after test...");
            Ok(())
        });
        
        // Global cleanup that runs once after all tests
        after_all(|_ctx| {
            println!("🏁 All tests completed!");
            Ok(())
        });
    }
    
    // Basic test using test_case! macro - works exactly like #[test]
    test_case!(test_calculator_new, |_ctx| {
        // Setup hooks for this test
        setup_hooks();
        
        let calc = Calculator::new();
        assert_eq!(calc.get_value(), 0);
        Ok(())
    });
    
    // Test with custom name using test_case_named! macro
    test_case_named!("test_calculator_addition", |_ctx| {
        // Setup hooks for this test
        setup_hooks();
        
        let mut calc = Calculator::new();
        calc.add(5);
        calc.add(3);
        assert_eq!(calc.get_value(), 8);
        Ok(())
    });
    
    // Test with Docker support using test_case_docker! macro
    test_case_docker!(test_with_docker, DockerRunOptions::new("alpine:latest"), |ctx| {
        // Setup hooks for this test
        setup_hooks();
        
        // This test runs with Docker context available
        // You can use ctx.docker_handle for Docker operations
        assert!(ctx.docker_handle.is_none()); // No Docker started by default
        Ok(())
    });
    
    // Standard Rust test that also works with cargo test
    #[test]
    fn test_calculator_subtraction() {
        // Setup hooks for this test
        setup_hooks();
        
        let mut calc = Calculator::new();
        calc.add(10);
        calc.subtract(3);
        assert_eq!(calc.get_value(), 7);
    }
    
    // Test with more complex logic
    test_case!(test_calculator_operations, |_ctx| {
        // Setup hooks for this test
        setup_hooks();
        
        let mut calc = Calculator::new();
        
        // Test addition
        calc.add(100);
        assert_eq!(calc.get_value(), 100);
        
        // Test subtraction
        calc.subtract(25);
        assert_eq!(calc.get_value(), 75);
        
        // Test multiple operations
        calc.add(50);
        calc.subtract(10);
        assert_eq!(calc.get_value(), 115);
        
        Ok(())
    });
    
    // Test that demonstrates error handling
    test_case!(test_calculator_edge_cases, |_ctx| {
        // Setup hooks for this test
        setup_hooks();
        
        let mut calc = Calculator::new();
        
        // Test with zero
        calc.add(0);
        assert_eq!(calc.get_value(), 0);
        
        // Test with negative numbers
        calc.subtract(5);
        assert_eq!(calc.get_value(), -5);
        
        // Test with large numbers
        calc.add(1000000);
        assert_eq!(calc.get_value(), 999995);
        
        Ok(())
    });
}

// You can also define tests outside of the mod tests block
// These will also be discovered by cargo test
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    fn setup_hooks() {
        before_all(|_ctx| {
            println!("🔧 Setting up integration tests...");
            Ok(())
        });
        
        before_each(|_ctx| {
            println!("📋 Preparing integration test...");
            Ok(())
        });
        
        after_each(|_ctx| {
            println!("🧹 Cleaning up integration test...");
            Ok(())
        });
        
        after_all(|_ctx| {
            println!("🏁 Integration tests completed!");
            Ok(())
        });
    }
    
    test_case!(test_calculator_integration, |_ctx| {
        // Setup hooks for this test
        setup_hooks();
        
        let mut calc = Calculator::new();
        
        // Simulate a more complex workflow
        calc.add(10);
        calc.subtract(2);
        calc.add(5);
        calc.subtract(1);
        
        let expected = 10 - 2 + 5 - 1;
        assert_eq!(calc.get_value(), expected);
        
        Ok(())
    });
    
    // Test with custom name
    test_case_named!("test_calculator_workflow", |_ctx| {
        // Setup hooks for this test
        setup_hooks();
        
        let mut calc = Calculator::new();
        
        // Simulate a business logic workflow
        calc.add(100);  // Starting balance
        calc.subtract(30); // Expenses
        calc.add(50);   // Income
        calc.subtract(20); // More expenses
        
        let final_balance = 100 - 30 + 50 - 20;
        assert_eq!(calc.get_value(), final_balance);
        
        Ok(())
    });
}

// You can even define tests at the module level
#[cfg(test)]
test_case!(test_module_level, |_ctx| {
    // This test is defined at the module level, not inside a mod tests block
    let calc = Calculator::new();
    assert_eq!(calc.get_value(), 0);
    Ok(())
});

// Alternative: Standard #[test] function for better IDE support
// This gives you the play button in RustRover while still using the framework
#[cfg(test)]
#[test]
fn test_module_level_standard() {
    // This standard test function will show a play button in RustRover
    // You can still use framework features by calling them manually
    let calc = Calculator::new();
    assert_eq!(calc.get_value(), 0);
    
    // If you want to use framework hooks, you can call them manually:
    // rust_test_harness::execute_before_each_hooks().unwrap();
    // ... your test logic ...
    // rust_test_harness::execute_after_each_hooks().unwrap();
}

// Example of how to use this in a real project:
// 
// 1. In your lib.rs or main.rs, you can have:
//    #[cfg(test)]
//    mod tests {
//        use super::*;
//        use rust_test_harness::test_case;
//        
//        test_case!(my_test, |ctx| {
//            // Your test logic here
//            Ok(())
//        });
//    }
//
// 2. Run with: cargo test
// 3. Your tests will be discovered and run just like standard Rust tests
// 4. You get all the benefits of your framework (hooks, Docker, etc.)
// 5. Plus standard Rust test features (parallel execution, filtering, etc.)

fn main() {
    // Your main application code here
    let mut calc = Calculator::new();
    calc.add(42);
    println!("Calculator value: {}", calc.get_value());
} 
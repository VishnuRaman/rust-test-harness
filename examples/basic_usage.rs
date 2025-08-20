//! Basic usage example demonstrating rust-test-harness with standard Rust testing patterns
//! 
//! This shows the fundamental concepts:
//! 1. Tests in mod tests blocks
//! 2. Using test_case! macro like #[test]
//! 3. Test hooks for setup and cleanup
//! 4. Standard Rust test discovery
//! 5. HTML report generation

use rust_test_harness::{
    test_case, before_all, before_each, after_each, after_all
};

// Example application code
pub struct Counter {
    value: i32,
}

impl Counter {
    pub fn new() -> Self {
        Self { value: 0 }
    }
    
    pub fn increment(&mut self) {
        self.value += 1;
    }
    
    pub fn decrement(&mut self) {
        self.value -= 1;
    }
    
    pub fn get_value(&self) -> i32 {
        self.value
    }
    
    pub fn reset(&mut self) {
        self.value = 0;
    }
}

// Tests follow standard Rust patterns
#[cfg(test)]
mod tests {
    use super::*;
    
    // Setup hooks that run for all tests
    fn setup_test_environment() {
        before_all(|_| {
            println!("🔧 Setting up test environment...");
            Ok(())
        });
        
        before_each(|_| {
            println!("  📝 Preparing test...");
            Ok(())
        });
        
        after_each(|_| {
            println!("  🧹 Test completed, cleaning up...");
            Ok(())
        });
        
        after_all(|_| {
            println!("🧹 Cleaning up test environment...");
            Ok(())
        });
    }
    
    // Basic test using test_case! macro
    test_case!(test_counter_new, |_ctx| {
        setup_test_environment();
        
        let counter = Counter::new();
        assert_eq!(counter.get_value(), 0);
        Ok(())
    });
    
    // Test counter increment
    test_case!(test_counter_increment, |_ctx| {
        setup_test_environment();
        
        let mut counter = Counter::new();
        counter.increment();
        assert_eq!(counter.get_value(), 1);
        
        counter.increment();
        assert_eq!(counter.get_value(), 2);
        Ok(())
    });
    
    // Test counter decrement
    test_case!(test_counter_decrement, |_ctx| {
        setup_test_environment();
        
        let mut counter = Counter::new();
        counter.decrement();
        assert_eq!(counter.get_value(), -1);
        
        counter.decrement();
        assert_eq!(counter.get_value(), -2);
        Ok(())
    });
    
    // Test counter reset
    test_case!(test_counter_reset, |_ctx| {
        setup_test_environment();
        
        let mut counter = Counter::new();
        counter.increment();
        counter.increment();
        counter.increment();
        assert_eq!(counter.get_value(), 3);
        
        counter.reset();
        assert_eq!(counter.get_value(), 0);
        Ok(())
    });
    
    // Test mixed operations
    test_case!(test_counter_mixed_operations, |_ctx| {
        setup_test_environment();
        
        let mut counter = Counter::new();
        
        // Start from 0
        assert_eq!(counter.get_value(), 0);
        
        // Increment a few times
        counter.increment();
        counter.increment();
        counter.increment();
        assert_eq!(counter.get_value(), 3);
        
        // Decrement once
        counter.decrement();
        assert_eq!(counter.get_value(), 2);
        
        // Reset and verify
        counter.reset();
        assert_eq!(counter.get_value(), 0);
        
        Ok(())
    });
    
    // Standard Rust test that also works
    #[test]
    fn test_counter_with_standard_test() {
        let mut counter = Counter::new();
        counter.increment();
        assert_eq!(counter.get_value(), 1);
    }
}

fn main() {
    println!("🚀 Basic Usage Example");
    println!("======================");
    println!("This example demonstrates basic counter operations.");
    println!("Run tests with: cargo test --example basic_usage");
    println!();
    
    // Demo the counter functionality
    let mut counter = Counter::new();
    println!("Initial value: {}", counter.get_value());
    
    counter.increment();
    println!("After increment: {}", counter.get_value());
    
    counter.increment();
    println!("After second increment: {}", counter.get_value());
    
    counter.decrement();
    println!("After decrement: {}", counter.get_value());
    
    counter.reset();
    println!("After reset: {}", counter.get_value());
    
    println!();
    println!("📊 HTML Reporting Demo");
    println!("=====================");
    println!("You can also generate HTML reports for your tests!");
    println!();
    println!("Option 1: Environment Variable");
    println!("  export TEST_HTML_REPORT=basic_usage_report.html");
    println!("  cargo test --example basic_usage");
    println!();
    println!("Option 2: Programmatic Configuration");
    println!("  use rust_test_harness::run_tests_with_config;");
    println!("  let config = TestConfig {{");
    println!("      html_report: Some(\"basic_usage_report.html\".to_string()),");
    println!("      ..Default::default()");
    println!("  }};");
    println!("  run_tests_with_config(config);");
    println!();
    println!("📖 HTML Report Features:");
    println!("  🔽 Expandable test details");
    println!("  🔍 Real-time search");
    println!("  ⌨️  Keyboard shortcuts");
    println!("  🚨 Auto-expand failed tests");
    println!("  📱 Responsive design");
} 
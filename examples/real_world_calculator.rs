use rust_test_harness::{
    before_all, before_each, after_each, after_all, 
    test, test_with_tags
};
use std::sync::{Arc, Mutex};

// Calculator type for testing
#[derive(Clone)]
pub struct Calculator {
    pub memory: f64,
    pub history: Vec<String>,
}

impl Calculator {
    pub fn new() -> Self {
        Self {
            memory: 0.0,
            history: Vec::new(),
        }
    }
}

fn main() {
    // Initialize logger
    env_logger::init();


    
    // Register global hooks
    before_all(|_| {
        println!("🧮 Starting Calculator Test Suite");
        println!("Testing basic arithmetic operations, error handling, and state management");
        Ok(())
    });

    after_all(|_| {
        println!("✅ Calculator Test Suite completed!");
        Ok(())
    });

    // Global calculator instance for tests
    let calculator = Arc::new(Mutex::new(Calculator::new()));

    before_each({
        let calculator = Arc::clone(&calculator);
        move |_| {
            // Each test gets a fresh calculator instance
            let mut calc = calculator.lock().unwrap();
            *calc = Calculator::new();
            println!("  📱 Initializing fresh calculator...");
            Ok(())
        }
    });

    after_each({
        let calculator = Arc::clone(&calculator);
        move |_| {
            // Clean up test data
            let calc = calculator.lock().unwrap();
            println!("  🧹 Test completed. History entries: {}", calc.history.len());
            Ok(())
        }
    });

    // Basic arithmetic tests
    test("addition works correctly", {
        let calculator = Arc::clone(&calculator);
        move |_| {
            let mut calc = calculator.lock().unwrap();
            
            // Simulate addition operations
            calc.memory = 2.5 + 3.5;
            assert_eq!(calc.memory, 6.0);
            
            calc.memory = -1.0 + 1.0;
            assert_eq!(calc.memory, 0.0);
            
            calc.memory = 0.0 + 0.0;
            assert_eq!(calc.memory, 0.0);
            
            Ok(())
        }
    });

    test("subtraction works correctly", {
        let calculator = Arc::clone(&calculator);
        move |_| {
            let mut calc = calculator.lock().unwrap();
            
            calc.memory = 5.0 - 3.0;
            assert_eq!(calc.memory, 2.0);
            
            calc.memory = 0.0 - 5.0;
            assert_eq!(calc.memory, -5.0);
            
            calc.memory = 10.0 - 10.0;
            assert_eq!(calc.memory, 0.0);
            
            Ok(())
        }
    });

    test("multiplication works correctly", {
        let calculator = Arc::clone(&calculator);
        move |_| {
            let mut calc = calculator.lock().unwrap();
            
            calc.memory = 3.0 * 4.0;
            assert_eq!(calc.memory, 12.0);
            
            calc.memory = -2.0 * 3.0;
            assert_eq!(calc.memory, -6.0);
            
            calc.memory = 0.0 * 100.0;
            assert_eq!(calc.memory, 0.0);
            
            Ok(())
        }
    });

    test("division works correctly", {
        let calculator = Arc::clone(&calculator);
        move |_| {
            let mut calc = calculator.lock().unwrap();
            
            calc.memory = 10.0 / 2.0;
            assert_eq!(calc.memory, 5.0);
            
            calc.memory = 7.0 / 2.0;
            assert_eq!(calc.memory, 3.5);
            
            calc.memory = 0.0 / 5.0;
            assert_eq!(calc.memory, 0.0);
            
            Ok(())
        }
    });

    // Error handling tests
    test("division by zero returns error", {
        let calculator = Arc::clone(&calculator);
        move |_| {
            let mut calc = calculator.lock().unwrap();
            
            // Simulate division by zero check
            let divisor = 0.0;
            if divisor == 0.0 {
                // In a real implementation, this would return an error
                calc.history.push("Division by zero error".to_string());
            } else {
                calc.memory = 10.0 / divisor;
            }
            
            // Verify error was recorded
            assert!(calc.history.contains(&"Division by zero error".to_string()));
            
            Ok(())
        }
    });

    // State management tests
    test("memory operations work correctly", {
        let calculator = Arc::clone(&calculator);
        move |_| {
            let mut calc = calculator.lock().unwrap();
            
            calc.memory = 42.0;
            assert_eq!(calc.memory, 42.0);
            
            calc.memory = -17.5;
            assert_eq!(calc.memory, -17.5);
            
            Ok(())
        }
    });

    test("history tracking works correctly", {
        let calculator = Arc::clone(&calculator);
        move |_| {
            let mut calc = calculator.lock().unwrap();
            
            // Perform some operations and record them
            calc.history.push("1 + 2 = 3".to_string());
            calc.history.push("3 * 4 = 12".to_string());
            calc.history.push("10 - 5 = 5".to_string());
            
            let history = &calc.history;
            assert_eq!(history.len(), 3);
            assert!(history[0].contains("1 + 2 = 3"));
            assert!(history[1].contains("3 * 4 = 12"));
            assert!(history[2].contains("10 - 5 = 5"));
            
            Ok(())
        }
    });

    // Data-driven tests
    test("floating point precision handling", {
        let calculator = Arc::clone(&calculator);
        move |_| {
            let mut calc = calculator.lock().unwrap();
            
            // Test floating point arithmetic
            calc.memory = 0.1 + 0.2;
            // Note: 0.1 + 0.2 != 0.3 due to floating point precision
            assert!((calc.memory - 0.30000000000000004).abs() < f64::EPSILON);
            
            Ok(())
        }
    });

    // Performance and stress tests
    test_with_tags("stress test - many operations", vec!["performance", "stress"], {
        let calculator = Arc::clone(&calculator);
        move |_| {
            let mut calc = calculator.lock().unwrap();
            
            // Perform many operations to test performance
            for i in 0..1000 {
                calc.history.push(format!("Operation {}", i));
            }
            
            assert_eq!(calc.history.len(), 1000);
            Ok(())
        }
    });

    // Integration-style tests
    test_with_tags("complex calculation workflow", vec!["integration", "workflow"], {
        let calculator = Arc::clone(&calculator);
        move |_| {
            let mut calc = calculator.lock().unwrap();
            
            // Simulate a real-world calculation workflow
            let initial_value = 100.0;
            calc.memory = initial_value;
            
            // Apply a series of operations
            let result1 = calc.memory + 50.0;  // 100 + 50 = 150
            calc.memory = result1;
            
            let result2 = calc.memory * 0.8;  // 150 * 0.8 = 120
            calc.memory = result2;
            
            let result3 = calc.memory - 20.0;  // 120 - 20 = 100
            calc.memory = result3;
            
            // Verify the final result
            assert_eq!(calc.memory, 100.0);
            
            // Verify history
            calc.history.push(format!("100 + 50 = 150"));
            calc.history.push(format!("150 * 0.8 = 120"));
            calc.history.push(format!("120 - 20 = 100"));
            
            let history = &calc.history;
            assert_eq!(history.len(), 3);
            assert!(history[0].contains("100 + 50 = 150"));
            assert!(history[1].contains("150 * 0.8 = 120"));
            assert!(history[2].contains("120 - 20 = 100"));
            
            Ok(())
        }
    });

    // Edge case tests
    test("edge cases - very large numbers", {
        let calculator = Arc::clone(&calculator);
        move |_| {
            let mut calc = calculator.lock().unwrap();
            
            let large_num = f64::MAX;
            calc.memory = large_num + 0.0;
            assert_eq!(calc.memory, large_num);
            
            Ok(())
        }
    });

    test("edge cases - very small numbers", {
        let calculator = Arc::clone(&calculator);
        move |_| {
            let mut calc = calculator.lock().unwrap();
            
            let small_num = f64::MIN_POSITIVE;
            calc.memory = small_num * 1.0;
            assert_eq!(calc.memory, small_num);
            
            Ok(())
        }
    });

    // Run all tests
    println!("\n🚀 Running Calculator Test Suite...\n");
    let exit_code = rust_test_harness::run_all();
    
    if exit_code == 0 {
        println!("\n🎉 All calculator tests passed!");
    } else {
        println!("\n❌ Some calculator tests failed!");
    }
    
    std::process::exit(exit_code);
} 
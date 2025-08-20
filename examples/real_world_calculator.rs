//! Real-world calculator example demonstrating comprehensive testing patterns
//! 
//! This shows real-world testing scenarios:
//! 1. Testing a complex stateful object (Calculator)
//! 2. Error handling and edge cases
//! 3. State management and memory operations
//! 4. Performance testing with tagged tests

use rust_test_harness::{
    test_case, test_case_named, before_all, before_each, after_each, after_all
};

// Calculator implementation for testing
#[derive(Clone, Debug, PartialEq)]
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
    
    pub fn add(&mut self, a: f64, b: f64) -> f64 {
        let result = a + b;
        self.memory = result;
        self.history.push(format!("{} + {} = {}", a, b, result));
        result
    }
    
    pub fn subtract(&mut self, a: f64, b: f64) -> f64 {
        let result = a - b;
        self.memory = result;
        self.history.push(format!("{} - {} = {}", a, b, result));
        result
    }
    
    pub fn multiply(&mut self, a: f64, b: f64) -> f64 {
        let result = a * b;
        self.memory = result;
        self.history.push(format!("{} * {} = {}", a, b, result));
        result
    }
    
    pub fn divide(&mut self, a: f64, b: f64) -> Result<f64, String> {
        if b == 0.0 {
            return Err("Division by zero".to_string());
        }
        let result = a / b;
        self.memory = result;
        self.history.push(format!("{} / {} = {}", a, b, result));
        Ok(result)
    }
    
    pub fn clear_memory(&mut self) {
        self.memory = 0.0;
        self.history.push("Memory cleared".to_string());
    }
    
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
    
    pub fn get_memory(&self) -> f64 {
        self.memory
    }
    
    pub fn get_history(&self) -> &Vec<String> {
        &self.history
    }
    
    pub fn get_history_count(&self) -> usize {
        self.history.len()
    }
}

// Basic arithmetic tests
#[cfg(test)]
mod arithmetic_tests {
    use super::*;
    
    fn setup_hooks() {
        before_all(|_| {
            println!("🧮 Starting Calculator Test Suite");
            println!("Testing basic arithmetic operations, error handling, and state management");
            Ok(())
        });
        
        before_each(|_| {
            println!("  📱 Initializing fresh calculator...");
            Ok(())
        });
        
        after_each(|_| {
            println!("  ✅ Calculator test completed");
            Ok(())
        });
        
        after_all(|_| {
            println!("✅ Calculator Test Suite completed!");
            Ok(())
        });
    }
    
    test_case!(test_calculator_creation, |_ctx| {
        setup_hooks();
        
        let calc = Calculator::new();
        assert_eq!(calc.get_memory(), 0.0);
        assert_eq!(calc.get_history_count(), 0);
        Ok(())
    });
    
    test_case!(test_basic_addition, |_ctx| {
        setup_hooks();
        
        let mut calc = Calculator::new();
        
        let result = calc.add(2.0, 3.0);
        assert_eq!(result, 5.0);
        assert_eq!(calc.get_memory(), 5.0);
        assert_eq!(calc.get_history_count(), 1);
        assert!(calc.get_history()[0].contains("2 + 3 = 5"));
        
        Ok(())
    });
    
    test_case!(test_basic_subtraction, |_ctx| {
        setup_hooks();
        
        let mut calc = Calculator::new();
        
        let result = calc.subtract(10.0, 3.0);
        assert_eq!(result, 7.0);
        assert_eq!(calc.get_memory(), 7.0);
        assert_eq!(calc.get_history_count(), 1);
        assert!(calc.get_history()[0].contains("10 - 3 = 7"));
        
        Ok(())
    });
    
    test_case!(test_basic_multiplication, |_ctx| {
        setup_hooks();
        
        let mut calc = Calculator::new();
        
        let result = calc.multiply(4.0, 5.0);
        assert_eq!(result, 20.0);
        assert_eq!(calc.get_memory(), 20.0);
        assert_eq!(calc.get_history_count(), 1);
        assert!(calc.get_history()[0].contains("4 * 5 = 20"));
        
        Ok(())
    });
    
    test_case!(test_basic_division, |_ctx| {
        setup_hooks();
        
        let mut calc = Calculator::new();
        
        let result = calc.divide(15.0, 3.0).unwrap();
        assert_eq!(result, 5.0);
        assert_eq!(calc.get_memory(), 5.0);
        assert_eq!(calc.get_history_count(), 1);
        assert!(calc.get_history()[0].contains("15 / 3 = 5"));
        
        Ok(())
    });
}

// Error handling tests
#[cfg(test)]
mod error_tests {
    use super::*;
    
    test_case!(test_division_by_zero, |_ctx| {
        let mut calc = Calculator::new();
        
        let result = calc.divide(10.0, 0.0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Division by zero");
        
        // Memory should not be updated on error
        assert_eq!(calc.get_memory(), 0.0);
        assert_eq!(calc.get_history_count(), 0);
        
        Ok(())
    });
    
    test_case!(test_negative_numbers, |_ctx| {
        let mut calc = Calculator::new();
        
        assert_eq!(calc.add(-5.0, 3.0), -2.0);
        assert_eq!(calc.subtract(-5.0, -3.0), -2.0);
        assert_eq!(calc.multiply(-4.0, 3.0), -12.0);
        assert_eq!(calc.divide(-12.0, 3.0).unwrap(), -4.0);
        
        Ok(())
    });
    
    test_case!(test_floating_point_precision, |_ctx| {
        let mut calc = Calculator::new();
        
        let result = calc.add(0.1, 0.2);
        // Use approx comparison for floating point
        assert!((result - 0.3).abs() < 1e-10);
        
        Ok(())
    });
}

// Memory and state tests
#[cfg(test)]
mod memory_tests {
    use super::*;
    
    test_case!(test_memory_operations, |_ctx| {
        let mut calc = Calculator::new();
        
        // Perform operations and check memory
        calc.add(10.0, 5.0);
        assert_eq!(calc.get_memory(), 15.0);
        
        calc.multiply(3.0, 4.0);
        assert_eq!(calc.get_memory(), 12.0);
        
        // Clear memory
        calc.clear_memory();
        assert_eq!(calc.get_memory(), 0.0);
        assert_eq!(calc.get_history_count(), 3); // 2 operations + 1 clear
        
        Ok(())
    });
    
    test_case!(test_history_tracking, |_ctx| {
        let mut calc = Calculator::new();
        
        calc.add(1.0, 2.0);
        calc.subtract(5.0, 3.0);
        calc.multiply(4.0, 6.0);
        
        assert_eq!(calc.get_history_count(), 3);
        
        let history = calc.get_history();
        assert!(history[0].contains("1 + 2 = 3"));
        assert!(history[1].contains("5 - 3 = 2"));
        assert!(history[2].contains("4 * 6 = 24"));
        
        // Clear history
        calc.clear_history();
        assert_eq!(calc.get_history_count(), 0);
        
        Ok(())
    });
    
    test_case_named!(test_calculator_state_persistence, |_ctx| {
        let mut calc = Calculator::new();
        
        // Perform multiple operations
        calc.add(10.0, 20.0);
        calc.subtract(50.0, 15.0);
        calc.multiply(7.0, 8.0);
        
        // Check final state
        assert_eq!(calc.get_memory(), 56.0);
        assert_eq!(calc.get_history_count(), 3);
        
        // Verify history order
        let history = calc.get_history();
        assert!(history[0].contains("10 + 20 = 30"));
        assert!(history[1].contains("50 - 15 = 35"));
        assert!(history[2].contains("7 * 8 = 56"));
        
        Ok(())
    });
}

// Complex workflow tests
#[cfg(test)]
mod workflow_tests {
    use super::*;
    
    test_case!(test_complex_calculation_workflow, |_ctx| {
        let mut calc = Calculator::new();
        
        // Simulate a complex calculation: (10 + 5) * 3 / 2 - 1
        calc.add(10.0, 5.0);              // = 15
        let temp1 = calc.get_memory();
        
        calc.multiply(temp1, 3.0);        // = 45
        let temp2 = calc.get_memory();
        
        calc.divide(temp2, 2.0).unwrap(); // = 22.5
        let temp3 = calc.get_memory();
        
        calc.subtract(temp3, 1.0);        // = 21.5
        
        assert_eq!(calc.get_memory(), 21.5);
        assert_eq!(calc.get_history_count(), 4);
        
        Ok(())
    });
    
    test_case!(test_calculator_clone, |_ctx| {
        let mut calc1 = Calculator::new();
        calc1.add(5.0, 3.0);
        calc1.multiply(2.0, 4.0);
        
        let calc2 = calc1.clone();
        
        // Both calculators should have the same state
        assert_eq!(calc1.get_memory(), calc2.get_memory());
        assert_eq!(calc1.get_history_count(), calc2.get_history_count());
        assert_eq!(calc1.get_history(), calc2.get_history());
        
        Ok(())
    });
    
    // Standard Rust test for comparison
    #[test]
    fn test_with_standard_rust_test() {
        let mut calc = Calculator::new();
        calc.add(2.0, 2.0);
        assert_eq!(calc.get_memory(), 4.0);
    }
}

fn main() {
    println!("🚀 Real-World Calculator Example");
    println!("=================================");
    println!("This example demonstrates comprehensive calculator testing.");
    println!("Run tests with: cargo test --example real_world_calculator");
    
    // Demo the calculator functionality
    let mut calc = Calculator::new();
    println!("Created calculator, memory: {}", calc.get_memory());
    
    println!("Performing calculations...");
    calc.add(10.0, 5.0);
    println!("10 + 5 = {}", calc.get_memory());
    
    calc.multiply(3.0, 4.0);
    println!("3 * 4 = {}", calc.get_memory());
    
    match calc.divide(20.0, 4.0) {
        Ok(result) => println!("20 / 4 = {}", result),
        Err(e) => println!("Error: {}", e),
    }
    
    println!("Final memory: {}", calc.get_memory());
    println!("History entries: {}", calc.get_history_count());
    
    println!("\nCalculation history:");
    for (i, entry) in calc.get_history().iter().enumerate() {
        println!("  {}: {}", i + 1, entry);
    }
} 
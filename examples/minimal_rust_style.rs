//! Minimal example showing how rust-test-harness works exactly like Rust's built-in testing
//! 
//! This demonstrates the key concept: you can use your framework in the same way
//! you use standard Rust testing, with the same patterns and discovery.

use rust_test_harness::test_case;

// Your application code - completely separate from tests
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

// Tests are defined in a separate module, just like Rust's standard testing
#[cfg(test)]
mod tests {
    use super::*;
    
    // Test using test_case! macro - works exactly like #[test]
    test_case!(test_add, |_ctx| {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
        assert_eq!(add(0, 0), 0);
        Ok(())
    });
    
    // Another test
    test_case!(test_multiply, |_ctx| {
        assert_eq!(multiply(2, 3), 6);
        assert_eq!(multiply(-2, 3), -6);
        assert_eq!(multiply(0, 5), 0);
        Ok(())
    });
    
    // Standard Rust test that also works unchanged
    #[test]
    fn test_both_functions() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(multiply(2, 3), 6);
    }
}

// You can also define tests outside of mod tests blocks
#[cfg(test)]
test_case!(test_module_level, |_ctx| {
    // This test is defined at the module level
    assert_eq!(add(10, 20), 30);
    Ok(())
});

fn main() {
    println!("2 + 3 = {}", add(2, 3));
    println!("2 * 3 = {}", multiply(2, 3));
}

// How to use this:
//
// 1. Run the application:
//    cargo run --example minimal_rust_style
//
// 2. Run the tests:
//    cargo test --example minimal_rust_style
//
// 3. Or run all tests in the project:
//    cargo test
//
// 4. Run specific tests:
//    cargo test test_add
//    cargo test multiply
//
// The key point: Your tests work exactly like standard Rust tests!
// They're discovered by cargo test, can use standard assertions,
// and follow the same patterns you're already familiar with. 
//! Cargo test integration example demonstrating compatibility with standard Rust testing
//! 
//! This shows how rust-test-harness integrates seamlessly with cargo test:
//! 1. Tests are discovered by cargo test automatically
//! 2. Works alongside standard #[test] functions
//! 3. Can use both framework features and standard assertions
//! 4. No special build configuration needed

use rust_test_harness::{
    test_case, test_case_named, before_all, before_each, after_each, after_all
};

// Example library code that we want to test
pub struct StringProcessor {
    case_sensitive: bool,
}

impl StringProcessor {
    pub fn new(case_sensitive: bool) -> Self {
        Self { case_sensitive }
    }
    
    pub fn process(&self, input: &str) -> String {
        if self.case_sensitive {
            input.to_string()
        } else {
            input.to_lowercase()
        }
    }
    
    pub fn contains(&self, haystack: &str, needle: &str) -> bool {
        if self.case_sensitive {
            haystack.contains(needle)
        } else {
            haystack.to_lowercase().contains(&needle.to_lowercase())
        }
    }
    
    pub fn count_words(&self, input: &str) -> usize {
        input.split_whitespace().count()
    }
    
    pub fn reverse(&self, input: &str) -> String {
        let processed = self.process(input);
        processed.chars().rev().collect()
    }
}

// Tests that demonstrate cargo test integration
#[cfg(test)]
mod basic_tests {
    use super::*;
    
    fn setup_hooks() {
        before_all(|_| {
            println!("🔧 Setting up test environment");
            Ok(())
        });
        
        before_each(|_| {
            println!("  📝 Preparing test");
            Ok(())
        });
        
        after_each(|_| {
            println!("  🧹 Test completed");
            Ok(())
        });
        
        after_all(|_| {
            println!("🧹 Cleaning up test environment");
            Ok(())
        });
    }
    
    // Test using test_case! macro
    test_case!(test_string_processor_creation, |_ctx| {
        setup_hooks();
        
        let processor = StringProcessor::new(true);
        assert_eq!(processor.case_sensitive, true);
        
        let processor_insensitive = StringProcessor::new(false);
        assert_eq!(processor_insensitive.case_sensitive, false);
        
        Ok(())
    });
    
    // Test case sensitive processing
    test_case!(test_case_sensitive_processing, |_ctx| {
        setup_hooks();
        
        let processor = StringProcessor::new(true);
        
        assert_eq!(processor.process("Hello World"), "Hello World");
        assert_eq!(processor.process("UPPERCASE"), "UPPERCASE");
        assert_eq!(processor.process("lowercase"), "lowercase");
        
        Ok(())
    });
    
    // Test case insensitive processing
    test_case!(test_case_insensitive_processing, |_ctx| {
        setup_hooks();
        
        let processor = StringProcessor::new(false);
        
        assert_eq!(processor.process("Hello World"), "hello world");
        assert_eq!(processor.process("UPPERCASE"), "uppercase");
        assert_eq!(processor.process("MiXeD cAsE"), "mixed case");
        
        Ok(())
    });
    
    // Standard Rust test - works alongside test_case! macros
    #[test]
    fn test_with_standard_rust_test() {
        let processor = StringProcessor::new(true);
        assert_eq!(processor.count_words("hello world"), 2);
    }
}

// More comprehensive tests
#[cfg(test)]
mod feature_tests {
    use super::*;
    
    test_case!(test_contains_case_sensitive, |_ctx| {
        let processor = StringProcessor::new(true);
        
        assert!(processor.contains("Hello World", "Hello"));
        assert!(processor.contains("Hello World", "World"));
        assert!(!processor.contains("Hello World", "hello")); // Case sensitive
        assert!(!processor.contains("Hello World", "WORLD")); // Case sensitive
        
        Ok(())
    });
    
    test_case!(test_contains_case_insensitive, |_ctx| {
        let processor = StringProcessor::new(false);
        
        assert!(processor.contains("Hello World", "hello"));
        assert!(processor.contains("Hello World", "WORLD"));
        assert!(processor.contains("Hello World", "Hello"));
        assert!(processor.contains("Hello World", "world"));
        assert!(!processor.contains("Hello World", "xyz"));
        
        Ok(())
    });
    
    test_case!(test_word_counting, |_ctx| {
        let processor = StringProcessor::new(true);
        
        assert_eq!(processor.count_words(""), 0);
        assert_eq!(processor.count_words("hello"), 1);
        assert_eq!(processor.count_words("hello world"), 2);
        assert_eq!(processor.count_words("  hello   world  "), 2);
        assert_eq!(processor.count_words("one two three four five"), 5);
        
        Ok(())
    });
    
    test_case_named!("test_string_reversal", |_ctx| {
        let processor_sensitive = StringProcessor::new(true);
        let processor_insensitive = StringProcessor::new(false);
        
        // Case sensitive reversal
        assert_eq!(processor_sensitive.reverse("hello"), "olleh");
        assert_eq!(processor_sensitive.reverse("Hello"), "olleH");
        
        // Case insensitive reversal (converts to lowercase first)
        assert_eq!(processor_insensitive.reverse("Hello"), "olleh");
        assert_eq!(processor_insensitive.reverse("WORLD"), "dlrow");
        
        Ok(())
    });
}

// Edge case tests
#[cfg(test)]
mod edge_case_tests {
    use super::*;
    
    test_case!(test_empty_strings, |_ctx| {
        let processor = StringProcessor::new(true);
        
        assert_eq!(processor.process(""), "");
        assert_eq!(processor.reverse(""), "");
        assert_eq!(processor.count_words(""), 0);
        assert!(!processor.contains("", "anything"));
        assert!(processor.contains("anything", ""));
        
        Ok(())
    });
    
    test_case!(test_special_characters, |_ctx| {
        let processor = StringProcessor::new(false);
        
        let special = "Hello, World! @#$%^&*()";
        let processed = processor.process(special);
        assert_eq!(processed, "hello, world! @#$%^&*()");
        
        assert_eq!(processor.count_words(special), 2);
        assert!(processor.contains(special, "hello"));
        assert!(processor.contains(special, "WORLD"));
        
        Ok(())
    });
    
    test_case!(test_unicode_handling, |_ctx| {
        let processor = StringProcessor::new(false);
        
        let unicode = "Héllo Wørld 你好";
        let processed = processor.process(unicode);
        
        // Basic processing should work with Unicode
        assert!(processed.len() > 0);
        assert_eq!(processor.count_words(unicode), 3);
        
        Ok(())
    });
    
    // Another standard Rust test
    #[test]
    fn test_mixed_testing_approaches() {
        // This demonstrates that you can mix standard Rust tests
        // with the test_case! macros in the same project
        let processor = StringProcessor::new(true);
        
        let result = processor.process("Test");
        assert_eq!(result, "Test");
        
        let reversed = processor.reverse("abc");
        assert_eq!(reversed, "cba");
    }
}

fn main() {
    println!("🚀 Cargo Test Integration Example");
    println!("==================================");
    println!("This example demonstrates how rust-test-harness integrates with cargo test.");
    println!("Run tests with: cargo test --example cargo_test_integration");
    println!("Or run specific tests: cargo test test_string_processor");
    
    // Demo the string processor functionality
    let processor_sensitive = StringProcessor::new(true);
    let processor_insensitive = StringProcessor::new(false);
    
    let sample_text = "Hello World!";
    
    println!("\nDemo with text: '{}'", sample_text);
    println!("Case sensitive: '{}'", processor_sensitive.process(sample_text));
    println!("Case insensitive: '{}'", processor_insensitive.process(sample_text));
    
    println!("Word count: {}", processor_sensitive.count_words(sample_text));
    println!("Contains 'hello' (case sensitive): {}", processor_sensitive.contains(sample_text, "hello"));
    println!("Contains 'hello' (case insensitive): {}", processor_insensitive.contains(sample_text, "hello"));
    
    println!("Reversed (case sensitive): '{}'", processor_sensitive.reverse(sample_text));
    println!("Reversed (case insensitive): '{}'", processor_insensitive.reverse(sample_text));
} 
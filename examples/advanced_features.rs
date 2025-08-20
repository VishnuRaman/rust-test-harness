//! Advanced features example demonstrating rust-test-harness with sophisticated patterns
//! 
//! This shows advanced concepts:
//! 1. Test hooks for complex setup/teardown
//! 2. Multiple test modules
//! 3. Different types of assertions
//! 4. Error handling in tests

use rust_test_harness::{
    test_case, test_case_named, before_all, before_each, after_each, after_all
};
use std::time::Duration;
use std::collections::HashMap;

// Example application - a simple in-memory database
pub struct Database {
    data: HashMap<String, String>,
    connection_count: usize,
}

impl Database {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            connection_count: 0,
        }
    }
    
    pub fn connect(&mut self) -> Result<(), String> {
        if self.connection_count >= 10 {
            return Err("Too many connections".to_string());
        }
        self.connection_count += 1;
        Ok(())
    }
    
    pub fn disconnect(&mut self) {
        if self.connection_count > 0 {
            self.connection_count -= 1;
        }
    }
    
    pub fn insert(&mut self, key: String, value: String) -> Result<(), String> {
        if self.connection_count == 0 {
            return Err("No active connection".to_string());
        }
        self.data.insert(key, value);
        Ok(())
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        if self.connection_count == 0 {
            return None;
        }
        self.data.get(key)
    }
    
    pub fn delete(&mut self, key: &str) -> bool {
        if self.connection_count == 0 {
            return false;
        }
        self.data.remove(key).is_some()
    }
    
    pub fn get_connection_count(&self) -> usize {
        self.connection_count
    }
    
    pub fn get_data_count(&self) -> usize {
        self.data.len()
    }
}

// Basic database tests
#[cfg(test)]
mod basic_tests {
    use super::*;
    
    fn setup_hooks() {
        before_all(|_| {
            println!("🔧 Setting up advanced test environment");
            Ok(())
        });
        
        before_each(|_| {
            println!("  📝 Preparing advanced test");
            Ok(())
        });
        
        after_each(|_| {
            println!("  🧹 Advanced test completed");
            Ok(())
        });
        
        after_all(|_| {
            println!("🧹 Cleaning up advanced test environment");
            Ok(())
        });
    }
    
    test_case!(test_database_creation, |_ctx| {
        setup_hooks();
        
        let db = Database::new();
        assert_eq!(db.get_connection_count(), 0);
        assert_eq!(db.get_data_count(), 0);
        Ok(())
    });
    
    test_case!(test_database_connection, |_ctx| {
        setup_hooks();
        
        let mut db = Database::new();
        
        // Test successful connection
        assert!(db.connect().is_ok());
        assert_eq!(db.get_connection_count(), 1);
        
        // Test multiple connections
        assert!(db.connect().is_ok());
        assert_eq!(db.get_connection_count(), 2);
        
        // Test disconnection
        db.disconnect();
        assert_eq!(db.get_connection_count(), 1);
        
        Ok(())
    });
    
    test_case!(test_database_max_connections, |_ctx| {
        setup_hooks();
        
        let mut db = Database::new();
        
        // Connect up to the limit
        for _ in 0..10 {
            assert!(db.connect().is_ok());
        }
        assert_eq!(db.get_connection_count(), 10);
        
        // Try to exceed the limit
        let result = db.connect();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Too many connections");
        
        Ok(())
    });
}

// Data operation tests
#[cfg(test)]
mod data_tests {
    use super::*;
    
    fn setup_hooks() {
        before_each(|_| {
            println!("  📝 Setting up data test");
            Ok(())
        });
        
        after_each(|_| {
            println!("  🧹 Cleaning up data test");
            Ok(())
        });
    }
    
    test_case!(test_insert_and_retrieve, |_ctx| {
        setup_hooks();
        
        let mut db = Database::new();
        db.connect().unwrap();
        
        // Test insertion
        assert!(db.insert("key1".to_string(), "value1".to_string()).is_ok());
        assert_eq!(db.get_data_count(), 1);
        
        // Test retrieval
        assert_eq!(db.get("key1"), Some(&"value1".to_string()));
        assert_eq!(db.get("nonexistent"), None);
        
        Ok(())
    });
    
    test_case!(test_insert_without_connection, |_ctx| {
        setup_hooks();
        
        let mut db = Database::new();
        
        // Try to insert without connection
        let result = db.insert("key1".to_string(), "value1".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No active connection");
        
        Ok(())
    });
    
    test_case!(test_delete_operations, |_ctx| {
        setup_hooks();
        
        let mut db = Database::new();
        db.connect().unwrap();
        
        // Insert some data
        db.insert("key1".to_string(), "value1".to_string()).unwrap();
        db.insert("key2".to_string(), "value2".to_string()).unwrap();
        assert_eq!(db.get_data_count(), 2);
        
        // Delete existing key
        assert!(db.delete("key1"));
        assert_eq!(db.get_data_count(), 1);
        assert_eq!(db.get("key1"), None);
        assert_eq!(db.get("key2"), Some(&"value2".to_string()));
        
        // Try to delete non-existent key
        assert!(!db.delete("nonexistent"));
        assert_eq!(db.get_data_count(), 1);
        
        Ok(())
    });
    
    // Test with custom name
    test_case_named!(test_complex_data_workflow, |_ctx| {
        setup_hooks();
        
        let mut db = Database::new();
        db.connect().unwrap();
        
        // Insert multiple items
        let items = vec![
            ("user:1".to_string(), "John Doe".to_string()),
            ("user:2".to_string(), "Jane Smith".to_string()),
            ("user:3".to_string(), "Bob Johnson".to_string()),
        ];
        
        for (key, value) in items {
            assert!(db.insert(key.clone(), value.clone()).is_ok());
            assert_eq!(db.get(&key), Some(&value));
        }
        
        assert_eq!(db.get_data_count(), 3);
        
        // Delete some items
        assert!(db.delete("user:2"));
        assert_eq!(db.get_data_count(), 2);
        
        // Verify remaining data
        assert_eq!(db.get("user:1"), Some(&"John Doe".to_string()));
        assert_eq!(db.get("user:2"), None);
        assert_eq!(db.get("user:3"), Some(&"Bob Johnson".to_string()));
        
        Ok(())
    });
}

// Error handling tests
#[cfg(test)]
mod error_tests {
    use super::*;
    
    test_case!(test_error_conditions, |_ctx| {
        let mut db = Database::new();
        
        // Test operations without connection
        assert_eq!(db.get("key"), None);
        assert!(!db.delete("key"));
        assert!(db.insert("key".to_string(), "value".to_string()).is_err());
        
        // Connect and test normal operations
        db.connect().unwrap();
        assert!(db.insert("key".to_string(), "value".to_string()).is_ok());
        assert_eq!(db.get("key"), Some(&"value".to_string()));
        assert!(db.delete("key"));
        
        Ok(())
    });
    
    // Standard Rust test for comparison
    #[test]
    fn test_with_standard_rust_test() {
        let mut db = Database::new();
        assert_eq!(db.get_connection_count(), 0);
        
        db.connect().unwrap();
        assert_eq!(db.get_connection_count(), 1);
    }
}

fn main() {
    println!("🚀 Advanced Features Example");
    println!("============================");
    println!("This example demonstrates advanced database operations.");
    println!("Run tests with: cargo test --example advanced_features");
    
    // Demo the database functionality
    let mut db = Database::new();
    println!("Created database, connections: {}", db.get_connection_count());
    
    db.connect().unwrap();
    println!("Connected, connections: {}", db.get_connection_count());
    
    db.insert("user:1".to_string(), "Alice".to_string()).unwrap();
    db.insert("user:2".to_string(), "Bob".to_string()).unwrap();
    println!("Inserted data, count: {}", db.get_data_count());
    
    if let Some(user) = db.get("user:1") {
        println!("Retrieved user:1 = {}", user);
    }
    
    db.delete("user:1");
    println!("Deleted user:1, remaining count: {}", db.get_data_count());
    
    db.disconnect();
    println!("Disconnected, connections: {}", db.get_connection_count());
} 
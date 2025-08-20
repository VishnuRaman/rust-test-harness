//! MongoDB integration example demonstrating Docker-based testing with rust-test-harness
//! 
//! This shows real-world database testing patterns:
//! 1. Testing with real Docker containers managed by hooks
//! 2. Database operations (CRUD)
//! 3. Error handling and edge cases
//! 4. Setup and teardown with hooks

use rust_test_harness::{test, run_tests_with_config, TestConfig, ContainerConfig, before_each};
use std::time::Duration;

// Mock MongoDB client for demonstration
struct MongoClient {
    container_id: String,
    connected: bool,
}

impl MongoClient {
    fn new(container_id: String) -> Self {
        Self {
            container_id,
            connected: false,
        }
    }
    
    fn connect(&mut self) -> Result<(), String> {
        // Simulate connection
        std::thread::sleep(Duration::from_millis(20));
        self.connected = true;
        Ok(())
    }
    
    fn disconnect(&mut self) -> Result<(), String> {
        // Simulate disconnection
        std::thread::sleep(Duration::from_millis(10));
        self.connected = false;
        Ok(())
    }
    
    fn insert_document(&self, collection: &str, document: &str) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected to MongoDB".to_string());
        }
        
        // Simulate document insertion
        std::thread::sleep(Duration::from_millis(5));
        println!("📄 Inserted document into collection '{}': {}", collection, document);
        Ok(())
    }
    
    fn find_documents(&self, collection: &str, query: &str) -> Result<Vec<String>, String> {
        if !self.connected {
            return Err("Not connected to MongoDB".to_string());
        }
        
        // Simulate document retrieval
        std::thread::sleep(Duration::from_millis(3));
        println!("🔍 Found documents in collection '{}' with query: {}", collection, query);
        
        // Return mock documents
        Ok(vec![
            format!("Document 1 matching: {}", query),
            format!("Document 2 matching: {}", query),
        ])
    }
}

fn main() {
    println!("🐳 MongoDB Integration Example with Container Hooks");
    println!("==================================================");
    println!();
    
    // Define container configuration
    let mongo_container = ContainerConfig::new("mongo:5.0")
        .port(27017, 27017)
        .env("MONGO_INITDB_ROOT_USERNAME", "admin")
        .env("MONGO_INITDB_ROOT_PASSWORD", "password")
        .name("test_mongodb")
        .ready_timeout(Duration::from_secs(30));
    
    println!("📋 Container Configuration:");
    println!("  Image: {}", mongo_container.image);
    println!("  Ports: {:?}", mongo_container.ports);
    println!("  Environment: {:?}", mongo_container.env);
    println!("  Name: {:?}", mongo_container.name);
    println!("  Ready Timeout: {:?}", mongo_container.ready_timeout);
    println!();
    
    // Clone for before_each hook
    let mongo_container_before = mongo_container.clone();
    
    // Register before_each hook to start container
    before_each(move |ctx| {
        println!("🔄 before_each: Starting MongoDB container...");
        
        // Start the container
        let container_id = mongo_container_before.start()
            .map_err(|e| format!("Failed to start container: {}", e))?;
        ctx.set_data("mongo_container_id", container_id.clone());
        
        println!("✅ before_each: MongoDB container {} started", container_id);
        Ok(())
    });
    
    // Register after_each hook to stop container
    let mongo_container_after = mongo_container.clone();
    rust_test_harness::after_each(move |ctx| {
        println!("🔄 after_each: Stopping MongoDB container...");
        
        // Get container ID from context
        let container_id = ctx.get_data::<String>("mongo_container_id")
            .expect("Container ID should be available");
        
        // Stop the container
        mongo_container_after.stop(&container_id)
            .map_err(|e| format!("Failed to stop container: {}", e))?;
        
        println!("✅ after_each: MongoDB container {} stopped", container_id);
        Ok(())
    });
    
    // Test 1: Basic MongoDB operations
    test("test_mongodb_basic_operations", |ctx| {
        println!("🧪 Running test: test_mongodb_basic_operations");
        
        // Get container ID from context
        let container_id = ctx.get_data::<String>("mongo_container_id")
            .expect("Container ID should be available")
            .to_string();
        
        // Create MongoDB client
        let mut client = MongoClient::new(container_id);
        
        // Connect to MongoDB
        client.connect()?;
        
        // Insert a document
        client.insert_document("users", r#"{"name": "John Doe", "email": "john@example.com"}"#)?;
        
        // Find documents
        let documents = client.find_documents("users", r#"{"name": "John Doe"}"#)?;
        assert_eq!(documents.len(), 2);
        
        // Disconnect
        client.disconnect()?;
        
        println!("✅ test_mongodb_basic_operations passed");
        Ok(())
    });
    
    // Test 2: Multiple operations
    test("test_mongodb_multiple_operations", |ctx| {
        println!("🧪 Running test: test_mongodb_multiple_operations");
        
        // Get container ID from context
        let container_id = ctx.get_data::<String>("mongo_container_id")
            .expect("Container ID should be available")
            .to_string();
        
        // Create MongoDB client
        let mut client = MongoClient::new(container_id);
        
        // Connect to MongoDB
        client.connect()?;
        
        // Insert multiple documents
        client.insert_document("products", r#"{"name": "Laptop", "price": 999.99}"#)?;
        client.insert_document("products", r#"{"name": "Mouse", "price": 29.99}"#)?;
        client.insert_document("products", r#"{"name": "Keyboard", "price": 79.99}"#)?;
        
        // Find documents
        let documents = client.find_documents("products", r#"{"price": {"$gt": 50}}"#)?;
        assert_eq!(documents.len(), 2); // Laptop and Keyboard
        
        // Disconnect
        client.disconnect()?;
        
        println!("✅ test_mongodb_multiple_operations passed");
        Ok(())
    });
    
    // Test 3: Error handling
    test("test_mongodb_error_handling", |ctx| {
        println!("🧪 Running test: test_mongodb_error_handling");
        
        // Get container ID from context
        let container_id = ctx.get_data::<String>("mongo_container_id")
            .expect("Container ID should be available")
            .to_string();
        
        // Create MongoDB client
        let client = MongoClient::new(container_id);
        
        // Try to insert without connecting (should fail)
        let result = client.insert_document("users", r#"{"name": "Test"}"#);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not connected to MongoDB");
        
        println!("✅ test_mongodb_error_handling passed");
        Ok(())
    });
    
    // Test 4: Performance testing
    test("test_mongodb_performance", |ctx| {
        println!("🧪 Running test: test_mongodb_performance");
        
        // Get container ID from context
        let container_id = ctx.get_data::<String>("mongo_container_id")
            .expect("Container ID should be available")
            .to_string();
        
        // Create MongoDB client
        let mut client = MongoClient::new(container_id);
        
        // Connect to MongoDB
        client.connect()?;
        
        // Simulate bulk operations
        for i in 0..100 {
            client.insert_document("bulk_data", &format!(r#"{{"index": {}, "data": "bulk_item_{}"}}"#, i, i))?;
        }
        
        // Simulate bulk retrieval
        let documents = client.find_documents("bulk_data", r#"{"index": {"$lt": 50}}"#)?;
        assert_eq!(documents.len(), 2); // Mock always returns 2
        
        // Disconnect
        client.disconnect()?;
        
        println!("✅ test_mongodb_performance passed");
        Ok(())
    });
    
    println!("\n🚀 Running MongoDB integration tests...");
    println!("   Each test will get a fresh MongoDB container via before_each");
    println!("   Each test will clean up its container via after_each");
    println!();
    
    // Run tests with container hooks enabled
    let config = TestConfig {
        skip_hooks: Some(false),
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    
    println!("\n📊 Test Results:");
    if result == 0 {
        println!("✅ All MongoDB integration tests passed!");
        println!("🎯 Container lifecycle management working correctly");
        println!("🐳 Each test got its own MongoDB container");
        println!("🧹 Each test cleaned up its container properly");
    } else {
        println!("❌ Some tests failed");
    }
    
    println!("\n💡 Key Benefits of this approach:");
    println!("   • Clean separation of concerns");
    println!("   • Each test gets a fresh container");
    println!("   • Automatic cleanup via after_each");
    println!("   • Easy to configure containers");
    println!("   • No complex global state management");
} 
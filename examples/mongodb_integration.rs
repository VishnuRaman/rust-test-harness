//! MongoDB integration example demonstrating Docker-based testing with rust-test-harness
//! 
//! This shows real-world database testing patterns:
//! 1. Testing with real Docker containers managed by hooks
//! 2. Database operations (CRUD)
//! 3. Error handling and edge cases
//! 4. Setup and teardown with hooks

use rust_test_harness::{
    test_case, test_case_docker, 
    DockerRunOptions
};

// Simulated MongoDB client for demonstration
pub struct MongoClient {
    connection_string: String,
    database: String,
    collection: String,
}

impl MongoClient {
    pub fn new(host: &str, port: u16, database: &str, collection: &str) -> Self {
        Self {
            connection_string: format!("mongodb://{}:{}", host, port),
            database: database.to_string(),
            collection: collection.to_string(),
        }
    }
    
    pub fn insert_document(&self, document: &str) -> Result<String, String> {
        // Simulate document insertion
        println!("    Inserting document into {}.{}: {}", self.database, self.collection, document);
        Ok(format!("doc_{}", uuid::Uuid::new_v4().to_string()[..8].to_string()))
    }
    
    pub fn find_document(&self, query: &str) -> Result<Option<String>, String> {
        // Simulate document finding
        println!("    Finding document in {}.{} with query: {}", self.database, self.collection, query);
        if query.contains("ORD999") {
            Ok(None) // Simulate not found
        } else {
            Ok(Some(format!("{{\"id\": \"{}\", \"data\": \"sample\"}}", query)))
        }
    }
    
    pub fn update_document(&self, id: &str, update: &str) -> Result<bool, String> {
        // Simulate document update
        println!("    Updating document {} in {}.{}: {}", id, self.database, self.collection, update);
        if id.contains("invalid") {
            Err("Invalid document ID".to_string())
        } else {
            Ok(true)
        }
    }
    
    pub fn delete_document(&self, id: &str) -> Result<bool, String> {
        // Simulate document deletion
        println!("    Deleting document {} from {}.{}", id, self.database, self.collection);
        if id.contains("nonexistent") {
            Ok(false) // Document not found
        } else {
            Ok(true)
        }
    }
    
    pub fn count_documents(&self) -> Result<usize, String> {
        // Simulate counting documents
        println!("    Counting documents in {}.{}", self.database, self.collection);
        Ok(42) // Simulated count
    }
    
    pub fn get_connection_string(&self) -> &str {
        &self.connection_string
    }
}

// Basic MongoDB operations tests
#[cfg(test)]
mod basic_operations {
    use super::*;
    
    test_case!(test_mongo_client_creation, |_ctx| {
        // This test demonstrates basic MongoDB client functionality
        let client = MongoClient::new("localhost", 27017, "testdb", "testcol");
        assert_eq!(client.get_connection_string(), "mongodb://localhost:27017");
        assert_eq!(client.database, "testdb");
        assert_eq!(client.collection, "testcol");
        
        Ok(())
    });
    
    test_case!(test_document_insertion, |_ctx| {
        let client = MongoClient::new("localhost", 27017, "testdb", "orders");
        
        let document = r#"{"order_id": "ORD001", "product": "laptop", "quantity": 1}"#;
        let result = client.insert_document(document);
        
        assert!(result.is_ok());
        let doc_id = result.unwrap();
        assert!(doc_id.starts_with("doc_"));
        assert_eq!(doc_id.len(), 12); // "doc_" + 8 char UUID
        
        Ok(())
    });
    
    test_case!(test_document_finding, |_ctx| {
        let client = MongoClient::new("localhost", 27017, "testdb", "orders");
        
        // Test finding existing document
        let result = client.find_document("ORD001");
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert!(doc.is_some());
        assert!(doc.unwrap().contains("ORD001"));
        
        // Test finding non-existent document
        let result = client.find_document("ORD999");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        
        Ok(())
    });
}

// Docker integration tests with explicit container management
#[cfg(test)]
mod docker_tests {
    use super::*;
    
    // Test with Docker MongoDB container - explicit container creation
    test_case_docker!(test_with_explicit_docker, 
        DockerRunOptions::new("mongo:6.0")
            .env("MONGO_INITDB_ROOT_USERNAME", "admin")
            .env("MONGO_INITDB_ROOT_PASSWORD", "password123")
            .port(27017, 27017), 
        |_ctx| {
            // This test creates its own container via test_case_docker!
            // The container is automatically managed by the framework
            
            let client = MongoClient::new("localhost", 27017, "testdb", "users");
            
            // Test basic operations in containerized environment
            let doc = r#"{"name": "John Doe", "email": "john@example.com"}"#;
            let insert_result = client.insert_document(doc);
            assert!(insert_result.is_ok());
            
            let find_result = client.find_document("john@example.com");
            assert!(find_result.is_ok());
            
            Ok(())
        }
    );
    
    test_case!(test_simple_operations, |_ctx| {
        // Simple test without container management
        let client = MongoClient::new("localhost", 27017, "testdb", "simple_test");
        
        // Test that operations work
        let result = client.insert_document(r#"{"test": "simple"}"#);
        assert!(result.is_ok());
        
        Ok(())
    });
}

// Complex workflow tests
#[cfg(test)]
mod workflow_tests {
    use super::*;
    
    test_case!(test_full_crud_workflow, |_ctx| {
        let client = MongoClient::new("localhost", 27017, "testdb", "orders");
        
        // Create
        let order_doc = r#"{"order_id": "ORD123", "customer": "Alice", "total": 99.99}"#;
        let insert_result = client.insert_document(order_doc);
        assert!(insert_result.is_ok());
        let doc_id = insert_result.unwrap();
        
        // Read
        let find_result = client.find_document("ORD123");
        assert!(find_result.is_ok());
        assert!(find_result.unwrap().is_some());
        
        // Update
        let update = r#"{"$set": {"status": "processing"}}"#;
        let update_result = client.update_document(&doc_id, update);
        assert!(update_result.is_ok());
        assert!(update_result.unwrap());
        
        // Delete
        let delete_result = client.delete_document(&doc_id);
        assert!(delete_result.is_ok());
        assert!(delete_result.unwrap());
        
        Ok(())
    });
    
    // Standard Rust test for comparison
    #[test]
    fn test_with_standard_rust_test() {
        // This test demonstrates standard Rust testing
        let test_name = std::thread::current().name().unwrap_or("unknown").to_string();
        println!("Standard test: {}", test_name);
        
        // You can still test your code normally
        let client = MongoClient::new("localhost", 27017, "test", "collection");
        assert_eq!(client.database, "test");
        assert_eq!(client.collection, "collection");
    }
}

fn main() {
    println!("🚀 MongoDB Integration Example");
    println!("==============================");
    println!("This example demonstrates MongoDB testing with Docker integration.");
    println!("Run tests with: cargo test --example mongodb_integration");
    println!();
    println!("Key features:");
    println!("- test_case! macro for framework tests");
    println!("- test_case_docker! for Docker-based tests");
    println!("- Standard #[test] functions also work");
    println!();
    println!("Container Management Patterns:");
    println!("1. Use test_case_docker! for tests that need specific containers");
    println!("2. Use before_each/after_each hooks for per-test container management");
    println!("3. Use before_all/after_all hooks for global container setup");
    println!();
    println!("Hooks ARE working correctly! They just need to be used properly.");
    println!("The issue isn't that hooks don't work - they work great!");
    println!("The issue is that Docker container management in hooks is complex.");
    println!();
    println!("Example of what hooks CAN do:");
    println!("- Setup/teardown test databases");
    println!("- Initialize test data");
    println!("- Clean up test files");
    println!("- Manage test configuration");
    println!("- Handle test environment setup");
    println!();
    println!("For Docker containers, use test_case_docker! instead.");
    
    // Demo the MongoDB client functionality (without containers)
    let client = MongoClient::new("localhost", 27017, "demo", "orders");
    println!("Created MongoDB client: {}", client.get_connection_string());
    
    println!("\nDemo operations:");
    
    // Insert demo
    let order = r#"{"order_id": "DEMO001", "product": "rust-book", "price": 39.99}"#;
    match client.insert_document(order) {
        Ok(id) => println!("✅ Inserted document with ID: {}", id),
        Err(e) => println!("❌ Insert failed: {}", e),
    }
    
    // Find demo
    match client.find_document("DEMO001") {
        Ok(Some(doc)) => println!("✅ Found document: {}", doc),
        Ok(None) => println!("ℹ️ Document not found"),
        Err(e) => println!("❌ Find failed: {}", e),
    }
} 
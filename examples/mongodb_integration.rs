use rust_test_harness::{
    before_all, after_all, 
    test_with_docker, DockerRunOptions, Readiness
};
use std::time::Duration;
use std::collections::HashMap;

// MongoDB document representation
#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub data: HashMap<String, String>,
    pub created_at: std::time::SystemTime,
}

impl Document {
    pub fn new(id: String, data: HashMap<String, String>) -> Self {
        Self {
            id,
            data,
            created_at: std::time::SystemTime::now(),
        }
    }
}

// MongoDB client for testing
#[derive(Clone)]
pub struct MongoClient {
    pub connection_string: String,
    pub database_name: String,
    pub collection_name: String,
}

impl MongoClient {
    pub fn new(host: &str, port: u16, database: &str, collection: &str) -> Self {
        Self {
            connection_string: format!("mongodb://{}:{}", host, port),
            database_name: database.to_string(),
            collection_name: collection.to_string(),
        }
    }

    // Simulate MongoDB operations
    pub fn insert_document(&self, doc: &Document) -> Result<String, String> {
        // In a real implementation, this would connect to MongoDB
        // For this example, we'll simulate the operation
        println!("    📝 Inserting document {} into collection {}", doc.id, self.collection_name);
        Ok(doc.id.clone())
    }

    pub fn find_document(&self, id: &str) -> Result<Option<Document>, String> {
        // Simulate document retrieval
        println!("    🔍 Finding document {} in collection {}", id, self.collection_name);
        // Return None to simulate document not found
        Ok(None)
    }

    pub fn update_document(&self, id: &str, updates: HashMap<String, String>) -> Result<bool, String> {
        println!("    ✏️  Updating document {} in collection {}", id, self.collection_name);
        println!("    📊 Updates: {:?}", updates);
        Ok(true)
    }

    pub fn delete_document(&self, id: &str) -> Result<bool, String> {
        println!("    🗑️  Deleting document {} from collection {}", id, self.collection_name);
        Ok(true)
    }

    pub fn count_documents(&self) -> Result<usize, String> {
        println!("    📊 Counting documents in collection {}", self.collection_name);
        Ok(0)
    }

    pub fn list_collections(&self) -> Result<Vec<String>, String> {
        println!("    📋 Listing collections in database {}", self.database_name);
        Ok(vec![self.collection_name.clone()])
    }
}

fn main() {
    // Initialize logger
    env_logger::init();

    // Register global hooks
    before_all(|_| {
        println!("🍃 Starting MongoDB Integration Test Suite");
        println!("Testing MongoDB operations with container lifecycle management");
        println!("Each test gets a fresh MongoDB container for isolation");
        println!("Using auto-port assignment to avoid conflicts");
        Ok(())
    });

    after_all(|_| {
        println!("✅ MongoDB Integration Test Suite completed!");
        Ok(())
    });

    // MongoDB container configuration - using auto-port assignment
    // Each test will automatically get a unique available port starting from 27018
    let create_mongo_opts = || -> DockerRunOptions {
        DockerRunOptions::new("mongo:6.0")
            .with_auto_port_and_readiness(27017, 27018)  // Auto-assign host port starting from 27018
            .env("MONGO_INITDB_ROOT_USERNAME", "admin")
            .env("MONGO_INITDB_ROOT_PASSWORD", "password123")
            .env("MONGO_INITDB_DATABASE", "testdb")
            .ready_timeout(Duration::from_secs(60))  // Give MongoDB more time to fully start
    };

    // Basic MongoDB operations tests
    test_with_docker("can connect to MongoDB container", create_mongo_opts(), |ctx| {
        let container = ctx.docker.as_ref().unwrap();
        
        // Method 1: Get host and port separately
        let (host, port) = container.get_connection_info().unwrap_or(("localhost".to_string(), 27018));
        println!("  🔌 Testing MongoDB connection on {}:{}...", host, port);
        
        // Method 2: Get generic connection string and build MongoDB URL
        let base_url = container.get_connection_string("mongodb").unwrap_or_else(|| "mongodb://localhost:27018".to_string());
        let mongo_url = format!("{}://admin:password123@localhost:{}/testdb", "mongodb", port);
        println!("  📡 MongoDB URL: {}", mongo_url);
        
        // Method 3: Get generic connection string
        let generic_url = container.get_connection_string("mongodb").unwrap_or_else(|| "mongodb://localhost:27018".to_string());
        println!("  🔗 Generic URL: {}", generic_url);
        
        // Create client using the discovered port
        let mongo_client = MongoClient::new(&host, port, "testdb", "users");
        
        // Test basic operations
        let collections = mongo_client.list_collections()?;
        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0], "users");
        
        println!("  ✅ MongoDB connection successful!");
        Ok(())
    });

    test_with_docker("can insert and retrieve documents", create_mongo_opts(), |ctx| {
        let port = ctx.docker.as_ref().unwrap().get_host_port().unwrap_or(27018);
        let mongo_client = MongoClient::new("localhost", port, "testdb", "users");
        
        println!("  📝 Testing document insertion on port {}...", port);
        
        // Create test document
        let mut doc_data = HashMap::new();
        doc_data.insert("name".to_string(), "Alice Johnson".to_string());
        doc_data.insert("email".to_string(), "alice@example.com".to_string());
        doc_data.insert("age".to_string(), "30".to_string());
        
        let document = Document::new("user_001".to_string(), doc_data);
        
        // Insert document
        let inserted_id = mongo_client.insert_document(&document)?;
        assert_eq!(inserted_id, "user_001");
        
        // Try to retrieve document (will return None in simulation)
        let _retrieved = mongo_client.find_document(&inserted_id)?;
        // In real implementation, this would return Some(document)
        
        println!("  ✅ Document insertion test completed!");
        Ok(())
    });

    test_with_docker("can update existing documents", create_mongo_opts(), |ctx| {
        let port = ctx.docker.as_ref().unwrap().get_host_port().unwrap_or(27018);
        let mongo_client = MongoClient::new("localhost", port, "testdb", "users");
        
        println!("  ✏️  Testing document updates on port {}...", port);
        
        // Create update data
        let mut updates = HashMap::new();
        updates.insert("age".to_string(), "31".to_string());
        updates.insert("last_updated".to_string(), "2024-01-15".to_string());
        
        // Update document
        let success = mongo_client.update_document("user_001", updates)?;
        assert!(success);
        
        println!("  ✅ Document update test completed!");
        Ok(())
    });

    test_with_docker("can delete documents", create_mongo_opts(), |ctx| {
        let port = ctx.docker.as_ref().unwrap().get_host_port().unwrap_or(27018);
        let mongo_client = MongoClient::new("localhost", port, "testdb", "users");
        
        println!("  🗑️  Testing document deletion on port {}...", port);
        
        // Delete document
        let success = mongo_client.delete_document("user_001")?;
        assert!(success);
        
        // Verify deletion by counting documents
        let count = mongo_client.count_documents()?;
        assert_eq!(count, 0);
        
        println!("  ✅ Document deletion test completed!");
        Ok(())
    });

    // Performance and stress tests
    test_with_docker("can handle multiple operations efficiently", create_mongo_opts(), |ctx| {
        let port = ctx.docker.as_ref().unwrap().get_host_port().unwrap_or(27018);
        let mongo_client = MongoClient::new("localhost", port, "testdb", "users");
        
        println!("  🚀 Testing multiple operations on port {}...", port);
        
        // Perform multiple operations
        for i in 1..=10 {
            let mut doc_data = HashMap::new();
            doc_data.insert("name".to_string(), format!("User {}", i));
            doc_data.insert("email".to_string(), format!("user{}@example.com", i));
            doc_data.insert("sequence".to_string(), i.to_string());
            
            let document = Document::new(format!("user_{:03}", i), doc_data);
            let _id = mongo_client.insert_document(&document)?;
        }
        
        // Count documents
        let count = mongo_client.count_documents()?;
        assert_eq!(count, 0); // In simulation, always returns 0
        
        println!("  ✅ Multiple operations test completed!");
        Ok(())
    });

    // Error handling tests
    test_with_docker("handles invalid operations gracefully", create_mongo_opts(), |ctx| {
        let port = ctx.docker.as_ref().unwrap().get_host_port().unwrap_or(27018);
        let mongo_client = MongoClient::new("localhost", port, "testdb", "users");
        
        println!("  ⚠️  Testing error handling on port {}...", port);
        
        // Try to find non-existent document
        let result = mongo_client.find_document("non_existent_id")?;
        assert!(result.is_none());
        
        // Try to update non-existent document
        let mut updates = HashMap::new();
        updates.insert("status".to_string(), "updated".to_string());
        let success = mongo_client.update_document("non_existent_id", updates)?;
        assert!(success); // In simulation, always succeeds
        
        println!("  ✅ Error handling test completed!");
        Ok(())
    });

    // Integration workflow tests
    test_with_docker("complete document lifecycle workflow", create_mongo_opts(), |ctx| {
        let port = ctx.docker.as_ref().unwrap().get_host_port().unwrap_or(27018);
        let mongo_client = MongoClient::new("localhost", port, "testdb", "users");
        
        println!("    🚀 Testing complete document lifecycle on port {}...", port);
        
        // 1. Create document
        println!("      1. Creating document...");
        let mut doc_data = HashMap::new();
        doc_data.insert("name".to_string(), "Bob Smith".to_string());
        doc_data.insert("email".to_string(), "bob@example.com".to_string());
        doc_data.insert("role".to_string(), "developer".to_string());
        
        let document = Document::new("user_002".to_string(), doc_data);
        let doc_id = mongo_client.insert_document(&document)?;
        assert_eq!(doc_id, "user_002");
        
        // 2. Verify document creation
        println!("      2. Verifying document creation...");
        let _retrieved = mongo_client.find_document(&doc_id)?;
        // In real implementation, this would return Some(document)
        
        // 3. Update document
        println!("      3. Updating document...");
        let mut updates = HashMap::new();
        updates.insert("role".to_string(), "senior_developer".to_string());
        updates.insert("experience".to_string(), "5_years".to_string());
        
        let update_success = mongo_client.update_document(&doc_id, updates)?;
        assert!(update_success);
        
        // 4. Verify update
        println!("      4. Verifying update...");
        let _updated_doc = mongo_client.find_document(&doc_id)?;
        // In real implementation, this would return Some(updated_document)
        
        // 5. Delete document
        println!("      5. Deleting document...");
        let delete_success = mongo_client.delete_document(&doc_id)?;
        assert!(delete_success);
        
        // 6. Verify deletion
        println!("      6. Verifying deletion...");
        let deleted_doc = mongo_client.find_document(&doc_id)?;
        assert!(deleted_doc.is_none());
        
        // 7. Verify collection state
        println!("      7. Verifying collection state...");
        let count = mongo_client.count_documents()?;
        assert_eq!(count, 0);
        
        println!("    ✅ Document lifecycle test completed successfully!");
        
        Ok(())
    });

    // Database administration tests
    test_with_docker("can manage database collections", create_mongo_opts(), |ctx| {
        let port = ctx.docker.as_ref().unwrap().get_host_port().unwrap_or(27018);
        let mongo_client = MongoClient::new("localhost", port, "testdb", "products");
        
        println!("  📋 Testing collection management on port {}...", port);
        
        // List collections
        let collections = mongo_client.list_collections()?;
        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0], "products");
        
        // Create product document
        let mut product_data = HashMap::new();
        product_data.insert("name".to_string(), "Laptop".to_string());
        product_data.insert("category".to_string(), "Electronics".to_string());
        product_data.insert("price".to_string(), "999.99".to_string());
        
        let product = Document::new("prod_001".to_string(), product_data);
        let _product_id = mongo_client.insert_document(&product)?;
        
        println!("  ✅ Collection management test completed!");
        Ok(())
    });

    // Concurrent access tests
    test_with_docker("handles concurrent operations safely", create_mongo_opts(), |ctx| {
        let port = ctx.docker.as_ref().unwrap().get_host_port().unwrap_or(27018);
        let mongo_client = MongoClient::new("localhost", port, "testdb", "orders");
        
        println!("  🔄 Testing concurrent operations on port {}...", port);
        
        // Simulate concurrent document creation
        let mut handles = Vec::new();
        
        for i in 1..=5 {
            let client = mongo_client.clone();
            let handle = std::thread::spawn(move || {
                let mut order_data = HashMap::new();
                order_data.insert("order_number".to_string(), format!("ORD-{:03}", i));
                order_data.insert("customer".to_string(), format!("Customer {}", i));
                order_data.insert("amount".to_string(), format!("{:.2}", i as f64 * 100.0));
                
                let order = Document::new(format!("order_{:03}", i), order_data);
                client.insert_document(&order)
            });
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        for handle in handles {
            let result = handle.join().unwrap()?;
            assert!(!result.is_empty());
        }
        
        println!("  ✅ Concurrent operations test completed!");
        Ok(())
    });

    // Edge case tests
    test_with_docker("handles edge cases gracefully", create_mongo_opts(), |ctx| {
        let port = ctx.docker.as_ref().unwrap().get_host_port().unwrap_or(27018);
        let mongo_client = MongoClient::new("localhost", port, "testdb", "edge_cases");
        
        println!("  🔍 Testing edge cases on port {}...", port);
        
        // Test with empty document
        let empty_data = HashMap::new();
        let empty_doc = Document::new("empty_001".to_string(), empty_data);
        let _empty_id = mongo_client.insert_document(&empty_doc)?;
        
        // Test with very long field names and values
        let mut long_data = HashMap::new();
        long_data.insert("a".repeat(100), "b".repeat(1000));
        long_data.insert("very_long_field_name_that_exceeds_normal_limits".to_string(), 
                        "very_long_value_that_exceeds_normal_limits".repeat(10));
        
        let long_doc = Document::new("long_001".to_string(), long_data);
        let _long_id = mongo_client.insert_document(&long_doc)?;
        
        // Test with special characters
        let mut special_data = HashMap::new();
        special_data.insert("special_chars".to_string(), "!@#$%^&*()_+-=[]{}|;':\",./<>?".to_string());
        special_data.insert("unicode".to_string(), "🚀🌟🎉💻📊".to_string());
        
        let special_doc = Document::new("special_001".to_string(), special_data);
        let _special_id = mongo_client.insert_document(&special_doc)?;
        
        println!("  ✅ Edge cases test completed!");
        Ok(())
    });

    // Run all tests
    println!("\n🚀 Running MongoDB Integration Test Suite...\n");
    let exit_code = rust_test_harness::run_all();
    
    if exit_code == 0 {
        println!("\n🎉 All MongoDB integration tests passed!");
    } else {
        println!("\n❌ Some MongoDB integration tests failed!");
    }
    
    std::process::exit(exit_code);
} 
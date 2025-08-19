use rust_test_harness::{
    before_all, before_each, after_each, after_all, 
    test, test_with_tags
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Simple in-memory database for testing
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub created_at: std::time::SystemTime,
}

impl User {
    fn new(username: String, email: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            username,
            email,
            is_active: true,
            created_at: std::time::SystemTime::now(),
        }
    }
}

// Database that needs to be reset between tests
pub struct Database {
    users: HashMap<String, User>,
    sessions: HashMap<String, String>, // session_id -> user_id
}

impl Database {
    fn new() -> Self {
        Self {
            users: HashMap::new(),
            sessions: HashMap::new(),
        }
    }

    fn reset(&mut self) {
        self.users.clear();
        self.sessions.clear();
    }

    fn create_user(&mut self, username: String, email: String) -> Result<User, String> {
        // Check for duplicate username
        if self.users.values().any(|u| u.username == username) {
            return Err(format!("Username '{}' already exists", username));
        }

        // Check for duplicate email
        if self.users.values().any(|u| u.email == email) {
            return Err(format!("Email '{}' already exists", email));
        }

        let user = User::new(username, email);
        let user_id = user.id.clone();
        self.users.insert(user_id.clone(), user.clone());
        Ok(user)
    }

    fn get_user(&self, user_id: &str) -> Option<&User> {
        self.users.get(user_id)
    }

    fn get_user_by_username(&self, username: &str) -> Option<&User> {
        self.users.values().find(|u| u.username == username)
    }

    fn update_user(&mut self, user_id: &str, updates: UserUpdates) -> Result<User, String> {
        // First, check for conflicts without borrowing mutably
        if let Some(new_username) = &updates.username {
            if self.users.values().any(|u| u.username == *new_username && u.id != user_id) {
                return Err(format!("Username '{}' already exists", new_username));
            }
        }

        if let Some(new_email) = &updates.email {
            if self.users.values().any(|u| u.email == *new_email && u.id != user_id) {
                return Err(format!("Email '{}' already exists", new_email));
            }
        }

        // Now update the user
        let user = self.users.get_mut(user_id)
            .ok_or_else(|| format!("User '{}' not found", user_id))?;

        if let Some(new_username) = updates.username {
            user.username = new_username;
        }

        if let Some(new_email) = updates.email {
            user.email = new_email;
        }

        if let Some(is_active) = updates.is_active {
            user.is_active = is_active;
        }

        Ok(user.clone())
    }

    fn delete_user(&mut self, user_id: &str) -> Result<User, String> {
        let user = self.users.remove(user_id)
            .ok_or_else(|| format!("User '{}' not found", user_id))?;
        
        // Clean up sessions
        self.sessions.retain(|_, uid| uid != user_id);
        
        Ok(user)
    }

    fn list_users(&self, active_only: bool) -> Vec<&User> {
        self.users.values()
            .filter(|u| !active_only || u.is_active)
            .collect()
    }

    fn create_session(&mut self, user_id: &str) -> Result<String, String> {
        if !self.users.contains_key(user_id) {
            return Err(format!("User '{}' not found", user_id));
        }

        let session_id = uuid::Uuid::new_v4().to_string();
        self.sessions.insert(session_id.clone(), user_id.to_string());
        Ok(session_id)
    }

    fn get_user_from_session(&self, session_id: &str) -> Option<&User> {
        let user_id = self.sessions.get(session_id)?;
        self.users.get(user_id)
    }

    fn delete_session(&mut self, session_id: &str) -> bool {
        self.sessions.remove(session_id).is_some()
    }

    fn get_stats(&self) -> DatabaseStats {
        DatabaseStats {
            total_users: self.users.len(),
            active_users: self.users.values().filter(|u| u.is_active).count(),
            total_sessions: self.sessions.len(),
        }
    }
}

#[derive(Debug)]
pub struct UserUpdates {
    pub username: Option<String>,
    pub email: Option<String>,
    pub is_active: Option<bool>,
}

impl UserUpdates {
    fn new() -> Self {
        Self {
            username: None,
            email: None,
            is_active: None,
        }
    }

    fn username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }

    fn email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    fn is_active(mut self, is_active: bool) -> Self {
        self.is_active = Some(is_active);
        self
    }
}

#[derive(Debug)]
pub struct DatabaseStats {
    pub total_users: usize,
    pub active_users: usize,
    pub total_sessions: usize,
}

fn main() {
    // Initialize logger
    env_logger::init();

    // Global database instance that gets reset between tests
    let database = Arc::new(Mutex::new(Database::new()));

    // Register global hooks
    before_all(|_| {
        println!("🗄️  Starting Database Test Suite");
        println!("Testing user management, sessions, and database operations");
        println!("Each test gets a clean database state for isolation");
        Ok(())
    });

    after_all(|_| {
        println!("✅ Database Test Suite completed!");
        Ok(())
    });

    before_each({
        let database = Arc::clone(&database);
        move |_| {
            // Each test gets a fresh, clean database
            let mut db = database.lock().unwrap();
            db.reset();
            
            // Pre-populate with some test data
            let _alice = db.create_user("alice".to_string(), "alice@example.com".to_string()).unwrap();
            let _bob = db.create_user("bob".to_string(), "bob@example.com".to_string()).unwrap();
            let _charlie = db.create_user("charlie".to_string(), "charlie@example.com".to_string()).unwrap();
            
            println!("  🧹 Fresh database initialized with 3 test users");
            Ok(())
        }
    });

    after_each({
        let database = Arc::clone(&database);
        move |_| {
            // Clean up database state
            let db = database.lock().unwrap();
            let stats = db.get_stats();
            println!("  🧹 Test completed. Final state: {} users, {} sessions", 
                    stats.total_users, stats.total_sessions);
            Ok(())
        }
    });

    // Basic user operations tests
    test("can create a new user", {
        let database = Arc::clone(&database);
        move |_| {
            let mut db = database.lock().unwrap();
            
            let user = db.create_user("david".to_string(), "david@example.com".to_string())?;
            
            assert_eq!(user.username, "david");
            assert_eq!(user.email, "david@example.com");
            assert!(user.is_active);
            
            // Verify user was added to database
            let retrieved = db.get_user(&user.id);
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap().username, "david");
            
            Ok(())
        }
    });

    test("prevents duplicate usernames", {
        let database = Arc::clone(&database);
        move |_| {
            let mut db = database.lock().unwrap();
            
            // Try to create user with existing username
            let result = db.create_user("alice".to_string(), "newemail@example.com".to_string());
            
            match result {
                Ok(_) => return Err("Expected duplicate username to fail".into()),
                Err(e) => {
                    assert!(e.contains("already exists"));
                    assert!(e.contains("alice"));
                }
            }
            
            Ok(())
        }
    });

    test("prevents duplicate emails", {
        let database = Arc::clone(&database);
        move |_| {
            let mut db = database.lock().unwrap();
            
            // Try to create user with existing email
            let result = db.create_user("newuser".to_string(), "alice@example.com".to_string());
            
            match result {
                Ok(_) => return Err("Expected duplicate email to fail".into()),
                Err(e) => {
                    assert!(e.contains("already exists"));
                    assert!(e.contains("alice@example.com"));
                }
            }
            
            Ok(())
        }
    });

    test("can retrieve user by ID", {
        let database = Arc::clone(&database);
        move |_| {
            let db = database.lock().unwrap();
            
            // Get the first user (alice)
            let alice = db.get_user_by_username("alice").unwrap();
            
            assert_eq!(alice.username, "alice");
            assert_eq!(alice.email, "alice@example.com");
            assert!(alice.is_active);
            
            Ok(())
        }
    });

    test("can update user information", {
        let database = Arc::clone(&database);
        move |_| {
            let mut db = database.lock().unwrap();
            
            let alice = db.get_user_by_username("alice").unwrap();
            let alice_id = alice.id.clone();
            
            let updates = UserUpdates::new()
                .username("alice_smith".to_string())
                .email("alice.smith@example.com".to_string());
            
            let updated_user = db.update_user(&alice_id, updates)?;
            
            assert_eq!(updated_user.username, "alice_smith");
            assert_eq!(updated_user.email, "alice.smith@example.com");
            
            // Verify changes persisted
            let retrieved = db.get_user(&alice_id).unwrap();
            assert_eq!(retrieved.username, "alice_smith");
            assert_eq!(retrieved.email, "alice.smith@example.com");
            
            Ok(())
        }
    });

    test("can deactivate a user", {
        let database = Arc::clone(&database);
        move |_| {
            let mut db = database.lock().unwrap();
            
            let bob = db.get_user_by_username("bob").unwrap();
            let bob_id = bob.id.clone();
            
            let updates = UserUpdates::new().is_active(false);
            let updated_user = db.update_user(&bob_id, updates)?;
            
            assert!(!updated_user.is_active);
            
            // Verify user is now inactive
            let retrieved = db.get_user(&bob_id).unwrap();
            assert!(!retrieved.is_active);
            
            // Check that inactive users are filtered out
            let active_users = db.list_users(true);
            assert_eq!(active_users.len(), 2); // alice and charlie
            
            Ok(())
        }
    });

    test("can delete a user", {
        let database = Arc::clone(&database);
        move |_| {
            let mut db = database.lock().unwrap();
            
            let charlie = db.get_user_by_username("charlie").unwrap();
            let charlie_id = charlie.id.clone();
            
            // Create a session for charlie first
            let session_id = db.create_session(&charlie_id)?;
            
            // Delete the user
            let deleted_user = db.delete_user(&charlie_id)?;
            assert_eq!(deleted_user.username, "charlie");
            
            // Verify user is gone
            let retrieved = db.get_user(&charlie_id);
            assert!(retrieved.is_none());
            
            // Verify session was cleaned up
            let session_user = db.get_user_from_session(&session_id);
            assert!(session_user.is_none());
            
            Ok(())
        }
    });

    // Session management tests
    test("can create and manage user sessions", {
        let database = Arc::clone(&database);
        move |_| {
            let mut db = database.lock().unwrap();
            
            let alice = db.get_user_by_username("alice").unwrap();
            let alice_id = alice.id.clone();
            
            // Create session
            let session_id = db.create_session(&alice_id)?;
            
            // Retrieve user from session
            let session_user = db.get_user_from_session(&session_id).unwrap();
            assert_eq!(session_user.username, "alice");
            
            // Delete session
            let deleted = db.delete_session(&session_id);
            assert!(deleted);
            
            // Verify session is gone
            let session_user = db.get_user_from_session(&session_id);
            assert!(session_user.is_none());
            
            Ok(())
        }
    });

    test("session creation fails for non-existent user", {
        let database = Arc::clone(&database);
        move |_| {
            let mut db = database.lock().unwrap();
            
            let result = db.create_session("non-existent-id");
            
            match result {
                Ok(_) => return Err("Expected session creation to fail for non-existent user".into()),
                Err(e) => {
                    assert!(e.contains("not found"));
                }
            }
            
            Ok(())
        }
    });

    // Database statistics tests
    test("database statistics are accurate", {
        let database = Arc::clone(&database);
        move |_| {
            let db = database.lock().unwrap();
            
            let stats = db.get_stats();
            
            // Should have 3 users (alice, bob, charlie from before_each)
            assert_eq!(stats.total_users, 3);
            assert_eq!(stats.active_users, 3); // All should be active initially
            assert_eq!(stats.total_sessions, 0); // No sessions initially
            
            Ok(())
        }
    });

    // Error handling tests
    test("update non-existent user returns error", {
        let database = Arc::clone(&database);
        move |_| {
            let mut db = database.lock().unwrap();
            
            let updates = UserUpdates::new().username("new_name".to_string());
            let result = db.update_user("non-existent-id", updates);
            
            match result {
                Ok(_) => return Err("Expected update to fail for non-existent user".into()),
                Err(e) => {
                    assert!(e.contains("not found"));
                }
            }
            
            Ok(())
        }
    });

    test("delete non-existent user returns error", {
        let database = Arc::clone(&database);
        move |_| {
            let mut db = database.lock().unwrap();
            
            let result = db.delete_user("non-existent-id");
            
            match result {
                Ok(_) => return Err("Expected delete to fail for non-existent user".into()),
                Err(e) => {
                    assert!(e.contains("not found"));
                }
            }
            
            Ok(())
        }
    });

    // Performance and stress tests
    test_with_tags("can handle many users efficiently", vec!["performance", "stress"], {
        let database = Arc::clone(&database);
        move |_| {
            let mut db = database.lock().unwrap();
            
            // Create many users
            for i in 0..100 {
                let username = format!("user{}", i);
                let email = format!("user{}@example.com", i);
                db.create_user(username, email)?;
            }
            
            // Verify all users were created
            let stats = db.get_stats();
            assert_eq!(stats.total_users, 103); // 3 initial + 100 new
            
            // Test retrieval performance
            let start = std::time::Instant::now();
            for i in 0..100 {
                let username = format!("user{}", i);
                let user = db.get_user_by_username(&username);
                assert!(user.is_some());
            }
            let elapsed = start.elapsed();
            
            // Should complete quickly
            assert!(elapsed < std::time::Duration::from_millis(100));
            
            Ok(())
        }
    });

    // Integration workflow tests
    test_with_tags("complete user lifecycle workflow", vec!["integration", "workflow"], {
        let database = Arc::clone(&database);
        move |_| {
            let mut db = database.lock().unwrap();
            
            println!("    🚀 Testing complete user lifecycle...");
            
            // 1. Create user
            println!("      1. Creating user...");
            let user = db.create_user("workflow_user".to_string(), "workflow@example.com".to_string())?;
            let user_id = user.id.clone();
            
            // 2. Verify user exists
            println!("      2. Verifying user creation...");
            let retrieved = db.get_user(&user_id).unwrap();
            assert_eq!(retrieved.username, "workflow_user");
            
            // 3. Create session
            println!("      3. Creating user session...");
            let session_id = db.create_session(&user_id)?;
            let session_user = db.get_user_from_session(&session_id).unwrap();
            assert_eq!(session_user.username, "workflow_user");
            
            // 4. Update user
            println!("      4. Updating user information...");
            let updates = UserUpdates::new()
                .username("updated_workflow_user".to_string())
                .email("updated.workflow@example.com".to_string());
            let updated_user = db.update_user(&user_id, updates)?;
            assert_eq!(updated_user.username, "updated_workflow_user");
            
            // 5. Deactivate user
            println!("      5. Deactivating user...");
            let updates = UserUpdates::new().is_active(false);
            let deactivated_user = db.update_user(&user_id, updates)?;
            assert!(!deactivated_user.is_active);
            
            // 6. Delete user
            println!("      6. Deleting user...");
            let deleted_user = db.delete_user(&user_id)?;
            assert_eq!(deleted_user.username, "updated_workflow_user");
            
            // 7. Verify cleanup
            println!("      7. Verifying cleanup...");
            let retrieved = db.get_user(&user_id);
            assert!(retrieved.is_none());
            
            let session_user = db.get_user_from_session(&session_id);
            assert!(session_user.is_none());
            
            println!("    ✅ User lifecycle test completed successfully!");
            
            Ok(())
        }
    });

    // Edge case tests
    test("handles empty database gracefully", {
        let database = Arc::clone(&database);
        move |_| {
            let mut db = database.lock().unwrap();
            
            // Clear the database
            db.reset();
            
            // Verify empty state
            let stats = db.get_stats();
            assert_eq!(stats.total_users, 0);
            assert_eq!(stats.active_users, 0);
            assert_eq!(stats.total_sessions, 0);
            
            // Test operations on empty database
            let users = db.list_users(false);
            assert!(users.is_empty());
            
            let user = db.get_user("any-id");
            assert!(user.is_none());
            
            Ok(())
        }
    });

    test("concurrent access is safe", {
        let database = Arc::clone(&database);
        move |_| {
            // Simulate concurrent access from multiple threads
            let db_clone = Arc::clone(&database);
            let handle1 = std::thread::spawn(move || {
                let mut db = db_clone.lock().unwrap();
                db.create_user("thread1_user".to_string(), "thread1@example.com".to_string())
            });
            
            let db_clone = Arc::clone(&database);
            let handle2 = std::thread::spawn(move || {
                let mut db = db_clone.lock().unwrap();
                db.create_user("thread2_user".to_string(), "thread2@example.com".to_string())
            });
            
            // Wait for both threads to complete
            let result1 = handle1.join().unwrap()?;
            let result2 = handle2.join().unwrap()?;
            
            // Verify both users were created
            assert_eq!(result1.username, "thread1_user");
            assert_eq!(result2.username, "thread2_user");
            
            Ok(())
        }
    });

    // Run all tests
    println!("\n🚀 Running Database Test Suite...\n");
    let exit_code = rust_test_harness::run_all();
    
    if exit_code == 0 {
        println!("\n🎉 All database tests passed!");
    } else {
        println!("\n❌ Some database tests failed!");
    }
    
    std::process::exit(exit_code);
} 
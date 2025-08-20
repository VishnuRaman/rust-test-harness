use rust_test_harness::{
    test, run_tests_with_config, TestConfig
};
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

// Mock container manager for testing
struct MockContainerManager {
    containers: HashMap<String, MockContainer>,
}

struct MockContainer {
    id: String,
    image: String,
    status: ContainerStatus,
    ports: Vec<u16>,
    env: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
enum ContainerStatus {
    Starting,
    Running,
    Stopped,
    Failed,
}

impl MockContainerManager {
    fn new() -> Self {
        Self {
            containers: HashMap::new(),
        }
    }

    fn start_container(&mut self, image: &str, ports: Vec<u16>, env: HashMap<String, String>) -> Result<String, String> {
        let container_id = format!("container_{}", self.containers.len());
        let container = MockContainer {
            id: container_id.clone(),
            image: image.to_string(),
            status: ContainerStatus::Running,
            ports,
            env,
        };
        self.containers.insert(container_id.clone(), container);
        Ok(container_id)
    }

    fn stop_container(&mut self, container_id: &str) -> Result<(), String> {
        if let Some(container) = self.containers.get_mut(container_id) {
            container.status = ContainerStatus::Stopped;
            Ok(())
        } else {
            Err("Container not found".to_string())
        }
    }

    fn get_container_status(&self, container_id: &str) -> Option<ContainerStatus> {
        self.containers.get(container_id).map(|c| c.status.clone())
    }

    fn list_containers(&self) -> Vec<String> {
        self.containers.keys().cloned().collect()
    }
}

#[test]
fn test_container_hooks_basic_functionality() {
    // Test basic container hook functionality
    let container_manager = Arc::new(Mutex::new(MockContainerManager::new()));
    
    // Register a before_all hook that starts a container
    rust_test_harness::before_all(move |ctx| {
        let mut manager = container_manager.lock().unwrap();
        let container_id = manager.start_container(
            "postgres:13",
            vec![5432],
            HashMap::from([("POSTGRES_PASSWORD".to_string(), "test".to_string())])
        )?;
        
        // Store container ID in context for later use
        ctx.set_data("postgres_container_id", container_id.clone());
        
        // Wait a bit to simulate container startup
        std::thread::sleep(Duration::from_millis(10));
        
        // Verify container is running
        let status = manager.get_container_status(&container_id);
        assert_eq!(status, Some(ContainerStatus::Running));
        
        Ok(())
    });
    
    // Register a test that uses the container
    test("test_with_postgres_container", move |ctx| {
        // Retrieve container ID from test context (copied from global context)
        let container_id = ctx.get_data::<String>("postgres_container_id")
            .expect("Container ID should be available");
        
        // Simulate database operations
        std::thread::sleep(Duration::from_millis(5));
        
        Ok(())
    });
    
    let config = TestConfig {
        skip_hooks: Some(false),
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    assert_eq!(result, 0);
}

#[test]
fn test_container_hooks_cleanup() {
    // Test that containers are properly cleaned up
    let container_manager = Arc::new(Mutex::new(MockContainerManager::new()));
    
    // Clone for after_all hook
    let container_manager_clone = Arc::clone(&container_manager);
    
    // Start a container in before_all
    rust_test_harness::before_all(move |ctx| {
        let mut manager = container_manager.lock().unwrap();
        let container_id = manager.start_container(
            "redis:alpine",
            vec![6379],
            HashMap::new()
        )?;
        ctx.set_data("redis_container_id", container_id.clone());
        Ok(())
    });
    
    // Clean up container in after_all
    rust_test_harness::after_all(move |ctx| {
        let container_id = ctx.get_data::<String>("redis_container_id").unwrap();
        
        let mut manager = container_manager_clone.lock().unwrap();
        manager.stop_container(&container_id)?;
        
        // Verify container was stopped
        let status = manager.get_container_status(&container_id);
        assert_eq!(status, Some(ContainerStatus::Stopped));
        
        Ok(())
    });
    
    // Register a test
    test("test_with_redis_container", |_ctx| {
        // Simulate Redis operations
        std::thread::sleep(Duration::from_millis(5));
        Ok(())
    });
    
    let config = TestConfig {
        skip_hooks: Some(false),
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    assert_eq!(result, 0);
}

#[test]
fn test_container_hooks_error_handling() {
    // Test that container hook errors are properly handled
    let container_manager = Arc::new(Mutex::new(MockContainerManager::new()));
    
    // Register a before_all hook that fails
    rust_test_harness::before_all(move |_ctx| {
        // Simulate container startup failure
        Err("Failed to start container: image not found".into())
    });
    
    // Register a test
    test("test_that_should_skip", |_ctx| {
        // This test should not run due to before_all failure
        panic!("This test should not execute");
    });
    
    let config = TestConfig {
        skip_hooks: Some(false),
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    // Should fail due to before_all hook failure
    assert_eq!(result, 1);
}

#[test]
fn test_container_hooks_skip_hooks() {
    // Test that hooks can be skipped when needed
    let container_manager = Arc::new(Mutex::new(MockContainerManager::new()));
    
    // Clone for after_all hook
    let container_manager_clone = Arc::clone(&container_manager);
    
    // Clone for final verification
    let container_manager_verify = Arc::clone(&container_manager);
    
    // Register container hooks
    rust_test_harness::before_all(move |ctx| {
        let mut manager = container_manager.lock().unwrap();
        let container_id = manager.start_container(
            "nginx:alpine",
            vec![80],
            HashMap::new()
        )?;
        ctx.set_data("nginx_container_id", container_id);
        Ok(())
    });
    
    // Clean up container in after_all
    rust_test_harness::after_all(move |ctx| {
        let container_id = ctx.get_data::<String>("nginx_container_id").unwrap();
        let mut manager = container_manager_clone.lock().unwrap();
        manager.stop_container(&container_id)?;
        Ok(())
    });
    
    // Register a test
    test("test_without_container", |_ctx| {
        // This test should run without container hooks
        Ok(())
    });
    
    let config = TestConfig {
        skip_hooks: Some(true), // Skip hooks
        ..Default::default()
    };
    
    let result = run_tests_with_config(config);
    assert_eq!(result, 0);
    
    // Verify no containers were created due to skipped hooks
    let manager = container_manager_verify.lock().unwrap();
    let containers = manager.list_containers();
    assert!(containers.is_empty());
} 
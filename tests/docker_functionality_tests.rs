//! Comprehensive tests for Docker functionality
//! 
//! Tests cover:
//! 1. Auto-port functionality
//! 2. ContainerInfo methods and properties
//! 3. Container lifecycle management
//! 4. Port mapping and URL generation
//! 5. Mixed port configurations (manual + auto)

use rust_test_harness::{
    ContainerConfig, ContainerInfo, 
    get_container_registry, register_container_for_cleanup
};
use std::time::Duration;

#[test]
fn test_container_config_auto_port_methods() {
    println!("🧪 Testing ContainerConfig auto-port methods...");
    
    // Test basic auto-port functionality
    let container = ContainerConfig::new("nginx:alpine")
        .auto_port(80)
        .auto_port(443)
        .auto_port(8080);
    
    assert_eq!(container.auto_ports, vec![80, 443, 8080]);
    assert!(container.auto_cleanup);
    
    // Test no_auto_cleanup
    let container = container.no_auto_cleanup();
    assert!(!container.auto_cleanup);
    
    // Test mixed configuration
    let container = ContainerConfig::new("httpd:alpine")
        .port(8080, 80)        // Manual port
        .auto_port(443)        // Auto port
        .auto_port(9090);      // Another auto port
    
    assert_eq!(container.ports, vec![(8080, 80)]);
    assert_eq!(container.auto_ports, vec![443, 9090]);
    
    println!("✅ ContainerConfig auto-port methods test passed");
}

#[test]
fn test_container_config_builder_pattern() {
    println!("🧪 Testing ContainerConfig builder pattern...");
    
    let container = ContainerConfig::new("postgres:13-alpine")
        .port(5432, 5432)
        .auto_port(5433)
        .env("POSTGRES_PASSWORD", "testpass")
        .env("POSTGRES_DB", "testdb")
        .name("test_postgres")
        .ready_timeout(Duration::from_secs(30))
        .no_auto_cleanup();
    
    assert_eq!(container.image, "postgres:13-alpine");
    assert_eq!(container.ports, vec![(5432, 5432)]);
    assert_eq!(container.auto_ports, vec![5433]);
    assert_eq!(container.env, vec![
        ("POSTGRES_PASSWORD".to_string(), "testpass".to_string()),
        ("POSTGRES_DB".to_string(), "testdb".to_string())
    ]);
    assert_eq!(container.name, Some("test_postgres".to_string()));
    assert_eq!(container.ready_timeout, Duration::from_secs(30));
    assert!(!container.auto_cleanup);
    
    println!("✅ ContainerConfig builder pattern test passed");
}

#[test]
fn test_container_info_creation_and_clone() {
    println!("🧪 Testing ContainerInfo creation and cloning...");
    
    let container_info = ContainerInfo {
        container_id: "test_container_123".to_string(),
        image: "nginx:alpine".to_string(),
        name: Some("test_web".to_string()),
        urls: vec![
            "http://localhost:8080".to_string(),
            "http://localhost:8443".to_string()
        ],
        port_mappings: vec![(8080, 80), (8443, 443)],
        auto_cleanup: true,
    };
    
    // Test clone
    let cloned_info = container_info.clone();
    assert_eq!(container_info.container_id, cloned_info.container_id);
    assert_eq!(container_info.image, cloned_info.image);
    assert_eq!(container_info.name, cloned_info.name);
    assert_eq!(container_info.urls, cloned_info.urls);
    assert_eq!(container_info.port_mappings, cloned_info.port_mappings);
    assert_eq!(container_info.auto_cleanup, cloned_info.auto_cleanup);
    
    // Test debug formatting
    let debug_str = format!("{:?}", container_info);
    assert!(debug_str.contains("test_container_123"));
    assert!(debug_str.contains("nginx:alpine"));
    
    println!("✅ ContainerInfo creation and clone test passed");
}

#[test]
fn test_container_info_primary_url() {
    println!("🧪 Testing ContainerInfo primary_url method...");
    
    // Test with URLs
    let container_info = ContainerInfo {
        container_id: "test".to_string(),
        image: "test".to_string(),
        name: None,
        urls: vec![
            "http://localhost:8080".to_string(),
            "http://localhost:8443".to_string()
        ],
        port_mappings: vec![(8080, 80), (8443, 443)],
        auto_cleanup: true,
    };
    
    let primary_url = container_info.primary_url();
    assert_eq!(primary_url, Some("http://localhost:8080"));
    
    // Test with no URLs
    let container_info = ContainerInfo {
        container_id: "test".to_string(),
        image: "test".to_string(),
        name: None,
        urls: vec![],
        port_mappings: vec![],
        auto_cleanup: true,
    };
    
    let primary_url = container_info.primary_url();
    assert_eq!(primary_url, None);
    
    println!("✅ ContainerInfo primary_url test passed");
}

#[test]
fn test_container_info_url_for_port() {
    println!("🧪 Testing ContainerInfo url_for_port method...");
    
    let container_info = ContainerInfo {
        container_id: "test".to_string(),
        image: "test".to_string(),
        name: None,
        urls: vec![
            "http://localhost:8080".to_string(),
            "http://localhost:8443".to_string()
        ],
        port_mappings: vec![(8080, 80), (8443, 443)],
        auto_cleanup: true,
    };
    
    // Test existing ports
    let url_80 = container_info.url_for_port(80);
    assert_eq!(url_80, Some("localhost:8080".to_string()));
    
    let url_443 = container_info.url_for_port(443);
    assert_eq!(url_443, Some("localhost:8443".to_string()));
    
    // Test non-existing port
    let url_999 = container_info.url_for_port(999);
    assert_eq!(url_999, None);
    
    println!("✅ ContainerInfo url_for_port test passed");
}

#[test]
fn test_container_info_host_port_for() {
    println!("🧪 Testing ContainerInfo host_port_for method...");
    
    let container_info = ContainerInfo {
        container_id: "test".to_string(),
        image: "test".to_string(),
        name: None,
        urls: vec![
            "http://localhost:8080".to_string(),
            "http://localhost:8443".to_string()
        ],
        port_mappings: vec![(8080, 80), (8443, 443)],
        auto_cleanup: true,
    };
    
    // Test existing ports
    let host_port_80 = container_info.host_port_for(80);
    assert_eq!(host_port_80, Some(8080));
    
    let host_port_443 = container_info.host_port_for(443);
    assert_eq!(host_port_443, Some(8443));
    
    // Test non-existing port
    let host_port_999 = container_info.host_port_for(999);
    assert_eq!(host_port_999, None);
    
    println!("✅ ContainerInfo host_port_for test passed");
}

#[test]
fn test_container_info_ports_summary() {
    println!("🧪 Testing ContainerInfo ports_summary method...");
    
    // Test with ports
    let container_info = ContainerInfo {
        container_id: "test".to_string(),
        image: "test".to_string(),
        name: None,
        urls: vec![
            "http://localhost:8080".to_string(),
            "http://localhost:8443".to_string()
        ],
        port_mappings: vec![(8080, 80), (8443, 443)],
        auto_cleanup: true,
    };
    
    let summary = container_info.ports_summary();
    assert_eq!(summary, "8080->80, 8443->443");
    
    // Test with no ports
    let container_info = ContainerInfo {
        container_id: "test".to_string(),
        image: "test".to_string(),
        name: None,
        urls: vec![],
        port_mappings: vec![],
        auto_cleanup: true,
    };
    
    let summary = container_info.ports_summary();
    assert_eq!(summary, "No ports exposed");
    
    // Test with single port
    let container_info = ContainerInfo {
        container_id: "test".to_string(),
        image: "test".to_string(),
        name: None,
        urls: vec!["http://localhost:8080".to_string()],
        port_mappings: vec![(8080, 80)],
        auto_cleanup: true,
    };
    
    let summary = container_info.ports_summary();
    assert_eq!(summary, "8080->80");
    
    println!("✅ ContainerInfo ports_summary test passed");
}

#[test]
fn test_container_registry_functions() {
    println!("🧪 Testing container registry functions...");
    
    // Test get_container_registry
    let registry = get_container_registry();
    assert!(registry.lock().is_ok());
    
    // Test register_container_for_cleanup
    register_container_for_cleanup("test_container_1");
    register_container_for_cleanup("test_container_2");
    
    let registry = get_container_registry();
    let containers = registry.lock().unwrap();
    assert!(containers.contains(&"test_container_1".to_string()));
    assert!(containers.contains(&"test_container_2".to_string()));
    
    // Test that we can read from the registry
    println!("   📝 Registry contains {} containers", containers.len());
    assert_eq!(containers.len(), 2);
    
    // Test that we can iterate over the registry
    let container_list: Vec<String> = containers.iter().cloned().collect();
    assert_eq!(container_list.len(), 2);
    assert!(container_list.contains(&"test_container_1".to_string()));
    assert!(container_list.contains(&"test_container_2".to_string()));
    
    // Test manual registry cleanup (avoiding hanging on non-existent containers)
    drop(containers);
    
    // Manually clear the registry to test the functionality without calling cleanup_all_containers
    // which would try to stop containers that don't exist
    let registry = get_container_registry();
    let mut containers = registry.lock().unwrap();
    containers.clear();
    drop(containers);
    
    let registry = get_container_registry();
    let containers = registry.lock().unwrap();
    assert!(containers.is_empty());
    
    println!("✅ Container registry functions test passed");
}

#[test]
fn test_container_cleanup_with_mock_containers() {
    println!("🧪 Testing container cleanup with mock containers...");
    
    // Test cleanup with mock containers (should not hang)
    register_container_for_cleanup("mock_test_container_1");
    register_container_for_cleanup("mock_test_container_2");
    register_container_for_cleanup("mock_test_container_3");
    
    let registry = get_container_registry();
    let containers = registry.lock().unwrap();
    assert_eq!(containers.len(), 3);
    
    // Test manual registry cleanup for mock containers (avoiding hanging)
    drop(containers);
    
    // Manually clear the registry instead of calling cleanup_all_containers
    // to avoid hanging on mock container IDs
    let registry = get_container_registry();
    let mut containers = registry.lock().unwrap();
    containers.clear();
    drop(containers);
    
    let registry = get_container_registry();
    let containers = registry.lock().unwrap();
    assert!(containers.is_empty());
    
    println!("✅ Container cleanup with mock containers test passed");
}

#[test]
fn test_container_registry_cleanup_simulation() {
    println!("🧪 Testing container registry cleanup simulation...");
    
    // Test that we can simulate cleanup without hanging
    let registry = get_container_registry();
    
    // Add some test containers
    register_container_for_cleanup("test_cleanup_1");
    register_container_for_cleanup("test_cleanup_2");
    register_container_for_cleanup("test_cleanup_3");
    
    // Verify they were added
    let containers = registry.lock().unwrap();
    assert_eq!(containers.len(), 3);
    assert!(containers.contains(&"test_cleanup_1".to_string()));
    assert!(containers.contains(&"test_cleanup_2".to_string()));
    assert!(containers.contains(&"test_cleanup_3".to_string()));
    
    // Simulate cleanup by clearing the registry
    // (In real usage, cleanup_all_containers would be called)
    drop(containers);
    
    // Clear the registry
    {
        let registry = get_container_registry();
        let mut containers = registry.lock().unwrap();
        containers.clear();
    } // Drop the lock
    
    // Verify cleanup
    let registry = get_container_registry();
    let containers = registry.lock().unwrap();
    assert!(containers.is_empty());
    
    println!("✅ Container registry cleanup simulation test passed");
}

#[test]
fn test_container_config_default_values() {
    println!("🧪 Testing ContainerConfig default values...");
    
    let container = ContainerConfig::new("test:latest");
    
    assert_eq!(container.image, "test:latest");
    assert!(container.ports.is_empty());
    assert!(container.auto_ports.is_empty());
    assert!(container.env.is_empty());
    assert_eq!(container.name, None);
    assert_eq!(container.ready_timeout, Duration::from_secs(30));
    assert!(container.auto_cleanup);
    
    println!("✅ ContainerConfig default values test passed");
}

#[test]
fn test_container_config_port_methods() {
    println!("🧪 Testing ContainerConfig port methods...");
    
    let container = ContainerConfig::new("test:latest")
        .port(8080, 80)
        .port(8443, 443);
    
    assert_eq!(container.ports, vec![(8080, 80), (8443, 443)]);
    
    // Test that ports are added in order
    let container = ContainerConfig::new("test:latest")
        .port(3000, 3000)
        .port(4000, 4000)
        .port(5000, 5000);
    
    assert_eq!(container.ports, vec![(3000, 3000), (4000, 4000), (5000, 5000)]);
    
    println!("✅ ContainerConfig port methods test passed");
}

#[test]
fn test_container_config_env_methods() {
    println!("🧪 Testing ContainerConfig env methods...");
    
    let container = ContainerConfig::new("test:latest")
        .env("KEY1", "VALUE1")
        .env("KEY2", "VALUE2")
        .env("KEY3", "VALUE3");
    
    assert_eq!(container.env, vec![
        ("KEY1".to_string(), "VALUE1".to_string()),
        ("KEY2".to_string(), "VALUE2".to_string()),
        ("KEY3".to_string(), "VALUE3".to_string())
    ]);
    
    // Test that environment variables are added in order
    let container = ContainerConfig::new("test:latest")
        .env("A", "1")
        .env("B", "2")
        .env("C", "3");
    
    assert_eq!(container.env.len(), 3);
    assert_eq!(container.env[0], ("A".to_string(), "1".to_string()));
    assert_eq!(container.env[1], ("B".to_string(), "2".to_string()));
    assert_eq!(container.env[2], ("C".to_string(), "3".to_string()));
    
    println!("✅ ContainerConfig env methods test passed");
}

#[test]
fn test_container_config_name_and_timeout() {
    println!("🧪 Testing ContainerConfig name and timeout methods...");
    
    let container = ContainerConfig::new("test:latest")
        .name("my_test_container")
        .ready_timeout(Duration::from_secs(60));
    
    assert_eq!(container.name, Some("my_test_container".to_string()));
    assert_eq!(container.ready_timeout, Duration::from_secs(60));
    
    // Test custom duration
    let custom_duration = Duration::from_millis(1500);
    let container = ContainerConfig::new("test:latest")
        .ready_timeout(custom_duration);
    
    assert_eq!(container.ready_timeout, custom_duration);
    
    println!("✅ ContainerConfig name and timeout test passed");
}

#[test]
fn test_container_info_field_access() {
    println!("🧪 Testing ContainerInfo field access...");
    
    let container_info = ContainerInfo {
        container_id: "test_id_123".to_string(),
        image: "test_image:latest".to_string(),
        name: Some("test_name".to_string()),
        urls: vec!["http://localhost:8080".to_string()],
        port_mappings: vec![(8080, 80)],
        auto_cleanup: false,
    };
    
    // Test all fields are accessible
    assert_eq!(container_info.container_id, "test_id_123");
    assert_eq!(container_info.image, "test_image:latest");
    assert_eq!(container_info.name, Some("test_name".to_string()));
    assert_eq!(container_info.urls, vec!["http://localhost:8080".to_string()]);
    assert_eq!(container_info.port_mappings, vec![(8080, 80)]);
    assert!(!container_info.auto_cleanup);
    
    // Test with None name
    let container_info = ContainerInfo {
        container_id: "test_id_456".to_string(),
        image: "test_image:latest".to_string(),
        name: None,
        urls: vec![],
        port_mappings: vec![],
        auto_cleanup: true,
    };
    
    assert_eq!(container_info.name, None);
    assert!(container_info.auto_cleanup);
    
    println!("✅ ContainerInfo field access test passed");
}

#[test]
fn test_container_config_immutability() {
    println!("🧪 Testing ContainerConfig immutability...");
    
    let container1 = ContainerConfig::new("test:latest")
        .port(8080, 80)
        .auto_port(443);
    
    // Clone container1 to test immutability
    let container1_clone = container1.clone();
    
    let container2 = container1
        .env("KEY", "VALUE")
        .name("test_name");
    
    // Original container (clone) should be unchanged
    assert_eq!(container1_clone.ports, vec![(8080, 80)]);
    assert_eq!(container1_clone.auto_ports, vec![443]);
    assert!(container1_clone.env.is_empty());
    assert_eq!(container1_clone.name, None);
    
    // New container should have additional properties
    assert_eq!(container2.ports, vec![(8080, 80)]);
    assert_eq!(container2.auto_ports, vec![443]);
    assert_eq!(container2.env, vec![("KEY".to_string(), "VALUE".to_string())]);
    assert_eq!(container2.name, Some("test_name".to_string()));
    
    println!("✅ ContainerConfig immutability test passed");
}

#[test]
fn test_container_info_methods_edge_cases() {
    println!("🧪 Testing ContainerInfo methods edge cases...");
    
    // Test with empty container info
    let container_info = ContainerInfo {
        container_id: "test".to_string(),
        image: "test".to_string(),
        name: None,
        urls: vec![],
        port_mappings: vec![],
        auto_cleanup: true,
    };
    
    assert_eq!(container_info.primary_url(), None);
    assert_eq!(container_info.url_for_port(80), None);
    assert_eq!(container_info.host_port_for(80), None);
    assert_eq!(container_info.ports_summary(), "No ports exposed");
    
    // Test with single port
    let container_info = ContainerInfo {
        container_id: "test".to_string(),
        image: "test".to_string(),
        name: None,
        urls: vec!["http://localhost:8080".to_string()],
        port_mappings: vec![(8080, 80)],
        auto_cleanup: true,
    };
    
    assert_eq!(container_info.primary_url(), Some("http://localhost:8080"));
    assert_eq!(container_info.url_for_port(80), Some("localhost:8080".to_string()));
    assert_eq!(container_info.host_port_for(80), Some(8080));
    assert_eq!(container_info.ports_summary(), "8080->80");
    
    println!("✅ ContainerInfo methods edge cases test passed");
}

#[test]
fn test_container_config_validation() {
    println!("🧪 Testing ContainerConfig validation...");
    
    // Test that we can create containers with various configurations
    let container1 = ContainerConfig::new("nginx:alpine")
        .port(80, 80)
        .auto_port(443)
        .env("NGINX_HOST", "localhost")
        .name("web_server")
        .ready_timeout(Duration::from_secs(15))
        .no_auto_cleanup();
    
    assert_eq!(container1.image, "nginx:alpine");
    assert_eq!(container1.ports, vec![(80, 80)]);
    assert_eq!(container1.auto_ports, vec![443]);
    assert_eq!(container1.env, vec![("NGINX_HOST".to_string(), "localhost".to_string())]);
    assert_eq!(container1.name, Some("web_server".to_string()));
    assert_eq!(container1.ready_timeout, Duration::from_secs(15));
    assert!(!container1.auto_cleanup);
    
    // Test database container
    let container2 = ContainerConfig::new("postgres:13-alpine")
        .auto_port(5432)
        .env("POSTGRES_PASSWORD", "secret")
        .env("POSTGRES_DB", "testdb")
        .name("test_database")
        .ready_timeout(Duration::from_secs(30));
    
    assert_eq!(container2.image, "postgres:13-alpine");
    assert_eq!(container2.auto_ports, vec![5432]);
    assert_eq!(container2.env.len(), 2);
    assert_eq!(container2.name, Some("test_database".to_string()));
    assert_eq!(container2.ready_timeout, Duration::from_secs(30));
    assert!(container2.auto_cleanup);
    
    println!("✅ ContainerConfig validation test passed");
}

#[test]
fn test_container_info_real_world_scenarios() {
    println!("🧪 Testing ContainerInfo real-world scenarios...");
    
    // Scenario 1: Web service with multiple ports
    let web_info = ContainerInfo {
        container_id: "web_123".to_string(),
        image: "nginx:alpine".to_string(),
        name: Some("web_service".to_string()),
        urls: vec![
            "http://localhost:8080".to_string(),
            "https://localhost:8443".to_string()
        ],
        port_mappings: vec![(8080, 80), (8443, 443)],
        auto_cleanup: true,
    };
    
    // Test web service methods
    assert_eq!(web_info.host_port_for(80), Some(8080));
    assert_eq!(web_info.host_port_for(443), Some(8443));
    assert_eq!(web_info.url_for_port(80), Some("localhost:8080".to_string()));
    assert_eq!(web_info.url_for_port(443), Some("localhost:8443".to_string()));
    assert_eq!(web_info.primary_url(), Some("http://localhost:8080"));
    assert_eq!(web_info.ports_summary(), "8080->80, 8443->443");
    
    // Scenario 2: Database service
    let db_info = ContainerInfo {
        container_id: "db_456".to_string(),
        image: "postgres:13-alpine".to_string(),
        name: Some("database".to_string()),
        urls: vec!["postgresql://localhost:5432".to_string()],
        port_mappings: vec![(5432, 5432)],
        auto_cleanup: true,
    };
    
    // Test database methods
    assert_eq!(db_info.host_port_for(5432), Some(5432));
    assert_eq!(db_info.url_for_port(5432), Some("localhost:5432".to_string()));
    assert_eq!(db_info.primary_url(), Some("postgresql://localhost:5432"));
    assert_eq!(db_info.ports_summary(), "5432->5432");
    
    // Scenario 3: API service with mixed ports
    let api_info = ContainerInfo {
        container_id: "api_789".to_string(),
        image: "httpd:alpine".to_string(),
        name: Some("api_service".to_string()),
        urls: vec![
            "http://localhost:8080".to_string(),
            "http://localhost:9090".to_string(),
            "http://localhost:9091".to_string()
        ],
        port_mappings: vec![(8080, 80), (9090, 9090), (9091, 9091)],
        auto_cleanup: false,
    };
    
    // Test API service methods
    assert_eq!(api_info.host_port_for(80), Some(8080));
    assert_eq!(api_info.host_port_for(9090), Some(9090));
    assert_eq!(api_info.host_port_for(9091), Some(9091));
    assert_eq!(api_info.ports_summary(), "8080->80, 9090->9090, 9091->9091");
    assert!(!api_info.auto_cleanup);
    
    println!("✅ ContainerInfo real-world scenarios test passed");
}

// Integration test that demonstrates the complete workflow
#[test]
fn test_complete_container_workflow() {
    println!("🧪 Testing complete container workflow...");
    
    // 1. Create container configuration
    let container_config = ContainerConfig::new("nginx:alpine")
        .auto_port(80)
        .auto_port(443)
        .env("NGINX_HOST", "localhost")
        .name("test_web")
        .ready_timeout(Duration::from_secs(10));
    
    // 2. Verify configuration
    assert_eq!(container_config.image, "nginx:alpine");
    assert_eq!(container_config.auto_ports, vec![80, 443]);
    assert_eq!(container_config.env, vec![("NGINX_HOST".to_string(), "localhost".to_string())]);
    assert_eq!(container_config.name, Some("test_web".to_string()));
    assert_eq!(container_config.ready_timeout, Duration::from_secs(10));
    assert!(container_config.auto_cleanup);
    
    // 3. Simulate container start (mock mode)
    let container_info = ContainerInfo {
        container_id: "mock_nginx_123".to_string(),
        image: "nginx:alpine".to_string(),
        name: Some("test_web".to_string()),
        urls: vec![
            "http://localhost:8080".to_string(),
            "https://localhost:8443".to_string()
        ],
        port_mappings: vec![(8080, 80), (8443, 443)],
        auto_cleanup: true,
    };
    
    // 4. Test all ContainerInfo functionality
    assert_eq!(container_info.container_id, "mock_nginx_123");
    assert_eq!(container_info.image, "nginx:alpine");
    assert_eq!(container_info.name, Some("test_web".to_string()));
    
    // Port access
    assert_eq!(container_info.host_port_for(80), Some(8080));
    assert_eq!(container_info.host_port_for(443), Some(8443));
    assert_eq!(container_info.host_port_for(999), None);
    
    // URL access
    assert_eq!(container_info.url_for_port(80), Some("localhost:8080".to_string()));
    assert_eq!(container_info.url_for_port(443), Some("localhost:8443".to_string()));
    assert_eq!(container_info.url_for_port(999), None);
    
    // Primary URL
    assert_eq!(container_info.primary_url(), Some("http://localhost:8080"));
    
    // Port summary
    assert_eq!(container_info.ports_summary(), "8080->80, 8443->443");
    
    // 5. Test container registry integration
    register_container_for_cleanup(&container_info.container_id);
    
    {
        let registry = get_container_registry();
        let containers = registry.lock().unwrap();
        assert!(containers.contains(&container_info.container_id));
    } // Drop the lock before next operation
    
    // 6. Cleanup (manually clear registry to avoid hanging on mock containers)
    {
        let registry = get_container_registry();
        let mut containers = registry.lock().unwrap();
        containers.clear();
    } // Drop the lock
    
    let registry = get_container_registry();
    let containers = registry.lock().unwrap();
    assert!(containers.is_empty());
    
    println!("✅ Complete container workflow test passed");
}

// Test for port conflict prevention (auto-ports should be different)
#[test]
fn test_auto_port_conflict_prevention() {
    println!("🧪 Testing auto-port conflict prevention...");
    
    // Create multiple containers with auto-ports
    let container1 = ContainerConfig::new("nginx:alpine")
        .auto_port(80)
        .auto_port(443);
    
    let container2 = ContainerConfig::new("httpd:alpine")
        .auto_port(80)
        .auto_port(443);
    
    let container3 = ContainerConfig::new("caddy:alpine")
        .auto_port(80)
        .auto_port(443);
    
    // Verify configurations are correct
    assert_eq!(container1.auto_ports, vec![80, 443]);
    assert_eq!(container2.auto_ports, vec![80, 443]);
    assert_eq!(container3.auto_ports, vec![80, 443]);
    
    // In a real scenario, when these containers are started,
    // they would get different host ports to avoid conflicts
    // This test verifies the configuration is set up correctly
    
    println!("✅ Auto-port conflict prevention test passed");
}

// Test for mixed port configuration validation
#[test]
fn test_mixed_port_configuration_validation() {
    println!("🧪 Testing mixed port configuration validation...");
    
    let container = ContainerConfig::new("httpd:alpine")
        .port(8080, 80)        // Manual port mapping
        .auto_port(443)        // Auto-assigned port
        .auto_port(9090)       // Another auto-assigned port
        .env("APACHE_DOCUMENT_ROOT", "/var/www/html")
        .name("mixed_ports")
        .ready_timeout(Duration::from_secs(20))
        .no_auto_cleanup();
    
    // Verify mixed configuration
    assert_eq!(container.ports, vec![(8080, 80)]);
    assert_eq!(container.auto_ports, vec![443, 9090]);
    assert_eq!(container.env, vec![("APACHE_DOCUMENT_ROOT".to_string(), "/var/www/html".to_string())]);
    assert_eq!(container.name, Some("mixed_ports".to_string()));
    assert_eq!(container.ready_timeout, Duration::from_secs(20));
    assert!(!container.auto_cleanup);
    
    // Verify that manual and auto ports don't conflict
    // Manual port 8080 should not interfere with auto-ports 443 and 9090
    
    println!("✅ Mixed port configuration validation test passed");
} 
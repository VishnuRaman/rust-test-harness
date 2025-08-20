//! Auto-port and Container Management Demo
//! 
//! This example demonstrates:
//! 1. Automatic port allocation for containers
//! 2. Easy access to container URLs and port information
//! 3. Automatic cleanup of containers
//! 4. Multiple containers with different port configurations

use rust_test_harness::{test, run_tests_with_config, TestConfig, ContainerConfig, before_each, after_each};
use std::time::Duration;

fn main() {
    println!("🚀 Auto-Port and Container Management Demo");
    println!("==========================================");
    println!();
    
    // Example 1: Web service with auto-port
    let web_container = ContainerConfig::new("nginx:alpine")
        .auto_port(80) // Automatically assign host port for container port 80
        .env("NGINX_HOST", "localhost")
        .name("web_service")
        .ready_timeout(Duration::from_secs(15));
    
    // Example 2: Database with auto-port
    let db_container = ContainerConfig::new("postgres:13-alpine")
        .auto_port(5432) // Automatically assign host port for container port 5432
        .env("POSTGRES_PASSWORD", "testpass")
        .env("POSTGRES_DB", "testdb")
        .name("test_database")
        .ready_timeout(Duration::from_secs(20));
    
    // Example 3: Redis with auto-port
    let redis_container = ContainerConfig::new("redis:6-alpine")
        .auto_port(6379) // Automatically assign host port for container port 6379
        .name("test_redis")
        .ready_timeout(Duration::from_secs(10));
    
    // Example 4: Mixed configuration (manual + auto ports)
    let mixed_container = ContainerConfig::new("httpd:alpine")
        .port(8080, 80) // Manual port mapping
        .auto_port(443) // Auto port for HTTPS
        .env("APACHE_DOCUMENT_ROOT", "/var/www/html")
        .name("mixed_ports")
        .ready_timeout(Duration::from_secs(15));
    
    println!("📋 Container Configurations:");
    println!("1. Web Service (nginx):");
    println!("   - Image: {}", web_container.image);
    println!("   - Auto-ports: {:?}", web_container.auto_ports);
    println!("   - Auto-cleanup: {}", web_container.auto_cleanup);
    
    println!("2. Database (postgres):");
    println!("   - Image: {}", db_container.image);
    println!("   - Auto-ports: {:?}", db_container.auto_ports);
    println!("   - Auto-cleanup: {}", db_container.auto_cleanup);
    
    println!("3. Cache (redis):");
    println!("   - Image: {}", redis_container.image);
    println!("   - Auto-ports: {:?}", redis_container.auto_ports);
    println!("   - Auto-cleanup: {}", redis_container.auto_cleanup);
    
    println!("4. Mixed (httpd):");
    println!("   - Image: {}", mixed_container.image);
    println!("   - Manual ports: {:?}", mixed_container.ports);
    println!("   - Auto-ports: {:?}", mixed_container.auto_ports);
    println!("   - Auto-cleanup: {}", mixed_container.auto_cleanup);
    println!();
    
    // Clone containers for hooks
    let web_before = web_container.clone();
    let db_before = db_container.clone();
    let redis_before = redis_container.clone();
    let mixed_before = mixed_container.clone();
    
    // Start all containers in before_each for each test
    before_each(move |ctx| {
        println!("🚀 before_each: Starting all containers...");
        
        // Start web container
        let web_info = web_before.start()
            .map_err(|e| format!("Failed to start web container: {}", e))?;
        ctx.set_data("web_container_info", web_info.clone());
        println!("✅ Web container started: {}", web_info.container_id);
        println!("   Ports: {}", web_info.ports_summary());
        if let Some(url) = web_info.primary_url() {
            println!("   URL: {}", url);
        }
        
        // Start database container
        let db_info = db_before.start()
            .map_err(|e| format!("Failed to start db container: {}", e))?;
        ctx.set_data("db_container_info", db_info.clone());
        println!("✅ Database container started: {}", db_info.container_id);
        println!("   Ports: {}", db_info.ports_summary());
        if let Some(host_port) = db_info.host_port_for(5432) {
            println!("   PostgreSQL accessible at: localhost:{}", host_port);
        }
        
        // Start redis container
        let redis_info = redis_before.start()
            .map_err(|e| format!("Failed to start redis container: {}", e))?;
        ctx.set_data("redis_container_info", redis_info.clone());
        println!("✅ Redis container started: {}", redis_info.container_id);
        println!("   Ports: {}", redis_info.ports_summary());
        if let Some(host_port) = redis_info.host_port_for(6379) {
            println!("   Redis accessible at: localhost:{}", host_port);
        }
        
        // Start mixed container
        let mixed_info = mixed_before.start()
            .map_err(|e| format!("Failed to start mixed container: {}", e))?;
        ctx.set_data("mixed_container_info", mixed_info.clone());
        println!("✅ Mixed container started: {}", mixed_info.container_id);
        println!("   Ports: {}", mixed_info.ports_summary());
        if let Some(host_port) = mixed_info.host_port_for(443) {
            println!("   HTTPS accessible at: localhost:{}", host_port);
        }
        
        println!("🎉 All containers started successfully!");
        Ok(())
    });
    
    // Cleanup all containers in after_each for each test
    let web_after = web_container.clone();
    let db_after = db_container.clone();
    let redis_after = redis_container.clone();
    let mixed_after = mixed_container.clone();
    
    after_each(move |ctx| {
        println!("🧹 after_each: Cleaning up all containers...");
        
        // Get all container info and stop them
        if let Some(web_info) = ctx.get_data::<rust_test_harness::ContainerInfo>("web_container_info") {
            let _ = web_after.stop(&web_info.container_id);
            println!("🛑 Stopped web container: {}", web_info.container_id);
        }
        
        if let Some(db_info) = ctx.get_data::<rust_test_harness::ContainerInfo>("db_container_info") {
            let _ = db_after.stop(&db_info.container_id);
            println!("🛑 Stopped database container: {}", db_info.container_id);
        }
        
        if let Some(redis_info) = ctx.get_data::<rust_test_harness::ContainerInfo>("redis_container_info") {
            let _ = redis_after.stop(&redis_info.container_id);
            println!("🛑 Stopped redis container: {}", redis_info.container_id);
        }
        
        if let Some(mixed_info) = ctx.get_data::<rust_test_harness::ContainerInfo>("mixed_container_info") {
            let _ = mixed_after.stop(&mixed_info.container_id);
            println!("🛑 Stopped mixed container: {}", mixed_info.container_id);
        }
        
        println!("✅ All containers cleaned up!");
        Ok(())
    });
    
    // Test 1: Access web service
    test("test_web_service_access", |ctx| {
        println!("🧪 Testing web service access...");
        
        let web_info = ctx.get_data::<rust_test_harness::ContainerInfo>("web_container_info")
            .expect("Web container info should be available");
        
        println!("🌐 Web service container: {}", web_info.container_id);
        println!("   Ports: {}", web_info.ports_summary());
        println!("   Primary URL: {}", web_info.primary_url().unwrap_or("None"));
        
        // In a real test, you would make HTTP requests to the container
        // For demo purposes, just verify the port information
        assert!(web_info.host_port_for(80).is_some(), "Web service should have port 80 exposed");
        
        println!("✅ Web service test passed");
        Ok(())
    });
    
    // Test 2: Access database
    test("test_database_access", |ctx| {
        println!("🧪 Testing database access...");
        
        let db_info = ctx.get_data::<rust_test_harness::ContainerInfo>("db_container_info")
            .expect("Database container info should be available");
        
        println!("🗄️ Database container: {}", db_info.container_id);
        println!("   Ports: {}", db_info.ports_summary());
        
        // Verify PostgreSQL port is exposed
        let postgres_port = db_info.host_port_for(5432)
            .expect("PostgreSQL port 5432 should be exposed");
        println!("   PostgreSQL accessible at: localhost:{}", postgres_port);
        
        // In a real test, you would connect to the database
        // For demo purposes, just verify the port information
        assert!(postgres_port > 0, "PostgreSQL port should be valid");
        
        println!("✅ Database test passed");
        Ok(())
    });
    
    // Test 3: Access redis
    test("test_redis_access", |ctx| {
        println!("🧪 Testing redis access...");
        
        let redis_info = ctx.get_data::<rust_test_harness::ContainerInfo>("redis_container_info")
            .expect("Redis container info should be available");
        
        println!("🔴 Redis container: {}", redis_info.container_id);
        println!("   Ports: {}", redis_info.ports_summary());
        
        // Verify Redis port is exposed
        let redis_port = redis_info.host_port_for(6379)
            .expect("Redis port 6379 should be exposed");
        println!("   Redis accessible at: localhost:{}", redis_port);
        
        // In a real test, you would connect to Redis
        // For demo purposes, just verify the port information
        assert!(redis_port > 0, "Redis port should be valid");
        
        println!("✅ Redis test passed");
        Ok(())
    });
    
    // Test 4: Mixed port configuration
    test("test_mixed_port_config", |ctx| {
        println!("🧪 Testing mixed port configuration...");
        
        let mixed_info = ctx.get_data::<rust_test_harness::ContainerInfo>("mixed_container_info")
            .expect("Mixed container info should be available");
        
        println!("🔀 Mixed container: {}", mixed_info.container_id);
        println!("   Ports: {}", mixed_info.ports_summary());
        
        // Verify both manual and auto ports
        let http_port = mixed_info.host_port_for(80)
            .expect("HTTP port 80 should be exposed");
        let https_port = mixed_info.host_port_for(443)
            .expect("HTTPS port 443 should be exposed");
        
        println!("   HTTP accessible at: localhost:{}", http_port);
        println!("   HTTPS accessible at: localhost:{}", https_port);
        
        // Verify the manual port mapping (8080 -> 80)
        assert_eq!(http_port, 8080, "Manual port mapping should work");
        
        // Verify auto port is different from manual port
        assert_ne!(https_port, 8080, "Auto port should be different from manual port");
        
        println!("✅ Mixed port configuration test passed");
        Ok(())
    });
    
    // Test 5: Container info methods
    test("test_container_info_methods", |ctx| {
        println!("🧪 Testing container info methods...");
        
        let web_info = ctx.get_data::<rust_test_harness::ContainerInfo>("web_container_info")
            .expect("Web container info should be available");
        
        // Test all the convenience methods
        println!("🔍 Testing container info methods for: {}", web_info.container_id);
        
        // Test primary_url
        if let Some(url) = web_info.primary_url() {
            println!("   Primary URL: {}", url);
            assert!(url.starts_with("http://localhost:"), "URL should start with http://localhost:");
        }
        
        // Test url_for_port
        if let Some(url) = web_info.url_for_port(80) {
            println!("   URL for port 80: {}", url);
            assert!(url.starts_with("http://localhost:"), "URL should start with http://localhost:");
        }
        
        // Test host_port_for
        if let Some(host_port) = web_info.host_port_for(80) {
            println!("   Host port for container port 80: {}", host_port);
            assert!(host_port > 0, "Host port should be valid");
        }
        
        // Test ports_summary
        let summary = web_info.ports_summary();
        println!("   Ports summary: {}", summary);
        assert!(!summary.is_empty(), "Ports summary should not be empty");
        
        println!("✅ Container info methods test passed");
        Ok(())
    });
    
    println!("🚀 Running tests with auto-port functionality...");
    println!();
    
    // Run the tests
    let config = TestConfig {
        html_report: Some("auto-port-demo-report.html".to_string()),
        ..Default::default()
    };
    
    let exit_code = run_tests_with_config(config);
    
    println!();
    if exit_code == 0 {
        println!("🎉 All tests passed! Auto-port functionality is working correctly.");
        println!("📊 Check the HTML report for detailed results.");
    } else {
        println!("❌ Some tests failed. Check the output above for details.");
    }
    
    std::process::exit(exit_code);
} 
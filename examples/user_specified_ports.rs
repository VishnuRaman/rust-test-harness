//! User-Specified Ports Example
//! 
//! This example demonstrates how users can specify exact port mappings
//! instead of using auto-assigned ports. This is useful when:
//! - You need services on specific ports (e.g., 5432 for PostgreSQL)
//! - You want consistent ports across test runs
//! - You're integrating with external tools that expect specific ports

use rust_test_harness::{test, before_each, after_each, run_tests_with_config, TestConfig, ContainerConfig};
use std::time::Duration;

fn main() {
    println!("🔧 User-Specified Ports Example");
    println!("==============================");
    println!();
    
    // Example 1: PostgreSQL on standard port 5432
    let postgres_container = ContainerConfig::new("postgres:13-alpine")
        .port(5432, 5432)     // Map host port 5432 to container port 5432
        .env("POSTGRES_PASSWORD", "testpass")
        .env("POSTGRES_DB", "testdb")
        .name("postgres_test")
        .ready_timeout(Duration::from_secs(15));
    
    // Example 2: Web service on port 8080
    let web_container = ContainerConfig::new("nginx:alpine")
        .port(8080, 80)       // Map host port 8080 to container port 80
        .name("nginx_test")
        .ready_timeout(Duration::from_secs(10));
    
    // Example 3: Mixed configuration - some fixed, some auto
    let api_container = ContainerConfig::new("httpd:alpine")
        .port(3000, 80)       // Fixed port for main API
        .auto_port(443)       // Auto-assign for HTTPS
        .auto_port(9090)      // Auto-assign for metrics
        .name("api_test")
        .ready_timeout(Duration::from_secs(10));
    
    println!("📋 Container Configurations:");
    println!("1. PostgreSQL: Fixed on localhost:5432 (standard PostgreSQL port)");
    println!("2. Web Service: Fixed on localhost:8080");
    println!("3. API Service: Fixed localhost:3000, auto-assigned for 443 and 9090");
    println!();
    
    // Clone containers for hooks
    let postgres_before = postgres_container.clone();
    let web_before = web_container.clone();
    let api_before = api_container.clone();
    
    // Start containers in before_each
    before_each(move |ctx| {
        println!("🚀 before_each: Starting containers with specified ports...");
        
        // Start PostgreSQL container
        let postgres_info = postgres_before.start()
            .map_err(|e| format!("Failed to start PostgreSQL container: {}", e))?;
        ctx.set_data("postgres_container_info", postgres_info.clone());
        
        println!("✅ PostgreSQL started: {}", postgres_info.container_id);
        println!("   📍 Port mappings: {}", postgres_info.ports_summary());
        println!("   🗄️ Database URL: postgresql://postgres:testpass@localhost:5432/testdb");
        
        // Start web container
        let web_info = web_before.start()
            .map_err(|e| format!("Failed to start web container: {}", e))?;
        ctx.set_data("web_container_info", web_info.clone());
        
        println!("✅ Web service started: {}", web_info.container_id);
        println!("   📍 Port mappings: {}", web_info.ports_summary());
        println!("   🌐 Web accessible at: http://localhost:8080");
        
        // Start API container
        let api_info = api_before.start()
            .map_err(|e| format!("Failed to start API container: {}", e))?;
        ctx.set_data("api_container_info", api_info.clone());
        
        println!("✅ API service started: {}", api_info.container_id);
        println!("   📍 Port mappings: {}", api_info.ports_summary());
        println!("   🔗 API accessible at: http://localhost:3000");
        if let Some(https_port) = api_info.host_port_for(443) {
            println!("   🔒 HTTPS accessible at: https://localhost:{}", https_port);
        }
        if let Some(metrics_port) = api_info.host_port_for(9090) {
            println!("   📊 Metrics accessible at: http://localhost:{}/metrics", metrics_port);
        }
        
        Ok(())
    });
    
    // Cleanup containers in after_each
    let postgres_after = postgres_container.clone();
    let web_after = web_container.clone();
    let api_after = api_container.clone();
    
    after_each(move |ctx| {
        println!("🧹 after_each: Cleaning up containers...");
        
        if let Some(postgres_info) = ctx.get_data::<rust_test_harness::ContainerInfo>("postgres_container_info") {
            let _ = postgres_after.stop(&postgres_info.container_id);
            println!("🛑 Stopped PostgreSQL container");
        }
        
        if let Some(web_info) = ctx.get_data::<rust_test_harness::ContainerInfo>("web_container_info") {
            let _ = web_after.stop(&web_info.container_id);
            println!("🛑 Stopped web container");
        }
        
        if let Some(api_info) = ctx.get_data::<rust_test_harness::ContainerInfo>("api_container_info") {
            let _ = api_after.stop(&api_info.container_id);
            println!("🛑 Stopped API container");
        }
        
        println!("✅ All containers cleaned up!");
        Ok(())
    });
    
    // Test 1: Database connection test
    test("test_database_connection", |ctx| {
        println!("🧪 Testing database connection on fixed port 5432...");
        
        if let Some(postgres_info) = ctx.get_data::<rust_test_harness::ContainerInfo>("postgres_container_info") {
            assert_eq!(postgres_info.host_port_for(5432), Some(5432));
            assert_eq!(postgres_info.url_for_port(5432), Some("localhost:5432".to_string()));
            println!("✅ Database is accessible on the expected port 5432");
        } else {
            return Err("PostgreSQL container info not found".into());
        }
        
        Ok(())
    });
    
    // Test 2: Web service test
    test("test_web_service_fixed_port", |ctx| {
        println!("🧪 Testing web service on fixed port 8080...");
        
        if let Some(web_info) = ctx.get_data::<rust_test_harness::ContainerInfo>("web_container_info") {
            assert_eq!(web_info.host_port_for(80), Some(8080));
            assert_eq!(web_info.url_for_port(80), Some("localhost:8080".to_string()));
            println!("✅ Web service is accessible on the expected port 8080");
        } else {
            return Err("Web container info not found".into());
        }
        
        Ok(())
    });
    
    // Test 3: Mixed configuration test
    test("test_mixed_port_configuration", |ctx| {
        println!("🧪 Testing mixed port configuration (fixed + auto)...");
        
        if let Some(api_info) = ctx.get_data::<rust_test_harness::ContainerInfo>("api_container_info") {
            // Test fixed port
            assert_eq!(api_info.host_port_for(80), Some(3000));
            assert_eq!(api_info.url_for_port(80), Some("localhost:3000".to_string()));
            println!("✅ Fixed port 3000 -> 80 is working");
            
            // Test auto-assigned ports
            if let Some(https_port) = api_info.host_port_for(443) {
                assert!(https_port > 1024); // Should be a high port
                println!("✅ Auto-assigned HTTPS port: {}", https_port);
            }
            
            if let Some(metrics_port) = api_info.host_port_for(9090) {
                assert!(metrics_port > 1024); // Should be a high port
                println!("✅ Auto-assigned metrics port: {}", metrics_port);
            }
            
            println!("✅ Mixed configuration is working correctly");
        } else {
            return Err("API container info not found".into());
        }
        
        Ok(())
    });
    
    println!("🚀 Running user-specified ports tests...");
    
    let config = TestConfig {
        html_report: Some("user-specified-ports-report.html".to_string()),
        ..Default::default()
    };
    
    let exit_code = run_tests_with_config(config);
    
    println!();
    println!("🎉 Tests completed!");
    println!("📊 Check the HTML report for detailed results.");
    println!();
    println!("💡 Key Takeaways:");
    println!("   • Use .port(host_port, container_port) for fixed port mappings");
    println!("   • Use .auto_port(container_port) for automatic port assignment");
    println!("   • Mix both approaches for maximum flexibility");
    println!("   • Fixed ports are great for services with standard ports (5432, 3306, etc.)");
    println!("   • Auto-ports prevent conflicts in parallel test environments");
    
    std::process::exit(exit_code);
} 
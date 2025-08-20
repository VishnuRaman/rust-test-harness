//! Container Port and URL Access Example
//! 
//! This example demonstrates how users can:
//! 1. Get exact host ports for container ports
//! 2. Get ready-to-use URLs for services
//! 3. Access all port mappings and summaries
//! 4. Use this information for testing and connections

use rust_test_harness::{test, before_each, after_each, run_tests_with_config, TestConfig, ContainerConfig};
use std::time::Duration;

fn main() {
    println!("🌐 Container Port and URL Access Example");
    println!("========================================");
    println!();
    
    // Example 1: Web service with multiple auto-ports
    let web_container = ContainerConfig::new("nginx:alpine")
        .auto_port(80)      // Auto-assign port for HTTP
        .auto_port(443)     // Auto-assign port for HTTPS
        .env("NGINX_HOST", "localhost")
        .name("web_service")
        .ready_timeout(Duration::from_secs(15));
    
    // Example 2: Database with auto-port
    let db_container = ContainerConfig::new("postgres:13-alpine")
        .auto_port(5432)    // Auto-assign port for PostgreSQL
        .env("POSTGRES_PASSWORD", "testpass")
        .env("POSTGRES_DB", "testdb")
        .name("test_database")
        .ready_timeout(Duration::from_secs(20));
    
    // Example 3: Mixed configuration (manual + auto ports)
    let api_container = ContainerConfig::new("httpd:alpine")
        .port(8080, 80)     // Manual port mapping
        .auto_port(9090)    // Auto-assign port for API
        .auto_port(9091)    // Auto-assign port for metrics
        .env("API_VERSION", "v1")
        .name("api_service")
        .ready_timeout(Duration::from_secs(15));
    
    println!("📋 Container Configurations:");
    println!("1. Web Service:");
    println!("   - Image: {}", web_container.image);
    println!("   - Auto-ports: {:?}", web_container.auto_ports);
    println!("   - Will auto-assign host ports for container ports 80 and 443");
    
    println!("2. Database:");
    println!("   - Image: {}", db_container.image);
    println!("   - Auto-ports: {:?}", db_container.auto_ports);
    println!("   - Will auto-assign host port for container port 5432");
    
    println!("3. API Service:");
    println!("   - Image: {}", api_container.image);
    println!("   - Manual ports: {:?}", api_container.ports);
    println!("   - Auto-ports: {:?}", api_container.auto_ports);
    println!("   - Manual mapping: 8080 -> 80, Auto-assign: 9090, 9091");
    println!();
    
    // Clone containers for hooks
    let web_before = web_container.clone();
    let db_before = db_container.clone();
    let api_before = api_container.clone();
    
    // Start containers in before_each
    before_each(move |ctx| {
        println!("🚀 before_each: Starting containers and capturing port info...");
        
        // Start web container
        let web_info = web_before.start()
            .map_err(|e| format!("Failed to start web container: {}", e))?;
        ctx.set_data("web_container_info", web_info.clone());
        
        println!("✅ Web container started: {}", web_info.container_id);
        println!("   📍 Port mappings: {}", web_info.ports_summary());
        println!("   🔗 Primary URL: {}", web_info.primary_url().unwrap_or("None"));
        
        // Show specific port access
        if let Some(http_port) = web_info.host_port_for(80) {
            println!("   🌐 HTTP accessible at: http://localhost:{}", http_port);
        }
        if let Some(https_port) = web_info.host_port_for(443) {
            println!("   🔒 HTTPS accessible at: https://localhost:{}", https_port);
        }
        
        // Start database container
        let db_info = db_before.start()
            .map_err(|e| format!("Failed to start db container: {}", e))?;
        ctx.set_data("db_container_info", db_info.clone());
        
        println!("✅ Database container started: {}", db_info.container_id);
        println!("   📍 Port mappings: {}", db_info.ports_summary());
        if let Some(db_port) = db_info.host_port_for(5432) {
            println!("   🗄️ PostgreSQL accessible at: localhost:{}", db_port);
            println!("   📝 Connection string: postgresql://user:pass@localhost:{}/testdb", db_port);
        }
        
        // Start API container
        let api_info = api_before.start()
            .map_err(|e| format!("Failed to start api container: {}", e))?;
        ctx.set_data("api_container_info", api_info.clone());
        
        println!("✅ API container started: {}", api_info.container_id);
        println!("   📍 Port mappings: {}", api_info.ports_summary());
        
        // Show both manual and auto ports
        if let Some(http_port) = api_info.host_port_for(80) {
            println!("   🌐 HTTP API at: http://localhost:{} (manual mapping)", http_port);
        }
        if let Some(api_port) = api_info.host_port_for(9090) {
            println!("   🔧 API endpoint at: http://localhost:{} (auto-assigned)", api_port);
        }
        if let Some(metrics_port) = api_info.host_port_for(9091) {
            println!("   📊 Metrics at: http://localhost:{} (auto-assigned)", metrics_port);
        }
        
        println!("🎉 All containers started with auto-assigned ports!");
        Ok(())
    });
    
    // Cleanup containers in after_each
    let web_after = web_container.clone();
    let db_after = db_container.clone();
    let api_after = api_container.clone();
    
    after_each(move |ctx| {
        println!("🧹 after_each: Cleaning up containers...");
        
        if let Some(web_info) = ctx.get_data::<rust_test_harness::ContainerInfo>("web_container_info") {
            let _ = web_after.stop(&web_info.container_id);
            println!("🛑 Stopped web container: {}", web_info.container_id);
        }
        
        if let Some(db_info) = ctx.get_data::<rust_test_harness::ContainerInfo>("db_container_info") {
            let _ = db_after.stop(&db_info.container_id);
            println!("🛑 Stopped database container: {}", db_info.container_id);
        }
        
        if let Some(api_info) = ctx.get_data::<rust_test_harness::ContainerInfo>("api_container_info") {
            let _ = api_after.stop(&api_info.container_id);
            println!("🛑 Stopped API container: {}", api_info.container_id);
        }
        
        println!("✅ All containers cleaned up!");
        Ok(())
    });
    
    // Test 1: Web service port access
    test("test_web_service_port_access", |ctx| {
        println!("🧪 Testing web service port access...");
        
        let web_info = ctx.get_data::<rust_test_harness::ContainerInfo>("web_container_info")
            .expect("Web container info should be available");
        
        println!("🔍 Demonstrating port access methods:");
        
        // Method 1: Get specific host port for container port
        if let Some(http_port) = web_info.host_port_for(80) {
            println!("   ✅ HTTP port 80 is mapped to host port: {}", http_port);
            println!("      💡 Use this for HTTP client connections");
        }
        
        if let Some(https_port) = web_info.host_port_for(443) {
            println!("   ✅ HTTPS port 443 is mapped to host port: {}", https_port);
            println!("      💡 Use this for HTTPS client connections");
        }
        
        // Method 2: Get ready-to-use URLs
        if let Some(http_url) = web_info.url_for_port(80) {
            println!("   🔗 HTTP URL: {}", http_url);
            println!("      💡 Ready to use in HTTP requests");
        }
        
        if let Some(https_url) = web_info.url_for_port(443) {
            println!("   🔗 HTTPS URL: {}", https_url);
            println!("      💡 Ready to use in HTTPS requests (change http:// to https://)");
        }
        
        // Method 3: Get all URLs
        println!("   📋 All service URLs:");
        for (i, url) in web_info.urls.iter().enumerate() {
            println!("      {}. {}", i + 1, url);
        }
        
        // Method 4: Get all port mappings
        println!("   📊 All port mappings:");
        for (host_port, container_port) in &web_info.port_mappings {
            println!("      Host {} -> Container {}", host_port, container_port);
        }
        
        // Verify ports are different (auto-assigned)
        assert!(web_info.host_port_for(80).unwrap() != web_info.host_port_for(443).unwrap(), 
                "Auto-assigned ports should be different");
        
        println!("✅ Web service port access test passed");
        Ok(())
    });
    
    // Test 2: Database connection string generation
    test("test_database_connection_info", |ctx| {
        println!("🧪 Testing database connection info generation...");
        
        let db_info = ctx.get_data::<rust_test_harness::ContainerInfo>("db_container_info")
            .expect("Database container info should be available");
        
        println!("🗄️ Database connection information:");
        
        // Get the auto-assigned port for PostgreSQL
        let db_port = db_info.host_port_for(5432)
            .expect("PostgreSQL port should be exposed");
        
        println!("   📍 PostgreSQL running on: localhost:{}", db_port);
        
        // Generate connection strings using the auto-assigned port
        let connection_string = format!("postgresql://admin:testpass@localhost:{}/testdb", db_port);
        let jdbc_url = format!("jdbc:postgresql://localhost:{}/testdb", db_port);
        let docker_internal = format!("postgresql://admin:testpass@{}:5432/testdb", db_info.container_id);
        
        println!("   🔗 Connection strings:");
        println!("      Standard: {}", connection_string);
        println!("      JDBC: {}", jdbc_url);
        println!("      Docker internal: {}", docker_internal);
        
        // Show how to use this in real applications
        println!("   💡 Usage examples:");
        println!("      - Set DATABASE_URL={}", connection_string);
        println!("      - Use in tests: connect to localhost:{}", db_port);
        println!("      - Container-to-container: use port 5432");
        
        // Verify port is valid
        assert!(db_port > 0, "Database port should be valid");
        println!("✅ Database connection info test passed");
        
        Ok(())
    });
    
    // Test 3: Mixed port configuration
    test("test_mixed_port_configuration", |ctx| {
        println!("🧪 Testing mixed port configuration (manual + auto)...");
        
        let api_info = ctx.get_data::<rust_test_harness::ContainerInfo>("api_container_info")
            .expect("API container info should be available");
        
        println!("🔀 Mixed port configuration analysis:");
        
        // Manual port mapping
        let http_port = api_info.host_port_for(80)
            .expect("HTTP port should be mapped");
        println!("   📌 Manual mapping - HTTP port 80 -> host port: {}", http_port);
        assert_eq!(http_port, 8080, "Manual port should be exactly as specified");
        
        // Auto-assigned ports
        let api_port = api_info.host_port_for(9090)
            .expect("API port should be auto-assigned");
        let metrics_port = api_info.host_port_for(9091)
            .expect("Metrics port should be auto-assigned");
        
        println!("   🎲 Auto-assigned - API port 9090 -> host port: {}", api_port);
        println!("   🎲 Auto-assigned - Metrics port 9091 -> host port: {}", metrics_port);
        
        // Verify auto-assigned ports are different from manual port
        assert_ne!(api_port, 8080, "Auto-assigned port should be different from manual port");
        assert_ne!(metrics_port, 8080, "Auto-assigned port should be different from manual port");
        assert_ne!(api_port, metrics_port, "Auto-assigned ports should be different from each other");
        
        println!("   🌐 Service endpoints:");
        println!("      Main API: http://localhost:{}/", http_port);
        println!("      API v2: http://localhost:{}/", api_port);
        println!("      Metrics: http://localhost:{}/metrics", metrics_port);
        
        // Show complete port summary
        println!("   📋 Complete port summary: {}", api_info.ports_summary());
        
        println!("✅ Mixed port configuration test passed");
        Ok(())
    });
    
    // Test 4: Container info convenience methods
    test("test_container_info_convenience_methods", |ctx| {
        println!("🧪 Testing ContainerInfo convenience methods...");
        
        let web_info = ctx.get_data::<rust_test_harness::ContainerInfo>("web_container_info")
            .expect("Web container info should be available");
        
        println!("🔧 Testing all ContainerInfo methods:");
        
        // Test primary_url()
        if let Some(primary_url) = web_info.primary_url() {
            println!("   ✅ primary_url(): {}", primary_url);
            assert!(primary_url.starts_with("http://localhost:"), "Primary URL should be valid");
        }
        
        // Test url_for_port()
        if let Some(url_80) = web_info.url_for_port(80) {
            println!("   ✅ url_for_port(80): {}", url_80);
            assert!(url_80.contains("localhost:"), "URL should contain localhost");
        }
        
        // Test host_port_for()
        if let Some(port_80) = web_info.host_port_for(80) {
            println!("   ✅ host_port_for(80): {}", port_80);
            assert!(port_80 > 0, "Port should be positive");
        }
        
        // Test ports_summary()
        let summary = web_info.ports_summary();
        println!("   ✅ ports_summary(): {}", summary);
        assert!(summary.contains("->"), "Summary should contain port mappings");
        
        // Test direct field access
        println!("   ✅ container_id: {}", web_info.container_id);
        println!("   ✅ image: {}", web_info.image);
        println!("   ✅ auto_cleanup: {}", web_info.auto_cleanup);
        
        println!("   📊 All URLs:");
        for (i, url) in web_info.urls.iter().enumerate() {
            println!("      {}. {}", i + 1, url);
        }
        
        println!("   📊 All port mappings:");
        for (host_port, container_port) in &web_info.port_mappings {
            println!("      {} -> {}", host_port, container_port);
        }
        
        println!("✅ Container info convenience methods test passed");
        Ok(())
    });
    
    // Test 5: Real-world usage patterns
    test("test_real_world_usage_patterns", |ctx| {
        println!("🧪 Testing real-world usage patterns...");
        
        let web_info = ctx.get_data::<rust_test_harness::ContainerInfo>("web_container_info")
            .expect("Web container info should be available");
        let db_info = ctx.get_data::<rust_test_harness::ContainerInfo>("db_container_info")
            .expect("Database container info should be available");
        
        println!("🌍 Real-world usage examples:");
        
        // Pattern 1: HTTP client testing
        if let Some(web_url) = web_info.primary_url() {
            println!("   🌐 HTTP Client Testing:");
            println!("      Base URL: {}", web_url);
            println!("      GET {}/health", web_url);
            println!("      POST {}/api/users", web_url);
            println!("      Code: let response = reqwest::get(\"{}/health\").await?;", web_url);
        }
        
        // Pattern 2: Database testing
        if let Some(db_port) = db_info.host_port_for(5432) {
            println!("   🗄️ Database Testing:");
            println!("      Connection: localhost:{}", db_port);
            println!("      Code: let conn = PgConnection::establish(\"postgresql://user:pass@localhost:{}/db\")?;", db_port);
        }
        
        // Pattern 3: Environment variable setup
        println!("   🔧 Environment Variables:");
        if let Some(web_port) = web_info.host_port_for(80) {
            println!("      export WEB_SERVICE_URL=http://localhost:{}", web_port);
        }
        if let Some(db_port) = db_info.host_port_for(5432) {
            println!("      export DATABASE_URL=postgresql://localhost:{}/testdb", db_port);
        }
        
        // Pattern 4: Docker-compose style networking
        println!("   🐳 Container Networking:");
        println!("      External: Use auto-assigned host ports");
        println!("      Internal: Use original container ports");
        println!("      Web -> DB: {}:5432 (container-to-container)", db_info.container_id);
        
        // Pattern 5: Health checks
        println!("   ❤️ Health Checks:");
        if let Some(web_url) = web_info.primary_url() {
            println!("      Health endpoint: {}/health", web_url);
            println!("      Ready endpoint: {}/ready", web_url);
        }
        
        println!("✅ Real-world usage patterns test passed");
        Ok(())
    });
    
    println!("🚀 Running container port access tests...");
    println!();
    
    // Run the tests
    let config = TestConfig {
        html_report: Some("container-port-access-report.html".to_string()),
        ..Default::default()
    };
    
    let exit_code = run_tests_with_config(config);
    
    println!();
    if exit_code == 0 {
        println!("🎉 All tests passed! Port access functionality is working correctly.");
        println!("📊 Check the HTML report for detailed results.");
        println!();
        println!("💡 Key Takeaways:");
        println!("   • Use auto_port() to avoid port conflicts");
        println!("   • Access host ports with host_port_for(container_port)");
        println!("   • Get ready-to-use URLs with url_for_port(container_port)");
        println!("   • Use ports_summary() for human-readable port info");
        println!("   • ContainerInfo provides complete access to all port information");
    } else {
        println!("❌ Some tests failed. Check the output above for details.");
    }
    
    std::process::exit(exit_code);
} 
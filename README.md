# Rust Test Harness

A modern, feature-rich testing framework for Rust with Docker integration, designed to provide a clean and intuitive testing experience similar to popular frameworks like ScalaTest.

## ✨ Features

- **Closure-based APIs**: Use `FnMut` closures for tests and hooks with proper error handling
- **Rich Error Types**: Comprehensive `TestError` enum with `Message`, `Panicked`, `Timeout`, and `DockerError` variants
- **Panic Safety**: Automatic panic recovery for both hooks and tests, ensuring cleanup always runs
- **Docker Integration**: Built-in Docker container management with readiness strategies
- **Test Lifecycle Hooks**: `before_all`, `before_each`, `after_each`, `after_all` hooks
- **Tagging System**: Tag tests and filter by tags for selective execution
- **Test Filtering**: Filter tests by name using environment variables or programmatic configuration
- **Deterministic Shuffling**: Optional test shuffling with seed-based reproducibility
- **Parallel Execution**: Configurable parallel test execution (currently falls back to sequential)
- **Colored Output**: Optional colored terminal output for better readability
- **JUnit XML**: Optional JUnit XML output for CI/CD integration
- **Mutex Recovery**: Automatic recovery from poisoned mutexes
- **Container Cleanup**: Automatic Docker container cleanup via `Drop` trait
- **Timeout Support**: Per-test timeout configuration (framework ready)
- **Signal Handling**: Unix signal handling for graceful cleanup

## 🚀 Quick Start

### Auto-Port Assignment for Docker Containers
The framework automatically handles port conflicts by finding available ports:

```rust
// Old way - manually specifying ports (error-prone)
let opts1 = DockerRunOptions::new("mongo:6.0").port(27018, 27017);
let opts2 = DockerRunOptions::new("mongo:6.0").port(27019, 27017);
let opts3 = DockerRunOptions::new("mongo:6.0").port(27020, 27017);

// New way - automatic port assignment
let create_opts = || -> DockerRunOptions {
    DockerRunOptions::new("mongo:6.0")
        .with_auto_port_and_readiness(27017, 27018)  // Auto-assign host port starting from 27018
        .env("MONGO_INITDB_ROOT_USERNAME", "admin")
        .env("MONGO_INITDB_ROOT_PASSWORD", "password123")
};

// Use in tests - each gets a unique port automatically
test_with_docker("test 1", create_opts(), |ctx| { /* ... */ });
test_with_docker("test 2", create_opts(), |ctx| { /* ... */ });
test_with_docker("test 3", create_opts(), |ctx| { /* ... */ });

// Get connection information programmatically
let container = ctx.docker.as_ref().unwrap();

// Method 1: Get host and port separately
let (host, port) = container.get_connection_info().unwrap_or(("localhost".to_string(), 27018));

// Method 2: Get generic connection string
let base_url = container.get_connection_string("mongodb").unwrap_or_else(|| "mongodb://localhost:27018".to_string());

// Method 3: Build MongoDB connection string
let mongo_url = format!("mongodb://admin:password123@{}:{}/testdb", host, port);

// Use the discovered connection info
let client = MongoClient::new(&host, port, "testdb", "users");
```

### Basic Usage

```rust
use rust_test_harness::{before_all, before_each, after_each, after_all, test, run_all};

fn main() {
    env_logger::init();

    before_all(|_| {
        println!("Setting up test environment...");
        Ok(())
    });

    after_all(|_| {
        println!("Cleaning up test environment...");
        Ok(())
    });

    before_each(|_| {
        println!("Preparing test...");
        Ok(())
    });

    after_each(|_| {
        println!("Cleaning up test...");
        Ok(())
    });

    test("basic arithmetic", |_| {
        assert_eq!(2 + 2, 4);
        Ok(())
    });

    test("string operations", |_| {
        let text = "hello world";
        assert!(text.contains("hello"));
        Ok(())
    });

    let exit_code = run_all();
    std::process::exit(exit_code);
}
```

### Docker Integration

```rust
use rust_test_harness::{test_with_docker, DockerRunOptions, Readiness};

let redis_opts = DockerRunOptions::new("redis:alpine")
    .port(6379, 6379)
    .env("REDIS_PASSWORD", "testpass")
    .ready_timeout(Duration::from_secs(10))
    .readiness(Readiness::PortOpen(6379));

test_with_docker("redis container test", redis_opts, |ctx| {
    if ctx.docker.is_some() {
        println!("Redis container is running!");
        Ok(())
    } else {
        Err("Docker container not available".into())
    }
});
```

### Tagged Tests

```rust
use rust_test_harness::test_with_tags;

test_with_tags("integration test", vec!["integration", "slow"], |_| {
    // This test will be tagged as integration and slow
    std::thread::sleep(Duration::from_millis(100));
    Ok(())
});
```

## 📚 Examples

The framework includes several comprehensive examples demonstrating different usage patterns:

### 1. Basic Usage (`examples/basic_usage.rs`)
Simple examples showing hooks, basic tests, and Docker integration.

### 2. Advanced Features (`examples/advanced_features.rs`)
Demonstrates configuration options, different test types, and Docker readiness strategies.

### 3. Real-World Calculator (`examples/real_world_calculator.rs`)
A practical example testing a calculator library without Docker, showing:
- **State Management**: Testing calculator memory and history
- **Error Handling**: Division by zero and edge cases
- **Data-Driven Tests**: Floating point precision testing
- **Performance Tests**: Stress testing with many operations
- **Integration Workflows**: Complex calculation sequences
- **Tagged Tests**: Performance and integration test categorization



### 5. Realistic Database (`examples/realistic_database.rs`)
A practical example demonstrating proper resource management in testing, showing:
- **Resource Isolation**: Each test gets a clean database state
- **Proper Cleanup**: Database is reset between tests using `before_each` and `after_each` hooks
- **Realistic Operations**: User management, sessions, CRUD operations
- **Error Handling**: Duplicate prevention, validation, edge cases
- **Performance Testing**: Stress testing with many users
- **Concurrent Access**: Thread-safe database operations
- **Integration Workflows**: Complete user lifecycle testing
- **No Unused Variables**: Every piece of code serves a purpose

### 6. MongoDB Integration (`examples/mongodb_integration.rs`)
A comprehensive example demonstrating Docker container lifecycle management with MongoDB, showing:
- **Container Per Test**: Each test gets a fresh MongoDB container for complete isolation
- **Docker Integration**: Proper container startup, readiness checks, and cleanup
- **Database Operations**: CRUD operations, collections, and document management
- **Performance Testing**: Multiple operations, concurrent access, and stress testing
- **Error Handling**: Graceful handling of invalid operations and edge cases
- **Integration Workflows**: Complete document lifecycle from creation to deletion
- **Container Readiness**: Port-based readiness strategy for MongoDB
- **Real-World Patterns**: Simulates actual MongoDB client usage patterns
- **Auto-Port Assignment**: Uses `with_auto_port_and_readiness()` to automatically avoid port conflicts
- **Clean API**: Single `create_mongo_opts()` function instead of 10 manual port configurations
- **Port Discovery**: Uses `get_host_port()` to dynamically connect to the assigned port
- **Automatic Cleanup**: Containers are automatically stopped and removed after each test
- **MongoDB Best Practices**: Proper authentication, database initialization, and connection handling

## 🏗️ Architecture

### Container Lifecycle Management
The framework provides robust Docker container lifecycle management:
- **Automatic Startup**: Containers start with configurable readiness strategies
- **Port Isolation**: Each test can use unique ports to avoid conflicts
- **Auto-Port Assignment**: `with_auto_port()` and `with_auto_port_and_readiness()` methods automatically find available ports
- **Readiness Checks**: Multiple strategies (Running, PortOpen, HttpOk, HealthCheck)
- **Automatic Cleanup**: Containers are stopped and removed after each test
- **Error Recovery**: Graceful fallback to force removal if normal cleanup fails
- **Resource Management**: Uses `--rm` flag and explicit cleanup for reliability
- **Port Discovery**: `get_host_port()` method retrieves the actual assigned port for connections
- **Connection Info**: `get_connection_info()` returns (host, port) tuple for easy URL building
- **Generic URLs**: `get_connection_string(protocol)` creates protocol://host:port URLs

### Core Components

- **`TestContext`**: Shared state between tests (Docker containers, custom objects)
- **`TestRunner`**: Manages test execution with configuration
- **`DockerRunOptions`**: Builder pattern for Docker container configuration
- **`Readiness`**: Container readiness strategies (Running, PortOpen, HttpOk, HealthCheck)

### Test Lifecycle

1. **Global Setup**: `before_all` hooks run once
2. **Test Setup**: `before_each` hooks run before each test
3. **Test Execution**: Test function runs with error handling
4. **Test Cleanup**: `after_each` hooks run after each test
5. **Global Cleanup**: `after_all` hooks run once

### Error Handling

The framework automatically converts panics into `TestError::Panicked` results, ensuring that:
- Tests don't crash the framework
- Cleanup hooks always run
- Error information is preserved and reported

## ⚙️ Configuration

### Environment Variables

```bash
# Test filtering
RUST_TEST_HARNESS_FILTER="calculator"
RUST_TEST_HARNESS_SKIP_TAGS="slow,integration"

# Output options
RUST_TEST_HARNESS_COLOR=true
RUST_TEST_HARNESS_JUNIT_XML="test-results.xml"

# Execution options
RUST_TEST_HARNESS_MAX_CONCURRENCY=4
RUST_TEST_HARNESS_SHUFFLE_SEED=42
```

### Programmatic Configuration

```rust
use rust_test_harness::{TestRunner, TestConfig};

let config = TestConfig {
    filter: Some("calculator".to_string()),
    skip_tags: vec!["slow".to_string()],
    max_concurrency: Some(2),
    shuffle_seed: Some(42),
    color: Some(true),
    junit_xml: Some("results.xml".to_string()),
};

let runner = TestRunner::with_config(config);
let exit_code = runner.run();
```

## 🐳 Docker Integration

### Container Readiness Strategies

- **`Running`**: Basic container running check
- **`PortOpen(port)`**: Wait for specific port to be open
- **`HttpOk { host, port, path }`**: HTTP endpoint health check
- **`HealthCheck`**: Docker health check command

### Container Management

```rust
let postgres_opts = DockerRunOptions::new("postgres:15-alpine")
    .port(5432, 5432)
    .env("POSTGRES_PASSWORD", "testpass")
    .env("POSTGRES_DB", "testdb")
    .ready_timeout(Duration::from_secs(45))
    .readiness(Readiness::PortOpen(5432));

test_with_docker("database test", postgres_opts, |ctx| {
    // Test database operations
    Ok(())
});
```

Containers are automatically stopped and cleaned up via the `Drop` trait, even if tests panic.

## 🔧 Advanced Usage

### Custom Test Context

```rust
use rust_test_harness::{TestContext, Calculator, ApiClient, DatabaseClient};

before_each(|ctx| {
    ctx.calculator = Some(Calculator::new());
    ctx.api_client = Some(ApiClient::new("http://localhost:3000".to_string()));
    ctx.db_client = Some(DatabaseClient::new("postgresql://localhost/testdb".to_string()));
    Ok(())
});

test("calculator operations", |ctx| {
    let calc = ctx.calculator.as_mut().unwrap();
    calc.memory = 42.0;
    assert_eq!(calc.memory, 42.0);
    Ok(())
});
```

### Test Filtering and Organization

```rust
// Run only tests matching a pattern
let config = TestConfig {
    filter: Some("calculator".to_string()),
    ..Default::default()
};

// Skip tests with specific tags
let config = TestConfig {
    skip_tags: vec!["slow".to_string(), "integration".to_string()],
    ..Default::default()
};
```

## 🧪 Running Tests

### Run All Tests
```bash
cargo run --example real_world_calculator
cargo run --example realistic_database
cargo run --example mongodb_integration
```

### Run Framework Tests
```bash
cargo test
```

### Run Specific Examples
```bash
cargo run --example basic_usage
cargo run --example advanced_features
```

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rust-test-harness = "0.1.0"
```

## 🤝 Contributing

This framework is designed to be extensible. Key areas for contribution:
- Parallel test execution implementation
- Additional Docker readiness strategies
- More output formats
- Performance optimizations

## 📄 License

Apache-2.0 - see LICENSE file for details.

## 🎯 Roadmap

- [ ] Full parallel test execution
- [ ] Additional container readiness strategies
- [ ] Test result caching
- [ ] Performance profiling
- [ ] More output formats (JSON, HTML reports)
- [ ] Test retry mechanisms
- [ ] CI/CD integration helpers 
# Rust Test Harness

A modern, feature-rich testing framework for Rust with Docker integration, hooks, and parallel execution.

## Features

- **Rust-Native Testing**: Works exactly like Rust's built-in `#[test]` attribute
- **Docker Integration**: Run tests in isolated containers
- **Test Hooks**: `before_all`, `before_each`, `after_each`, `after_all`
- **Parallel Execution**: Run tests concurrently with configurable concurrency
- **Tag-based Filtering**: Organize and filter tests by tags
- **Test Timeouts**: Set maximum execution time for tests
- **HTML Reports**: Generate beautiful, interactive test reports for CI/CD pipelines

## Quick Start

### Basic Usage (Rust-Style)

Your test harness works exactly like Rust's built-in testing framework! You can use it in `mod tests` blocks and outside of `main()` functions:

```rust
use rust_test_harness::test_case;

pub struct Calculator {
    value: i32,
}

impl Calculator {
    pub fn new() -> Self { Self { value: 0 } }
    pub fn add(&mut self, x: i32) { self.value += x; }
    pub fn get_value(&self) -> i32 { self.value }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Test using test_case! macro - works exactly like #[test]
    test_case!(test_calculator_new, |_ctx| {
        let calc = Calculator::new();
        assert_eq!(calc.get_value(), 0);
        Ok(())
    });
    
    // Standard Rust test that also works
    #[test]
    fn test_subtraction() {
        let mut calc = Calculator::new();
        calc.add(10);
        // Your test logic here
        assert_eq!(calc.get_value(), 10);
    }
}
```

### Running Tests

```bash
# Run all tests (discovered by cargo test)
cargo test

# Run specific test
cargo test test_calculator_new

# Run tests with filter
cargo test calculator

# Run specific example
cargo test --example rust_style_tests

# Run specific test in an example
cargo test --example rust_style_tests test_calculator_new
```

### Advanced Features

#### Docker Integration

```rust
use rust_test_harness::{test_case_docker, DockerRunOptions};

test_case_docker!(test_with_nginx, 
    DockerRunOptions::new("nginx:alpine")
        .port(80, 8080)
        .env("NGINX_HOST", "localhost"), 
    |ctx| {
        // Your test runs with Docker context
        // The container is automatically managed by the framework
        Ok(())
    }
);
```

#### Test Hooks

**Important Note**: Hooks are built into the framework and work automatically. You don't need to manually call them.

```rust
use rust_test_harness::test_case;

#[cfg(test)]
mod tests {
    use super::*;
    
    // The framework automatically handles hooks for:
    // - before_all: Global setup before all tests
    // - before_each: Setup before each test
    // - after_each: Cleanup after each test
    // - after_all: Global cleanup after all tests
    
    test_case!(my_test, |_ctx| {
        // Your test logic here
        // Hooks run automatically
        Ok(())
    });
}
```

**What Hooks Can Do:**
- ✅ Setup/teardown test databases
- ✅ Initialize test data
- ✅ Clean up test files
- ✅ Manage test configuration
- ✅ Handle test environment setup

**What Hooks Are NOT For:**
- ❌ Docker container management (use `test_case_docker!` instead)
- ❌ Complex resource orchestration (use specialized macros)

#### Tag-based Filtering

```rust
use rust_test_harness::test_with_tags;

test_with_tags("slow_integration_test", vec!["slow", "integration"], |_ctx| {
    // This test has tags: "slow" and "integration"
    std::thread::sleep(Duration::from_secs(1));
    Ok(())
});

// Skip slow tests: TEST_SKIP_TAGS=slow cargo test
```

#### HTML Reports

Generate beautiful, interactive HTML reports for your test results:

```rust
use rust_test_harness::run_tests_with_config;

let config = TestConfig {
    html_report: Some("test-results.html".to_string()),
    ..Default::default()
};

let result = run_tests_with_config(config);
// Generates test-results.html with comprehensive test results
```

**HTML Report Features:**
- 📊 **Visual Summary**: Pass/fail/skip statistics with color-coded cards
- 📋 **Detailed Results**: Individual test results with status and error details
- 🎨 **Modern Design**: Responsive, mobile-friendly interface with gradients
- 🔍 **Test Information**: Tags, timeouts, Docker configuration, and error messages
- ⏱️ **Execution Time**: Total test execution duration
- 📱 **Responsive Layout**: Works on desktop, tablet, and mobile devices
- 🔽 **Expandable Details**: Click any test to view detailed metadata and configuration
- 🔍 **Search Functionality**: Search tests by name, status, or tags in real-time
- ⌨️ **Keyboard Shortcuts**: Ctrl+F (search), Ctrl+A (expand all), Ctrl+Z (collapse all)
- 🚨 **Auto-Expand Failed**: Failed tests automatically expand for better visibility

**Environment Variable:**
```bash
export TEST_HTML_REPORT=test-results.html
cargo test
```

#### **Interactive Features**

The HTML reports include several interactive features to enhance your testing experience:

**🔽 Expandable Test Details**
- Click on any test header to expand/collapse detailed information
- View test metadata including tags, timeouts, and Docker configuration
- Error details are automatically displayed for failed tests

**🔍 Real-Time Search**
- Search box filters tests by name, status, or tags
- Results update instantly as you type
- Case-insensitive search for better usability

**⌨️ Keyboard Shortcuts**
- `Ctrl+F` (or `Cmd+F` on Mac): Focus search box
- `Ctrl+A` (or `Cmd+A` on Mac): Expand all test details
- `Ctrl+Z` (or `Cmd+Z` on Mac): Collapse all test details

**🚨 Smart Defaults**
- Failed tests automatically expand for immediate visibility
- Hover effects provide visual feedback
- Responsive design adapts to all screen sizes

## Framework vs. Standard Testing

| Feature | Rust Standard | Rust Test Harness |
|---------|---------------|-------------------|
| Basic Tests | `#[test]` | `test_case!()` or `#[test]` |
| Test Discovery | ✅ | ✅ |
| Parallel Execution | ✅ | ✅ |
| Test Hooks | ❌ | ✅ (Automatic) |
| Docker Integration | ❌ | ✅ |
| Tag-based Filtering | ❌ | ✅ |
| Test Timeouts | ❌ | ✅ |
| HTML Reports | ❌ | ✅ |

## Migration from Standard Tests

Converting from standard Rust tests is simple:

**Before (Standard Rust):**
```rust
#[test]
fn test_my_function() {
    assert_eq!(my_function(2, 2), 4);
}
```

**After (Rust Test Harness):**
```rust
test_case!(test_my_function, |_ctx| {
    assert_eq!(my_function(2, 2), 4);
    Ok(())
});
```

**Or keep using `#[test]` unchanged:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_my_function() {
        // Your existing test code works unchanged
        assert_eq!(my_function(2, 2), 4);
    }
}
```

## Examples

Check out the `examples/` directory for comprehensive examples:

### Core Examples
- `minimal_rust_style.rs` - Minimal example showing Rust-style testing
- `rust_style_tests.rs` - Comprehensive Rust-style testing patterns
- `basic_usage.rs` - Basic framework usage with `test_case!`

### Real-World Examples
- `advanced_features.rs` - Advanced features and patterns
- `real_world_calculator.rs` - Real-world application testing
- `mongodb_integration.rs` - Database testing with Docker integration
- `cargo_test_integration.rs` - Integration with cargo test

### Running Examples

```bash
# Run a specific example
cargo run --example rust_style_tests

# Test a specific example
cargo test --example rust_style_tests

# Run specific test in an example
cargo test --example rust_style_tests test_calculator_new
```

## Container Management Patterns

### For Docker Containers

**Use `test_case_docker!` (Recommended):**
```rust
test_case_docker!(test_with_mongo, 
    DockerRunOptions::new("mongo:6.0")
        .env("MONGO_INITDB_ROOT_USERNAME", "admin")
        .env("MONGO_INITDB_ROOT_PASSWORD", "password123")
        .port(27017, 27017), 
    |_ctx| {
        // Container is automatically managed
        let client = MongoClient::new("localhost", 27017, "testdb", "users");
        // Your test logic here
        Ok(())
    }
);
```

**Why Not Hooks for Containers?**
- Container lifecycle management is complex
- Parallel test execution can cause conflicts
- Port management becomes tricky
- Use `test_case_docker!` for automatic container management

### For Other Resources

**Hooks work great for:**
- Database connections
- File system operations
- Configuration management
- Test data setup/cleanup

## Configuration

Set environment variables to configure test execution:

```bash
# Filter tests by name
export TEST_FILTER=integration

# Skip tests with specific tags
export TEST_SKIP_TAGS=slow,flaky

# Set maximum concurrency
export TEST_MAX_CONCURRENCY=8

# Generate HTML report
export TEST_HTML_REPORT=test-results.html

# Skip hooks (for debugging)
export TEST_SKIP_HOOKS=true
```

## IDE Support

### RustRover and Similar IDEs

**`test_case!` macros:**
- ❌ No play buttons (macros aren't expanded during compilation)
- ✅ Tests are discoverable by `cargo test`
- ✅ Tests run correctly

**Standard `#[test]` functions:**
- ✅ Play buttons work normally
- ✅ Full IDE integration

**Recommended Approach:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // Use test_case! for framework features
    test_case!(test_with_framework, |_ctx| {
        // Framework-specific logic
        Ok(())
    });
    
    // Use #[test] for IDE play button support
    #[test]
    fn test_standard_rust() {
        // Standard Rust testing
        assert_eq!(2 + 2, 4);
    }
}
```

## Why This Approach?

1. **Familiar**: Works exactly like Rust's built-in testing
2. **Discoverable**: Tests are found by `cargo test` automatically
3. **Compatible**: Existing Rust tests work unchanged
4. **Enhanced**: Adds powerful features without breaking existing patterns
5. **Flexible**: Use framework features when needed, standard patterns when not
6. **Automatic**: Hooks work automatically without manual setup

## Contributing

Contributions are welcome! This framework aims to enhance Rust's testing capabilities while maintaining full compatibility with existing testing patterns. 
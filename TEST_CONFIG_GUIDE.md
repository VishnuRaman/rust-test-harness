# TestConfig Guide

The `TestConfig` struct allows you to customize how your tests run. Users will typically create their own configurations for different environments and use cases.

## 🔧 **TestConfig Options**

```rust
use rust_test_harness::{TestRunner, TestConfig};

let config = TestConfig {
    filter: Some("calculator".to_string()),    // Filter tests by name
    skip_tags: vec!["slow".to_string()],       // Skip tests with specific tags
    max_concurrency: Some(4),                  // Number of parallel workers
    shuffle_seed: Some(42),                    // Randomize test order
    color: Some(true),                         // Colored output
    junit_xml: Some("results.xml".to_string()),// JUnit XML output
    skip_hooks: Some(true),                    // Performance mode
};

let runner = TestRunner::with_config(config);
let exit_code = runner.run();
```

## 📋 **Configuration Options**

### **`filter: Option<String>`**
Filter tests by name (case-insensitive substring match).

```rust
// Run only calculator tests
let config = TestConfig {
    filter: Some("calculator".to_string()),
    ..Default::default()
};

// Run only tests containing "auth"
let config = TestConfig {
    filter: Some("auth".to_string()),
    ..Default::default()
};
```

**Environment Variable**: `TEST_FILTER=calculator`

### **`skip_tags: Vec<String>`**
Skip tests that have any of the specified tags.

```rust
// Skip slow and integration tests
let config = TestConfig {
    skip_tags: vec!["slow".to_string(), "integration".to_string()],
    ..Default::default()
};

// Skip only flaky tests
let config = TestConfig {
    skip_tags: vec!["flaky".to_string()],
    ..Default::default()
};
```

**Environment Variable**: `TEST_SKIP_TAGS=slow,integration`

### **`max_concurrency: Option<usize>`**
Control parallel execution.

```rust
// Use 4 worker threads
let config = TestConfig {
    max_concurrency: Some(4),
    ..Default::default()
};

// Use all available CPU cores (auto-detect)
let config = TestConfig {
    max_concurrency: None,
    ..Default::default()
};

// Force sequential execution
let config = TestConfig {
    max_concurrency: Some(1),
    ..Default::default()
};
```

**Environment Variable**: `TEST_MAX_CONCURRENCY=4`

### **`shuffle_seed: Option<u64>`**
Randomize test execution order for better test isolation.

```rust
// Use specific seed for reproducible randomization
let config = TestConfig {
    shuffle_seed: Some(42),
    ..Default::default()
};

// No shuffling (alphabetical order)
let config = TestConfig {
    shuffle_seed: None,
    ..Default::default()
};
```

**Environment Variable**: `TEST_SHUFFLE=42`

### **`color: Option<bool>`**
Control colored terminal output.

```rust
// Force colors on
let config = TestConfig {
    color: Some(true),
    ..Default::default()
};

// Force colors off (good for CI)
let config = TestConfig {
    color: Some(false),
    ..Default::default()
};

// Auto-detect (default)
let config = TestConfig {
    color: None,
    ..Default::default()
};
```

**Environment Variables**: 
- `TEST_COLOR=true/false`
- `NO_COLOR=1` (disable colors)

### **`junit_xml: Option<String>`**
Generate JUnit XML output for CI integration.

```rust
// Generate XML report
let config = TestConfig {
    junit_xml: Some("test-results.xml".to_string()),
    ..Default::default()
};

// No XML output
let config = TestConfig {
    junit_xml: None,
    ..Default::default()
};
```

**Environment Variable**: `TEST_JUNIT_XML=test-results.xml`

### **`skip_hooks: Option<bool>`**
Performance mode - skip beforeEach/afterEach hooks.

```rust
// Performance mode (faster, but no test isolation)
let config = TestConfig {
    skip_hooks: Some(true),
    ..Default::default()
};

// Normal mode (slower, but proper test isolation)
let config = TestConfig {
    skip_hooks: Some(false),
    ..Default::default()
};
```

**Environment Variable**: `TEST_SKIP_HOOKS=true`

## 🎯 **Common Use Cases**

### **Development Testing**
```rust
let dev_config = TestConfig {
    filter: Some("unit".to_string()),    // Only unit tests
    max_concurrency: Some(2),            // Don't overwhelm development machine
    color: Some(true),                   // Pretty output
    skip_hooks: Some(false),             // Proper test isolation
    ..Default::default()
};
```

### **CI/CD Pipeline**
```rust
let ci_config = TestConfig {
    max_concurrency: Some(8),            // Use all CI cores
    color: Some(false),                  // No colors in logs
    junit_xml: Some("results.xml".to_string()), // For test reporting
    skip_hooks: Some(true),              // Maximum speed
    ..Default::default()
};
```

### **Integration Testing**
```rust
let integration_config = TestConfig {
    filter: Some("integration".to_string()), // Only integration tests
    max_concurrency: Some(1),                // Sequential for resource isolation
    skip_hooks: Some(false),                 // Need proper setup/teardown
    ..Default::default()
};
```

### **Performance Testing**
```rust
let perf_config = TestConfig {
    skip_tags: vec!["slow".to_string()],  // Skip slow tests
    max_concurrency: Some(16),            // Maximum parallelism
    skip_hooks: Some(true),               // Maximum speed
    shuffle_seed: Some(123),              // Consistent ordering
    ..Default::default()
};
```

### **Debug/Troubleshooting**
```rust
let debug_config = TestConfig {
    filter: Some("failing_test".to_string()), // Single failing test
    max_concurrency: Some(1),                 // Sequential for debugging
    color: Some(true),                        // Easy to read output
    skip_hooks: Some(false),                  // Full test lifecycle
    ..Default::default()
};
```

## 🔄 **Environment Variable Overrides**

All config options can be overridden with environment variables:

```bash
# Override specific options
export TEST_FILTER="calculator"
export TEST_MAX_CONCURRENCY=8
export TEST_SKIP_HOOKS=true
export TEST_COLOR=false

# Run with overrides
cargo run --example my_tests
```

## 📊 **Performance Recommendations**

### **For Speed:**
```rust
TestConfig {
    max_concurrency: Some(8),     // Use more workers
    skip_hooks: Some(true),       // Skip hook overhead
    color: Some(false),           // Reduce output overhead
    ..Default::default()
}
```

### **For Reliability:**
```rust
TestConfig {
    max_concurrency: Some(1),     // Sequential execution
    skip_hooks: Some(false),      // Full test isolation
    shuffle_seed: Some(42),       // Consistent but randomized
    ..Default::default()
}
```

### **For CI/CD:**
```rust
TestConfig {
    max_concurrency: None,        // Auto-detect cores
    skip_hooks: Some(true),       // Maximum speed
    junit_xml: Some("results.xml".to_string()), // Test reporting
    color: Some(false),           // Clean logs
    ..Default::default()
}
```

## 🎛️ **Advanced Configuration Patterns**

### **Multi-Environment Setup**
```rust
fn get_test_config() -> TestConfig {
    match std::env::var("ENVIRONMENT").as_deref() {
        Ok("ci") => TestConfig {
            max_concurrency: Some(8),
            skip_hooks: Some(true),
            junit_xml: Some("results.xml".to_string()),
            color: Some(false),
            ..Default::default()
        },
        Ok("dev") => TestConfig {
            max_concurrency: Some(2),
            color: Some(true),
            skip_hooks: Some(false),
            ..Default::default()
        },
        _ => TestConfig::default()
    }
}
```

### **Test Suite Composition**
```rust
// Fast unit tests
let unit_config = TestConfig {
    filter: Some("unit".to_string()),
    max_concurrency: Some(8),
    skip_hooks: Some(true),
    ..Default::default()
};

// Careful integration tests
let integration_config = TestConfig {
    filter: Some("integration".to_string()),
    max_concurrency: Some(1),
    skip_hooks: Some(false),
    ..Default::default()
};

// Run both suites
TestRunner::with_config(unit_config).run();
TestRunner::with_config(integration_config).run();
```

## 💡 **Best Practices**

1. **Use environment variables** for flexible configuration across environments
2. **Create config presets** for common scenarios (dev, ci, debug)
3. **Start with defaults** and only override what you need
4. **Use performance mode sparingly** - only when speed is critical
5. **Test your configs** to ensure they work as expected
6. **Document your team's conventions** for consistent usage

## 🔍 **Troubleshooting**

### **Tests Running Too Slowly?**
- Increase `max_concurrency`
- Enable `skip_hooks: true`
- Disable `color` for CI environments

### **Tests Interfering With Each Other?**
- Decrease `max_concurrency` to 1
- Ensure `skip_hooks: false`
- Use different `shuffle_seed` values

### **Can't Find Specific Tests?**
- Check your `filter` string
- Verify `skip_tags` isn't excluding them
- Use broader filter patterns

This flexible configuration system allows users to optimize their testing workflow for any scenario! 🚀 
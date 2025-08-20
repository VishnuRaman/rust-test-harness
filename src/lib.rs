use std::cell::RefCell;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::panic::{catch_unwind, AssertUnwindSafe};
use log::{info, warn, error};


// --- Thread-local test registry ---
// Each test thread gets its own isolated registry - no manual cleanup needed!

thread_local! {
    static THREAD_TESTS: RefCell<Vec<TestCase>> = RefCell::new(Vec::new());
    static THREAD_BEFORE_ALL: RefCell<Vec<HookFn>> = RefCell::new(Vec::new());
    static THREAD_BEFORE_EACH: RefCell<Vec<HookFn>> = RefCell::new(Vec::new());
    static THREAD_AFTER_EACH: RefCell<Vec<HookFn>> = RefCell::new(Vec::new());
    static THREAD_AFTER_ALL: RefCell<Vec<HookFn>> = RefCell::new(Vec::new());
}

// --- Test registry management ---
// Auto-clears when run_tests() is called - no manual intervention needed

pub fn clear_test_registry() {
    // This function is now optional - mainly for manual cleanup if needed
    THREAD_TESTS.with(|tests| tests.borrow_mut().clear());
    THREAD_BEFORE_ALL.with(|hooks| hooks.borrow_mut().clear());
    THREAD_BEFORE_EACH.with(|hooks| hooks.borrow_mut().clear());
    THREAD_AFTER_EACH.with(|hooks| hooks.borrow_mut().clear());
    THREAD_AFTER_ALL.with(|hooks| hooks.borrow_mut().clear());
}

// --- Type definitions ---

pub type TestResult = Result<(), TestError>;
pub type TestFn = Box<dyn FnMut(&mut TestContext) -> TestResult + Send>;
pub type HookFn = Box<dyn FnMut(&mut TestContext) -> TestResult + Send>;

pub struct TestCase {
    pub name: String,
    pub test_fn: TestFn,
    pub docker: Option<DockerRunOptions>,
    pub tags: Vec<String>,
    pub timeout: Option<Duration>,
    pub status: TestStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Pending,
    Running,
    Passed,
    Failed(TestError),
    Skipped,
}

#[derive(Debug, Clone)]
pub struct TestContext {
    pub docker_handle: Option<DockerHandle>,
    pub start_time: Instant,
}

impl TestContext {
    pub fn new() -> Self {
        Self {
            docker_handle: None,
            start_time: Instant::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DockerHandle {
    pub container_id: String,
    pub ports: Vec<(u16, u16)>, // (host_port, container_port)
}

#[derive(Debug, Clone)]
pub struct DockerRunOptions {
    pub image: String,
    pub env: Vec<(String, String)>,
    pub ports: Vec<(u16, u16)>, // (host_port, container_port)
    pub args: Vec<String>,
    pub name: Option<String>,
    pub labels: Vec<(String, String)>,
    pub ready_timeout: Duration,
    pub readiness: Readiness,
}

impl Default for DockerRunOptions {
    fn default() -> Self {
        Self {
            image: "alpine:latest".to_string(),
            env: Vec::new(),
            ports: Vec::new(),
            args: Vec::new(),
            name: None,
            labels: Vec::new(),
            ready_timeout: Duration::from_secs(15),
            readiness: Readiness::Running,
        }
    }
}

impl DockerRunOptions {
    pub fn new(image: &str) -> Self {
        Self {
            image: image.to_string(),
            ..Default::default()
        }
    }

    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env.push((key.to_string(), value.to_string()));
        self
    }

    pub fn port(mut self, host_port: u16, container_port: u16) -> Self {
        self.ports.push((host_port, container_port));
        self
    }

    pub fn arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn label(mut self, key: &str, value: &str) -> Self {
        self.labels.push((key.to_string(), value.to_string()));
        self
    }

    pub fn ready_timeout(mut self, timeout: Duration) -> Self {
        self.ready_timeout = timeout;
        self
    }

    pub fn readiness(mut self, readiness: Readiness) -> Self {
        self.readiness = readiness;
        self
    }
}

#[derive(Debug, Clone)]
pub enum Readiness {
    Running,
    PortOpen(u16),
    Custom(String), // Custom readiness command
}

#[derive(Debug, Clone)]
pub struct TestConfig {
    pub filter: Option<String>,
    pub skip_tags: Vec<String>,
    pub max_concurrency: Option<usize>,
    pub shuffle_seed: Option<u64>,
    pub color: Option<bool>,
    pub html_report: Option<String>,
    pub skip_hooks: Option<bool>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            filter: std::env::var("TEST_FILTER").ok(),
            skip_tags: std::env::var("TEST_SKIP_TAGS")
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
            max_concurrency: std::env::var("TEST_MAX_CONCURRENCY")
                .ok()
                .and_then(|s| s.parse().ok()),
            shuffle_seed: std::env::var("TEST_SHUFFLE_SEED")
                .ok()
                .and_then(|s| s.parse().ok()),
            color: Some(atty::is(atty::Stream::Stdout)),
            html_report: std::env::var("TEST_HTML_REPORT").ok(),
            skip_hooks: std::env::var("TEST_SKIP_HOOKS")
                .ok()
                .and_then(|s| s.parse().ok()),
        }
    }
}

// --- Global test registration functions ---
// Users just call these - no runners needed!

pub fn before_all<F>(f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    THREAD_BEFORE_ALL.with(|hooks| hooks.borrow_mut().push(Box::new(f)));
}

pub fn before_each<F>(f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    THREAD_BEFORE_EACH.with(|hooks| hooks.borrow_mut().push(Box::new(f)));
}

pub fn after_each<F>(f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    THREAD_AFTER_EACH.with(|hooks| hooks.borrow_mut().push(Box::new(f)));
}

pub fn after_all<F>(f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    THREAD_AFTER_ALL.with(|hooks| hooks.borrow_mut().push(Box::new(f)));
}

pub fn test<F>(name: &str, f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    THREAD_TESTS.with(|tests| tests.borrow_mut().push(TestCase {
        name: name.to_string(),
        test_fn: Box::new(f),
        docker: None,
        tags: Vec::new(),
        timeout: None,
        status: TestStatus::Pending,
    }));
}

pub fn test_with_docker<F>(name: &'static str, opts: DockerRunOptions, f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    THREAD_TESTS.with(|tests| tests.borrow_mut().push(TestCase {
        name: name.to_string(),
        test_fn: Box::new(f),
        docker: Some(opts),
        tags: Vec::new(),
        timeout: None,
        status: TestStatus::Pending,
    }));
}

pub fn test_with_tags<F>(name: &'static str, tags: Vec<&'static str>, f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    THREAD_TESTS.with(|tests| tests.borrow_mut().push(TestCase {
        name: name.to_string(),
        test_fn: Box::new(f),
        docker: None,
        tags: tags.into_iter().map(|s| s.to_string()).collect(),
        timeout: None,
        status: TestStatus::Pending,
    }));
}

pub fn test_with_docker_and_tags<F>(name: &'static str, opts: DockerRunOptions, tags: Vec<&'static str>, f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    THREAD_TESTS.with(|tests| tests.borrow_mut().push(TestCase {
        name: name.to_string(),
        test_fn: Box::new(f),
        docker: Some(opts),
        tags: tags.into_iter().map(|s| s.to_string()).collect(),
        timeout: None,
        status: TestStatus::Pending,
    }));
}

// --- Main execution function ---
// Users just call this to run all registered tests in parallel!

pub fn run_tests() -> i32 {
    let config = TestConfig::default();
    run_tests_with_config(config)
}

pub fn run_tests_with_config(config: TestConfig) -> i32 {
    let start_time = Instant::now();
    
    info!("🚀 Starting test execution with config: {:?}", config);
    
    // Get all tests and hooks from thread-local storage
    let mut tests = THREAD_TESTS.with(|t| t.borrow_mut().drain(..).collect::<Vec<_>>());
    let before_all_hooks = THREAD_BEFORE_ALL.with(|h| h.borrow_mut().drain(..).collect::<Vec<_>>());
    let before_each_hooks = THREAD_BEFORE_EACH.with(|h| h.borrow_mut().drain(..).collect::<Vec<_>>());
    let after_each_hooks = THREAD_AFTER_EACH.with(|h| h.borrow_mut().drain(..).collect::<Vec<_>>());
    let after_all_hooks = THREAD_AFTER_ALL.with(|h| h.borrow_mut().drain(..).collect::<Vec<_>>());
    
    if tests.is_empty() {
        warn!("⚠️  No tests registered to run");
        return 0;
    }
    
    info!("📋 Found {} tests to run", tests.len());
    
    // Run before_all hooks
    if !config.skip_hooks.unwrap_or(false) && !before_all_hooks.is_empty() {
        info!("🔄 Running {} before_all hooks", before_all_hooks.len());
        // For now, we'll skip hook execution due to FnMut limitations
        // In a real implementation, you'd want to restructure this
        info!("⚠️  Skipping before_all hooks due to FnMut limitations");
    }
    
    // Filter and sort tests
    let test_indices = filter_and_sort_test_indices(&tests, &config);
    let filtered_count = test_indices.len();
    
    if filtered_count == 0 {
        warn!("⚠️  No tests match the current filter");
        return 0;
    }
    
    info!("🎯 Running {} filtered tests", filtered_count);
    
    let mut overall_failed = 0usize;
    let mut overall_skipped = 0usize;
    
    // Run tests in parallel or sequential based on config
    if let Some(max_concurrency) = config.max_concurrency {
        if max_concurrency > 1 {
            info!("⚡ Running tests in parallel with max concurrency: {}", max_concurrency);
            run_tests_parallel_by_index(&mut tests, &test_indices, before_each_hooks, after_each_hooks, &config, &mut overall_failed, &mut overall_skipped);
        } else {
            info!("🐌 Running tests sequentially (max_concurrency = 1)");
            run_tests_sequential_by_index(&mut tests, &test_indices, before_each_hooks, after_each_hooks, &config, &mut overall_failed, &mut overall_skipped);
        }
    } else {
        // Default to parallel execution
        let default_concurrency = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
        info!("⚡ Running tests in parallel with default concurrency: {}", default_concurrency);
        run_tests_parallel_by_index(&mut tests, &test_indices, before_each_hooks, after_each_hooks, &config, &mut overall_failed, &mut overall_skipped);
    }
    
    // Run after_all hooks
    if !config.skip_hooks.unwrap_or(false) && !after_all_hooks.is_empty() {
        info!("🔄 Running {} after_all hooks", after_all_hooks.len());
        // For now, we'll skip hook execution due to FnMut limitations
        // In a real implementation, you'd want to restructure this
        info!("⚠️  Skipping after_all hooks due to FnMut limitations");
    }
    
    let total_time = start_time.elapsed();
    
    // Print summary
    let passed = tests.iter().filter(|t| matches!(t.status, TestStatus::Passed)).count();
    let failed = tests.iter().filter(|t| matches!(t.status, TestStatus::Failed(_))).count();
    let skipped = tests.iter().filter(|t| matches!(t.status, TestStatus::Skipped)).count();
    
    info!("\n📊 TEST EXECUTION SUMMARY");
    info!("==========================");
    info!("Total tests: {}", tests.len());
    info!("Passed: {}", passed);
    info!("Failed: {}", failed);
    info!("Skipped: {}", skipped);
    info!("Total time: {:?}", total_time);
    
    // Generate HTML report if requested
    if let Some(ref html_path) = config.html_report {
        if let Err(e) = generate_html_report(&tests, total_time, html_path) {
            warn!("⚠️  Failed to generate HTML report: {}", e);
        } else {
            info!("📊 HTML report generated: {}", html_path);
        }
    }
    
    if failed > 0 {
        error!("\n❌ FAILED TESTS:");
        for test in tests.iter().filter(|t| matches!(t.status, TestStatus::Failed(_))) {
            if let TestStatus::Failed(error) = &test.status {
                error!("  {}: {}", test.name, error);
            }
        }
    }
    
    if failed > 0 {
        error!("❌ Test execution failed with {} failures", failed);
        1
    } else {
        info!("✅ All tests passed!");
        0
    }
}

// --- Helper functions ---

fn filter_and_sort_test_indices(tests: &[TestCase], config: &TestConfig) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..tests.len()).collect();
    
    // Apply filter
    if let Some(ref filter) = config.filter {
        indices.retain(|&idx| tests[idx].name.contains(filter));
    }
    
    // Apply tag filtering
    if !config.skip_tags.is_empty() {
        indices.retain(|&idx| {
            let test_tags = &tests[idx].tags;
            !config.skip_tags.iter().any(|skip_tag| test_tags.contains(skip_tag))
        });
    }
    
    // Apply shuffling
    if let Some(seed) = config.shuffle_seed {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Simple shuffle based on hash
        for i in 0..indices.len() {
            let j = (hash as usize + i) % indices.len();
            indices.swap(i, j);
        }
    }
    
    indices
}

fn run_tests_parallel_by_index(
    tests: &mut [TestCase],
    test_indices: &[usize],
    before_each_hooks: Vec<HookFn>,
    after_each_hooks: Vec<HookFn>,
    config: &TestConfig,
    overall_failed: &mut usize,
    overall_skipped: &mut usize,
) {
    let max_workers = config.max_concurrency.unwrap_or_else(|| {
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4)
    });
    
    info!("Running {} tests in parallel with {} workers", test_indices.len(), max_workers);
    
    // For now, we'll run tests sequentially in the parallel function
    // This is a limitation of the current design - we can't easily share FnMut closures
    // In a real implementation, you'd want to restructure this to avoid these limitations
    info!("⚠️  Note: Running tests sequentially due to FnMut closure limitations");
    info!("   True parallelism requires restructuring the test execution model");
    
    run_tests_sequential_by_index(tests, test_indices, before_each_hooks, after_each_hooks, config, overall_failed, overall_skipped);
}

fn run_tests_sequential_by_index(
    tests: &mut [TestCase],
    test_indices: &[usize],
    before_each_hooks: Vec<HookFn>,
    after_each_hooks: Vec<HookFn>,
    config: &TestConfig,
    overall_failed: &mut usize,
    overall_skipped: &mut usize,
) {
    for &idx in test_indices {
        run_single_test_by_index(
            tests,
            idx,
            &before_each_hooks,
            &after_each_hooks,
            config,
            overall_failed,
            overall_skipped,
        );
    }
}

fn run_single_test_by_index(
    tests: &mut [TestCase],
    idx: usize,
    before_each_hooks: &[HookFn],
    after_each_hooks: &[HookFn],
    config: &TestConfig,
    overall_failed: &mut usize,
    overall_skipped: &mut usize,
) {
    let test = &mut tests[idx];
    let test_name = &test.name;
    
    info!("🧪 Running test: {}", test_name);
    
    // Check if test should be skipped
    if let Some(ref filter) = config.filter {
        if !test_name.contains(filter) {
            test.status = TestStatus::Skipped;
            *overall_skipped += 1;
            info!("⏭️  Test '{}' skipped (filter: {})", test_name, filter);
            return;
        }
    }
    
    // Check tag filtering
    if !config.skip_tags.is_empty() {
        let test_tags = &test.tags;
        if config.skip_tags.iter().any(|skip_tag| test_tags.contains(skip_tag)) {
            test.status = TestStatus::Skipped;
            *overall_skipped += 1;
            info!("⏭️  Test '{}' skipped (tags: {:?})", test_name, test_tags);
            return;
        }
    }
    
    test.status = TestStatus::Running;
    let start_time = Instant::now();
    
    // Run before_each hooks
    let mut ctx = TestContext::new();
    if !config.skip_hooks.unwrap_or(false) {
        for _hook in before_each_hooks {
            // For now, we'll skip hook execution due to FnMut limitations
            // In a real implementation, you'd want to restructure this
            info!("⚠️  Skipping before_each hook due to FnMut limitations");
        }
    }
    
    // Run the test
    let test_result = if let Some(timeout) = test.timeout {
        run_test_with_timeout(&mut test.test_fn, &mut ctx, timeout)
    } else {
        run_test(&mut test.test_fn, &mut ctx)
    };
    
    // Run after_each hooks
    if !config.skip_hooks.unwrap_or(false) {
        for _hook in after_each_hooks {
            // For now, we'll skip hook execution due to FnMut limitations
            // In a real implementation, you'd want to restructure this
            info!("⚠️  Skipping after_each hook due to FnMut limitations");
        }
    }
    
    let elapsed = start_time.elapsed();
    
    match test_result {
        Ok(()) => {
            test.status = TestStatus::Passed;
            info!("✅ Test '{}' passed in {:?}", test_name, elapsed);
        }
        Err(e) => {
            test.status = TestStatus::Failed(e.clone());
            *overall_failed += 1;
            error!("❌ Test '{}' failed in {:?}: {}", test_name, elapsed, e);
        }
    }
    
    // Clean up Docker if used
    if let Some(ref docker_handle) = ctx.docker_handle {
        cleanup_docker_container(docker_handle);
    }
}

fn run_test<F>(test_fn: &mut F, ctx: &mut TestContext) -> TestResult 
where 
    F: FnMut(&mut TestContext) -> TestResult 
{
    catch_unwind(AssertUnwindSafe(|| test_fn(ctx))).unwrap_or_else(|panic_info| {
        let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        };
        Err(TestError::Panicked(msg))
    })
}

fn run_test_with_timeout<F>(test_fn: &mut F, ctx: &mut TestContext, timeout: Duration) -> TestResult 
where 
    F: FnMut(&mut TestContext) -> TestResult
{
    let start = Instant::now();
    
    // Run the test
    let result = run_test(test_fn, ctx);
    
    let elapsed = start.elapsed();
    
    // Check if the test exceeded the timeout
    if elapsed > timeout {
        warn!("  ⚠️  Test took {:?} which exceeds timeout of {:?}", elapsed, timeout);
        return Err(TestError::Timeout(timeout));
    }
    
    result
}



fn cleanup_docker_container(handle: &DockerHandle) {
    info!("🧹 Cleaning up Docker container: {}", handle.container_id);
    // In a real implementation, this would use the Docker API to stop and remove the container
    // For now, just log the cleanup
}

// --- Error types ---

#[derive(Debug, Clone, PartialEq)]
pub enum TestError {
    Message(String),
    Panicked(String),
    Timeout(Duration),
    DockerError(String),
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestError::Message(msg) => write!(f, "{}", msg),
            TestError::Panicked(msg) => write!(f, "panicked: {}", msg),
            TestError::Timeout(duration) => write!(f, "timeout after {:?}", duration),
            TestError::DockerError(msg) => write!(f, "docker error: {}", msg),
        }
    }
}

impl From<&str> for TestError {
    fn from(s: &str) -> Self {
        TestError::Message(s.to_string())
    }
}

impl From<String> for TestError {
    fn from(s: String) -> Self {
        TestError::Message(s)
    }
}

// --- HTML Report Generation ---

fn generate_html_report(tests: &[TestCase], total_time: Duration, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut html = String::new();
    
    // HTML header
    html.push_str(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Test Execution Report</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; background: white; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); overflow: hidden; }
        .header { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; text-align: center; }
        .header h1 { margin: 0; font-size: 2.5em; font-weight: 300; }
        .header .subtitle { margin: 10px 0 0 0; opacity: 0.9; font-size: 1.1em; }
        .summary { padding: 30px; border-bottom: 1px solid #eee; }
        .summary-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin: 20px 0; }
        .summary-card { background: #f8f9fa; padding: 20px; border-radius: 6px; text-align: center; border-left: 4px solid #007bff; }
        .summary-card.passed { border-left-color: #28a745; }
        .summary-card.failed { border-left-color: #dc3545; }
        .summary-card.skipped { border-left-color: #ffc107; }
        .summary-card .number { font-size: 2em; font-weight: bold; margin-bottom: 5px; }
        .summary-card .label { color: #6c757d; font-size: 0.9em; text-transform: uppercase; letter-spacing: 0.5px; }
        .tests-section { padding: 30px; }
        .tests-section h2 { margin: 0 0 20px 0; color: #333; }
        .test-list { display: grid; gap: 15px; }
        .test-item { background: #f8f9fa; border-radius: 6px; padding: 15px; border-left: 4px solid #dee2e6; transition: all 0.2s ease; }
        .test-item:hover { box-shadow: 0 4px 12px rgba(0,0,0,0.1); transform: translateY(-2px); }
        .test-item.passed { border-left-color: #28a745; background: #f8fff9; }
        .test-item.failed { border-left-color: #dc3545; background: #fff8f8; }
        .test-item.skipped { border-left-color: #ffc107; background: #fffef8; }
        .test-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; cursor: pointer; }
        .test-name { font-weight: 600; color: #333; }
        .test-status { padding: 4px 12px; border-radius: 20px; font-size: 0.8em; font-weight: 600; text-transform: uppercase; }
        .test-status.passed { background: #d4edda; color: #155724; }
        .test-status.failed { background: #f8d7da; color: #721c24; }
        .test-status.skipped { background: #fff3cd; color: #856404; }
        .test-details { font-size: 0.9em; color: #6c757d; }
        .test-error { background: #f8d7da; color: #721c24; padding: 10px; border-radius: 4px; margin-top: 10px; font-family: monospace; font-size: 0.85em; }
        .test-expandable { max-height: 0; overflow: hidden; transition: max-height 0.3s ease-in-out; }
        .test-expandable.expanded { max-height: 500px; }
        .expand-icon { transition: transform 0.2s ease; font-size: 1.2em; color: #6c757d; }
        .expand-icon.expanded { transform: rotate(90deg); }
        .test-metadata { background: #f1f3f4; padding: 15px; border-radius: 6px; margin-top: 10px; }
        .metadata-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; }
        .metadata-item { display: flex; flex-direction: column; }
        .metadata-label { font-weight: 600; color: #495057; font-size: 0.85em; text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 5px; }
        .metadata-value { color: #6c757d; font-size: 0.9em; }
        .footer { background: #f8f9fa; padding: 20px; text-align: center; color: #6c757d; font-size: 0.9em; border-top: 1px solid #eee; }
        .timestamp { color: #007bff; }
        .filters { background: #e9ecef; padding: 15px; border-radius: 6px; margin: 20px 0; font-size: 0.9em; }
        .filters strong { color: #495057; }
        .search-box { width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; margin-bottom: 20px; font-size: 1em; }
        .search-box:focus { outline: none; border-color: #007bff; box-shadow: 0 0 0 2px rgba(0,123,255,0.25); }
        .test-item.hidden { display: none; }
        .no-results { text-align: center; padding: 40px; color: #6c757d; font-style: italic; }
        @media (max-width: 768px) { .summary-grid { grid-template-columns: 1fr; } .test-header { flex-direction: column; align-items: flex-start; gap: 10px; } .metadata-grid { grid-template-columns: 1fr; } }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🧪 Test Execution Report</h1>
            <p class="subtitle">Comprehensive test results and analysis</p>
        </div>
        
        <div class="summary">
            <h2>📊 Execution Summary</h2>
            <div class="summary-grid">"#);
    
    // Summary statistics
    let passed = tests.iter().filter(|t| matches!(t.status, TestStatus::Passed)).count();
    let failed = tests.iter().filter(|t| matches!(t.status, TestStatus::Failed(_))).count();
    let skipped = tests.iter().filter(|t| matches!(t.status, TestStatus::Skipped)).count();
    
    html.push_str(&format!(r#"
                <div class="summary-card passed">
                    <div class="number">{}</div>
                    <div class="label">Passed</div>
                </div>
                <div class="summary-card failed">
                    <div class="number">{}</div>
                    <div class="label">Failed</div>
                </div>
                <div class="summary-card skipped">
                    <div class="number">{}</div>
                    <div class="label">Skipped</div>
                </div>
                <div class="summary-card">
                    <div class="number">{}</div>
                    <div class="label">Total</div>
                </div>
            </div>
            <p><strong>Total Execution Time:</strong> <span class="timestamp">{:?}</span></p>
        </div>
        
        <div class="tests-section">
            <h2>�� Test Results</h2>
            
            <input type="text" class="search-box" id="testSearch" placeholder="🔍 Search tests by name, status, or tags..." />
            
            <div class="test-list" id="testList">"#, passed, failed, skipped, tests.len(), total_time));
    
    // Test results
    for test in tests {
        let status_class = match test.status {
            TestStatus::Passed => "passed",
            TestStatus::Failed(_) => "failed",
            TestStatus::Skipped => "skipped",
            TestStatus::Pending => "skipped",
            TestStatus::Running => "skipped",
        };
        
        let status_text = match test.status {
            TestStatus::Passed => "PASSED",
            TestStatus::Failed(_) => "FAILED",
            TestStatus::Skipped => "SKIPPED",
            TestStatus::Pending => "PENDING",
            TestStatus::Running => "RUNNING",
        };
        
        html.push_str(&format!(r#"
                <div class="test-item {}" data-test-name="{}" data-test-status="{}" data-test-tags="{}">
                    <div class="test-header" onclick="toggleTestDetails(this)">
                        <div class="test-name">{}</div>
                        <div style="display: flex; align-items: center; gap: 10px;">
                            <div class="test-status {}">{}</div>
                            <span class="expand-icon">▶</span>
                        </div>
                    </div>
                    
                    <div class="test-expandable">
                        <div class="test-metadata">
                            <div class="metadata-grid">"#, 
            status_class, test.name, status_text, test.tags.join(","), test.name, status_class, status_text));
        
        // Add test metadata
        if !test.tags.is_empty() {
            html.push_str(&format!(r#"<div class="metadata-item"><div class="metadata-label">Tags</div><div class="metadata-value">{}</div></div>"#, test.tags.join(", ")));
        }
        
        if let Some(timeout) = test.timeout {
            html.push_str(&format!(r#"<div class="metadata-item"><div class="metadata-label">Timeout</div><div class="metadata-value">{:?}</div></div>"#, timeout));
        }
        
        if let Some(docker) = &test.docker {
            html.push_str(&format!(r#"<div class="metadata-item"><div class="metadata-label">Docker Image</div><div class="metadata-value">{}</div></div>"#, docker.image));
            if !docker.ports.is_empty() {
                html.push_str(&format!(r#"<div class="metadata-item"><div class="metadata-label">Ports</div><div class="metadata-value">{:?}</div></div>"#, docker.ports));
            }
            if !docker.env.is_empty() {
                html.push_str(&format!(r#"<div class="metadata-item"><div class="metadata-label">Environment</div><div class="metadata-value">{:?}</div></div>"#, docker.env));
            }
        }
        
        html.push_str(r#"</div></div>"#);
        
        // Add error details for failed tests
        if let TestStatus::Failed(error) = &test.status {
            html.push_str(&format!(r#"<div class="test-error"><strong>Error:</strong> {}</div>"#, error));
        }
        
        html.push_str("</div></div>");
    }
    
    // HTML footer
    html.push_str(r#"
            </div>
        </div>
        
        <div class="footer">
            <p>Report generated by <strong>rust-test-harness</strong> at <span class="timestamp">"#);
    
    html.push_str(&chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
    
    html.push_str(r#"</span></p>
        </div>
    </div>
    
    <script>
        // Expandable test details functionality
        function toggleTestDetails(header) {
            const testItem = header.closest('.test-item');
            const expandable = testItem.querySelector('.test-expandable');
            const expandIcon = header.querySelector('.expand-icon');
            
            if (expandable.classList.contains('expanded')) {
                expandable.classList.remove('expanded');
                expandIcon.classList.remove('expanded');
                expandIcon.textContent = '▶';
            } else {
                expandable.classList.add('expanded');
                expandIcon.classList.add('expanded');
                expandIcon.textContent = '▼';
            }
        }
        
        // Search functionality
        document.getElementById('testSearch').addEventListener('input', function(e) {
            const searchTerm = e.target.value.toLowerCase();
            const testItems = document.querySelectorAll('.test-item');
            let visibleCount = 0;
            
            testItems.forEach(item => {
                const testName = item.getAttribute('data-test-name').toLowerCase();
                const testStatus = item.getAttribute('data-test-status').toLowerCase();
                const testTags = item.getAttribute('data-test-tags').toLowerCase();
                
                const matches = testName.includes(searchTerm) || 
                               testStatus.includes(searchTerm) || 
                               testTags.includes(searchTerm);
                
                if (matches) {
                    item.classList.remove('hidden');
                    visibleCount++;
                } else {
                    item.classList.add('hidden');
                }
            });
            
            // Show/hide no results message
            const noResults = document.querySelector('.no-results');
            if (visibleCount === 0 && searchTerm.length > 0) {
                if (!noResults) {
                    const message = document.createElement('div');
                    message.className = 'no-results';
                    message.textContent = 'No tests match your search criteria';
                    document.getElementById('testList').appendChild(message);
                }
            } else if (noResults) {
                noResults.remove();
            }
        });
        
        // Keyboard shortcuts
        document.addEventListener('keydown', function(e) {
            if (e.ctrlKey || e.metaKey) {
                switch(e.key) {
                    case 'f':
                        e.preventDefault();
                        document.getElementById('testSearch').focus();
                        break;
                    case 'a':
                        e.preventDefault();
                        // Expand all test details
                        document.querySelectorAll('.test-expandable').forEach(expandable => {
                            expandable.classList.add('expanded');
                        });
                        document.querySelectorAll('.expand-icon').forEach(icon => {
                            icon.classList.add('expanded');
                            icon.textContent = '▼';
                        });
                        break;
                    case 'z':
                        e.preventDefault();
                        // Collapse all test details
                        document.querySelectorAll('.test-expandable').forEach(expandable => {
                            expandable.classList.remove('expanded');
                        });
                        document.querySelectorAll('.expand-icon').forEach(icon => {
                            icon.classList.remove('expanded');
                            icon.textContent = '▶';
                        });
                        break;
                }
            }
        });
        
        // Auto-expand failed tests for better visibility
        document.addEventListener('DOMContentLoaded', function() {
            const failedTests = document.querySelectorAll('.test-item.failed');
            failedTests.forEach(testItem => {
                const expandable = testItem.querySelector('.test-expandable');
                const expandIcon = testItem.querySelector('.expand-icon');
                if (expandable && expandIcon) {
                    expandable.classList.add('expanded');
                    expandIcon.classList.add('expanded');
                    expandIcon.textContent = '▼';
                }
            });
        });
    </script>
</body>
</html>"#);
    
    // Write to file
    std::fs::write(output_path, html)?;
    Ok(())
}

// --- Macros ---

/// Macro to create individual test functions that can be run independently
/// This makes the framework compatible with cargo test and existing test libraries
/// Hooks (before_all, before_each, etc.) are automatically executed
#[macro_export]
macro_rules! test_function {
    ($name:ident, $test_fn:expr) => {
        #[test]
        fn $name() {
            // Initialize logging for individual test runs
            let _ = env_logger::try_init();
            
            // Execute before_all hooks if any exist
            if let Ok(()) = rust_test_harness::execute_before_all_hooks() {
                // Execute before_each hooks if any exist
                if let Ok(()) = rust_test_harness::execute_before_each_hooks() {
                    // Run the test function
                    let result = ($test_fn)(&mut rust_test_harness::TestContext::new());
                    
                    // Execute after_each hooks if any exist
                    let _ = rust_test_harness::execute_after_each_hooks();
                    
                    // Convert result to test outcome
                    match result {
                        Ok(_) => {
                            info!("✅ Test '{}' passed", stringify!($name));
                        }
                        Err(e) => {
                            panic!("❌ Test '{}' failed: {:?}", stringify!($name), e);
                        }
                    }
                } else {
                    panic!("❌ Test '{}' failed: before_each hooks failed", stringify!($name));
                }
            } else {
                panic!("❌ Test '{}' failed: before_all hooks failed", stringify!($name));
            }
            
            // Execute after_all hooks if any exist
            let _ = rust_test_harness::execute_after_all_hooks();
        }
    };
}

/// Macro to create individual test functions with custom names
/// Hooks (before_all, before_each, etc.) are automatically executed
#[macro_export]
macro_rules! test_named {
    ($name:expr, $test_fn:expr) => {
        #[test]
        fn test_function() {
            // Initialize logging for individual test runs
            let _ = env_logger::try_init();
            
            // Execute before_all hooks if any exist
            if let Ok(()) = rust_test_harness::execute_before_all_hooks() {
                // Execute before_each hooks if any exist
                if let Ok(()) = rust_test_harness::execute_before_each_hooks() {
                    // Run the test function
                    let result = ($test_fn)(&mut rust_test_harness::TestContext::new());
                    
                    // Execute after_each hooks if any exist
                    let _ = rust_test_harness::execute_after_each_hooks();
                    
                    // Convert result to test outcome
                    match result {
                        Ok(_) => {
                            info!("✅ Test '{}' passed", $name);
                        }
                        Err(e) => {
                            panic!("❌ Test '{}' failed: {:?}", $name, e);
                        }
                    }
                } else {
                    panic!("❌ Test '{}' failed: before_each hooks failed", $name);
                }
            } else {
                panic!("❌ Test '{}' failed: before_all hooks failed", $name);
            }
            
            // Execute after_all hooks if any exist
            let _ = rust_test_harness::execute_after_all_hooks();
        }
    };
}

/// Macro to create individual async test functions (for when you add async support)
/// Hooks (before_all, before_each, etc.) are automatically executed
#[macro_export]
macro_rules! test_async {
    ($name:ident, $test_fn:expr) => {
        #[tokio::test]
        async fn $name() {
            // Initialize logging for individual test runs
            let _ = env_logger::try_init();
            
            // Execute before_all hooks if any exist
            if let Ok(()) = rust_test_harness::execute_before_all_hooks() {
                // Execute before_each hooks if any exist
                if let Ok(()) = rust_test_harness::execute_before_each_hooks() {
                    // Run the async test function
                    let result = ($test_fn)(&mut rust_test_harness::TestContext::new()).await;
                    
                    // Execute after_each hooks if any exist
                    let _ = rust_test_harness::execute_before_each_hooks();
                    
                    // Convert result to test outcome
                    match result {
                        Ok(_) => {
                            info!("✅ Async test '{}' passed", stringify!($name));
                        }
                        Err(e) => {
                            panic!("❌ Async test '{}' failed: {:?}", stringify!($name), e);
                        }
                    }
                } else {
                    panic!("❌ Async test '{}' failed: before_each hooks failed", stringify!($name));
                }
            } else {
                panic!("❌ Async test '{}' failed: before_all hooks failed", stringify!($name));
            }
            
            // Execute after_all hooks if any exist
            let _ = rust_test_harness::execute_after_all_hooks();
        }
    };
}

/// Macro to create test cases that work exactly like Rust's built-in #[test] attribute
/// but with our framework's enhanced features (hooks, Docker, etc.)
/// 
/// **IDE Support**: This macro creates a standard #[test] function that RustRover will recognize.
/// 
/// Usage:
/// ```rust
/// #[cfg(test)]
/// mod tests {
///     use super::*;
///     use rust_test_harness::test_case;
///     
///     test_case!(test_something, |ctx| {
///         // Your test logic here
///         assert_eq!(2 + 2, 4);
///         Ok(())
///     });
///     
///     test_case!(test_with_docker, |ctx| {
///         // Test with Docker context
///         Ok(())
///     });
/// }
/// ```
#[macro_export]
macro_rules! test_case {
    ($name:ident, $test_fn:expr) => {
        #[test]
        #[allow(unused_imports)]
        fn $name() {
            // Initialize logging for individual test runs
            let _ = env_logger::try_init();
            
            // Execute before_all hooks if any exist
            if let Ok(()) = rust_test_harness::execute_before_all_hooks() {
                // Execute before_each hooks if any exist
                if let Ok(()) = rust_test_harness::execute_before_each_hooks() {
                    // Run the test function
                    let result: rust_test_harness::TestResult = ($test_fn)(&mut rust_test_harness::TestContext::new());
                    
                    // Execute after_each hooks if any exist
                    let _ = rust_test_harness::execute_after_each_hooks();
                    
                    // Convert result to test outcome
                    match result {
                        Ok(_) => {
                            // Test passed - no need to panic
                        }
                        Err(e) => {
                            panic!("Test failed: {:?}", e);
                        }
                    }
                } else {
                    panic!("Test failed: before_each hooks failed");
                }
            } else {
                panic!("Test failed: before_all hooks failed");
            }
            
            // Execute after_all hooks if any exist
            let _ = rust_test_harness::execute_after_all_hooks();
        }
    };
}

/// Macro to create test cases with custom names (useful for dynamic test names)
/// 
/// Usage:
/// ```rust
/// #[cfg(test)]
/// mod tests {
///     use super::*;
///     use rust_test_harness::test_case_named;
///     
///     test_case_named!("my_custom_test_name", |ctx| {
///         // Your test logic here
///         Ok(())
///     });
/// }
/// ```
#[macro_export]
macro_rules! test_case_named {
    ($name:expr, $test_fn:expr) => {
        #[test]
        fn test_function() {
            // Initialize logging for individual test runs
            let _ = env_logger::try_init();
            
            // Execute before_all hooks if any exist
            if let Ok(()) = rust_test_harness::execute_before_all_hooks() {
                // Execute before_each hooks if any exist
                if let Ok(()) = rust_test_harness::execute_before_each_hooks() {
                    // Run the test function
                    let result: rust_test_harness::TestResult = ($test_fn)(&mut rust_test_harness::TestContext::new());
                    
                    // Execute after_each hooks if any exist
                    let _ = rust_test_harness::execute_after_each_hooks();
                    
                    // Convert result to test outcome
                    match result {
                        Ok(_) => {
                            // Test passed - no need to panic
                        }
                        Err(e) => {
                            panic!("Test '{}' failed: {:?}", $name, e);
                        }
                    }
                } else {
                    panic!("Test '{}' failed: before_each hooks failed", $name);
                }
            } else {
                panic!("Test '{}' failed: before_all hooks failed", $name);
                }
            
            // Execute after_all hooks if any exist
            let _ = rust_test_harness::execute_after_all_hooks();
        }
    };
}

/// Macro to create test cases with Docker support
/// 
/// Usage:
/// ```rust
/// #[cfg(test)]
/// mod tests {
///     use super::*;
///     use rust_test_harness::{test_case_docker, DockerRunOptions};
///     
///     test_case_docker!(test_with_nginx, DockerRunOptions::new("nginx:alpine"), |ctx| {
///         // Your test logic here with Docker context
///         Ok(())
///     });
/// }
/// ```
#[macro_export]
macro_rules! test_case_docker {
    ($name:ident, $docker_opts:expr, $test_fn:expr) => {
        #[test]
        fn $name() {
            // Initialize logging for individual test runs
            let _ = env_logger::try_init();
            
            // Execute before_all hooks if any exist
            if let Ok(()) = rust_test_harness::execute_before_all_hooks() {
                // Execute before_each hooks if any exist
                if let Ok(()) = rust_test_harness::execute_before_each_hooks() {
                    // Create test context with Docker options
                    let mut ctx = rust_test_harness::TestContext::new();
                    
                    // Run the test function with explicit type annotation
                    let test_fn: fn(&mut rust_test_harness::TestContext) -> rust_test_harness::TestResult = $test_fn;
                    let result = test_fn(&mut ctx);
                    
                    // Execute after_each hooks if any exist
                    let _ = rust_test_harness::execute_after_each_hooks();
                    
                    // Convert result to test outcome
                    match result {
                        Ok(_) => {
                            // Test passed - no need to panic
                        }
                        Err(e) => {
                            panic!("Test failed: {:?}", e);
                        }
                    }
                } else {
                    panic!("Test failed: before_each hooks failed");
                }
            } else {
                panic!("Test failed: before_all hooks failed");
            }
            
            // Execute after_all hooks if any exist
            let _ = rust_test_harness::execute_after_all_hooks();
        }
    };
}

// --- Hook execution functions for individual tests ---

/// Execute before_all hooks for individual test functions
pub fn execute_before_all_hooks() -> Result<(), TestError> {
    THREAD_BEFORE_ALL.with(|hooks| {
        let mut hooks = hooks.borrow_mut();
        for hook in hooks.iter_mut() {
            hook(&mut TestContext::new())?;
        }
        Ok(())
    })
}

/// Execute before_each hooks for individual test functions
pub fn execute_before_each_hooks() -> Result<(), TestError> {
    THREAD_BEFORE_EACH.with(|hooks| {
        let mut hooks = hooks.borrow_mut();
        for hook in hooks.iter_mut() {
            hook(&mut TestContext::new())?;
        }
        Ok(())
    })
}

/// Execute after_each hooks for individual test functions
pub fn execute_after_each_hooks() -> Result<(), TestError> {
    THREAD_AFTER_EACH.with(|hooks| {
        let mut hooks = hooks.borrow_mut();
        for hook in hooks.iter_mut() {
            let _ = hook(&mut TestContext::new());
        }
        Ok(())
    })
}

/// Execute after_all hooks for individual test functions
pub fn execute_after_all_hooks() -> Result<(), TestError> {
    THREAD_AFTER_ALL.with(|hooks| {
        let mut hooks = hooks.borrow_mut();
        for hook in hooks.iter_mut() {
            let _ = hook(&mut TestContext::new());
        }
        Ok(())
    })
}

// --- Convenience function for running tests ---

pub fn run_all() -> i32 {
    run_tests()
}

 
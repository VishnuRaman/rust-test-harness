use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::collections::HashMap;
use std::any::Any;
use std::cell::RefCell;
use once_cell::sync::OnceCell;
use log::{info, warn, error};

// Global shared context for before_all/after_all hooks
static GLOBAL_SHARED_DATA: OnceCell<Arc<Mutex<HashMap<String, String>>>> = OnceCell::new();

// Global container registry for automatic cleanup
static CONTAINER_REGISTRY: OnceCell<Arc<Mutex<Vec<String>>>> = OnceCell::new();

pub fn get_global_context() -> Arc<Mutex<HashMap<String, String>>> {
    GLOBAL_SHARED_DATA.get_or_init(|| Arc::new(Mutex::new(HashMap::new()))).clone()
}

pub fn clear_global_context() {
    if let Some(global_ctx) = GLOBAL_SHARED_DATA.get() {
        if let Ok(mut map) = global_ctx.lock() {
            map.clear();
        }
    }
}

pub fn get_container_registry() -> Arc<Mutex<Vec<String>>> {
    CONTAINER_REGISTRY.get_or_init(|| Arc::new(Mutex::new(Vec::new()))).clone()
}

pub fn register_container_for_cleanup(container_id: &str) {
    if let Ok(mut containers) = get_container_registry().lock() {
        containers.push(container_id.to_string());
        info!("📝 Registered container {} for automatic cleanup", container_id);
    }
}

pub fn cleanup_all_containers() {
    if let Ok(mut containers) = get_container_registry().lock() {
        info!("🧹 Cleaning up {} registered containers", containers.len());
        let container_ids: Vec<String> = containers.drain(..).collect();
        drop(containers); // Drop the lock before processing
        
        // Clean up containers with timeout protection
        for container_id in container_ids {
            let config = ContainerConfig::new("dummy"); // dummy config for cleanup
            
            // Use a timeout to prevent hanging
            let stop_result = std::panic::catch_unwind(|| {
                // Set a reasonable timeout for container stop operations
                let stop_future = config.stop(&container_id);
                
                // In a real implementation, we'd use async/await with timeout
                // For now, we'll just attempt the stop and log any issues
                match stop_future {
                    Ok(_) => info!("✅ Successfully stopped container {}", container_id),
                    Err(e) => warn!("Failed to cleanup container {}: {}", container_id, e),
                }
            });
            
            if let Err(panic_info) = stop_result {
                warn!("Panic while stopping container {}: {:?}", container_id, panic_info);
            }
        }
    }
}

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
pub type TestFn = Box<dyn FnOnce(&mut TestContext) -> TestResult + Send + 'static>;
pub type HookFn = Arc<Mutex<Box<dyn FnMut(&mut TestContext) -> TestResult + Send>>>;

pub struct TestCase {
    pub name: String,
    pub test_fn: Option<TestFn>, // Changed to Option to allow safe Send+Sync
    pub tags: Vec<String>,
    pub timeout: Option<Duration>,
    pub status: TestStatus,
}

impl Clone for TestCase {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            test_fn: None, // Clone with None since we can't clone the function
            tags: self.tags.clone(),
            timeout: self.timeout.clone(),
            status: self.status.clone(),
        }
    }
}

// TestCase is now automatically Send + Sync since test_fn is Option<TestFn>
// and all other fields are already Send + Sync

#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Pending,
    Running,
    Passed,
    Failed(TestError),
    Skipped,
}

#[derive(Debug)]
pub struct TestContext {
    pub docker_handle: Option<DockerHandle>,
    pub start_time: Instant,
    pub data: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl TestContext {
    pub fn new() -> Self {
        Self {
            docker_handle: None,
            start_time: Instant::now(),
            data: HashMap::new(),
        }
    }
    
    /// Store arbitrary data in the test context
    pub fn set_data<T: Any + Send + Sync>(&mut self, key: &str, value: T) {
        self.data.insert(key.to_string(), Box::new(value));
    }
    
    /// Retrieve data from the test context
    pub fn get_data<T: Any + Send + Sync>(&self, key: &str) -> Option<&T> {
        self.data.get(key).and_then(|boxed| boxed.downcast_ref::<T>())
    }
    
    /// Check if data exists in the test context
    pub fn has_data(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
    
    /// Remove data from the test context
    pub fn remove_data<T: Any + Send + Sync>(&mut self, key: &str) -> Option<T> {
        self.data.remove(key).and_then(|boxed| {
            match boxed.downcast::<T>() {
                Ok(value) => Some(*value),
                Err(_) => None,
            }
        })
    }
    
    // Removed get_global_data function - it was a footgun that never worked
    // Use get_data() instead, which properly accesses data set by before_all hooks
}

impl Clone for TestContext {
    fn clone(&self) -> Self {
        Self {
            docker_handle: self.docker_handle.clone(),
            start_time: self.start_time,
            data: HashMap::new(), // Can't clone Box<dyn Any>, start fresh
        }
    }
}

#[derive(Debug, Clone)]
pub struct DockerHandle {
    pub container_id: String,
    pub ports: Vec<(u16, u16)>, // (host_port, container_port)
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
    pub timeout_config: TimeoutConfig,
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
            timeout_config: TimeoutConfig::default(),
        }
    }
}

// --- Global test registration functions ---
// Users just call these - no runners needed!

pub fn before_all<F>(f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    THREAD_BEFORE_ALL.with(|hooks| hooks.borrow_mut().push(Arc::new(Mutex::new(Box::new(f)))));
}

pub fn before_each<F>(f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    THREAD_BEFORE_EACH.with(|hooks| hooks.borrow_mut().push(Arc::new(Mutex::new(Box::new(f)))));
}

pub fn after_each<F>(f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    THREAD_AFTER_EACH.with(|hooks| hooks.borrow_mut().push(Arc::new(Mutex::new(Box::new(f)))));
}

pub fn after_all<F>(f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    THREAD_AFTER_ALL.with(|hooks| hooks.borrow_mut().push(Arc::new(Mutex::new(Box::new(f)))));
}

pub fn test<F>(name: &str, f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    THREAD_TESTS.with(|tests| tests.borrow_mut().push(TestCase {
        name: name.to_string(),
        test_fn: Some(Box::new(f)),
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
        test_fn: Some(Box::new(f)),
        tags: tags.into_iter().map(|s| s.to_string()).collect(),
        timeout: None,
        status: TestStatus::Pending,
    }));
}



pub fn test_with_timeout<F>(name: &str, timeout: Duration, f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    THREAD_TESTS.with(|tests| tests.borrow_mut().push(TestCase {
        name: name.to_string(),
        test_fn: Some(Box::new(f)),
        tags: Vec::new(),
        timeout: Some(timeout),
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
    
    info!("📋 Found {} tests to run", tests.len());
    
    if tests.is_empty() {
        warn!("⚠️  No tests registered to run");
        return 0;
    }
    
    // Run before_all hooks ONCE at the beginning
    let mut shared_context = TestContext::new();
    if !config.skip_hooks.unwrap_or(false) && !before_all_hooks.is_empty() {
        info!("🔄 Running {} before_all hooks", before_all_hooks.len());
        
        // Execute each before_all hook with the shared context
        for hook in before_all_hooks {
            // Wrap hook execution with panic safety
            let result = catch_unwind(AssertUnwindSafe(|| {
                if let Ok(mut hook_fn) = hook.lock() {
                    hook_fn(&mut shared_context)
                } else {
                    Err(TestError::Message("Failed to acquire hook lock".into()))
                }
            }));
            match result {
                Ok(Ok(())) => {
                    // Hook succeeded
                }
                Ok(Err(e)) => {
                    error!("❌ before_all hook failed: {}", e);
                    return 1; // Fail the entire test run
                }
                Err(panic_info) => {
                    let panic_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_info.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic".to_string()
                    };
                    error!("💥 before_all hook panicked: {}", panic_msg);
                    return 1; // Fail the entire test run
                }
            }
        }
        
        info!("✅ before_all hooks completed");
        
        // Copy data from shared context to global context for individual tests
        let global_ctx = get_global_context();
        clear_global_context(); // Clear any existing data
        for (key, value) in &shared_context.data {
            if let Some(string_value) = value.downcast_ref::<String>() {
                if let Ok(mut map) = global_ctx.lock() {
                    map.insert(key.clone(), string_value.clone());
                }
            }
        }
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
            run_tests_parallel_by_index(&mut tests, &test_indices, before_each_hooks, after_each_hooks, &config, &mut overall_failed, &mut overall_skipped, &mut shared_context);
        } else {
            info!("🐌 Running tests sequentially (max_concurrency = 1)");
            run_tests_sequential_by_index(&mut tests, &test_indices, before_each_hooks, after_each_hooks, &config, &mut overall_failed, &mut overall_skipped, &mut shared_context);
        }
    } else {
        // Default to parallel execution
        let default_concurrency = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
        info!("⚡ Running tests in parallel with default concurrency: {}", default_concurrency);
        run_tests_parallel_by_index(&mut tests, &test_indices, before_each_hooks, after_each_hooks, &config, &mut overall_failed, &mut overall_skipped, &mut shared_context);
    }
    

    
    // Run after_all hooks
    if !config.skip_hooks.unwrap_or(false) && !after_all_hooks.is_empty() {
        info!("🔄 Running {} after_all hooks", after_all_hooks.len());
        
        // Execute each after_all hook with the same shared context
        for hook in after_all_hooks {
            // Wrap hook execution with panic safety
            let result = catch_unwind(AssertUnwindSafe(|| {
                if let Ok(mut hook_fn) = hook.lock() {
                    hook_fn(&mut shared_context)
                } else {
                    Err(TestError::Message("Failed to acquire hook lock".into()))
                }
            }));
            match result {
                Ok(Ok(())) => {
                    // Hook succeeded
                }
                Ok(Err(e)) => {
                    warn!("⚠️  after_all hook failed: {}", e);
                    // Don't fail the entire test run for after_all hook failures
                }
                Err(panic_info) => {
                    let panic_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_info.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic".to_string()
                    };
                    warn!("💥 after_all hook panicked: {}", panic_msg);
                    // Don't fail the entire test run for after_all hook panics
                }
            }
        }
        
        info!("✅ after_all hooks completed");
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
    
    // Apply shuffling using Fisher-Yates algorithm with seeded PRNG
    if let Some(seed) = config.shuffle_seed {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        // Create a simple seeded PRNG using the hash
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        let mut rng_state = hasher.finish();
        
        // Fisher-Yates shuffle
        for i in (1..indices.len()).rev() {
            // Generate next pseudo-random number
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            let j = (rng_state as usize) % (i + 1);
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
    _shared_context: &mut TestContext,
) {
    let max_workers = config.max_concurrency.unwrap_or_else(|| {
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4)
    });
    
    info!("Running {} tests in parallel with {} workers", test_indices.len(), max_workers);
    
    // Use rayon for true parallel execution
    use rayon::prelude::*;
    

    
    // Create a thread pool with the specified concurrency
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(max_workers)
        .build()
        .expect("Failed to create thread pool");
    

    
    // Extract test functions and create test data before parallel execution to avoid borrowing issues
    let mut test_functions: Vec<Arc<Mutex<TestFn>>> = Vec::new();
    let mut test_data: Vec<(String, Vec<String>, Option<Duration>, TestStatus)> = Vec::new();
    
    for idx in test_indices {
        let test_fn = std::mem::replace(&mut tests[*idx].test_fn, None).unwrap_or_else(|| Box::new(|_| Ok(())));
        test_functions.push(Arc::new(Mutex::new(test_fn)));
        
        // Extract all the data we need from the test
        let test = &tests[*idx];
        test_data.push((
            test.name.clone(),
            test.tags.clone(),
            test.timeout.clone(),
            test.status.clone(),
        ));
    }
    
    // Collect results from parallel execution
    let results: Vec<_> = pool.install(|| {
        test_indices.par_iter().enumerate().map(|(i, &idx)| {
            // Create a new test from the extracted data
            let (name, tags, timeout, status) = &test_data[i];
            let mut test = TestCase {
                name: name.clone(),
                test_fn: None, // Will be set to None since we extracted the function
                tags: tags.clone(),
                timeout: *timeout,
                status: status.clone(),
            };
            
            let test_fn = test_functions[i].clone();
            
            // Clone hooks for this thread
            let before_hooks = before_each_hooks.clone();
            let after_hooks = after_each_hooks.clone();
            
            // Run the test in parallel with the extracted function
            run_single_test_by_index_parallel_with_fn(
                &mut test,
                test_fn,
                &before_hooks,
                &after_hooks,
                config,
            );
            
            (idx, test)
        }).collect()
    });
    
    // Update the original test array with results
    for (idx, test_result) in results {
        tests[idx] = test_result;
        
        // Update counters
        match &tests[idx].status {
            TestStatus::Failed(_) => *overall_failed += 1,
            TestStatus::Skipped => *overall_skipped += 1,
            _ => {}
        }
    }
}

fn run_tests_sequential_by_index(
    tests: &mut [TestCase],
    test_indices: &[usize],
    mut before_each_hooks: Vec<HookFn>,
    mut after_each_hooks: Vec<HookFn>,
    config: &TestConfig,
    overall_failed: &mut usize,
    overall_skipped: &mut usize,
    shared_context: &mut TestContext,
) {
    for &idx in test_indices {
        run_single_test_by_index(
            tests,
            idx,
            &mut before_each_hooks,
            &mut after_each_hooks,
            config,
            overall_failed,
            overall_skipped,
            shared_context,
        );
    }
}

fn run_single_test_by_index(
    tests: &mut [TestCase],
    idx: usize,
    before_each_hooks: &mut [HookFn],
    after_each_hooks: &mut [HookFn],
    config: &TestConfig,
    overall_failed: &mut usize,
    overall_skipped: &mut usize,
    _shared_context: &mut TestContext,
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
    
    // Create test context
    let mut ctx = TestContext::new();
    
    // Copy data from global context to test context
    // This allows tests to access data set by before_all hooks
    let global_ctx = get_global_context();
    if let Ok(map) = global_ctx.lock() {
        for (key, value) in map.iter() {
            ctx.set_data(key, value.clone());
        }
    }
    
    // Run before_each hooks
    if !config.skip_hooks.unwrap_or(false) {
        for hook in before_each_hooks.iter_mut() {
            // Wrap hook execution with panic safety
            let result = catch_unwind(AssertUnwindSafe(|| {
                if let Ok(mut hook_fn) = hook.lock() {
                    hook_fn(&mut ctx)
                } else {
                    Err(TestError::Message("Failed to acquire hook lock".into()))
                }
            }));
            match result {
                Ok(Ok(())) => {
                    // Hook succeeded
                }
                Ok(Err(e)) => {
                    error!("❌ before_each hook failed: {}", e);
                    test.status = TestStatus::Failed(e.clone());
                    *overall_failed += 1;
                    return;
                }
                Err(panic_info) => {
                    let panic_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_info.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic".to_string()
                    };
                    error!("💥 before_each hook panicked: {}", panic_msg);
                    test.status = TestStatus::Failed(TestError::Panicked(panic_msg));
                    *overall_failed += 1;
                    return;
                }
            }
        }
    }
    
    // Run the test
    let test_result = if let Some(timeout) = test.timeout {
        let test_fn = std::mem::replace(&mut test.test_fn, None).unwrap_or_else(|| Box::new(|_| Ok(())));
        run_test_with_timeout(test_fn, &mut ctx, timeout)
    } else {
        let test_fn = std::mem::replace(&mut test.test_fn, None).unwrap_or_else(|| Box::new(|_| Ok(())));
        run_test(test_fn, &mut ctx)
    };
    
    // Run after_each hooks
    if !config.skip_hooks.unwrap_or(false) {
        for hook in after_each_hooks.iter_mut() {
            // Wrap hook execution with panic safety
            let result = catch_unwind(AssertUnwindSafe(|| {
                if let Ok(mut hook_fn) = hook.lock() {
                    hook_fn(&mut ctx)
                } else {
                    Err(TestError::Message("Failed to acquire hook lock".into()))
                }
            }));
            match result {
                Ok(Ok(())) => {
                    // Hook succeeded
                }
                Ok(Err(e)) => {
                    warn!("⚠️  after_each hook failed: {}", e);
                    // Don't fail the test for after_each hook failures
                }
                Err(panic_info) => {
                    let panic_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_info.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic".to_string()
                    };
                    warn!("💥 after_each hook panicked: {}", panic_msg);
                    // Don't fail the test for after_each hook panics
                }
            }
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

fn run_single_test_by_index_parallel_with_fn(
    test: &mut TestCase,
    test_fn: Arc<Mutex<TestFn>>,
    before_each_hooks: &[HookFn],
    after_each_hooks: &[HookFn],
    config: &TestConfig,
) {
    let test_name = &test.name;
    
    info!("🧪 Running test: {}", test_name);
    
    // Check if test should be skipped
    if let Some(ref filter) = config.filter {
        if !test_name.contains(filter) {
            test.status = TestStatus::Skipped;
            info!("⏭️  Test '{}' skipped (filter: {})", test_name, filter);
            return;
        }
    }
    
    // Check tag filtering
    if !config.skip_tags.is_empty() {
        let test_tags = &test.tags;
        if config.skip_tags.iter().any(|skip_tag| test_tags.contains(skip_tag)) {
            test.status = TestStatus::Skipped;
            info!("⏭️  Test '{}' skipped (tags: {:?})", test_name, test_tags);
            return;
        }
    }
    
    let start_time = Instant::now();
    
    // Create test context
    let mut ctx = TestContext::new();
    // Copy data from global context to test context
    // This allows tests to access data set by before_all hooks
    let global_ctx = get_global_context();
    if let Ok(map) = global_ctx.lock() {
        for (key, value) in map.iter() {
            ctx.set_data(key, value.clone());
        }
    }
    
    // Run before_each hooks
    if !config.skip_hooks.unwrap_or(false) {
        for hook in before_each_hooks.iter() {
            // Wrap hook execution with panic safety
            let result = catch_unwind(AssertUnwindSafe(|| {
                if let Ok(mut hook_fn) = hook.lock() {
                    hook_fn(&mut ctx)
                } else {
                    Err(TestError::Message("Failed to acquire hook lock".into()))
                }
            }));
            match result {
                Ok(Ok(())) => {
                    // Hook succeeded
                }
                Ok(Err(e)) => {
                    error!("❌ before_each hook failed: {}", e);
                    test.status = TestStatus::Failed(e.clone());
                    return;
                }
                Err(panic_info) => {
                    let panic_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_info.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic".to_string()
                    };
                    error!("💥 before_each hook panicked: {}", panic_msg);
                    test.status = TestStatus::Failed(TestError::Panicked(panic_msg));
                    return;
                }
            }
        }
    }
    
    // Run the test
    let test_result = if let Some(timeout) = test.timeout {
        if let Ok(mut fn_box) = test_fn.lock() {
            let test_fn = std::mem::replace(&mut *fn_box, Box::new(|_| Ok(())));
            run_test_with_timeout(test_fn, &mut ctx, timeout)
        } else {
            Err(TestError::Message("Failed to acquire test function lock".into()))
        }
    } else {
        if let Ok(mut fn_box) = test_fn.lock() {
            let test_fn = std::mem::replace(&mut *fn_box, Box::new(|_| Ok(())));
            run_test(test_fn, &mut ctx)
        } else {
            Err(TestError::Message("Failed to acquire test function lock".into()))
        }
    };
    
    // Run after_each hooks
    if !config.skip_hooks.unwrap_or(false) {
        for hook in after_each_hooks.iter() {
            // Wrap hook execution with panic safety
            let result = catch_unwind(AssertUnwindSafe(|| {
                if let Ok(mut hook_fn) = hook.lock() {
                    hook_fn(&mut ctx)
                } else {
                    Err(TestError::Message("Failed to acquire hook lock".into()))
                }
            }));
            match result {
                Ok(Ok(())) => {
                    // Hook succeeded
                }
                Ok(Err(e)) => {
                    warn!("⚠️  after_each hook failed: {}", e);
                    // Don't fail the test for after_each hook failures
                }
                Err(panic_info) => {
                    let panic_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_info.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic".to_string()
                    };
                    warn!("💥 after_each hook panicked: {}", panic_msg);
                    // Don't fail the test for after_each hook panics
                }
            }
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
            error!("❌ Test '{}' failed in {:?}: {}", test_name, elapsed, e);
        }
    }
    
    // Clean up Docker if used
    if let Some(ref docker_handle) = ctx.docker_handle {
        cleanup_docker_container(docker_handle);
    }
}

fn run_test<F>(test_fn: F, ctx: &mut TestContext) -> TestResult 
where 
    F: FnOnce(&mut TestContext) -> TestResult
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

fn run_test_with_timeout<F>(test_fn: F, ctx: &mut TestContext, timeout: Duration) -> TestResult 
where 
    F: FnOnce(&mut TestContext) -> TestResult + Send + 'static
{
    // Use the enhanced timeout with configurable strategies
    run_test_with_timeout_enhanced(test_fn, ctx, timeout, &TimeoutConfig::default())
}

fn run_test_with_timeout_enhanced<F>(
    test_fn: F, 
    ctx: &mut TestContext, 
    timeout: Duration, 
    config: &TimeoutConfig
) -> TestResult 
where 
    F: FnOnce(&mut TestContext) -> TestResult + Send + 'static
{
    use std::sync::mpsc;
    
    let (tx, rx) = mpsc::channel();
    
    // Spawn test in worker thread with a new context
    let handle = std::thread::spawn(move || {
        let mut worker_ctx = TestContext::new();
        let result = catch_unwind(AssertUnwindSafe(|| test_fn(&mut worker_ctx)));
        let _ = tx.send((result, worker_ctx));
    });
    
    // Wait for result with timeout based on strategy
    let recv_result = match config.strategy {
        TimeoutStrategy::Simple => {
            // Simple strategy - just wait for the full timeout
            rx.recv_timeout(timeout)
        }
        TimeoutStrategy::Aggressive => {
            // Aggressive strategy - interrupt immediately on timeout
            rx.recv_timeout(timeout)
        }
        TimeoutStrategy::Graceful(cleanup_time) => {
            // Graceful strategy - allow cleanup time
            let main_timeout = timeout.saturating_sub(cleanup_time);
            match rx.recv_timeout(main_timeout) {
                Ok(result) => Ok(result),
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Give cleanup time, then force timeout
                    match rx.recv_timeout(cleanup_time) {
                        Ok(result) => Ok(result),
                        Err(_) => Err(mpsc::RecvTimeoutError::Timeout),
                    }
                }
                Err(e) => Err(e),
            }
        }
    };
    
    match recv_result {
        Ok((Ok(test_result), worker_ctx)) => {
            // Test completed without panic
            match test_result {
                Ok(()) => {
                    // Test passed - copy any data changes back to original context
                    for (key, value) in &worker_ctx.data {
                        if let Some(string_value) = value.downcast_ref::<String>() {
                            ctx.set_data(key, string_value.clone());
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    // Test failed with error
                    Err(e)
                }
            }
        }
        Ok((Err(panic_info), _)) => {
            // Test panicked
            let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown panic".to_string()
            };
            Err(TestError::Panicked(msg))
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // Test timed out - handle based on strategy
            match config.strategy {
                TimeoutStrategy::Simple => {
                    warn!("  ⚠️  Test took longer than {:?} (Simple strategy)", timeout);
                    Err(TestError::Timeout(timeout))
                }
                TimeoutStrategy::Aggressive => {
                    warn!("  ⚠️  Test timed out after {:?} - interrupting", timeout);
                    drop(handle); // This will join the thread when it goes out of scope
                    Err(TestError::Timeout(timeout))
                }
                TimeoutStrategy::Graceful(_) => {
                    warn!("  ⚠️  Test timed out after {:?} - graceful cleanup attempted", timeout);
                    drop(handle);
                    Err(TestError::Timeout(timeout))
                }
            }
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            // Worker thread error
            Err(TestError::Message("worker thread error".into()))
        }
    }
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
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestError::Message(msg) => write!(f, "{}", msg),
            TestError::Panicked(msg) => write!(f, "panicked: {}", msg),
                    TestError::Timeout(duration) => write!(f, "timeout after {:?}", duration),
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

#[derive(Debug, Clone)]
pub enum TimeoutStrategy {
    /// Simple timeout - just report when exceeded
    Simple,
    /// Aggressive timeout - attempt to interrupt the test
    Aggressive,
    /// Graceful timeout - allow cleanup before interruption
    Graceful(Duration),
}

impl Default for TimeoutStrategy {
    fn default() -> Self {
        TimeoutStrategy::Aggressive
    }
}

#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    pub strategy: TimeoutStrategy,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            strategy: TimeoutStrategy::default(),
        }
    }
}

// --- HTML Report Generation ---

fn generate_html_report(tests: &[TestCase], total_time: Duration, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔧 generate_html_report called with {} tests, duration: {:?}, output: {}", tests.len(), total_time, output_path);
    
    // Ensure the target directory exists and create the full path
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    let html_dir = format!("{}/test-reports", target_dir);
    info!("📁 Creating directory: {}", html_dir);
    std::fs::create_dir_all(&html_dir)?;
    info!("✅ Directory created/verified: {}", html_dir);
    
    // Determine the final path - if output_path is absolute, use it directly; otherwise place in target/test-reports/
    let final_path = if std::path::Path::new(output_path).is_absolute() {
        output_path.to_string()
    } else {
        // Extract just the filename from the path and place it in target/test-reports/
        let filename = std::path::Path::new(output_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("test-report.html");
        format!("{}/{}", html_dir, filename)
    };
    info!("📄 Final HTML path: {}", final_path);
    
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
            <h2>📊 Test Results</h2>
            
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
    
    // Write to file in target/test-reports directory
    std::fs::write(&final_path, html)?;
    
    // Log the actual file location for user convenience
    info!("📄 HTML report written to: {}", final_path);
    
    Ok(())
}

// --- Macros ---

/// Macro to create individual test functions that can be run independently
/// This makes the framework compatible with cargo test and existing test libraries
/// Note: Hooks are only executed when using the main test runner, not individual macros
#[macro_export]
macro_rules! test_function {
    ($name:ident, $test_fn:expr) => {
        #[test]
        fn $name() {
            // Initialize logging for individual test runs
            let _ = env_logger::try_init();
            
            // Run the test function
            let result = ($test_fn)(&mut rust_test_harness::TestContext::new());
            
            // Convert result to test outcome
            match result {
                Ok(_) => {
                    // Test passed - no need to panic
                }
                Err(e) => {
                    panic!("❌ Test '{}' failed: {:?}", stringify!($name), e);
                }
            }
        }
    };
}

/// Macro to create individual test functions with custom names
/// Note: Hooks are only executed when using the main test runner, not individual macros
#[macro_export]
macro_rules! test_named {
    ($name:expr, $test_fn:expr) => {
        #[test]
        fn test_named_function() {
            // Initialize logging for individual test runs
            let _ = env_logger::try_init();
            
            // Run the test function
            let result = ($test_fn)(&mut rust_test_harness::TestContext::new());
            
            // Convert result to test outcome
            match result {
                Ok(_) => {
                    // Test passed - no need to panic
                }
                Err(e) => {
                    panic!("❌ Test '{}' failed: {:?}", $name, e);
                }
            }
        }
    };
}

/// Macro to create individual async test functions (for when you add async support)
/// Note: Hooks are only executed when using the main test runner, not individual macros
#[macro_export]
macro_rules! test_async {
    ($name:ident, $test_fn:expr) => {
        #[tokio::test]
        async fn $name() {
            // Initialize logging for individual test runs
            let _ = env_logger::try_init();
            
            // Run the async test function
            let result = ($test_fn)(&mut rust_test_harness::TestContext::new()).await;
            
            // Convert result to test outcome
            match result {
                Ok(_) => {
                    // Test passed - no need to panic
                }
                Err(e) => {
                    panic!("❌ Async test '{}' failed: {:?}", stringify!($name), e);
                }
            }
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
            
            // Run the test function
            let result: rust_test_harness::TestResult = ($test_fn)(&mut rust_test_harness::TestContext::new());
            
            // Convert result to test outcome
            match result {
                Ok(_) => {
                    // Test passed - no need to panic
                }
                Err(e) => {
                    panic!("Test failed: {:?}", e);
                }
            }
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
///     test_case_named!(my_custom_test_name, |ctx| {
///         // Your test logic here
///         Ok(())
///     });
/// }
/// ```
#[macro_export]
macro_rules! test_case_named {
    ($name:ident, $test_fn:expr) => {
        #[test]
        fn $name() {
            // Initialize logging for individual test runs
            let _ = env_logger::try_init();
            
            // Run the test function
            let result: rust_test_harness::TestResult = ($test_fn)(&mut rust_test_harness::TestContext::new());
            
            // Convert result to test outcome
            match result {
                Ok(_) => {
                    // Test passed - no need to panic
                }
                Err(e) => {
                    panic!("Test '{}' failed: {:?}", stringify!($name), e);
                }
            }
        }
    };
}



#[derive(Debug, Clone)]
pub struct ContainerConfig {
    pub image: String,
    pub ports: Vec<(u16, u16)>, // (host_port, container_port)
    pub auto_ports: Vec<u16>, // container ports that should get auto-assigned host ports
    pub env: Vec<(String, String)>,
    pub name: Option<String>,
    pub ready_timeout: Duration,
    pub auto_cleanup: bool, // automatically cleanup on drop/test end
}

#[derive(Debug, Clone)]
pub struct ContainerInfo {
    pub container_id: String,
    pub image: String,
    pub name: Option<String>,
    pub urls: Vec<String>, // URLs for all exposed ports
    pub port_mappings: Vec<(u16, u16)>, // (host_port, container_port) for all ports
    pub auto_cleanup: bool,
}

impl ContainerInfo {
    /// Get the primary URL (first port)
    pub fn primary_url(&self) -> Option<&str> {
        self.urls.first().map(|s| s.as_str())
    }
    
    /// Get host:port for a specific container port
    pub fn url_for_port(&self, container_port: u16) -> Option<String> {
        self.port_mappings.iter()
            .find(|(_, cp)| *cp == container_port)
            .map(|(host_port, _)| format!("localhost:{}", host_port))
    }
    
    /// Get host port for a specific container port
    pub fn host_port_for(&self, container_port: u16) -> Option<u16> {
        self.port_mappings.iter()
            .find(|(_, cp)| *cp == container_port)
            .map(|(host_port, _)| *host_port)
    }
    
    /// Get all exposed ports as a formatted string
    pub fn ports_summary(&self) -> String {
        if self.port_mappings.is_empty() {
            "No ports exposed".to_string()
        } else {
            self.port_mappings.iter()
                .map(|(host_port, container_port)| format!("{}->{}", host_port, container_port))
                .collect::<Vec<_>>()
                .join(", ")
        }
    }
}

impl ContainerConfig {
    pub fn new(image: &str) -> Self {
        Self {
            image: image.to_string(),
            ports: Vec::new(),
            auto_ports: Vec::new(),
            env: Vec::new(),
            name: None,
            ready_timeout: Duration::from_secs(30),
            auto_cleanup: true, // enable auto-cleanup by default
        }
    }
    
    pub fn port(mut self, host_port: u16, container_port: u16) -> Self {
        self.ports.push((host_port, container_port));
        self
    }
    
    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env.push((key.to_string(), value.to_string()));
        self
    }
    
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }
    
    pub fn ready_timeout(mut self, timeout: Duration) -> Self {
        self.ready_timeout = timeout;
        self
    }
    
    /// Add a port that should be automatically assigned an available host port
    pub fn auto_port(mut self, container_port: u16) -> Self {
        self.auto_ports.push(container_port);
        self
    }
    
    /// Disable automatic cleanup (containers will persist after tests)
    pub fn no_auto_cleanup(mut self) -> Self {
        self.auto_cleanup = false;
        self
    }
    
    /// Find an available port on the host
    fn find_available_port() -> Result<u16, Box<dyn std::error::Error + Send + Sync>> {
        use std::net::TcpListener;
        
        // Try to bind to port 0 to let the OS assign an available port
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;
        Ok(addr.port())
    }
    
    /// Start a container with this configuration using Docker API
    pub fn start(&self) -> Result<ContainerInfo, Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(feature = "docker")]
        {
            // Real Docker API implementation - spawn Tokio runtime for async operations
            let runtime = tokio::runtime::Runtime::new()
                .map_err(|e| format!("Failed to create Tokio runtime: {}", e))?;
            
            let result = runtime.block_on(async {
                use bollard::Docker;
                use bollard::models::{ContainerCreateBody, HostConfig, PortBinding, PortMap};
                
                // Connect to Docker daemon
                let docker = Docker::connect_with_local_defaults()
                    .map_err(|e| format!("Failed to connect to Docker: {}", e))?;
                
                // Build port bindings - handle both manual and auto-ports
                let mut port_bindings = PortMap::new();
                let mut auto_port_mappings = Vec::new();
                
                // Handle manual port mappings
                for (host_port, container_port) in &self.ports {
                    let binding = vec![PortBinding {
                        host_ip: Some("127.0.0.1".to_string()),
                        host_port: Some(host_port.to_string()),
                    }];
                    port_bindings.insert(format!("{}/tcp", container_port), Some(binding));
                }
                
                // Handle auto-ports - find available host ports
                for container_port in &self.auto_ports {
                    let host_port = Self::find_available_port()
                        .map_err(|e| format!("Failed to find available port: {}", e))?;
                    
                    let binding = vec![PortBinding {
                        host_ip: Some("127.0.0.1".to_string()),
                        host_port: Some(host_port.to_string()),
                    }];
                    port_bindings.insert(format!("{}/tcp", container_port), Some(binding));
                    
                    // Store the mapping for return
                    auto_port_mappings.push((host_port, *container_port));
                }
                
                // Build environment variables
                let env_vars: Vec<String> = self.env.iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                
                // Create container configuration using the correct bollard 0.19 API
                let container_config = ContainerCreateBody {
                    image: Some(self.image.clone()),
                    env: Some(env_vars),
                    host_config: Some(HostConfig {
                        port_bindings: Some(port_bindings),
                        ..Default::default()
                    }),
                    ..Default::default()
                };
                
                // Create the container
                let container = docker.create_container(None::<bollard::query_parameters::CreateContainerOptions>, container_config)
                    .await
                    .map_err(|e| format!("Failed to create container: {}", e))?;
                let id = container.id;
                
                // Start the container
                docker.start_container(&id, None::<bollard::query_parameters::StartContainerOptions>)
                    .await
                    .map_err(|e| format!("Failed to start container: {}", e))?;
                
                // Wait for container to be ready
                self.wait_for_ready_async(&docker, &id).await?;
                
                // Build port mappings and URLs
                let mut all_port_mappings = self.ports.clone();
                all_port_mappings.extend(auto_port_mappings);
                
                let urls: Vec<String> = all_port_mappings.iter()
                    .map(|(host_port, _)| format!("http://localhost:{}", host_port))
                    .collect();
                
                let container_info = ContainerInfo {
                    container_id: id.clone(),
                    image: self.image.clone(),
                    name: self.name.clone(),
                    urls,
                    port_mappings: all_port_mappings,
                    auto_cleanup: self.auto_cleanup,
                };
                
                Ok::<ContainerInfo, Box<dyn std::error::Error + Send + Sync>>(container_info)
            });
            
            match result {
                Ok(container_info) => {
                    info!("🚀 Started Docker container {} with image {}", container_info.container_id, self.image);
                    
                    // Register for auto-cleanup if enabled
                    if container_info.auto_cleanup {
                        register_container_for_cleanup(&container_info.container_id);
                    }
                    
                    // Log port information
                    if !container_info.port_mappings.is_empty() {
                        info!("🌐 Container {} exposed on ports:", container_info.container_id);
                        for (host_port, container_port) in &container_info.port_mappings {
                            info!("   {} -> {} (http://localhost:{})", host_port, container_port, host_port);
                        }
                    }
                    
                    Ok(container_info)
                }
                Err(e) => Err(e),
            }
        }
        
        #[cfg(not(feature = "docker"))]
        {
            // Mock implementation for when Docker feature is not enabled
            let container_id = format!("mock_{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
            info!("🚀 Starting mock container {} with image {}", container_id, self.image);
            
            // Simulate container startup time
            std::thread::sleep(Duration::from_millis(100));
            
            // Build port mappings and URLs for mock
            let mut all_port_mappings = self.ports.clone();
            let mut auto_port_mappings = Vec::new();
            
            // Generate mock auto-ports
            for container_port in &self.auto_ports {
                let mock_host_port = 10000 + (container_port % 1000); // deterministic mock port
                auto_port_mappings.push((mock_host_port, *container_port));
            }
            all_port_mappings.extend(auto_port_mappings);
            
            let urls: Vec<String> = all_port_mappings.iter()
                .map(|(host_port, _)| format!("http://localhost:{}", host_port))
                .collect();
            
            let container_info = ContainerInfo {
                container_id: container_id.clone(),
                image: self.image.clone(),
                name: self.name.clone(),
                urls,
                port_mappings: all_port_mappings,
                auto_cleanup: self.auto_cleanup,
            };
            
            info!("✅ Mock container {} started successfully", container_id);
            if !container_info.port_mappings.is_empty() {
                info!("🌐 Mock container {} exposed on ports:", container_id);
                for (host_port, container_port) in &container_info.port_mappings {
                    info!("   {} -> {} (http://localhost:{})", host_port, container_port, host_port);
                }
            }
            
            Ok(container_info)
        }
    }
    
    /// Stop a container by ID using Docker API
    pub fn stop(&self, container_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(feature = "docker")]
        {
            // Real Docker API implementation - spawn Tokio runtime for async operations
            let runtime = tokio::runtime::Runtime::new()
                .map_err(|e| format!("Failed to create Tokio runtime: {}", e))?;
            
            let result = runtime.block_on(async {
                use bollard::Docker;
                use tokio::time::{timeout, Duration as TokioDuration};
                
                // Connect to Docker with timeout
                let docker_result = timeout(
                    TokioDuration::from_secs(5), // 5 second timeout for connection
                    Docker::connect_with_local_defaults()
                ).await;
                
                let docker = match docker_result {
                    Ok(Ok(docker)) => docker,
                    Ok(Err(e)) => return Err(format!("Failed to connect to Docker: {}", e).into()),
                    Err(_) => return Err("Docker connection timeout".into()),
                };
                
                // Stop the container with timeout (ignore errors for non-existent containers)
                let stop_result = timeout(
                    TokioDuration::from_secs(10), // 10 second timeout for stop
                    docker.stop_container(container_id, None::<bollard::query_parameters::StopContainerOptions>)
                ).await;
                
                match stop_result {
                    Ok(Ok(())) => info!("🛑 Container {} stopped successfully", container_id),
                    Ok(Err(e)) => {
                        let error_msg = e.to_string();
                        if error_msg.contains("No such container") || error_msg.contains("not found") {
                            info!("ℹ️ Container {} already removed or doesn't exist", container_id);
                        } else {
                            warn!("Failed to stop container {}: {}", container_id, e);
                            // Don't return error for cleanup operations - just log and continue
                        }
                    },
                    Err(_) => {
                        warn!("Container stop timeout for {}", container_id);
                        // Don't return error for cleanup operations - just log and continue
                    },
                }
                
                // Remove the container with timeout (ignore errors for non-existent containers)
                let remove_result = timeout(
                    TokioDuration::from_secs(10), // 10 second timeout for remove
                    docker.remove_container(container_id, None::<bollard::query_parameters::RemoveContainerOptions>)
                ).await;
                
                match remove_result {
                    Ok(Ok(())) => info!("🗑️ Container {} removed successfully", container_id),
                    Ok(Err(e)) => {
                        let error_msg = e.to_string();
                        if error_msg.contains("No such container") || error_msg.contains("not found") {
                            info!("ℹ️ Container {} already removed or doesn't exist", container_id);
                        } else {
                            warn!("Failed to remove container {}: {}", container_id, e);
                            // Don't return error for cleanup operations - just log and continue
                        }
                    },
                    Err(_) => {
                        warn!("Container remove timeout for {}", container_id);
                        // Don't return error for cleanup operations - just log and continue
                    },
                }
                
                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            });
            
            match result {
                Ok(()) => {
                    info!("🛑 Stopped and removed Docker container {}", container_id);
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        
        #[cfg(not(feature = "docker"))]
        {
            // Mock implementation
            info!("🛑 Stopping mock container {}", container_id);
            std::thread::sleep(Duration::from_millis(50));
            info!("✅ Mock container {} stopped successfully", container_id);
            Ok(())
        }
    }
    
    #[cfg(feature = "docker")]
    async fn wait_for_ready_async(&self, docker: &bollard::Docker, container_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::time::{sleep, Duration as TokioDuration};
        
        // Wait for container to be ready by checking its status
        let start_time = std::time::Instant::now();
        let timeout = self.ready_timeout;
        
        loop {
            if start_time.elapsed() > timeout {
                return Err("Container readiness timeout".into());
            }
            
            // Inspect container to check status
            let inspect_result = docker.inspect_container(container_id, None::<bollard::query_parameters::InspectContainerOptions>).await;
            if let Ok(container_info) = inspect_result {
                if let Some(state) = container_info.state {
                    if let Some(running) = state.running {
                        if running {
                            if let Some(health) = state.health {
                                if let Some(status) = health.status {
                                    if status.to_string() == "healthy" {
                                        info!("✅ Container {} is healthy and ready", container_id);
                                        return Ok(());
                                    }
                                }
                            } else {
                                // No health check, assume ready if running
                                info!("✅ Container {} is running and ready", container_id);
                                return Ok(());
                            }
                        }
                    }
                }
            }
            
            // Wait a bit before checking again
            sleep(TokioDuration::from_millis(500)).await;
        }
    }
}

// --- Hook execution functions for individual tests ---

/// Execute before_all hooks for individual test functions
pub fn execute_before_all_hooks() -> Result<(), TestError> {
    THREAD_BEFORE_ALL.with(|hooks| {
        let mut hooks = hooks.borrow_mut();
        for hook in hooks.iter_mut() {
            if let Ok(mut hook_fn) = hook.lock() {
                hook_fn(&mut TestContext::new())?;
            }
        }
        Ok(())
    })
}

/// Execute before_each hooks for individual test functions
pub fn execute_before_each_hooks() -> Result<(), TestError> {
    THREAD_BEFORE_EACH.with(|hooks| {
        let mut hooks = hooks.borrow_mut();
        for hook in hooks.iter_mut() {
            if let Ok(mut hook_fn) = hook.lock() {
                hook_fn(&mut TestContext::new())?;
            }
        }
        Ok(())
    })
}

/// Execute after_each hooks for individual test functions
pub fn execute_after_each_hooks() -> Result<(), TestError> {
    THREAD_AFTER_EACH.with(|hooks| {
        let mut hooks = hooks.borrow_mut();
        for hook in hooks.iter_mut() {
            if let Ok(mut hook_fn) = hook.lock() {
                let _ = hook_fn(&mut TestContext::new());
            }
        }
        Ok(())
    })
}

/// Execute after_all hooks for individual test functions
pub fn execute_after_all_hooks() -> Result<(), TestError> {
    THREAD_AFTER_ALL.with(|hooks| {
        let mut hooks = hooks.borrow_mut();
        for hook in hooks.iter_mut() {
            if let Ok(mut hook_fn) = hook.lock() {
                let _ = hook_fn(&mut TestContext::new());
            }
        }
        Ok(())
    })
}

// --- Convenience function for running tests ---

pub fn run_all() -> i32 {
    run_tests()
}

 
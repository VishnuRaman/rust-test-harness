use once_cell::sync::Lazy;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use log::{info, warn, error};
use std::sync::MutexGuard;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::thread;
use std::sync::mpsc;
use std::env;


// --- Error types ---

#[derive(Debug)]
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

pub type TestResult = Result<(), TestError>;



// --- Test context ---

pub struct TestContext {
    pub docker: Option<ContainerHandle>,
}

impl TestContext {
    pub fn new() -> Self { 
        Self { 
            docker: None,
        } 
    }
}

// --- Closure-based test and hook functions ---

pub type TestFn = Box<dyn FnMut(&mut TestContext) -> TestResult + Send + 'static>;
pub type HookFn = Box<dyn FnMut(&mut TestContext) -> TestResult + Send + 'static>;

// --- Docker options with builder pattern ---

#[derive(Clone, Debug)]
pub struct DockerRunOptions {
    pub image: &'static str,
    pub env: Vec<(&'static str, &'static str)>,
    pub ports: Vec<(u16, u16)>, // (host:container)
    pub args: Vec<&'static str>,
    pub ready_timeout: Duration,
    pub name: Option<&'static str>,
    pub labels: Vec<(&'static str, &'static str)>,
    pub readiness: Readiness,
}

#[derive(Clone, Debug)]
pub enum Readiness {
    Running,
    PortOpen(u16),
    HttpOk { host: String, port: u16, path: String },
    HealthCheck,
}

impl Default for Readiness {
    fn default() -> Self {
        Readiness::Running
    }
}

impl Default for DockerRunOptions {
    fn default() -> Self {
        Self {
            image: "alpine:latest",
            env: vec![],
            ports: vec![],
            args: vec![],
            ready_timeout: Duration::from_secs(15),
            name: None,
            labels: vec![],
            readiness: Readiness::Running,
        }
    }
}

impl DockerRunOptions {
    pub fn new(image: &'static str) -> Self {
        Self {
            image,
            ..Default::default()
        }
    }

    pub fn env(mut self, key: &'static str, value: &'static str) -> Self {
        self.env.push((key, value));
        self
    }

    pub fn port(mut self, host: u16, container: u16) -> Self {
        self.ports.push((host, container));
        self
    }

    pub fn arg(mut self, arg: &'static str) -> Self {
        self.args.push(arg);
        self
    }

    pub fn name(mut self, name: &'static str) -> Self {
        self.name = Some(name);
        self
    }

    pub fn label(mut self, key: &'static str, value: &'static str) -> Self {
        self.labels.push((key, value));
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

// --- Test case with tags and timeout ---

pub struct TestCase {
    pub name: &'static str,
    pub test_fn: TestFn,
    pub docker: Option<DockerRunOptions>,
    pub tags: Vec<&'static str>,
    pub timeout: Option<Duration>,
}

// --- Mutex recovery utility ---

pub fn lock_or_recover<T>(mutex: &Mutex<T>) -> MutexGuard<T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            warn!("Mutex was poisoned, recovering...");
            poisoned.into_inner()
        }
    }
}

// --- Static collections ---

pub static BEFORE_ALL: Lazy<Mutex<Vec<HookFn>>> = Lazy::new(|| Mutex::new(Vec::new()));
pub static BEFORE_EACH: Lazy<Mutex<Vec<HookFn>>> = Lazy::new(|| Mutex::new(Vec::new()));
pub static AFTER_EACH: Lazy<Mutex<Vec<HookFn>>> = Lazy::new(|| Mutex::new(Vec::new()));
pub static AFTER_ALL: Lazy<Mutex<Vec<HookFn>>> = Lazy::new(|| Mutex::new(Vec::new()));
pub static TESTS: Lazy<Mutex<Vec<TestCase>>> = Lazy::new(|| Mutex::new(Vec::new()));

// --- Public registration API ---

pub fn before_all<F>(f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    lock_or_recover(&BEFORE_ALL).push(Box::new(f));
}

pub fn before_each<F>(f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    lock_or_recover(&BEFORE_EACH).push(Box::new(f));
}

pub fn after_each<F>(f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    lock_or_recover(&AFTER_EACH).push(Box::new(f));
}

pub fn after_all<F>(f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    lock_or_recover(&AFTER_ALL).push(Box::new(f));
}

pub fn test<F>(name: &'static str, f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    lock_or_recover(&TESTS).push(TestCase { 
        name, 
        test_fn: Box::new(f), 
        docker: None,
        tags: vec![],
        timeout: None,
    });
}

pub fn test_with_docker<F>(name: &'static str, opts: DockerRunOptions, f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    lock_or_recover(&TESTS).push(TestCase { 
        name, 
        test_fn: Box::new(f), 
        docker: Some(opts),
        tags: vec![],
        timeout: None,
    });
}

pub fn test_with_tags<F>(name: &'static str, tags: Vec<&'static str>, f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    lock_or_recover(&TESTS).push(TestCase { 
        name, 
        test_fn: Box::new(f), 
        docker: None,
        tags,
        timeout: None,
    });
}

pub fn test_with_docker_and_tags<F>(name: &'static str, opts: DockerRunOptions, tags: Vec<&'static str>, f: F) 
where 
    F: FnMut(&mut TestContext) -> TestResult + Send + 'static 
{
    lock_or_recover(&TESTS).push(TestCase { 
        name, 
        test_fn: Box::new(f), 
        docker: Some(opts),
        tags,
        timeout: None,
    });
}

// --- Container handle with Drop implementation ---

#[derive(Clone, Debug)]
pub struct ContainerHandle { 
    pub id: String 
}

impl ContainerHandle {
    pub fn stop(&mut self) -> Result<(), String> {
        let output = Command::new("docker")
            .arg("stop")
            .arg(&self.id)
            .output()
            .map_err(|e| format!("Failed to stop container: {}", e))?;
        
        if !output.status.success() {
            return Err(format!("docker stop failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        Ok(())
    }
    
    /// Get the host port that this container is using
    /// This is useful for connecting to the container from the host
    pub fn get_host_port(&self) -> Option<u16> {
        // Parse the port from docker inspect output
        let output = Command::new("docker")
            .args(["inspect", "-f", "{{range $k, $v := .NetworkSettings.Ports}}{{$k}}{{end}}", &self.id])
            .output();
        
        if let Ok(output) = output {
            if output.status.success() {
                let ports_str = String::from_utf8_lossy(&output.stdout);
                // Parse the port mapping (e.g., "27017/tcp->0.0.0.0:27018")
                if let Some(port_part) = ports_str.split("->").nth(1) {
                    if let Some(host_port) = port_part.split(':').nth(1) {
                        if let Some(port_str) = host_port.split('/').next() {
                            if let Ok(port) = port_str.parse::<u16>() {
                                return Some(port);
                            }
                        }
                    }
                }
            }
        }
        None
    }
    
    /// Get the connection information for this container
    /// Returns a tuple of (host, port) for easy connection string creation
    pub fn get_connection_info(&self) -> Option<(String, u16)> {
        self.get_host_port().map(|port| ("localhost".to_string(), port))
    }
    
    /// Get a generic connection string for this container
    /// Useful for any service that needs host:port format
    pub fn get_connection_string(&self, protocol: &str) -> Option<String> {
        self.get_host_port().map(|port| format!("{}://localhost:{}", protocol, port))
    }
}

impl Drop for ContainerHandle {
    fn drop(&mut self) {
        if let Err(e) = self.stop() {
            warn!("Failed to stop container {} during drop: {}", self.id, e);
        }
    }
}

impl ContainerHandle {
    // Force remove container even if stop fails
    pub fn force_remove(&mut self) -> Result<(), String> {
        let output = Command::new("docker")
            .arg("rm")
            .arg("-f")
            .arg(&self.id)
            .output()
            .map_err(|e| format!("Failed to force remove container: {}", e))?;
        
        if !output.status.success() {
            return Err(format!("docker rm -f failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        Ok(())
    }
}

// --- Docker utilities ---

fn docker_available() -> Result<bool, String> {
    let output = Command::new("docker")
        .arg("version")
        .output()
        .map_err(|e| format!("docker command failed: {}", e))?;
    
    if output.status.success() {
        Ok(true)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("command not found") {
            Ok(false)
        } else {
            Ok(false)
        }
    }
}

fn start_docker(opts: &DockerRunOptions) -> Result<ContainerHandle, String> {
    if !docker_available()? {
        return Err("Docker is not available on PATH or not running".into());
    }

    let mut args: Vec<String> = vec!["run".into(), "-d".into(), "--rm".into()];
    
    for (k, v) in &opts.env {
        args.push("-e".into());
        args.push(format!("{}={}", k, v));
    }
    
    for (host, container) in &opts.ports {
        args.push("-p".into());
        args.push(format!("{}:{}", host, container));
    }
    
    if let Some(name) = opts.name {
        args.push("--name".into());
        args.push(name.into());
    }
    
    for (k, v) in &opts.labels {
        args.push("-l".into());
        args.push(format!("{}={}", k, v));
    }
    
    for a in &opts.args {
        args.push((*a).into());
    }
    
    args.push(opts.image.into());

    let output = Command::new("docker")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to start docker container: {}", e))?;
        
    if !output.status.success() {
        return Err(format!("docker run failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Wait for readiness based on strategy
    let start = Instant::now();
    while start.elapsed() < opts.ready_timeout {
        match &opts.readiness {
            Readiness::Running => {
                if is_container_running(&id)? {
                    break;
                }
            }
            Readiness::PortOpen(port) => {
                if is_port_open("127.0.0.1", *port)? {
                    // Add a small delay to ensure the service is actually ready to accept connections
                    thread::sleep(Duration::from_millis(2000));
                    break;
                }
            }
            Readiness::HttpOk { host, port, path } => {
                if is_http_ok(host, *port, path)? {
                    break;
                }
            }
            Readiness::HealthCheck => {
                if is_health_check_ok(&id)? {
                    break;
                }
            }

        }
        thread::sleep(Duration::from_millis(100));
    }

    if start.elapsed() >= opts.ready_timeout {
        return Err(format!("Container did not become ready within {:?}", opts.ready_timeout));
    }

    Ok(ContainerHandle { id })
}

fn is_container_running(id: &str) -> Result<bool, String> {
    let output = Command::new("docker")
        .args(["inspect", "-f", "{{.State.Running}}", id])
        .output()
        .map_err(|e| format!("docker inspect failed: {}", e))?;
    
    if output.status.success() {
        let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(status == "true")
    } else {
        Ok(false)
    }
}

fn is_port_open(host: &str, port: u16) -> Result<bool, String> {
    use std::net::TcpStream;
    match TcpStream::connect(format!("{}:{}", host, port)) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

fn is_http_ok(host: &str, port: u16, path: &str) -> Result<bool, String> {
    use std::net::TcpStream;
    use std::io::{Read, Write};
    
    let mut stream = TcpStream::connect(format!("{}:{}", host, port))
        .map_err(|e| format!("Failed to connect: {}", e))?;
    
    let request = format!("GET {} HTTP/1.1\r\nHost: {}:{}\r\nConnection: close\r\n\r\n", path, host, port);
    stream.write_all(request.as_bytes())
        .map_err(|e| format!("Failed to write request: {}", e))?;
    
    let mut response = String::new();
    stream.read_to_string(&mut response)
        .map_err(|e| format!("Failed to read response: {}", e))?;
    
    Ok(response.contains("HTTP/1.1 200"))
}

fn is_health_check_ok(id: &str) -> Result<bool, String> {
    let output = Command::new("docker")
        .args(["inspect", "-f", "{{.State.Health.Status}}", id])
        .output()
        .map_err(|e| format!("docker inspect failed: {}", e))?;
    
    if output.status.success() {
        let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(status == "healthy")
    } else {
        Ok(false)
    }
}

// --- Test execution with timeout ---

fn run_with_timeout<F>(test_fn: F, timeout: Duration) -> TestResult 
where 
    F: FnOnce() -> TestResult + Send + 'static 
{
    let (tx, rx) = mpsc::channel();
    
    thread::spawn(move || {
        let result = test_fn();
        let _ = tx.send(result);
    });
    
    match rx.recv_timeout(timeout) {
        Ok(result) => result,
        Err(mpsc::RecvTimeoutError::Timeout) => Err(TestError::Timeout(timeout)),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(TestError::Message("Test thread disconnected".into())),
    }
}

// --- Test filtering and configuration ---

#[derive(Debug, Clone)]
pub struct TestConfig {
    pub filter: Option<String>,
    pub skip_tags: Vec<String>,
    pub max_concurrency: Option<usize>,
    pub shuffle_seed: Option<u64>,
    pub color: Option<bool>,
    pub junit_xml: Option<String>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            filter: env::var("TEST_FILTER").ok(),
            skip_tags: env::var("TEST_SKIP_TAGS")
                .ok()
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
            max_concurrency: env::var("TEST_MAX_CONCURRENCY")
                .ok()
                .and_then(|s| s.parse().ok()),
            shuffle_seed: env::var("TEST_SHUFFLE")
                .ok()
                .and_then(|s| s.parse().ok()),
            color: env::var("TEST_COLOR")
                .ok()
                .and_then(|s| s.parse().ok())
                .or_else(|| {
                    if env::var("NO_COLOR").is_ok() {
                        Some(false)
                    } else {
                        Some(atty::is(atty::Stream::Stdout))
                    }
                }),
            junit_xml: env::var("TEST_JUNIT_XML").ok(),
        }
    }
}

// --- Test runner ---

pub struct TestRunner {
    config: TestConfig,
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            config: TestConfig::default(),
        }
    }

    pub fn with_config(config: TestConfig) -> Self {
        Self { config }
    }

    pub fn run(&self) -> i32 {
        let mut overall_failed = 0usize;
        let mut overall_skipped = 0usize;

        let mut global_ctx = TestContext::new();

        // Print header
        self.print_header();

        // beforeAll
        for hook in lock_or_recover(&BEFORE_ALL).iter_mut() {
            if let Err(e) = self.run_hook(hook, &mut global_ctx, "beforeAll") {
                error!("[beforeAll] failed: {}", e);
                overall_failed += 1;
            }
        }

        // Get and filter tests - we need to work with references since TestCase can't be cloned
        let test_count;
        let mut test_indices: Vec<usize>;
        
        {
            let tests = lock_or_recover(&TESTS);
            test_count = tests.len();
            test_indices = (0..test_count).collect();
        }
        
        self.filter_and_sort_test_indices(&mut test_indices);

        info!("Running {} test(s)...", test_count);

        if let Some(_max_concurrency) = self.config.max_concurrency {
            self.run_tests_parallel_by_index(&mut test_indices, &mut overall_failed, &mut overall_skipped);
        } else {
            self.run_tests_sequential_by_index(&mut test_indices, &mut overall_failed, &mut overall_skipped);
        }

        // afterAll
        for hook in lock_or_recover(&AFTER_ALL).iter_mut() {
            if let Err(e) = self.run_hook(hook, &mut global_ctx, "afterAll") {
                error!("[afterAll] failed: {}", e);
                overall_failed += 1;
            }
        }

        self.print_summary(overall_failed, overall_skipped);
        
        if overall_failed > 0 { 1 } else { 0 }
    }

    fn print_header(&self) {
        if let Some(filter) = &self.config.filter {
            info!("Test filter: {}", filter);
        }
        if !self.config.skip_tags.is_empty() {
            info!("Skip tags: {}", self.config.skip_tags.join(", "));
        }
        if let Some(concurrency) = self.config.max_concurrency {
            info!("Max concurrency: {}", concurrency);
        }
        if self.config.shuffle_seed.is_some() {
            info!("Shuffle enabled");
        }
    }

    fn filter_and_sort_test_indices(&self, test_indices: &mut Vec<usize>) {
        let tests = lock_or_recover(&TESTS);
        
        // Apply filter
        if let Some(filter) = &self.config.filter {
            test_indices.retain(|&idx| {
                let tc = &tests[idx];
                tc.name.to_lowercase().contains(&filter.to_lowercase())
            });
        }

        // Apply tag filtering
        if !self.config.skip_tags.is_empty() {
            test_indices.retain(|&idx| {
                let tc = &tests[idx];
                !tc.tags.iter().any(|tag| {
                    self.config.skip_tags.iter().any(|skip_tag| {
                        tag.to_lowercase() == skip_tag.to_lowercase()
                    })
                })
            });
        }

        // Sort or shuffle
        if let Some(seed) = self.config.shuffle_seed {
            self.shuffle_test_indices(test_indices, seed);
        } else {
            test_indices.sort_by(|&a, &b| tests[a].name.cmp(&tests[b].name));
        }
    }

    fn shuffle_test_indices(&self, test_indices: &mut Vec<usize>, seed: u64) {
        // Simple LCG PRNG
        let mut rng = seed;
        for i in (1..test_indices.len()).rev() {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            let j = (rng >> 16) as usize % (i + 1);
            test_indices.swap(i, j);
        }
    }

    fn run_tests_sequential_by_index(&self, test_indices: &mut Vec<usize>, overall_failed: &mut usize, overall_skipped: &mut usize) {
        for &idx in test_indices.iter() {
            self.run_single_test_by_index(idx, overall_failed, overall_skipped);
        }
    }

    fn run_tests_parallel_by_index(&self, test_indices: &mut Vec<usize>, overall_failed: &mut usize, overall_skipped: &mut usize) {
        // For now, fall back to sequential execution since TestCase can't be cloned
        // TODO: Implement proper parallel execution with test context handling
        warn!("Parallel execution not yet implemented, falling back to sequential");
        self.run_tests_sequential_by_index(test_indices, overall_failed, overall_skipped);
    }

    fn run_single_test_by_index(&self, idx: usize, overall_failed: &mut usize, overall_skipped: &mut usize) {
        let mut tests = lock_or_recover(&TESTS);
        let tc = &mut tests[idx];
        
        info!("- {}", tc.name);
        let mut ctx = TestContext::new();

        // beforeEach
        for hook in lock_or_recover(&BEFORE_EACH).iter_mut() {
            if let Err(e) = self.run_hook(hook, &mut ctx, "beforeEach") {
                error!("  [beforeEach] failed: {}", e);
            }
        }

        let mut started_container: Option<ContainerHandle> = None;
        let mut skipped = false;

        if let Some(opts) = tc.docker.as_ref() {
            match start_docker(opts) {
                Ok(handle) => {
                    ctx.docker = Some(handle.clone());
                    started_container = Some(handle);
                    info!("  Docker started: {}", opts.image);
                }
                Err(err) => {
                    warn!("  SKIPPED (docker): {}", err);
                    skipped = true;
                    *overall_skipped += 1;
                }
            }
        }

        if !skipped {
            let start = Instant::now();
            let result = if let Some(_timeout) = tc.timeout {
                // For timeout tests, we need to handle the closure differently
                // Since we can't easily share the closure between threads, we'll just run it directly
                // and let the test framework handle timeouts at a higher level
                warn!("  Timeout not yet implemented for this test");
                self.run_test(&mut tc.test_fn, &mut ctx)
            } else {
                self.run_test(&mut tc.test_fn, &mut ctx)
            };
            let elapsed = start.elapsed();
            
            match result {
                Ok(()) => self.print_result("PASSED", elapsed, None),
                Err(e) => {
                    self.print_result("FAILED", elapsed, Some(&e));
                    *overall_failed += 1;
                }
            }
        }

        // afterEach - always run, even if test panicked
        for hook in lock_or_recover(&AFTER_EACH).iter_mut() {
            if let Err(e) = self.run_hook(hook, &mut ctx, "afterEach") {
                error!("  [afterEach] failed: {}", e);
            }
        }

        if let Some(mut handle) = started_container {
            // Try to stop gracefully first
            if let Err(e) = handle.stop() {
                warn!("  Failed to stop Docker gracefully: {}", e);
                // Force remove if stop fails
                if let Err(e) = handle.force_remove() {
                    warn!("  Failed to force remove Docker container: {}", e);
                } else {
                    info!("  Docker container force removed");
                }
            } else {
                info!("  Docker stopped");
            }
        }
    }

    fn run_test<F>(&self, test_fn: &mut F, ctx: &mut TestContext) -> TestResult 
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

    fn run_hook<F>(&self, hook: &mut F, ctx: &mut TestContext, _hook_name: &str) -> TestResult 
    where 
        F: FnMut(&mut TestContext) -> TestResult 
    {
        catch_unwind(AssertUnwindSafe(|| hook(ctx))).unwrap_or_else(|panic_info| {
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

    fn print_result(&self, status: &str, elapsed: Duration, error: Option<&TestError>) {
        let color = self.config.color.unwrap_or(false);
        let elapsed_ms = elapsed.as_millis();
        
        if color {
            match status {
                "PASSED" => info!("  \x1b[32m{}\x1b[0m in {} ms", status, elapsed_ms),
                "FAILED" => {
                    error!("  \x1b[31m{}\x1b[0m in {} ms", status, elapsed_ms);
                    if let Some(e) = error {
                        error!("    {}", e);
                    }
                }
                "SKIPPED" => warn!("  \x1b[33m{}\x1b[0m in {} ms", status, elapsed_ms),
                _ => info!("  {} in {} ms", status, elapsed_ms),
            }
        } else {
            match status {
                "PASSED" => info!("  {} in {} ms", status, elapsed_ms),
                "FAILED" => {
                    error!("  {} in {} ms", status, elapsed_ms);
                    if let Some(e) = error {
                        error!("    {}", e);
                    }
                }
                "SKIPPED" => warn!("  {} in {} ms", status, elapsed_ms),
                _ => info!("  {} in {} ms", status, elapsed_ms),
            }
        }
    }

    fn print_summary(&self, failed: usize, skipped: usize) {
        info!("Summary: failures={}, skipped={}", failed, skipped);
    }
}

// --- Port utilities ---

// Find an available port starting from the given port
fn find_available_port(start_port: u16) -> u16 {
    use std::net::TcpListener;
    
    for port in start_port..start_port + 100 {
        if TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok() {
            return port;
        }
    }
    start_port // Fallback to original port if none found
}

// --- Docker utilities ---

impl DockerRunOptions {
    /// Create a new Docker run options with an automatically assigned available port
    /// This is useful for avoiding port conflicts when running multiple containers
    pub fn with_auto_port(mut self, container_port: u16, start_host_port: u16) -> Self {
        let available_port = find_available_port(start_host_port);
        self.ports.push((available_port, container_port));
        self
    }
    
    /// Create a new Docker run options with an automatically assigned available port
    /// and set the readiness check to that port
    pub fn with_auto_port_and_readiness(mut self, container_port: u16, start_host_port: u16) -> Self {
        let available_port = find_available_port(start_host_port);
        self.ports.push((available_port, container_port));
        self.readiness = Readiness::PortOpen(available_port);
        self
    }
    
    /// Get the host port that was assigned (useful for connecting to the container)
    pub fn get_host_port(&self) -> Option<u16> {
        self.ports.first().map(|(host, _)| *host)
    }
}

// --- Convenience function for running tests ---

pub fn run_all() -> i32 {
    TestRunner::new().run()
} 
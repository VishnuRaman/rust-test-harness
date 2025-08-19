use rust_test_harness::{before_all, before_each, after_each, after_all, test, test_with_docker, DockerRunOptions, Readiness, run_all};

fn register_examples() {
    before_all(|_| { 
        log::info!("[beforeAll] global setup"); 
        Ok(()) 
    });
    
    after_all(|_| { 
        log::info!("[afterAll] global teardown"); 
        Ok(()) 
    });
    
    before_each(|_| { 
        log::info!("  [beforeEach]"); 
        Ok(()) 
    });
    
    after_each(|_| { 
        log::info!("  [afterEach]"); 
        Ok(()) 
    });

    test("addition works", |_| {
        if 2 + 2 == 4 { 
            Ok(()) 
        } else { 
            Err("math broke".into()) 
        }
    });

    test("basic assertion", |_| {
        assert_eq!(1 + 1, 2);
        Ok(())
    });

    // Docker-backed test (will be skipped if Docker is unavailable)
    let docker_opts = DockerRunOptions::new("nginx:alpine")
        .port(8080, 80)
        .ready_timeout(std::time::Duration::from_secs(10))
        .readiness(Readiness::PortOpen(80));
        
    test_with_docker("nginx container starts", docker_opts, |ctx| {
        if ctx.docker.is_some() { 
            Ok(()) 
        } else { 
            Err("no docker".into()) 
        }
    });
}

fn main() {
    // Initialize logger (configure level via RUST_LOG, e.g., RUST_LOG=info)
    env_logger::init();

    // Register hooks and tests
    register_examples();

    let code = run_all();
    std::process::exit(code);
}

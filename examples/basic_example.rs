use connect::{TestSetup, print_example_header, print_success};

#[tokio::main]
async fn main() {
    print_example_header("TiDB Basic Connection Test");
    
    // Use the shared example setup
    let mut setup = match TestSetup::new() {
        Ok(setup) => setup,
        Err(e) => {
            eprintln!("Failed to initialize example setup: {}", e);
            std::process::exit(1);
        }
    };
    
    // Run the state machine with job monitoring
    if let Err(e) = setup.run_with_job_monitoring().await {
        eprintln!("Example failed: {}", e);
        std::process::exit(1);
    }
    
    print_success("Basic connection test completed successfully!");
} 
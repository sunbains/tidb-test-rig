use connect::{CommonArgsSetup, print_example_header, print_success, print_error_and_exit, ErrorContext};
use tracing::{info, debug, warn, error, instrument};
use std::time::Instant;

#[instrument(skip_all)]
async fn perform_logged_operation(operation_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    
    info!("Starting operation: {}", operation_name);
    debug!("Operation details: {}", operation_name);
    
    // Simulate some work
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Log performance metric
    connect::log_performance_metric(operation_name, start_time.elapsed());
    
    // Log memory usage (simulated)
    connect::log_memory_usage(operation_name, 1024 * 1024); // 1MB
    
    info!("Completed operation: {}", operation_name);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_example_header("TiDB Logging Example");
    
    // Use the shared example setup
    let mut setup = CommonArgsSetup::new()?;
    
    // Log different levels
    info!("Starting TiDB logging example");
    debug!("Connection parameters: host={}, user={}, database={:?}", 
           setup.args.host, setup.args.user, setup.args.database);
    warn!("This is a warning message");
    error!("This is an error message (for demonstration)");
    
    // Perform some logged operations
    perform_logged_operation("database_connection").await?;
    perform_logged_operation("query_execution").await?;
    perform_logged_operation("data_processing").await?;
    
    // Run the state machine with error context
    let start_time = Instant::now();
    match setup.state_machine.run().await {
        Ok(_) => {
            let duration = start_time.elapsed();
            connect::log_performance_metric("state_machine_execution", duration);
            
            info!("Logging example completed successfully");
            print_success("Logging example completed successfully!");
            println!("Check the logs for detailed information.");
        }
        Err(e) => {
            let error_context = ErrorContext::new("state_machine_execution", "Failed to complete state machine");
            error_context.log_error(&*e);
            
            print_error_and_exit("Logging example failed", &*e);
        }
    }
    
    Ok(())
} 
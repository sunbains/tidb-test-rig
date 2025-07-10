use clap::Parser;
use connect::state_machine::{StateMachine, State};
use connect::{InitialHandler, ParsingConfigHandler, ConnectingHandler, TestingConnectionHandler, VerifyingDatabaseHandler, GettingVersionHandler};
use connect::{log_performance_metric, log_memory_usage, ErrorContext};
use tracing::{info, debug, warn, error, instrument};
use std::time::Instant;
use connect::cli::CommonArgs;

#[instrument(skip_all)]
async fn perform_logged_operation(operation_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    
    info!("Starting operation: {}", operation_name);
    debug!("Operation details: {}", operation_name);
    
    // Simulate some work
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Log performance metric
    log_performance_metric(operation_name, start_time.elapsed());
    
    // Log memory usage (simulated)
    log_memory_usage(operation_name, 1024 * 1024); // 1MB
    
    info!("Completed operation: {}", operation_name);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("TiDB Logging Example");
    println!("=====================");
    
    // Parse command line arguments
    let args = CommonArgs::parse();
    
    // Initialize logging with CLI arguments
    args.init_logging()?;
    
    // Print connection info
    args.print_connection_info();
    
    // Get connection info
    let (host, user, password, database) = args.get_connection_info()?;
    
    // Log different levels
    info!("Starting TiDB logging example");
    debug!("Connection parameters: host={}, user={}, database={:?}", host, user, database);
    warn!("This is a warning message");
    error!("This is an error message (for demonstration)");
    
    // Perform some logged operations
    perform_logged_operation("database_connection").await?;
    perform_logged_operation("query_execution").await?;
    perform_logged_operation("data_processing").await?;
    
    // Create and configure the state machine
    let mut state_machine = StateMachine::new();
    
    // Register state handlers
    state_machine.register_handler(State::Initial, Box::new(InitialHandler));
    state_machine.register_handler(
        State::ParsingConfig,
        Box::new(ParsingConfigHandler::new(
            host,
            user,
            password,
            database
        ))
    );
    state_machine.register_handler(State::Connecting, Box::new(ConnectingHandler));
    state_machine.register_handler(State::TestingConnection, Box::new(TestingConnectionHandler));
    state_machine.register_handler(State::VerifyingDatabase, Box::new(VerifyingDatabaseHandler));
    state_machine.register_handler(State::GettingVersion, Box::new(GettingVersionHandler));
    
    // Run the state machine with error context
    let start_time = Instant::now();
    match state_machine.run().await {
        Ok(_) => {
            let duration = start_time.elapsed();
            log_performance_metric("state_machine_execution", duration);
            
            info!("Logging example completed successfully");
            println!("\n✅ Logging example completed successfully!");
            println!("Check the logs for detailed information.");
        }
        Err(e) => {
            let error_context = ErrorContext::new("state_machine_execution", "Failed to complete state machine");
            error_context.log_error(&*e);
            
            eprintln!("\n❌ Logging example failed: {e}");
            std::process::exit(1);
        }
    }
    
    Ok(())
} 
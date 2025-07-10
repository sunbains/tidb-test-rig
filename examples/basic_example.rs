use std::process;
use connect::state_machine::{StateMachine, State};
use connect::{InitialHandler, ParsingConfigHandler, ConnectingHandler, TestingConnectionHandler, VerifyingDatabaseHandler, GettingVersionHandler};
use connect::{JobMonitor, parse_args};

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let args = parse_args().expect("Failed to parse arguments");
    
    // Initialize logging
    args.init_logging().expect("Failed to initialize logging");
    
    // Print connection info
    args.print_connection_info();
    
    // Get connection info
    let (host, user, password, database) = args.get_connection_info().expect("Failed to get connection info");
    
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
    
    // Run the state machine
    match state_machine.run().await {
        Ok(_) => {
            println!("Connection test completed successfully!");
            
            // If monitor duration is specified, run job monitoring
            if args.monitor_duration > 0 {
                println!("\nStarting import job monitoring...");
                
                // Create job monitor with the connection from the main state machine
                let mut job_monitor = JobMonitor::new(args.monitor_duration);
                
                // Transfer the connection to the job monitor
                if let Some(conn) = state_machine.get_context_mut().connection.take() {
                    job_monitor.get_context_mut().connection = Some(conn);
                    job_monitor.get_context_mut().host = state_machine.get_context().host.clone();
                    job_monitor.get_context_mut().port = state_machine.get_context().port;
                    job_monitor.get_context_mut().username = state_machine.get_context().username.clone();
                    job_monitor.get_context_mut().password = state_machine.get_context().password.clone();
                    job_monitor.get_context_mut().database = state_machine.get_context().database.clone();
                    
                    // Run the job monitor
                    if let Err(e) = job_monitor.run().await {
                        eprintln!("✗ Job monitoring failed: {e}");
                    }
                } else {
                    eprintln!("✗ No connection available for job monitoring");
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to complete connection test: {e}");
            
            // Try to provide more specific error messages
            let error_msg = e.to_string().to_lowercase();
            if error_msg.contains("access denied") || error_msg.contains("authentication") {
                eprintln!("  → Check your username and password");
            } else if error_msg.contains("connection refused") || error_msg.contains("timeout") {
                eprintln!("  → Check if TiDB is running on the specified host and port");
            } else if error_msg.contains("unknown database") {
                eprintln!("  → Database does not exist");
            }
            
            process::exit(1);
        }
    }
} 
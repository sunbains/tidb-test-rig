use std::process;
use connect::state_machine::{StateMachine, State};
use connect::state_handlers::{InitialHandler, ParsingConfigHandler, ConnectingHandler, TestingConnectionHandler, VerifyingDatabaseHandler, GettingVersionHandler};
use connect::{CheckingImportJobsHandler, ShowingImportJobDetailsHandler, parse_args};

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
    state_machine.register_handler(State::CheckingImportJobs, Box::new(CheckingImportJobsHandler));
    state_machine.register_handler(
        State::ShowingImportJobDetails, 
        Box::new(ShowingImportJobDetailsHandler::new(args.monitor_duration))
    );
    
    // Run the state machine
    match state_machine.run().await {
        Ok(_) => {
            println!("Connection test completed successfully!");
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


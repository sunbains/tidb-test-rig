use clap::Parser;
use std::process;
use connect::state_machine::{StateMachine, State};
use connect::state_handlers::{InitialHandler, ParsingConfigHandler, ConnectingHandler, TestingConnectionHandler, VerifyingDatabaseHandler, GettingVersionHandler};
use connect::{CheckingImportJobsHandler, ShowingImportJobDetailsHandler};
use rpassword::prompt_password;

#[derive(Parser)]
#[command(name = "tidb-client")]
#[command(about = "A basic TiDB client for connection testing")]
struct Args {
    /// Hostname and port in format hostname:port
    #[arg(short = 'H', long, default_value = "tidb.qyruvz1u6xtd.clusters.dev.tidb-cloud.com:4000")]
    host: String,
    
    /// Username for database authentication
    #[arg(short = 'u', long)]
    user: String,
    
    /// Database name (optional)
    #[arg(short = 'd', long)]
    database: Option<String>,

    /// Duration to monitor import jobs in seconds (default: 60)
    #[arg(short = 't', long, default_value = "60")]
    monitor_duration: u64,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Prompt for password securely
    let password = prompt_password("Password: ").expect("Failed to read password");
    
    // Create and configure the state machine
    let mut state_machine = StateMachine::new();
    
    // Register state handlers
    state_machine.register_handler(State::Initial, Box::new(InitialHandler));
    state_machine.register_handler(
        State::ParsingConfig,
        Box::new(ParsingConfigHandler::new(
            args.host.clone(),
            args.user.clone(),
            password,
            args.database.clone()
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
            eprintln!("✗ Failed to complete connection test: {}", e);
            
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


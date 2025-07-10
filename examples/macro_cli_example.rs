use connect::state_machine::{StateMachine, State};
use connect::state_handlers::{InitialHandler, ParsingConfigHandler, ConnectingHandler, TestingConnectionHandler, VerifyingDatabaseHandler, GettingVersionHandler};
use connect::{generate_cli_args, generate_cli_impl};
use clap::Parser;

// Generate CLI arguments specific to this example
generate_cli_args!(isolation_test);

// Generate CLI implementation specific to this example
generate_cli_impl!(isolation_test);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("TiDB Macro-based CLI Example");
    println!("=============================");
    
    // Parse command line arguments using the macro-generated CLI
    let args = CommonArgs::parse();
    
    // Print connection info (includes test_rows for isolation test)
    args.print_connection_info();
    
    // Get connection info
    let (host, user, password, database) = args.get_connection_info()?;
    let database = database.unwrap_or_else(|| "test".to_string());
    
    // Access example-specific arguments
    println!("  Test Rows: {}", args.test_rows);
    
    // Create and configure the state machine
    let mut state_machine = StateMachine::new();
    
    // Register standard handlers
    state_machine.register_handler(State::Initial, Box::new(InitialHandler));
    state_machine.register_handler(
        State::ParsingConfig,
        Box::new(ParsingConfigHandler::new(
            host,
            user,
            password,
            Some(database)
        ))
    );
    state_machine.register_handler(State::Connecting, Box::new(ConnectingHandler));
    state_machine.register_handler(State::TestingConnection, Box::new(TestingConnectionHandler));
    state_machine.register_handler(State::VerifyingDatabase, Box::new(VerifyingDatabaseHandler));
    state_machine.register_handler(State::GettingVersion, Box::new(GettingVersionHandler));
    
    // Run the state machine
    match state_machine.run().await {
        Ok(_) => {
            println!("\n✅ Macro-based CLI example completed successfully!");
            println!("Used {} test rows configuration", args.test_rows);
        }
        Err(e) => {
            eprintln!("\n❌ Macro-based CLI example failed: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
} 
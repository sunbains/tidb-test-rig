use connect::state_machine::{StateMachine, State};
use connect::{InitialHandler, ParsingConfigHandler, ConnectingHandler, TestingConnectionHandler, VerifyingDatabaseHandler, GettingVersionHandler};
use connect::cli::CommonArgs;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("TiDB Simple Connection Test");
    println!("===========================");
    
    // Parse command line arguments using the common CLI library
    let args = CommonArgs::parse();
    
    // Print connection info
    args.print_connection_info();
    
    // Get connection info
    let (host, user, password, database) = args.get_connection_info()?;
    
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
            println!("\n✅ Simple connection test completed successfully!");
        }
        Err(e) => {
            eprintln!("\n❌ Simple connection test failed: {e}");
            std::process::exit(1);
        }
    }
    
    Ok(())
} 
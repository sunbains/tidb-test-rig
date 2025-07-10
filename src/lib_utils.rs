use crate::state_machine::{StateMachine, State};
use crate::{
    InitialHandler, ParsingConfigHandler, ConnectingHandler, 
    TestingConnectionHandler, VerifyingDatabaseHandler,
};
use crate::state_handlers::GettingVersionHandler;
use crate::cli::CommonArgs;
use crate::errors::{StateError, Result};
use clap::Parser;
use std::process;

/// Common setup for tests using the new CommonArgs approach
pub struct TestSetup {
    pub args: CommonArgs,
}

impl TestSetup {
    /// Create a new test setup with CommonArgs
    pub fn new(args: &CommonArgs) -> Self {
        Self {
            args: args.clone(),
        }
    }
    
    /// Run the basic connection workflow
    pub async fn run_basic_workflow(&self) -> Result<()> {
        // Get connection info
        let (host, user, password, database) = self.args.get_connection_info()
            .map_err(|e| StateError::from(e.to_string()))?;
        
        // Create and configure the state machine
        let mut state_machine = StateMachine::new();
        
        // Register standard state handlers
        register_standard_handlers(&mut state_machine, host, user, password, database);
        
        // Run the state machine
        match state_machine.run().await {
            Ok(_) => {
                println!("Connection test completed successfully!");
                Ok(())
            }
            Err(e) => {
                Self::handle_connection_error(&e);
                Err(e)
            }
        }
    }
    
    /// Handle connection errors with helpful messages
    pub fn handle_connection_error(e: &StateError) {
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

// Register the standard set of state handlers as a top-level function
pub fn register_standard_handlers(
    state_machine: &mut StateMachine,
    host: String,
    user: String,
    password: String,
    database: Option<String>,
) {
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
}

/// Common setup for tests using the new CommonArgs approach
pub struct CommonArgsSetup {
    pub args: CommonArgs,
    pub state_machine: StateMachine,
}

impl CommonArgsSetup {
    /// Create a new test setup with CommonArgs
    pub fn new() -> std::result::Result<Self, Box<dyn std::error::Error>> {
        // Parse command line arguments
        let args = CommonArgs::parse();
        
        // Initialize logging
        args.init_logging()?;
        
        // Print connection info
        args.print_connection_info();
        
        // Get connection info
        let (host, user, password, database) = args.get_connection_info()?;
        
        // Create and configure the state machine
        let mut state_machine = StateMachine::new();
        
        // Register standard state handlers
        register_standard_handlers(&mut state_machine, host, user, password, database);
        
        Ok(Self {
            args,
            state_machine,
        })
    }
    
    /// Run the state machine with standard error handling
    pub async fn run_with_error_handling(&mut self) -> Result<()> {
        match self.state_machine.run().await {
            Ok(_) => {
                println!("Connection test completed successfully!");
                Ok(())
            }
            Err(e) => {
                TestSetup::handle_connection_error(&e);
                Err(e)
            }
        }
    }
}

/// Helper function to print a standard test header
pub fn print_test_header(title: &str) {
    println!("{}", title);
    println!("{}", "=".repeat(title.len()));
}

/// Helper function to print a success message
pub fn print_success(message: &str) {
    println!("\n✅ {}", message);
}

/// Helper function to print an error message and exit
pub fn print_error_and_exit(message: &str, error: &dyn std::error::Error) {
    eprintln!("\n❌ {}: {}", message, error);
    std::process::exit(1);
}

/// Helper function to create a state machine with standard handlers for multi-connection scenarios
pub fn create_state_machine_with_handlers(
    host: String,
    user: String,
    password: String,
    database: Option<String>,
) -> crate::state_machine::StateMachine {
    let mut state_machine = crate::state_machine::StateMachine::new();
    register_standard_handlers(&mut state_machine, host, user, password, database);
    state_machine
} 

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_test_header_and_success() {
        print_test_header("Header");
        print_success("Success");
    }

    // print_error_and_exit cannot be tested as it exits the process
} 
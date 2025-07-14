//! # Library Utilities
//!
//! Common utility functions and helpers for the test_rig framework.
//! Provides error reporting, success reporting, and test output formatting.

use crate::cli::CommonArgs;
use crate::errors::{ConnectError, Result};
use crate::state_handlers::InitialHandler;
use crate::state_machine::{State, StateMachine};
use std::process;

/// Common setup for tests using the new `CommonArgs` approach
pub struct TestSetup {
    pub args: CommonArgs,
}

impl TestSetup {
    /// Create a new test setup with `CommonArgs`
    #[must_use]
    pub fn new(args: &CommonArgs) -> Self {
        Self { args: args.clone() }
    }

    /// Run the basic workflow
    ///
    /// # Errors
    ///
    /// Returns an error if the workflow execution fails.
    pub async fn run_basic_workflow(&self) -> Result<()> {
        // Get connection info
        let (host, user, password, database) = self
            .args
            .get_connection_info()
            .map_err(|e| ConnectError::StateMachine(e.to_string()))?;

        // Create and configure the state machine
        let mut state_machine = StateMachine::new();

        // Register standard state handlers
        register_standard_handlers(&mut state_machine, host, user, password, database);

        // Run the state machine
        match state_machine.run().await {
            Ok(()) => {
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
    pub fn handle_connection_error(e: &ConnectError) {
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
    _host: String,
    _user: String,
    _password: String,
    _database: Option<String>,
) {
    state_machine.register_handler(State::Initial, Box::new(InitialHandler));
    // Note: For extensible state handling, use the dynamic state machine system
    // The core StateMachine now only supports Initial, Completed, and Error states
    // Use DynamicStateMachine for complex workflows with custom states
}

/// Helper function to print a standard test header
pub fn print_test_header(title: &str) {
    println!("{title}");
    println!("{}", "=".repeat(title.len()));
}

/// Helper function to print a success message
pub fn print_success(message: &str) {
    println!("\n✅ {message}");
}

/// Helper function to print an error message and exit
pub fn print_error_and_exit(message: &str, error: &dyn std::error::Error) {
    eprintln!("\n❌ {message}: {error}");
    std::process::exit(1);
}

/// Helper function to create a state machine with standard handlers for multi-connection scenarios
#[must_use]
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

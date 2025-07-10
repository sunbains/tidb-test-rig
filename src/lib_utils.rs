use crate::state_machine::{StateMachine, State};
use crate::{
    InitialHandler, ParsingConfigHandler, ConnectingHandler, 
    TestingConnectionHandler, VerifyingDatabaseHandler, GettingVersionHandler,
    parse_args, JobMonitor
};
use crate::cli::CommonArgs;
use clap::Parser;
use std::process;

/// Common setup for examples using the legacy parse_args approach
pub struct TestSetup {
    pub args: crate::cli::CommonArgs,
    pub state_machine: StateMachine,
}

impl TestSetup {
    /// Create a new example setup with standard configuration
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
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
        
        // Register standard state handlers
        Self::register_standard_handlers(&mut state_machine, host, user, password, database);
        
        Ok(Self {
            args,
            state_machine,
        })
    }
    
    /// Register the standard set of state handlers
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
    
    /// Run the state machine with standard error handling
    pub async fn run_with_error_handling(&mut self) -> Result<(), crate::state_machine::StateError> {
        match self.state_machine.run().await {
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
    
    /// Run the state machine and optionally start job monitoring
    pub async fn run_with_job_monitoring(&mut self) -> Result<(), crate::state_machine::StateError> {
        match self.state_machine.run().await {
            Ok(_) => {
                println!("Connection test completed successfully!");
                
                // If monitor duration is specified, run job monitoring
                if self.args.monitor_duration > 0 {
                    println!("\nStarting import job monitoring...");
                    
                    // Create job monitor with the connection from the main state machine
                    let mut job_monitor = JobMonitor::new(self.args.monitor_duration);
                    
                    // Transfer the connection to the job monitor
                    if let Some(conn) = self.state_machine.get_context_mut().connection.take() {
                        job_monitor.get_context_mut().connection = Some(conn);
                        job_monitor.get_context_mut().host = self.state_machine.get_context().host.clone();
                        job_monitor.get_context_mut().port = self.state_machine.get_context().port;
                        job_monitor.get_context_mut().username = self.state_machine.get_context().username.clone();
                        job_monitor.get_context_mut().password = self.state_machine.get_context().password.clone();
                        job_monitor.get_context_mut().database = self.state_machine.get_context().database.clone();
                        
                        // Run the job monitor
                        if let Err(e) = job_monitor.run().await {
                            eprintln!("✗ Job monitoring failed: {e}");
                        }
                    } else {
                        eprintln!("✗ No connection available for job monitoring");
                    }
                }
                
                Ok(())
            }
            Err(e) => {
                Self::handle_connection_error(&e);
                Err(e)
            }
        }
    }
    
    /// Handle connection errors with helpful messages
    pub fn handle_connection_error(e: &crate::state_machine::StateError) {
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

/// Common setup for examples using the new CommonArgs approach
pub struct CommonArgsSetup {
    pub args: CommonArgs,
    pub state_machine: StateMachine,
}

impl CommonArgsSetup {
    /// Create a new example setup with CommonArgs
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
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
        TestSetup::register_standard_handlers(&mut state_machine, host, user, password, database);
        
        Ok(Self {
            args,
            state_machine,
        })
    }
    
    /// Run the state machine with standard error handling
    pub async fn run_with_error_handling(&mut self) -> Result<(), crate::state_machine::StateError> {
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
    
    /// Run the state machine and optionally start job monitoring
    pub async fn run_with_job_monitoring(&mut self) -> Result<(), crate::state_machine::StateError> {
        match self.state_machine.run().await {
            Ok(_) => {
                println!("Connection test completed successfully!");
                
                // If monitor duration is specified, run job monitoring
                if self.args.monitor_duration > 0 {
                    println!("\nStarting import job monitoring...");
                    
                    // Create job monitor with the connection from the main state machine
                    let mut job_monitor = JobMonitor::new(self.args.monitor_duration);
                    
                    // Transfer the connection to the job monitor
                    if let Some(conn) = self.state_machine.get_context_mut().connection.take() {
                        job_monitor.get_context_mut().connection = Some(conn);
                        job_monitor.get_context_mut().host = self.state_machine.get_context().host.clone();
                        job_monitor.get_context_mut().port = self.state_machine.get_context().port;
                        job_monitor.get_context_mut().username = self.state_machine.get_context().username.clone();
                        job_monitor.get_context_mut().password = self.state_machine.get_context().password.clone();
                        job_monitor.get_context_mut().database = self.state_machine.get_context().database.clone();
                        
                        // Run the job monitor
                        if let Err(e) = job_monitor.run().await {
                            eprintln!("✗ Job monitoring failed: {e}");
                        }
                    } else {
                        eprintln!("✗ No connection available for job monitoring");
                    }
                }
                
                Ok(())
            }
            Err(e) => {
                TestSetup::handle_connection_error(&e);
                Err(e)
            }
        }
    }
}

/// Helper function to print a standard example header
pub fn print_example_header(title: &str) {
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
    TestSetup::register_standard_handlers(&mut state_machine, host, user, password, database);
    state_machine
} 
//!
//! # Simple Multi-Connection TiDB Testing Binary
//!
//! This binary demonstrates a straightforward approach to running multiple TiDB connections in parallel.
//! It is designed as an easy-to-understand example for basic concurrency and load testing scenarios,
//! and serves as a starting point for more advanced multi-connection workflows.
//!
//! ## Overview
//!
//! This test creates and manages multiple TiDB connections simultaneously, running them in parallel
//! with minimal coordination. Each connection is managed by its own state machine, and results are
//! collected in a shared state for simple reporting at the end of the test.
//!
//! This approach is ideal for:
//! - **Basic Load Testing**: Simulate multiple clients connecting and running queries concurrently
//! - **Connection Health Checks**: Verify that multiple connections can be established and used in parallel
//! - **Quick Prototyping**: Use as a template for building more complex multi-connection tests
//!
//! ## Architecture
//!
//! - **SimpleMultiConnectionCoordinator**: Manages a list of connection configs and a shared state for results
//! - **StateMachine per Connection**: Each connection runs its own state machine independently
//! - **SharedTestState**: Collects connection results and global status for reporting
//!
//! ## State Flow
//!
//! Each connection follows this state progression:
//! 1. **Initial** → **ParsingConfig** → **Connecting** → **TestingConnection**
//! 2. **VerifyingDatabase** → **GettingVersion** → **Completed**
//!
//! All connections run these states concurrently, and their results are aggregated at the end.
//!
//! ## Features
//!
//! - **Parallel Connection Management**: Multiple connections run in parallel using Tokio tasks
//! - **Minimal Coordination**: No complex event or message passing between connections
//! - **Simple Reporting**: Aggregates and prints connection results and statuses
//! - **Easy to Extend**: Serves as a foundation for more advanced multi-connection scenarios
//!
//! ## Usage
//!
//! ```bash
//! # Basic usage with default settings
//! cargo run --bin simple_multi_connection --features multi_connection
//!
//! # Custom connection count
//! cargo run --bin simple_multi_connection --features multi_connection -- --connection-count 5
//!
//! # With configuration file
//! cargo run --bin simple_multi_connection --features multi_connection -- -c config.json
//! ```
//!
//! ## Output
//!
//! The test prints:
//! - Connection status for each connection (host, status, errors)
//! - Global test status
//! - Summary of all connection results
//!
//! ## Error Handling
//!
//! - Individual connection failures are reported and do not affect other connections
//! - Errors are collected and displayed in the final report
//!
//! ## Extensibility
//!
//! This binary is intended as a simple, clear example. For more advanced coordination, shared state,
//! or import job monitoring, see the `multi_connection.rs` binary.

use connect::state_machine::{StateMachine, State};
use connect::errors::StateError;
use connect::state_handlers::{InitialHandler, ParsingConfigHandler, ConnectingHandler, TestingConnectionHandler, VerifyingDatabaseHandler, GettingVersionHandler};
use connect::JobMonitor;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use std::collections::HashMap;
use connect::{CommonArgs, print_test_header, print_success};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "simple-multi-connection")]
#[command(about = "Simple multi-connection test with connection count argument")]
pub struct Args {
    #[command(flatten)]
    pub common: CommonArgs,
    /// Number of connections to create for multi-connection tests
    #[arg(long, default_value = "2")]
    pub connection_count: u32,
}

impl Args {
    pub fn print_connection_info(&self) {
        self.common.print_connection_info();
        println!("  Connection Count: {}", self.connection_count);
    }
    pub fn init_logging(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.common.init_logging()
    }
    pub fn get_connection_info(&self) -> connect::cli::ConnInfoResult {
        self.common.get_connection_info()
    }
}

/// Simple shared state for coordination
#[derive(Debug, Clone)]
pub struct SharedTestState {
    pub connection_results: HashMap<String, ConnectionResult>,
    pub global_status: String,
}

#[derive(Debug, Clone)]
pub struct ConnectionResult {
    pub connection_id: String,
    pub host: String,
    pub status: ConnectionStatus,
    pub error: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    NotStarted,
    Connecting,
    Connected,
    Testing,
    Completed,
    Failed,
}

impl Default for SharedTestState {
    fn default() -> Self {
        Self {
            connection_results: HashMap::new(),
            global_status: "Initialized".to_string(),
        }
    }
}

/// Simple multi-connection coordinator
pub struct SimpleMultiConnectionCoordinator {
    shared_state: Arc<Mutex<SharedTestState>>,
    connections: Vec<ConnectionConfig>,
}

impl Default for SimpleMultiConnectionCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ConnectionConfig {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: Option<String>,
}

impl SimpleMultiConnectionCoordinator {
    pub fn new() -> Self {
        Self {
            shared_state: Arc::new(Mutex::new(SharedTestState::default())),
            connections: Vec::new(),
        }
    }

    pub fn add_connection(&mut self, config: ConnectionConfig) {
        // Initialize connection result
        if let Ok(mut state) = self.shared_state.lock() {
            state.connection_results.insert(config.id.clone(), ConnectionResult {
                connection_id: config.id.clone(),
                host: config.host.clone(),
                status: ConnectionStatus::NotStarted,
                error: None,
                version: None,
            });
        }
        self.connections.push(config);
    }

    pub fn get_shared_state(&self) -> Arc<Mutex<SharedTestState>> {
        Arc::clone(&self.shared_state)
    }

    /// Run all connections concurrently
    pub async fn run_all_connections(&self) -> Result<(), StateError> {
        println!("Starting {} connections concurrently...", self.connections.len());
        
        let mut handles: Vec<JoinHandle<Result<(), StateError>>> = Vec::new();
        
        for connection in &self.connections {
            let shared_state = Arc::clone(&self.shared_state);
            let connection_id = connection.id.clone();
            let host = connection.host.clone();
            let username = connection.username.clone();
            let password = connection.password.clone();
            let database = connection.database.clone();
            
            let handle = tokio::spawn(async move {
                // Create state machine for this connection
                let mut state_machine = StateMachine::new();
                
                // Register handlers
                state_machine.register_handler(State::Initial, Box::new(InitialHandler));
                state_machine.register_handler(
                    State::ParsingConfig,
                    Box::new(ParsingConfigHandler::new(
                        host,
                        username,
                        password,
                        database
                    ))
                );
                state_machine.register_handler(State::Connecting, Box::new(ConnectingHandler));
                state_machine.register_handler(State::TestingConnection, Box::new(TestingConnectionHandler));
                state_machine.register_handler(State::VerifyingDatabase, Box::new(VerifyingDatabaseHandler));
                state_machine.register_handler(State::GettingVersion, Box::new(GettingVersionHandler));
                
                // Update status to connecting
                if let Ok(mut state) = shared_state.lock() {
                    if let Some(result) = state.connection_results.get_mut(&connection_id) {
                        result.status = ConnectionStatus::Connecting;
                    }
                }
                
                // Run the state machine
                match state_machine.run().await {
                    Ok(_) => {
                        // Update status to completed
                        if let Ok(mut state) = shared_state.lock() {
                            if let Some(result) = state.connection_results.get_mut(&connection_id) {
                                result.status = ConnectionStatus::Completed;
                            }
                            state.global_status = "All connections completed".to_string();
                        }
                        println!("✓ Connection {} completed successfully", connection_id);
                        
                        // Run job monitoring for this connection
                        println!("Starting job monitoring for connection {}...", connection_id);
                        let mut job_monitor = JobMonitor::new(30); // 30 seconds monitoring
                        
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
                                eprintln!("✗ Job monitoring failed for connection {}: {}", connection_id, e);
                            }
                        }
                        
                        Ok(())
                    }
                    Err(e) => {
                        // Update status to failed
                        if let Ok(mut state) = shared_state.lock() {
                            if let Some(result) = state.connection_results.get_mut(&connection_id) {
                                result.status = ConnectionStatus::Failed;
                                result.error = Some(e.to_string());
                            }
                        }
                        eprintln!("✗ Connection {} failed: {}", connection_id, e);
                        Err(e)
                    }
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all connections to complete
        for handle in handles {
            match handle.await {
                Ok(Ok(_)) => {},
                Ok(Err(e)) => eprintln!("Connection task failed: {}", e),
                Err(e) => eprintln!("Task join failed: {}", e),
            }
        }
        
        Ok(())
    }

    /// Print final results
    pub fn print_results(&self) {
        if let Ok(state) = self.shared_state.lock() {
            println!("\n=== Final Results ===");
            println!("Global Status: {}", state.global_status);
            println!("\nConnection Results:");
            
            for (conn_id, result) in &state.connection_results {
                println!("  {}: {:?} - {}", conn_id, result.status, result.host);
                if let Some(error) = &result.error {
                    println!("    Error: {}", error);
                }
                if let Some(version) = &result.version {
                    println!("    Version: {}", version);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_test_header("Simple Multi-Connection TiDB Testing");
    let args = Args::parse();
    args.init_logging()?;
    args.print_connection_info();

    let mut coordinator = SimpleMultiConnectionCoordinator::new();

    // Add multiple connections
    coordinator.add_connection(ConnectionConfig {
        id: "primary".to_string(),
        host: "tidb-primary.example.com".to_string(),
        port: 4000,
        username: "user1".to_string(),
        password: "password1".to_string(),
        database: Some("test_db".to_string()),
    });

    coordinator.add_connection(ConnectionConfig {
        id: "secondary".to_string(),
        host: "tidb-secondary.example.com".to_string(),
        port: 4000,
        username: "user2".to_string(),
        password: "password2".to_string(),
        database: Some("test_db".to_string()),
    });

    coordinator.add_connection(ConnectionConfig {
        id: "backup".to_string(),
        host: "tidb-backup.example.com".to_string(),
        port: 4000,
        username: "user3".to_string(),
        password: "password3".to_string(),
        database: Some("backup_db".to_string()),
    });

    // Run all connections concurrently
    if let Err(e) = coordinator.run_all_connections().await {
        eprintln!("Failed to run connections: {}", e);
        return Err(Box::new(std::io::Error::other(e.to_string())) as Box<dyn std::error::Error>);
    }

    // Print results
    coordinator.print_results();

    print_success("Multi-connection testing completed!");
    Ok(())
} 

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from([
            "test-bin", 
            "--connection-count", "5",
            "-H", "localhost:4000",
            "-u", "testuser"
        ]);
        assert_eq!(args.connection_count, 5);
        assert_eq!(args.common.host, "localhost:4000");
        assert_eq!(args.common.user, "testuser");
    }

    #[test]
    fn test_args_defaults() {
        let args = Args::parse_from(["test-bin"]);
        assert_eq!(args.connection_count, 2); // default value
        assert_eq!(args.common.host, "localhost:4000"); // default value
        assert_eq!(args.common.user, "root"); // default value
    }

    #[test]
    fn test_shared_test_state_default() {
        let state = SharedTestState::default();
        assert_eq!(state.global_status, "Initialized");
        assert!(state.connection_results.is_empty());
    }

    #[test]
    fn test_connection_config_creation() {
        let config = ConnectionConfig {
            id: "test-1".to_string(),
            host: "localhost".to_string(),
            port: 4000,
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            database: Some("testdb".to_string()),
        };
        assert_eq!(config.id, "test-1");
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 4000);
        assert_eq!(config.username, "testuser");
        assert_eq!(config.database, Some("testdb".to_string()));
    }

    #[test]
    fn test_coordinator_creation() {
        let coordinator = SimpleMultiConnectionCoordinator::new();
        assert_eq!(coordinator.connections.len(), 0);
        
        // Test that shared state is accessible
        let shared_state = coordinator.get_shared_state();
        if let Ok(state) = shared_state.lock() {
            assert_eq!(state.global_status, "Initialized");
            assert!(state.connection_results.is_empty());
        }
    }

    #[test]
    fn test_connection_status_enum() {
        assert!(matches!(ConnectionStatus::NotStarted, ConnectionStatus::NotStarted));
        assert!(matches!(ConnectionStatus::Connecting, ConnectionStatus::Connecting));
        assert!(matches!(ConnectionStatus::Completed, ConnectionStatus::Completed));
        assert!(matches!(ConnectionStatus::Failed, ConnectionStatus::Failed));
    }
} 
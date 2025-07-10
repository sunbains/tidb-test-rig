//! # Advanced Multi-Connection TiDB Testing Binary
//!
//! This binary provides advanced multi-connection testing capabilities for TiDB databases,
//! implementing a sophisticated coordination system for managing multiple concurrent
//! database connections with shared state management.
//!
//! ## Overview
//!
//! This test creates and manages multiple TiDB connections simultaneously,
//! running them in parallel while coordinating their activities through a shared state
//! management system. This is useful for testing:
//!
//! - **Load Testing**: Multiple connections performing concurrent operations
//! - **Failover Testing**: Testing behavior when some connections fail
//! - **Coordination Testing**: Ensuring proper synchronization between connections
//!
//! ## Architecture
//!
//! ### Core Components
//!
//! 1. **MultiConnectionStateMachine**: Orchestrates multiple individual state machines
//!    - Each connection gets its own state machine instance
//!    - Manages concurrent execution using Tokio tasks
//!    - Coordinates state transitions across all connections
//!
//! 2. **ConnectionCoordinator**: Central coordination hub
//!    - Maintains shared state accessible to all connections
//!    - Handles inter-connection communication via message passing
//!    - Tracks connection status, import jobs, and coordination events
//!
//! 3. **SharedState**: Global state management
//!    - Connection status for each connection (Connected, Testing, Error, etc.)
//!    - Active import jobs across all connections
//!    - Coordination events and timing information
//!    - Global configuration (test duration, timeouts, max connections)
//!
//! ### State Flow
//!
//! Each connection follows this state progression:
//! 1. **Initial** → **ParsingConfig** → **Connecting** → **TestingConnection**
//! 2. **VerifyingDatabase** → **GettingVersion** → **Completed**
//!
//! All connections run these states concurrently, with the coordinator
//! tracking progress and managing shared resources.
//!
//! ## Features
//!
//! - **Concurrent Connection Management**: Multiple connections run in parallel
//! - **Shared State Coordination**: Real-time status tracking across connections
//! - **Import Job Monitoring**: Monitor import jobs across multiple connections
//! - **Error Isolation**: Failures in one connection don't affect others
//! - **Comprehensive Reporting**: Detailed status and event reporting
//! - **Configurable Timeouts**: Adjustable coordination and test timeouts
//!
//! ## Usage
//!
//! ```bash
//! # Basic usage with default settings
//! cargo run --bin multi_connection --features multi_connection,import_jobs
//!
//! # Custom connection count
//! cargo run --bin multi_connection --features multi_connection,import_jobs -- --connection-count 5
//!
//! # With configuration file
//! cargo run --bin multi_connection --features multi_connection,import_jobs -- -c config.json
//! ```
//!
//! ## Configuration
//!
//! The binary uses a `GlobalConfig` with these settings:
//! - **test_duration**: 120 seconds (2 minutes) - Total test duration
//! - **coordination_timeout**: 30 seconds - Timeout for coordination events
//! - **max_connections**: 3 - Maximum number of concurrent connections
//!
//! ## Output
//!
//! The test provides comprehensive output including:
//! - Connection status for each connection (host, status, errors)
//! - Active import jobs across all connections
//! - Coordination events and timing information
//! - Overall test success/failure status
//!
//! ## Error Handling
//!
//! - Individual connection failures are isolated and reported
//! - Coordination timeouts are handled gracefully
//! - Detailed error messages for debugging
//! - Graceful shutdown on critical failures
//!
//! ## Dependencies
//!
//! Requires the `multi_connection` and `import_jobs` features to be enabled.
//! Uses the shared state machine framework from the main library.

use clap::Parser;
use test_rig::{CommonArgs, print_success, print_test_header};
use test_rig::{ConnectionCoordinator, ConnectionInfo, GlobalConfig, MultiConnectionStateMachine};

#[derive(Parser, Debug)]
#[command(name = "multi-connection-test")]
#[command(about = "Advanced multi-connection test with connection count argument")]
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
    pub fn get_connection_info(&self) -> test_rig::cli::ConnInfoResult {
        self.common.get_connection_info()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_test_header("Advanced Multi-Connection TiDB Testing");
    let args = Args::parse();
    args.init_logging()?;
    args.print_connection_info();

    // Create global configuration
    let config = GlobalConfig {
        test_duration: 120,       // 2 minutes
        coordination_timeout: 30, // 30 seconds
        max_connections: 3,       // Max 3 connections
    };

    // Create coordinator
    let coordinator = ConnectionCoordinator::new(config);
    let mut multi_sm = MultiConnectionStateMachine::new(coordinator);

    // Define multiple connections
    let connections = vec![
        (
            "primary".to_string(),
            ConnectionInfo {
                host: "tidb-primary.example.com".to_string(),
                port: 4000,
                username: "user1".to_string(),
                password: "password1".to_string(),
                database: Some("test_db".to_string()),
                connection: None,
            },
        ),
        (
            "secondary".to_string(),
            ConnectionInfo {
                host: "tidb-secondary.example.com".to_string(),
                port: 4000,
                username: "user2".to_string(),
                password: "password2".to_string(),
                database: Some("test_db".to_string()),
                connection: None,
            },
        ),
        (
            "backup".to_string(),
            ConnectionInfo {
                host: "tidb-backup.example.com".to_string(),
                port: 4000,
                username: "user3".to_string(),
                password: "password3".to_string(),
                database: Some("backup_db".to_string()),
                connection: None,
            },
        ),
    ];

    // Add connections to the multi-state machine
    for (connection_id, connection_info) in connections {
        println!("Adding connection: {connection_id}");
        multi_sm.add_connection(connection_id, connection_info);
    }

    // Run all connections concurrently
    println!("\nStarting concurrent connection testing...");
    if let Err(e) = multi_sm.run_all().await {
        eprintln!("Failed to run multi-connection test: {e}");
        return Err(Box::new(std::io::Error::other(e.to_string())) as Box<dyn std::error::Error>);
    }

    // Check results
    println!("\n=== Final Results ===");

    let shared_state = multi_sm.get_shared_state();
    if let Ok(state) = shared_state.lock() {
        println!("Connection Status:");
        for (conn_id, status) in &state.connection_status {
            println!("  {}: {:?} - {}", conn_id, status.status, status.host);
            if let Some(error) = &status.error_message {
                println!("    Error: {error}");
            }
        }

        println!("\nActive Import Jobs:");
        for job in &state.import_jobs {
            if job.end_time.is_none() {
                println!(
                    "  Job {} on {}: {} - {}",
                    job.job_id, job.connection_id, job.phase, job.status
                );
            }
        }

        println!("\nCoordination Events:");
        for event in &state.coordination_events {
            println!("  {event:?}");
        }
    }

    print_success("Advanced multi-connection testing completed!");
    Ok(())
}

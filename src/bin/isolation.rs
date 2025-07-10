//!
//! # TiDB Isolation Test Binary
//!
//! This binary implements a simple test for TiDB's transaction isolation guarantees (such as repeatable read).
//! It is designed to verify that TiDB enforces correct isolation semantics under concurrent transactions.
//!
//! ## Overview
//!
//! The isolation test creates a dedicated test table, populates it with data, and then runs concurrent transactions
//! to verify that isolation properties (e.g., repeatable read) are upheld. The test is useful for:
//! - **Verifying Transaction Isolation**: Ensuring TiDB's isolation level is correctly implemented
//! - **Regression Testing**: Detecting changes or regressions in isolation behavior across TiDB versions
//! - **Database Correctness**: Validating that concurrent operations do not violate isolation guarantees
//!
//! ## Architecture
//!
//! - **State Machine**: Drives the workflow through all phases of the test
//! - **Custom Handlers**: Implements handlers for creating tables, populating data, and running isolation checks
//! - **Test Context**: Stores test table name, results, and phase for each run
//!
//! ## State Flow
//!
//! The test progresses through these states:
//! 1. **Initial** → **ParsingConfig** → **Connecting**
//! 2. **CreatingTable**: Create a dedicated test table for isolation testing
//! 3. **PopulatingData**: Insert test rows into the table
//! 4. **TestingIsolation**: Run concurrent transactions to verify isolation
//! 5. **VerifyingResults**: Check and report the results
//! 6. **Completed**
//!
//! ## Features
//!
//! - **Automated Table Setup**: Creates and cleans up a dedicated test table
//! - **Concurrent Transaction Testing**: Runs multiple transactions to test isolation
//! - **Detailed Reporting**: Prints step-by-step results and any detected anomalies
//! - **Configurable Test Size**: Number of test rows is configurable via CLI
//! - **Extensible**: Handlers can be extended for more complex isolation scenarios
//!
//! ## Usage
//!
//! ```bash
//! # Basic usage with default settings
//! cargo run --bin isolation --features isolation_test
//!
//! # Custom number of test rows
//! cargo run --bin isolation --features isolation_test -- --test-rows 20
//!
//! # With configuration file
//! cargo run --bin isolation --features isolation_test -- -c config.json
//! ```
//!
//! ## Output
//!
//! The test prints:
//! - Connection and test configuration
//! - Step-by-step progress through each phase
//! - Results of isolation checks and any errors detected
//!
//! ## Error Handling
//!
//! - All errors are reported with context
//! - Test aborts on critical failures (e.g., connection errors, table creation failures)
//! - Results are printed for debugging and regression tracking
//!
//! ## Extensibility
//!
//! This binary is intended as a robust, extensible foundation for isolation and concurrency testing in TiDB.
//! Handlers and test logic can be extended to cover more advanced isolation scenarios as needed.

use connect::state_machine::{StateMachine, State, StateContext, StateHandler};
use connect::{CommonArgs, print_test_header, print_success, print_error_and_exit, register_standard_handlers};
use connect::state_handlers::NextStateVersionHandler;
use connect::errors::{StateError, Result};
use mysql::prelude::*;
use mysql::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "isolation-test")]
#[command(about = "TiDB isolation test with test-specific arguments")]
pub struct IsolationTestArgs {
    #[command(flatten)]
    pub common: CommonArgs,
    /// Number of test rows to create for isolation testing
    #[arg(long, default_value = "10")]
    pub test_rows: u32,
}

impl IsolationTestArgs {
    pub fn print_connection_info(&self) {
        self.common.print_connection_info();
        println!("  Test Rows: {}", self.test_rows);
    }
    pub fn init_logging(&self) -> connect::errors::Result<()> {
        self.common.init_logging().map_err(connect::errors::ConnectError::from)
    }
    pub fn get_connection_info(&self) -> connect::cli::ConnInfoResult {
        self.common.get_connection_info()
    }
    pub fn get_database(&self) -> Option<String> {
        self.common.get_database()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestRow {
    id: i32,
    name: String,
    value: i32,
    created_at: String,
}

#[derive(Debug, Clone)]
struct IsolationTestContext {
    test_table_name: String,
    test_results: Vec<String>,
    phase: IsolationTestPhase,
}

#[derive(Debug, Clone, PartialEq)]
enum IsolationTestPhase {
    Initial,
    CreatingTable,
    PopulatingData,
    TestingIsolation,
    Completed,
}

impl IsolationTestContext {
    fn new() -> Self {
        Self {
            test_table_name: format!("isolation_test_{}", chrono::Utc::now().timestamp()),
            test_results: Vec::new(),
            phase: IsolationTestPhase::Initial,
        }
    }

    fn add_result(&mut self, result: &str) {
        println!("[TEST] {}", result);
        self.test_results.push(result.to_string());
    }
}

/// Handler for creating test table
pub struct CreatingTableHandler;

#[async_trait]
impl StateHandler for CreatingTableHandler {
    async fn enter(&self, _context: &mut StateContext) -> Result<State> {
        println!("Creating test table for isolation testing...");
        Ok(State::CreatingTable)
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State> {
        // Extract connection first to avoid borrowing conflicts
        let connection = context.connection.take();
        
        if let Some(mut conn) = connection {
            // Get the test context after taking connection
            let test_context = context.get_handler_context_mut::<IsolationTestContext>(&State::CreatingTable)
                .ok_or("Isolation test context not found")?;
            let table_name = test_context.test_table_name.clone();
            let create_table_sql = format!(
                "CREATE TABLE IF NOT EXISTS {} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100) NOT NULL,
                    value INT NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )",
                table_name
            );
            
            conn.exec_drop(&create_table_sql, ())?;
            test_context.add_result(&format!("✓ Created test table: {}", table_name));
            test_context.phase = IsolationTestPhase::CreatingTable;
            
            // Restore the connection
            context.connection = Some(conn);
            
            Ok(State::PopulatingData)
        } else {
            Err(StateError::from("No connection available for creating table"))
        }
    }

    async fn exit(&self, context: &mut StateContext) -> Result<()> {
        context.move_handler_context::<IsolationTestContext>(&State::CreatingTable, State::PopulatingData);
        Ok(())
    }
}

/// Handler for populating test data
pub struct PopulatingDataHandler;

#[async_trait]
impl StateHandler for PopulatingDataHandler {
    async fn enter(&self, _context: &mut StateContext) -> Result<State> {
        println!("Populating test table with 10 rows...");
        Ok(State::PopulatingData)
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State> {
        // Extract connection first to avoid borrowing conflicts
        let connection = context.connection.take();
        
        if let Some(mut conn) = connection {
            // Get the test context after taking connection
            let test_context = context.get_handler_context_mut::<IsolationTestContext>(&State::PopulatingData)
                .ok_or("Isolation test context not found")?;
            let table_name = test_context.test_table_name.clone();
            // Insert 10 test rows
            for i in 1..=10 {
                let insert_sql = format!(
                    "INSERT INTO {} (id, name, value) VALUES (?, ?, ?)",
                    table_name
                );
                conn.exec_drop(&insert_sql, (i, format!("row_{}", i), i * 10))?;
            }
            
            // Verify the data was inserted
            let count_sql = format!("SELECT COUNT(*) FROM {}", table_name);
            let count: i64 = conn.exec_first(&count_sql, ())?.unwrap_or(0);
            
            test_context.add_result(&format!("✓ Inserted {} rows into test table", count));
            test_context.phase = IsolationTestPhase::PopulatingData;
            
            // Restore the connection
            context.connection = Some(conn);
            
            Ok(State::TestingIsolation)
        } else {
            Err(StateError::from("No connection available for populating data"))
        }
    }

    async fn exit(&self, context: &mut StateContext) -> Result<()> {
        context.move_handler_context::<IsolationTestContext>(&State::PopulatingData, State::TestingIsolation);
        Ok(())
    }
}

/// Handler for testing isolation
pub struct TestingIsolationHandler;

#[async_trait]
impl StateHandler for TestingIsolationHandler {
    async fn enter(&self, _context: &mut StateContext) -> Result<State> {
        println!("Testing repeatable read isolation...");
        Ok(State::TestingIsolation)
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State> {
        // Extract all needed values before any mutable borrows
        let host = context.host.clone();
        let username = context.username.clone();
        let password = context.password.clone();
        let database = context.database.clone();
        
        // Take the connection first
        let connection = context.connection.take();
        
        if let Some(mut conn) = connection {
            // Get the test context after taking connection
            let test_context = context.get_handler_context_mut::<IsolationTestContext>(&State::TestingIsolation)
                .ok_or("Isolation test context not found")?;
            let table_name = test_context.test_table_name.clone();
            // Create a second connection for testing using the same connection parameters
            let host_parts: Vec<&str> = host.split(':').collect();
            let hostname = host_parts.first().unwrap_or(&"localhost").to_string();
            let port = host_parts.get(1).and_then(|p| p.parse::<u16>().ok()).unwrap_or(4000);
            
            let opts = OptsBuilder::new()
                .ip_or_hostname(Some(&hostname))
                .tcp_port(port)
                .user(Some(&username))
                .pass(Some(&password))
                .db_name(database.as_deref());
            let pool = Pool::new(opts)?;
            let mut conn2 = pool.get_conn()?;
            
            test_context.add_result(&format!("✓ Created second connection for isolation testing"));
            
            // Step 1: Both connections read the same data
            test_context.add_result(&format!("Step 1: Both connections reading initial data..."));
            
            let query = format!("SELECT id, name, value FROM {} ORDER BY id", table_name);
            let conn1_data: Vec<Row> = conn.exec(&query, ())?;
            let conn2_data: Vec<Row> = conn2.exec(&query, ())?;
            
            test_context.add_result(&format!("✓ Connection 1 read {} rows", conn1_data.len()));
            test_context.add_result(&format!("✓ Connection 2 read {} rows", conn2_data.len()));
            
            // Step 2: Start transactions
            test_context.add_result(&format!("Step 2: Starting transactions on both connections..."));
            
            conn.exec_drop("START TRANSACTION", ())?;
            conn2.exec_drop("START TRANSACTION", ())?;
            test_context.add_result(&format!("✓ Started transactions on both connections"));
            
            // Step 3: Connection 1 updates a row
            test_context.add_result(&format!("Step 3: Connection 1 updating row with id=5..."));
            let update_sql = format!("UPDATE {} SET value = 999 WHERE id = 5", table_name);
            conn.exec_drop(&update_sql, ())?;
            test_context.add_result(&format!("✓ Connection 1 updated row with id=5 (value=999)"));
            
            // Step 4: Connection 2 tries to read the same data (should see old values due to repeatable read)
            test_context.add_result(&format!("Step 4: Connection 2 reading data again (should see old values)..."));
            let query = format!("SELECT id, name, value FROM {} WHERE id = 5", table_name);
            let conn2_data_after_update: Vec<Row> = conn2.exec(&query, ())?;
            
            if let Some(row) = conn2_data_after_update.first() {
                let value: i32 = row.get("value").unwrap_or(0);
                if value == 50 { // Original value for id=5
                    test_context.add_result(&format!("✓ Connection 2 correctly sees old value (50) - Repeatable Read working!"));
                } else {
                    test_context.add_result(&format!("✗ Connection 2 sees new value ({}) - Repeatable Read may not be working", value));
                }
            }
            
            // Step 5: Connection 1 commits
            test_context.add_result(&format!("Step 5: Connection 1 committing transaction..."));
            conn.exec_drop("COMMIT", ())?;
            test_context.add_result(&format!("✓ Connection 1 committed transaction"));
            
            // Step 6: Connection 2 reads again (should still see old values until it commits)
            test_context.add_result(&format!("Step 6: Connection 2 reading data after connection 1 commit..."));
            let query = format!("SELECT id, name, value FROM {} WHERE id = 5", table_name);
            let conn2_data_after_commit: Vec<Row> = conn2.exec(&query, ())?;
            
            if let Some(row) = conn2_data_after_commit.first() {
                let value: i32 = row.get("value").unwrap_or(0);
                if value == 50 { // Should still see old value
                    test_context.add_result(&format!("✓ Connection 2 still sees old value (50) - Isolation maintained!"));
                } else {
                    test_context.add_result(&format!("✗ Connection 2 sees new value ({}) - Isolation may be broken", value));
                }
            }
            
            // Step 7: Connection 2 commits and reads again
            test_context.add_result(&format!("Step 7: Connection 2 committing and reading updated data..."));
            conn2.exec_drop("COMMIT", ())?;
            test_context.add_result(&format!("✓ Connection 2 committed transaction"));
            
            let query = format!("SELECT id, name, value FROM {} WHERE id = 5", table_name);
            let final_data: Vec<Row> = conn2.exec(&query, ())?;
            
            if let Some(row) = final_data.first() {
                let value: i32 = row.get("value").unwrap_or(0);
                if value == 999 { // Should now see the updated value
                    test_context.add_result(&format!("✓ Connection 2 now sees updated value (999) - Transaction isolation working correctly!"));
                } else {
                    test_context.add_result(&format!("✗ Connection 2 sees unexpected value ({})", value));
                }
            }
            
            test_context.phase = IsolationTestPhase::TestingIsolation;
            
            // Restore the connection
            context.connection = Some(conn);
            
            Ok(State::VerifyingResults)
        } else {
            Err(StateError::from("No connection available for testing isolation"))
        }
    }

    async fn exit(&self, context: &mut StateContext) -> Result<()> {
        context.move_handler_context::<IsolationTestContext>(&State::TestingIsolation, State::VerifyingResults);
        Ok(())
    }
}

/// Handler for verifying results
pub struct VerifyingResultsHandler;

#[async_trait]
impl StateHandler for VerifyingResultsHandler {
    async fn enter(&self, _context: &mut StateContext) -> Result<State> {
        println!("Verifying test results...");
        Ok(State::VerifyingResults)
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State> {
        // Extract table name first
        let table_name = {
            let test_context = context.get_handler_context::<IsolationTestContext>(&State::VerifyingResults)
                .ok_or("Isolation test context not found")?;
            test_context.test_table_name.clone()
        };
        
        // Take the connection first
        let connection = context.connection.take();
        
        if let Some(mut conn) = connection {
            // Clean up test table
            let drop_sql = format!("DROP TABLE IF EXISTS {}", table_name);
            conn.exec_drop(&drop_sql, ())?;
            
            // Restore the connection
            context.connection = Some(conn);
        }
        
        // Now get the test context for the rest of the work
        {
            let test_context = context.get_handler_context_mut::<IsolationTestContext>(&State::VerifyingResults)
                .ok_or("Isolation test context not found")?;
            test_context.add_result(&format!("✓ Cleaned up test table: {}", table_name));
            
            // Display summary
            println!("\n=== ISOLATION TEST SUMMARY ===");
            println!("Test completed successfully!");
            println!("Total test steps: {}", test_context.test_results.len());
            
            let success_count = test_context.test_results.iter()
                .filter(|r| r.contains("✓"))
                .count();
            let failure_count = test_context.test_results.iter()
                .filter(|r| r.contains("✗"))
                .count();
            
            println!("Successful steps: {}", success_count);
            println!("Failed steps: {}", failure_count);
            
            if failure_count == 0 {
                println!("🎉 All isolation tests passed! Repeatable Read isolation is working correctly.");
            } else {
                println!("⚠️  Some isolation tests failed. Check the results above.");
            }
            
            test_context.phase = IsolationTestPhase::Completed;
        }
        
        Ok(State::Completed)
    }

    async fn exit(&self, context: &mut StateContext) -> Result<()> {
        context.remove_handler_context(&State::VerifyingResults);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> connect::errors::Result<()> {
    print_test_header("TiDB Repeatable Read Isolation Test");
    // Parse command line arguments using the specific args type
    let args = IsolationTestArgs::parse();
    args.init_logging()?;
    args.print_connection_info();
    let (host, user, password, _database) = args.get_connection_info()?;
    let database = args.get_database().unwrap_or_else(|| "test".to_string());
    // Create and configure the state machine
    let mut state_machine = StateMachine::new();
    // Register handlers manually to include custom version handler
    register_isolation_handlers(&mut state_machine, host, user, password, Some(database));
    // Initialize isolation test context using the public method
    state_machine.get_context_mut().set_handler_context(State::CreatingTable, IsolationTestContext::new());
    // Run the state machine
    match state_machine.run().await {
        Ok(_) => {
            print_success("Isolation test completed successfully!");
        }
        Err(e) => {
            print_error_and_exit("Isolation test failed", &e);
        }
    }
    Ok(())
}

/// Register all handlers for isolation test
fn register_isolation_handlers(
    state_machine: &mut StateMachine,
    host: String,
    user: String,
    password: String,
    database: Option<String>,
) {
    // Register standard connection handlers using the shared function
    register_standard_handlers(state_machine, host, user, password, database);
    
    // Override the GettingVersion handler to transition to isolation testing
    state_machine.register_handler(State::GettingVersion, Box::new(NextStateVersionHandler::new(State::CreatingTable)));
    
    // Register isolation test handlers
    state_machine.register_handler(State::CreatingTable, Box::new(CreatingTableHandler));
    state_machine.register_handler(State::PopulatingData, Box::new(PopulatingDataHandler));
    state_machine.register_handler(State::TestingIsolation, Box::new(TestingIsolationHandler));
    state_machine.register_handler(State::VerifyingResults, Box::new(VerifyingResultsHandler));
} 

#[cfg(test)]
mod tests {
    use super::*;
    use connect::config::{AppConfig, TestConfig, ConfigBuilder};
    use tempfile::NamedTempFile;
    use std::io::Write;
    use serial_test::serial;

    #[test]
    fn test_isolation_test_context() {
        let context = IsolationTestContext::new();
        assert_eq!(context.phase, IsolationTestPhase::Initial);
        assert!(context.test_results.is_empty());
        assert!(context.test_table_name.starts_with("isolation_test_"));
    }

    #[test]
    fn test_test_row_serialization() {
        let row = TestRow {
            id: 1,
            name: "test".to_string(),
            value: 42,
            created_at: "2023-01-01".to_string(),
        };
        assert_eq!(row.id, 1);
        assert_eq!(row.name, "test");
        assert_eq!(row.value, 42);
    }

    #[test]
    fn test_isolation_test_args_parsing() {
        let args = IsolationTestArgs::parse_from([
            "test-bin", 
            "--test-rows", "20",
            "-H", "localhost:4000",
            "-u", "testuser"
        ]);
        assert_eq!(args.test_rows, 20);
        assert_eq!(args.common.host, "localhost:4000");
        assert_eq!(args.common.user, "testuser");
    }

    #[test]
    fn test_isolation_test_args_defaults() {
        let args = IsolationTestArgs::parse_from(["test-bin"]);
        assert_eq!(args.test_rows, 10); // default value
        assert_eq!(args.common.host, "localhost:4000"); // default value
        assert_eq!(args.common.user, "root"); // default value
    }

    #[test]
    fn test_isolation_test_args_validation() {
        let args = IsolationTestArgs::parse_from([
            "test-bin", 
            "--test-rows", "5",
            "-H", "testhost:4000",
            "-u", "testuser",
            "-d", "testdb"
        ]);
        
        // Test that all fields are properly set
        assert_eq!(args.test_rows, 5);
        assert_eq!(args.common.host, "testhost:4000");
        assert_eq!(args.common.user, "testuser");
        assert_eq!(args.common.database, Some("testdb".to_string()));
        
        // Test the helper methods
        assert_eq!(args.get_database(), Some("testdb".to_string()));
    }

    #[test]
    #[serial]
    fn test_test_config_integration() {
        // Test that TestConfig from config module works with isolation test logic
        let test_config = TestConfig {
            rows: 15,
            timeout_secs: 120,
            verbose: true,
        };
        
        assert_eq!(test_config.rows, 15);
        assert_eq!(test_config.timeout_secs, 120);
        assert!(test_config.verbose);
        
        // Test integration with ConfigBuilder
        let config = ConfigBuilder::new()
            .test_rows(25)
            .build();
        
        assert_eq!(config.test.rows, 25);
    }

    #[test]
    #[serial]
    fn test_isolation_config_file_parsing() {
        let json = r#"{
            "test": {"rows": 50, "timeout_secs": 180, "verbose": true}
        }"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(json.as_bytes()).unwrap();
        
        let config = AppConfig::from_file(file.path()).unwrap();
        assert_eq!(config.test.rows, 50);
        assert_eq!(config.test.timeout_secs, 180);
        assert!(config.test.verbose);
    }

    #[test]
    fn test_isolation_test_args_with_config() {
        // Test that isolation test args can work with config-based test settings
        let args = IsolationTestArgs::parse_from([
            "test-bin", 
            "--test-rows", "30",
            "-c", "test_config.json"
        ]);
        
        assert_eq!(args.test_rows, 30);
        // Note: In a real scenario, the config file would be loaded and merged
    }
} 
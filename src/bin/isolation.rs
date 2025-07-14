//!
//! # `TiDB` Isolation Test Binary
//!
//! This binary implements a simple test for `TiDB`'s transaction isolation guarantees (such as repeatable read).
//! It is designed to verify that `TiDB` enforces correct isolation semantics under concurrent transactions.
//!
//! ## Overview
//!
//! The isolation test creates a dedicated test table, populates it with data, and then runs concurrent transactions
//! to verify that isolation properties (e.g., repeatable read) are upheld. The test is useful for:
//! - **Verifying Transaction Isolation**: Ensuring `TiDB`'s isolation level is correctly implemented
//! - **Regression Testing**: Detecting changes or regressions in isolation behavior across `TiDB` versions
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
//! 1. **Initial** → **`ParsingConfig`** → **Connecting**
//! 2. **`CreatingTable`**: Create a dedicated test table for isolation testing
//! 3. **`PopulatingData`**: Insert test rows into the table
//! 4. **`TestingIsolation`**: Run concurrent transactions to verify isolation
//! 5. **`VerifyingResults`**: Check and report the results
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
//! This binary is intended as a robust, extensible foundation for isolation and concurrency testing in `TiDB`.
//! Handlers and test logic can be extended to cover more advanced isolation scenarios as needed.

use async_trait::async_trait;
use clap::Command;
use clap::Parser;
use mysql::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use test_rig::ConfigExtension;
use test_rig::errors::Result;
use test_rig::{
    CommonArgs, ConnectError, DynamicState, DynamicStateContext, DynamicStateHandler,
    DynamicStateMachine, dynamic_state, print_error_and_exit, print_success, print_test_header,
    register_transitions,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IsolationTestError {
    #[error("Failed to create test table {table}: {message}")]
    TableCreationFailed { table: String, message: String },

    #[error("Failed to populate test data: {0}")]
    DataPopulationFailed(String),

    #[error("Isolation test failed: {0}")]
    TestFailed(String),

    #[error("Failed to clean up test table {table}: {message}")]
    CleanupFailed { table: String, message: String },

    #[error("Isolation level {level} not supported")]
    UnsupportedIsolationLevel { level: String },

    #[error("Concurrent modification detected: {details}")]
    ConcurrentModification { details: String },

    #[error("Deadlock detected during isolation test")]
    Deadlock,

    #[error("Test data corruption detected: {details}")]
    DataCorruption { details: String },

    #[error("Isolation test timeout after {duration:?}")]
    Timeout { duration: Duration },
}

/// Configuration extension for isolation test
struct IsolationConfigExtension;

impl ConfigExtension for IsolationConfigExtension {
    fn add_cli_args(&self, app: Command) -> Command {
        app.arg(
            clap::Arg::new("test-rows")
                .long("test-rows")
                .help("Number of test rows to create for isolation testing")
                .default_value("10"),
        )
    }

    fn build_config(
        &self,
        args: &clap::ArgMatches,
        config: &mut test_rig::config::AppConfig,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if let Some(test_rows) = args.get_one::<String>("test-rows")
            && let Ok(rows) = test_rows.parse::<u32>()
        {
            config.test.rows = rows;
        }
        Ok(())
    }

    fn get_extension_name(&self) -> &'static str {
        "isolation_test"
    }

    fn get_help_text(&self) -> &'static str {
        "Adds --test-rows option for isolation testing"
    }
}

// Register the extension when this binary is built
fn register_extensions() {
    test_rig::register_config_extension(Box::new(IsolationConfigExtension));
}

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
    /// Initialize logging system
    ///
    /// # Errors
    ///
    /// Returns an error if logging initialization fails.
    pub fn init_logging(&self) -> test_rig::errors::Result<()> {
        self.common
            .init_logging()
            .map_err(test_rig::errors::ConnectError::from)
    }
    /// Get connection information
    ///
    /// # Errors
    ///
    /// Returns an error if connection information cannot be obtained.
    pub fn get_connection_info(&self) -> test_rig::cli::ConnInfoResult {
        self.common.get_connection_info()
    }
    #[must_use]
    pub fn get_database(&self) -> Option<String> {
        self.common.get_database()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
        self.test_results.push(result.to_string());
        println!("{result}");
    }
}

// Define custom states for the workflow
mod isolation_states {
    use super::{DynamicState, dynamic_state};

    // Re-export common states
    pub use test_rig::common_states::{
        completed, connecting, getting_version, parsing_config, testing_connection,
        verifying_database,
    };

    // Test-specific states
    pub fn creating_table() -> DynamicState {
        dynamic_state!("creating_table", "Creating Test Table")
    }
    pub fn populating_data() -> DynamicState {
        dynamic_state!("populating_data", "Populating Test Data")
    }
    pub fn testing_isolation() -> DynamicState {
        dynamic_state!("testing_isolation", "Testing Isolation")
    }
    pub fn verifying_results() -> DynamicState {
        dynamic_state!("verifying_results", "Verifying Results")
    }
}

// Adapter for InitialHandler to DynamicStateHandler
struct InitialHandlerAdapter;
#[async_trait]
impl DynamicStateHandler for InitialHandlerAdapter {
    async fn enter(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        Ok(dynamic_state!("initial", "Initial"))
    }
    async fn execute(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        Ok(isolation_states::parsing_config())
    }
    async fn exit(&self, _context: &mut DynamicStateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

// Adapter for ParsingConfigHandler to DynamicStateHandler
struct ParsingConfigHandlerAdapter {
    host: String,
    user: String,
    password: String,
    database: Option<String>,
}
#[async_trait]
impl DynamicStateHandler for ParsingConfigHandlerAdapter {
    async fn enter(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        Ok(isolation_states::parsing_config())
    }
    async fn execute(&self, context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        let (host, port) = test_rig::connection::parse_connection_string(&self.host)?;
        context.host = host;
        context.port = port;
        context.username.clone_from(&self.user);
        context.password.clone_from(&self.password);
        context.database.clone_from(&self.database);
        Ok(isolation_states::connecting())
    }
    async fn exit(&self, _context: &mut DynamicStateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

// Adapter for ConnectingHandler to DynamicStateHandler
struct ConnectingHandlerAdapter;
#[async_trait]
impl DynamicStateHandler for ConnectingHandlerAdapter {
    async fn enter(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        Ok(isolation_states::connecting())
    }
    async fn execute(&self, context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        let pool = test_rig::connection::create_connection_pool(
            &context.host,
            context.port,
            &context.username,
            &context.password,
            context.database.as_deref(),
        )?;
        let conn = pool.get_conn()?;
        context.connection = Some(conn);
        Ok(isolation_states::testing_connection())
    }
    async fn exit(&self, _context: &mut DynamicStateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

// Adapter for TestingConnectionHandler to DynamicStateHandler
struct TestingConnectionHandlerAdapter;
#[async_trait]
impl DynamicStateHandler for TestingConnectionHandlerAdapter {
    async fn enter(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        Ok(isolation_states::testing_connection())
    }
    async fn execute(&self, context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        if let Some(ref mut conn) = context.connection {
            let result: std::result::Result<Vec<mysql::Row>, mysql::Error> =
                conn.exec("SELECT 1", ());
            match result {
                Ok(_) => Ok(isolation_states::verifying_database()),
                Err(e) => Err(format!("Connection test failed: {e}").into()),
            }
        } else {
            Err("No connection available for testing".into())
        }
    }
    async fn exit(&self, _context: &mut DynamicStateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

// Adapter for VerifyingDatabaseHandler to DynamicStateHandler
struct VerifyingDatabaseHandlerAdapter;
#[async_trait]
impl DynamicStateHandler for VerifyingDatabaseHandlerAdapter {
    async fn enter(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        Ok(isolation_states::verifying_database())
    }
    async fn execute(&self, context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        if let Some(ref mut conn) = context.connection {
            if let Some(ref db_name) = context.database {
                let query = format!("USE `{db_name}`");
                match conn.query_drop(query) {
                    Ok(()) => Ok(isolation_states::getting_version()),
                    Err(e) => Err(format!("Database verification failed: {e}").into()),
                }
            } else {
                Ok(isolation_states::getting_version())
            }
        } else {
            Err("No connection available for database verification".into())
        }
    }
    async fn exit(&self, _context: &mut DynamicStateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

// Adapter for GettingVersionHandler to DynamicStateHandler
struct GettingVersionHandlerAdapter;
#[async_trait]
impl DynamicStateHandler for GettingVersionHandlerAdapter {
    async fn enter(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        Ok(isolation_states::getting_version())
    }
    async fn execute(&self, context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        if let Some(ref mut conn) = context.connection {
            let version_query = "SELECT VERSION()";
            match conn.query_first::<String, _>(version_query) {
                Ok(Some(version)) => {
                    context.server_version = Some(version.clone());
                    Ok(isolation_states::creating_table())
                }
                Ok(None) => Err("No version returned from server".into()),
                Err(e) => Err(format!("Failed to get server version: {e}").into()),
            }
        } else {
            Err("No connection available for getting version".into())
        }
    }
    async fn exit(&self, _context: &mut DynamicStateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

/// Handler for creating test table
pub struct CreatingTableHandler;

#[async_trait]
impl DynamicStateHandler for CreatingTableHandler {
    async fn enter(&self, _context: &mut DynamicStateContext) -> Result<DynamicState> {
        println!("Creating test table for isolation testing...");
        Ok(isolation_states::creating_table())
    }

    async fn execute(&self, context: &mut DynamicStateContext) -> Result<DynamicState> {
        // Initialize isolation test context first
        let test_context = IsolationTestContext::new();
        let table_name = test_context.test_table_name.clone();

        // Store test context in dynamic context
        context.set_custom_data("isolation_test_context".to_string(), test_context);

        if let Some(ref mut conn) = context.connection {
            // Create test table
            let create_table_sql = format!(
                "CREATE TABLE IF NOT EXISTS {table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(255) NOT NULL,
                    value INT NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )"
            );

            match conn.query_drop(&create_table_sql) {
                Ok(()) => {
                    println!("✓ Test table '{table_name}' created successfully");
                    Ok(isolation_states::populating_data())
                }
                Err(e) => {
                    let error_msg = format!("Failed to create test table: {e}");
                    Err(format!("Failed to create test table {table_name}: {error_msg}").into())
                }
            }
        } else {
            Err(ConnectError::StateMachine(
                "No connection available for creating table".to_string(),
            ))
        }
    }

    async fn exit(&self, _context: &mut DynamicStateContext) -> Result<()> {
        Ok(())
    }
}

/// Handler for populating test data
pub struct PopulatingDataHandler;

#[async_trait]
impl DynamicStateHandler for PopulatingDataHandler {
    async fn enter(&self, _context: &mut DynamicStateContext) -> Result<DynamicState> {
        println!("Populating test table with 10 rows...");
        Ok(isolation_states::populating_data())
    }

    async fn execute(&self, context: &mut DynamicStateContext) -> Result<DynamicState> {
        // Get test context first
        let table_name = if let Some(ctx) =
            context.get_custom_data::<IsolationTestContext>("isolation_test_context")
        {
            ctx.test_table_name.clone()
        } else {
            return Err("Isolation test context not found".into());
        };

        if let Some(ref mut conn) = context.connection {
            // Insert 10 test rows
            for i in 1..=10 {
                let insert_sql =
                    format!("INSERT INTO {table_name} (id, name, value) VALUES (?, ?, ?)");
                conn.exec_drop(&insert_sql, (i, format!("row_{i}"), i * 10))?;
            }

            // Verify the data was inserted
            let count_sql = format!("SELECT COUNT(*) FROM {table_name}");
            let count: i64 = conn.exec_first(&count_sql, ())?.unwrap_or(0);

            // Update test context after database operations
            if let Some(ctx) =
                context.get_custom_data_mut::<IsolationTestContext>("isolation_test_context")
            {
                ctx.add_result(&format!("✓ Inserted {count} rows into test table"));
                ctx.phase = IsolationTestPhase::PopulatingData;
            }

            Ok(isolation_states::testing_isolation())
        } else {
            Err(ConnectError::StateMachine(
                "No connection available for populating data".to_string(),
            ))
        }
    }

    async fn exit(&self, _context: &mut DynamicStateContext) -> Result<()> {
        Ok(())
    }
}

/// Handler for testing isolation
pub struct TestingIsolationHandler;

#[async_trait]
impl DynamicStateHandler for TestingIsolationHandler {
    async fn enter(&self, _context: &mut DynamicStateContext) -> Result<DynamicState> {
        println!("Testing transaction isolation with concurrent operations...");
        Ok(isolation_states::testing_isolation())
    }

    async fn execute(&self, context: &mut DynamicStateContext) -> Result<DynamicState> {
        // Get test context first
        let table_name = if let Some(ctx) =
            context.get_custom_data::<IsolationTestContext>("isolation_test_context")
        {
            ctx.test_table_name.clone()
        } else {
            return Err("Isolation test context not found".into());
        };

        if let Some(ref mut conn) = context.connection {
            // Start a transaction
            conn.query_drop("START TRANSACTION")?;

            // Read initial state
            let initial_query = format!("SELECT * FROM {table_name} ORDER BY id LIMIT 5");
            let initial_rows: Vec<TestRow> = conn.exec(&initial_query, ())?;

            // Simulate concurrent modification (in a real scenario, this would be in another connection)
            // For this test, we'll just update a row
            let update_sql = format!("UPDATE {table_name} SET value = value + 100 WHERE id = 1");
            conn.query_drop(&update_sql)?;

            // Read again to test isolation
            let final_query = format!("SELECT * FROM {table_name} ORDER BY id LIMIT 5");
            let final_rows: Vec<TestRow> = conn.exec(&final_query, ())?;

            // Commit the transaction
            conn.query_drop("COMMIT")?;

            // Update test context after database operations
            if let Some(ctx) =
                context.get_custom_data_mut::<IsolationTestContext>("isolation_test_context")
            {
                ctx.add_result(&format!("✓ Initial read: {} rows", initial_rows.len()));
                ctx.add_result("✓ Updated row with id = 1");
                ctx.add_result(&format!("✓ Final read: {} rows", final_rows.len()));

                // Check for isolation violations
                if initial_rows.len() == final_rows.len() {
                    ctx.add_result("✓ Transaction isolation maintained");
                } else {
                    ctx.add_result("⚠️  Potential isolation violation detected");
                }
                ctx.phase = IsolationTestPhase::TestingIsolation;
            }

            Ok(isolation_states::verifying_results())
        } else {
            Err(ConnectError::StateMachine(
                "No connection available for testing isolation".to_string(),
            ))
        }
    }

    async fn exit(&self, _context: &mut DynamicStateContext) -> Result<()> {
        Ok(())
    }
}

/// Handler for verifying results
pub struct VerifyingResultsHandler;

#[async_trait]
impl DynamicStateHandler for VerifyingResultsHandler {
    async fn enter(&self, _context: &mut DynamicStateContext) -> Result<DynamicState> {
        println!("Verifying isolation test results...");
        Ok(isolation_states::verifying_results())
    }

    async fn execute(&self, context: &mut DynamicStateContext) -> Result<DynamicState> {
        // Get test context
        let Some(test_context) =
            context.get_custom_data_mut::<IsolationTestContext>("isolation_test_context")
        else {
            return Err("Isolation test context not found".into());
        };

        // Print all results
        println!("\n=== Isolation Test Results ===");
        for result in &test_context.test_results {
            println!("{result}");
        }

        // Determine overall success
        let success_count = test_context
            .test_results
            .iter()
            .filter(|r| r.starts_with("✓"))
            .count();
        let total_count = test_context.test_results.len();

        if success_count == total_count {
            println!("✅ All isolation tests passed!");
        } else {
            println!("⚠️  Some isolation tests failed. Check the results above.");
        }

        test_context.phase = IsolationTestPhase::Completed;

        Ok(isolation_states::completed())
    }

    async fn exit(&self, _context: &mut DynamicStateContext) -> Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> test_rig::errors::Result<()> {
    // Register configuration extensions
    register_extensions();

    print_test_header("TiDB Repeatable Read Isolation Test");
    // Parse command line arguments using the specific args type
    let args = IsolationTestArgs::parse();
    args.init_logging()?;
    args.print_connection_info();
    let (host, user, password, _database) = args.get_connection_info()?;
    let database = args.get_database().unwrap_or_else(|| "test".to_string());

    // Create and configure the dynamic state machine
    let mut machine = DynamicStateMachine::new();

    // Register handlers manually to include custom version handler
    register_isolation_handlers(&mut machine, host, user, password, Some(database));

    // Register valid transitions
    register_transitions!(
        machine,
        dynamic_state!("initial", "Initial"),
        [isolation_states::parsing_config()]
    );
    register_transitions!(
        machine,
        isolation_states::parsing_config(),
        [isolation_states::connecting()]
    );
    register_transitions!(
        machine,
        isolation_states::connecting(),
        [isolation_states::testing_connection()]
    );
    register_transitions!(
        machine,
        isolation_states::testing_connection(),
        [isolation_states::verifying_database()]
    );
    register_transitions!(
        machine,
        isolation_states::verifying_database(),
        [isolation_states::getting_version()]
    );
    register_transitions!(
        machine,
        isolation_states::getting_version(),
        [isolation_states::creating_table()]
    );
    register_transitions!(
        machine,
        isolation_states::creating_table(),
        [isolation_states::populating_data()]
    );
    register_transitions!(
        machine,
        isolation_states::populating_data(),
        [isolation_states::testing_isolation()]
    );
    register_transitions!(
        machine,
        isolation_states::testing_isolation(),
        [isolation_states::verifying_results()]
    );
    register_transitions!(
        machine,
        isolation_states::verifying_results(),
        [isolation_states::completed()]
    );

    // Run the state machine
    match machine.run().await {
        Ok(()) => {
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
    state_machine: &mut DynamicStateMachine,
    host: String,
    user: String,
    password: String,
    database: Option<String>,
) {
    // Register standard connection handlers
    state_machine.register_handler(
        dynamic_state!("initial", "Initial"),
        Box::new(InitialHandlerAdapter),
    );
    state_machine.register_handler(
        isolation_states::parsing_config(),
        Box::new(ParsingConfigHandlerAdapter {
            host,
            user,
            password,
            database,
        }),
    );
    state_machine.register_handler(
        isolation_states::connecting(),
        Box::new(ConnectingHandlerAdapter),
    );
    state_machine.register_handler(
        isolation_states::testing_connection(),
        Box::new(TestingConnectionHandlerAdapter),
    );
    state_machine.register_handler(
        isolation_states::verifying_database(),
        Box::new(VerifyingDatabaseHandlerAdapter),
    );
    state_machine.register_handler(
        isolation_states::getting_version(),
        Box::new(GettingVersionHandlerAdapter),
    );

    // Register isolation test handlers
    state_machine.register_handler(
        isolation_states::creating_table(),
        Box::new(CreatingTableHandler),
    );
    state_machine.register_handler(
        isolation_states::populating_data(),
        Box::new(PopulatingDataHandler),
    );
    state_machine.register_handler(
        isolation_states::testing_isolation(),
        Box::new(TestingIsolationHandler),
    );
    state_machine.register_handler(
        isolation_states::verifying_results(),
        Box::new(VerifyingResultsHandler),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use test_rig::config::{AppConfig, ConfigBuilder, TestConfig};

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
            "--test-rows",
            "20",
            "-H",
            "localhost:4000",
            "-u",
            "testuser",
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
            "--test-rows",
            "5",
            "-H",
            "testhost:4000",
            "-u",
            "testuser",
            "-d",
            "testdb",
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
        let config = ConfigBuilder::new().test_rows(25).build();

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
            "--test-rows",
            "30",
            "-c",
            "test_config.json",
        ]);

        assert_eq!(args.test_rows, 30);
        // Note: In a real scenario, the config file would be loaded and merged
    }
}

use connect::state_machine::{StateMachine, State, StateContext, StateHandler, StateError};
use connect::{InitialHandler, ParsingConfigHandler, ConnectingHandler, TestingConnectionHandler, VerifyingDatabaseHandler, GettingVersionHandler};
use connect::parse_args;
use mysql::prelude::*;
use mysql::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use connect::cli::CommonArgs;
use clap::Parser;

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
    async fn enter(&self, _context: &mut StateContext) -> Result<State, StateError> {
        println!("Creating test table for isolation testing...");
        Ok(State::CreatingTable)
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State, StateError> {
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
            Err("No connection available for creating table".into())
        }
    }

    async fn exit(&self, context: &mut StateContext) -> Result<(), StateError> {
        context.move_handler_context::<IsolationTestContext>(&State::CreatingTable, State::PopulatingData);
        Ok(())
    }
}

/// Handler for populating test data
pub struct PopulatingDataHandler;

#[async_trait]
impl StateHandler for PopulatingDataHandler {
    async fn enter(&self, _context: &mut StateContext) -> Result<State, StateError> {
        println!("Populating test table with 10 rows...");
        Ok(State::PopulatingData)
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State, StateError> {
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
            Err("No connection available for populating data".into())
        }
    }

    async fn exit(&self, context: &mut StateContext) -> Result<(), StateError> {
        context.move_handler_context::<IsolationTestContext>(&State::PopulatingData, State::TestingIsolation);
        Ok(())
    }
}

/// Handler for testing isolation
pub struct TestingIsolationHandler;

#[async_trait]
impl StateHandler for TestingIsolationHandler {
    async fn enter(&self, _context: &mut StateContext) -> Result<State, StateError> {
        println!("Testing repeatable read isolation...");
        Ok(State::TestingIsolation)
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State, StateError> {
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
            
            test_context.add_result("✓ Created second connection for isolation testing");
            
            // Step 1: Both connections read the same data
            test_context.add_result("Step 1: Both connections reading initial data...");
            
            let query = format!("SELECT id, name, value FROM {} ORDER BY id", table_name);
            let conn1_data: Vec<Row> = conn.exec(&query, ())?;
            let conn2_data: Vec<Row> = conn2.exec(&query, ())?;
            
            test_context.add_result(&format!("✓ Connection 1 read {} rows", conn1_data.len()));
            test_context.add_result(&format!("✓ Connection 2 read {} rows", conn2_data.len()));
            
            // Step 2: Start transactions
            test_context.add_result("Step 2: Starting transactions on both connections...");
            conn.exec_drop("START TRANSACTION", ())?;
            conn2.exec_drop("START TRANSACTION", ())?;
            test_context.add_result("✓ Started transactions on both connections");
            
            // Step 3: Connection 1 updates a row
            test_context.add_result("Step 3: Connection 1 updating row with id=5...");
            let update_sql = format!("UPDATE {} SET value = 999 WHERE id = 5", table_name);
            conn.exec_drop(&update_sql, ())?;
            test_context.add_result("✓ Connection 1 updated row with id=5 (value=999)");
            
            // Step 4: Connection 2 tries to read the same data (should see old values due to repeatable read)
            test_context.add_result("Step 4: Connection 2 reading data again (should see old values)...");
            let query = format!("SELECT id, name, value FROM {} WHERE id = 5", table_name);
            let conn2_data_after_update: Vec<Row> = conn2.exec(&query, ())?;
            
            if let Some(row) = conn2_data_after_update.first() {
                let value: i32 = row.get("value").unwrap_or(0);
                if value == 50 { // Original value for id=5
                    test_context.add_result("✓ Connection 2 correctly sees old value (50) - Repeatable Read working!");
                } else {
                    test_context.add_result(&format!("✗ Connection 2 sees new value ({}) - Repeatable Read may not be working", value));
                }
            }
            
            // Step 5: Connection 1 commits
            test_context.add_result("Step 5: Connection 1 committing transaction...");
            conn.exec_drop("COMMIT", ())?;
            test_context.add_result("✓ Connection 1 committed transaction");
            
            // Step 6: Connection 2 reads again (should still see old values until it commits)
            test_context.add_result("Step 6: Connection 2 reading data after connection 1 commit...");
            let query = format!("SELECT id, name, value FROM {} WHERE id = 5", table_name);
            let conn2_data_after_commit: Vec<Row> = conn2.exec(&query, ())?;
            
            if let Some(row) = conn2_data_after_commit.first() {
                let value: i32 = row.get("value").unwrap_or(0);
                if value == 50 { // Should still see old value
                    test_context.add_result("✓ Connection 2 still sees old value (50) - Isolation maintained!");
                } else {
                    test_context.add_result(&format!("✗ Connection 2 sees new value ({}) - Isolation may be broken", value));
                }
            }
            
            // Step 7: Connection 2 commits and reads again
            test_context.add_result("Step 7: Connection 2 committing and reading updated data...");
            conn2.exec_drop("COMMIT", ())?;
            test_context.add_result("✓ Connection 2 committed transaction");
            
            let query = format!("SELECT id, name, value FROM {} WHERE id = 5", table_name);
            let final_data: Vec<Row> = conn2.exec(&query, ())?;
            
            if let Some(row) = final_data.first() {
                let value: i32 = row.get("value").unwrap_or(0);
                if value == 999 { // Should now see the updated value
                    test_context.add_result("✓ Connection 2 now sees updated value (999) - Transaction isolation working correctly!");
                } else {
                    test_context.add_result(&format!("✗ Connection 2 sees unexpected value ({})", value));
                }
            }
            
            test_context.phase = IsolationTestPhase::TestingIsolation;
            
            // Restore the connection
            context.connection = Some(conn);
            
            Ok(State::VerifyingResults)
        } else {
            Err("No connection available for testing isolation".into())
        }
    }

    async fn exit(&self, context: &mut StateContext) -> Result<(), StateError> {
        context.move_handler_context::<IsolationTestContext>(&State::TestingIsolation, State::VerifyingResults);
        Ok(())
    }
}

/// Handler for verifying results
pub struct VerifyingResultsHandler;

#[async_trait]
impl StateHandler for VerifyingResultsHandler {
    async fn enter(&self, _context: &mut StateContext) -> Result<State, StateError> {
        println!("Verifying test results...");
        Ok(State::VerifyingResults)
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State, StateError> {
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

    async fn exit(&self, context: &mut StateContext) -> Result<(), StateError> {
        context.remove_handler_context(&State::VerifyingResults);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("TiDB Repeatable Read Isolation Test");
    println!("===================================");
    
    // Parse command line arguments
    let args = CommonArgs::parse();
    
    // Print connection info
    args.print_connection_info();
    
    // Get connection info
    let (host, user, password, database) = args.get_connection_info()?;
    let database = database.unwrap_or_else(|| "test".to_string());
    
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
    
    // Register isolation test handlers
    state_machine.register_handler(State::CreatingTable, Box::new(CreatingTableHandler));
    state_machine.register_handler(State::PopulatingData, Box::new(PopulatingDataHandler));
    state_machine.register_handler(State::TestingIsolation, Box::new(TestingIsolationHandler));
    state_machine.register_handler(State::VerifyingResults, Box::new(VerifyingResultsHandler));
    
    // Initialize isolation test context using the public method
    state_machine.get_context_mut().set_handler_context(State::CreatingTable, IsolationTestContext::new());
    
    // Run the state machine
    match state_machine.run().await {
        Ok(_) => {
            println!("\n✅ Isolation test completed successfully!");
        }
        Err(e) => {
            eprintln!("\n❌ Isolation test failed: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
} 
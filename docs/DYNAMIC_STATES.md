# Dynamic States System

This guide covers the dynamic states system in the test_rig framework, which allows you to create extensible workflows with custom states defined at compile time.

## Overview

The dynamic states system provides a flexible way to define custom workflows without modifying the core state machine. It's particularly useful for test-specific scenarios that require custom states and logic.

## Key Components

### Dynamic State Machine

The `DynamicStateMachine` drives extensible workflows with custom states:

```rust
use test_rig::{DynamicStateMachine, DynamicStateHandler, DynamicStateContext};

let mut machine = DynamicStateMachine::new();
// Register handlers for custom states
machine.run().await?;
```

### Dynamic States

Dynamic states are string-based and can be defined at compile time:

```rust
// Define custom states using the macro
dynamic_state!(creating_table);
dynamic_state!(populating_data);
dynamic_state!(testing_isolation);
dynamic_state!(cleanup);
```

### Dynamic State Handlers

Handlers implement the `DynamicStateHandler` trait:

```rust
use test_rig::{DynamicStateHandler, DynamicStateContext, ConnectError};

pub struct CreatingTableHandler;

#[async_trait]
impl DynamicStateHandler for CreatingTableHandler {
    async fn execute(&self, context: &mut DynamicStateContext) -> Result<String, ConnectError> {
        // Your custom logic here
        Ok("populating_data".to_string())
    }
}
```

## Usage Patterns

### Basic Dynamic Workflow

Create a simple dynamic workflow:

```rust
use test_rig::{DynamicStateMachine, DynamicStateHandler, DynamicStateContext, common_states::*};

// Define custom states
dynamic_state!(creating_table);
dynamic_state!(populating_data);
dynamic_state!(testing_queries);

// Create handlers
struct CreatingTableHandler;
struct PopulatingDataHandler;
struct TestingQueriesHandler;

#[async_trait]
impl DynamicStateHandler for CreatingTableHandler {
    async fn execute(&self, context: &mut DynamicStateContext) -> Result<String, ConnectError> {
        if let Some(conn) = &mut context.connection {
            conn.query_drop("DROP TABLE IF EXISTS test_table").await?;
            conn.query_drop("CREATE TABLE test_table (id INT, name VARCHAR(50))").await?;
            Ok("populating_data".to_string())
        } else {
            Ok("connecting".to_string())
        }
    }
}

#[async_trait]
impl DynamicStateHandler for PopulatingDataHandler {
    async fn execute(&self, context: &mut DynamicStateContext) -> Result<String, ConnectError> {
        if let Some(conn) = &mut context.connection {
            for i in 1..=10 {
                conn.query_drop(&format!(
                    "INSERT INTO test_table (id, name) VALUES ({}, 'Item{}')",
                    i, i
                )).await?;
            }
            Ok("testing_queries".to_string())
        } else {
            Ok("connecting".to_string())
        }
    }
}

#[async_trait]
impl DynamicStateHandler for TestingQueriesHandler {
    async fn execute(&self, context: &mut DynamicStateContext) -> Result<String, ConnectError> {
        if let Some(conn) = &mut context.connection {
            let result: Vec<mysql::Row> = conn.query("SELECT COUNT(*) FROM test_table").await?;
            println!("Table contains {} rows", result[0].get::<i32, _>(0).unwrap());
            Ok("completed".to_string())
        } else {
            Ok("connecting".to_string())
        }
    }
}

// Create and run the machine
let mut machine = DynamicStateMachine::new();

// Register handlers for common states
machine.register_handler(parsing_config(), Box::new(ParsingConfigHandlerAdapter));
machine.register_handler(connecting(), Box::new(ConnectingHandlerAdapter));
machine.register_handler(testing_connection(), Box::new(TestingConnectionHandlerAdapter));

// Register handlers for custom states
machine.register_handler(creating_table(), Box::new(CreatingTableHandler));
machine.register_handler(populating_data(), Box::new(PopulatingDataHandler));
machine.register_handler(testing_queries(), Box::new(TestingQueriesHandler));

machine.run().await?;
```

### Advanced Dynamic Workflow

Create a more complex workflow with conditional logic:

```rust
use test_rig::{DynamicStateMachine, DynamicStateHandler, DynamicStateContext, ConnectError};

// Define states for a complex test scenario
dynamic_state!(setup_environment);
dynamic_state!(create_test_data);
dynamic_state!(run_concurrent_operations);
dynamic_state!(verify_results);
dynamic_state!(cleanup_environment);

struct SetupEnvironmentHandler;

#[async_trait]
impl DynamicStateHandler for SetupEnvironmentHandler {
    async fn execute(&self, context: &mut DynamicStateContext) -> Result<String, ConnectError> {
        if let Some(conn) = &mut context.connection {
            // Create test database
            conn.query_drop("CREATE DATABASE IF NOT EXISTS test_db").await?;
            conn.query_drop("USE test_db").await?;
            
            // Store database name in context
            context.set_data("database_name", "test_db");
            
            Ok("create_test_data".to_string())
        } else {
            Ok("connecting".to_string())
        }
    }
}

struct CreateTestDataHandler;

#[async_trait]
impl DynamicStateHandler for CreateTestDataHandler {
    async fn execute(&self, context: &mut DynamicStateContext) -> Result<String, ConnectError> {
        if let Some(conn) = &mut context.connection {
            // Create test tables
            conn.query_drop("DROP TABLE IF EXISTS users").await?;
            conn.query_drop("DROP TABLE IF EXISTS orders").await?;
            
            conn.query_drop("""
                CREATE TABLE users (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    email VARCHAR(100),
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )
            """).await?;
            
            conn.query_drop("""
                CREATE TABLE orders (
                    id INT PRIMARY KEY,
                    user_id INT,
                    amount DECIMAL(10,2),
                    status VARCHAR(20),
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (user_id) REFERENCES users(id)
                )
            """).await?;
            
            // Insert initial data
            for i in 1..=5 {
                conn.query_drop(&format!(
                    "INSERT INTO users (id, name, email) VALUES ({}, 'User{}', 'user{}@example.com')",
                    i, i, i
                )).await?;
            }
            
            Ok("run_concurrent_operations".to_string())
        } else {
            Ok("connecting".to_string())
        }
    }
}

struct RunConcurrentOperationsHandler;

#[async_trait]
impl DynamicStateHandler for RunConcurrentOperationsHandler {
    async fn execute(&self, context: &mut DynamicStateContext) -> Result<String, ConnectError> {
        if let Some(conn) = &mut context.connection {
            // Simulate concurrent operations
            let operations = vec![
                "INSERT INTO orders (id, user_id, amount, status) VALUES (1, 1, 100.00, 'pending')",
                "INSERT INTO orders (id, user_id, amount, status) VALUES (2, 2, 200.00, 'pending')",
                "UPDATE orders SET status = 'completed' WHERE id = 1",
                "SELECT COUNT(*) FROM orders WHERE status = 'completed'",
            ];
            
            for operation in operations {
                conn.query_drop(operation).await?;
            }
            
            Ok("verify_results".to_string())
        } else {
            Ok("connecting".to_string())
        }
    }
}

struct VerifyResultsHandler;

#[async_trait]
impl DynamicStateHandler for VerifyResultsHandler {
    async fn execute(&self, context: &mut DynamicStateContext) -> Result<String, ConnectError> {
        if let Some(conn) = &mut context.connection {
            // Verify test results
            let user_count: Vec<mysql::Row> = conn.query("SELECT COUNT(*) FROM users").await?;
            let order_count: Vec<mysql::Row> = conn.query("SELECT COUNT(*) FROM orders").await?;
            let completed_orders: Vec<mysql::Row> = conn.query("SELECT COUNT(*) FROM orders WHERE status = 'completed'").await?;
            
            println!("Users: {}", user_count[0].get::<i32, _>(0).unwrap());
            println!("Orders: {}", order_count[0].get::<i32, _>(0).unwrap());
            println!("Completed orders: {}", completed_orders[0].get::<i32, _>(0).unwrap());
            
            // Store results in context
            context.set_data("user_count", user_count[0].get::<i32, _>(0).unwrap());
            context.set_data("order_count", order_count[0].get::<i32, _>(0).unwrap());
            
            Ok("cleanup_environment".to_string())
        } else {
            Ok("connecting".to_string())
        }
    }
}

struct CleanupEnvironmentHandler;

#[async_trait]
impl DynamicStateHandler for CleanupEnvironmentHandler {
    async fn execute(&self, context: &mut DynamicStateContext) -> Result<String, ConnectError> {
        if let Some(conn) = &mut context.connection {
            // Clean up test data
            conn.query_drop("DROP TABLE IF EXISTS orders").await?;
            conn.query_drop("DROP TABLE IF EXISTS users").await?;
            
            // Get database name from context
            if let Some(db_name) = context.get_data::<String>("database_name") {
                conn.query_drop(&format!("DROP DATABASE IF EXISTS {}", db_name)).await?;
            }
            
            Ok("completed".to_string())
        } else {
            Ok("completed".to_string())
        }
    }
}
```

## Context Management

### Using Dynamic State Context

The `DynamicStateContext` provides flexible data storage:

```rust
use test_rig::DynamicStateContext;

// Store data in context
context.set_data("table_name", "my_test_table");
context.set_data("row_count", 1000);
context.set_data("test_config", TestConfig { timeout: 30 });

// Retrieve data from context
if let Some(table_name) = context.get_data::<String>("table_name") {
    println!("Using table: {}", table_name);
}

if let Some(row_count) = context.get_data::<i32>("row_count") {
    println!("Expected rows: {}", row_count);
}

if let Some(config) = context.get_data::<TestConfig>("test_config") {
    println!("Timeout: {} seconds", config.timeout);
}
```

### Type-Safe Context Data

Create type-safe context data structures:

```rust
#[derive(Clone)]
struct TestConfig {
    timeout: u32,
    retry_count: u32,
    batch_size: usize,
}

#[derive(Clone)]
struct TestResults {
    success_count: u32,
    failure_count: u32,
    total_duration: Duration,
}

// Store and retrieve typed data
context.set_data("config", TestConfig {
    timeout: 30,
    retry_count: 3,
    batch_size: 100,
});

if let Some(config) = context.get_data::<TestConfig>("config") {
    println!("Test timeout: {} seconds", config.timeout);
}
```

## Error Handling

### Error Propagation

Handle errors properly in dynamic state handlers:

```rust
#[async_trait]
impl DynamicStateHandler for MyHandler {
    async fn execute(&self, context: &mut DynamicStateContext) -> Result<String, ConnectError> {
        if let Some(conn) = &mut context.connection {
            match conn.query_drop("SELECT 1").await {
                Ok(_) => {
                    println!("Operation successful");
                    Ok("next_state".to_string())
                }
                Err(e) => {
                    println!("Operation failed: {}", e);
                    // Return error state or retry
                    Ok("error_state".to_string())
                }
            }
        } else {
            Err(ConnectError::Connection("No database connection".into()))
        }
    }
}
```

### Retry Logic

Implement retry logic in handlers:

```rust
use std::time::Duration;
use tokio::time::sleep;

struct RetryableHandler {
    max_attempts: u32,
    attempt: u32,
}

#[async_trait]
impl DynamicStateHandler for RetryableHandler {
    async fn execute(&self, context: &mut DynamicStateContext) -> Result<String, ConnectError> {
        self.attempt += 1;
        
        if let Some(conn) = &mut context.connection {
            match perform_operation(conn).await {
                Ok(_) => Ok("next_state".to_string()),
                Err(e) => {
                    if self.attempt < self.max_attempts {
                        println!("Attempt {} failed, retrying...", self.attempt);
                        sleep(Duration::from_secs(1)).await;
                        Ok("current_state".to_string()) // Retry same state
                    } else {
                        println!("Max attempts reached, giving up");
                        Ok("error_state".to_string())
                    }
                }
            }
        } else {
            Ok("connecting".to_string())
        }
    }
}
```

## Integration with Common States

### Using Common States

Leverage common states for standard workflow steps:

```rust
use test_rig::common_states::*;

let mut machine = DynamicStateMachine::new();

// Register handlers for common states
machine.register_handler(parsing_config(), Box::new(ParsingConfigHandlerAdapter));
machine.register_handler(connecting(), Box::new(ConnectingHandlerAdapter));
machine.register_handler(testing_connection(), Box::new(TestingConnectionHandlerAdapter));
machine.register_handler(verifying_database(), Box::new(VerifyingDatabaseHandlerAdapter));
machine.register_handler(getting_version(), Box::new(GettingVersionHandlerAdapter));

// Register handlers for custom states
machine.register_handler(creating_table(), Box::new(CreatingTableHandler));
machine.register_handler(populating_data(), Box::new(PopulatingDataHandler));
machine.register_handler(testing_queries(), Box::new(TestingQueriesHandler));
```

### State Transitions

Handle state transitions properly:

```rust
#[async_trait]
impl DynamicStateHandler for MyHandler {
    async fn execute(&self, context: &mut DynamicStateContext) -> Result<String, ConnectError> {
        // Check prerequisites
        if !context.connection.is_some() {
            return Ok("connecting".to_string());
        }
        
        // Perform operation
        match perform_operation(context).await {
            Ok(_) => Ok("next_custom_state".to_string()),
            Err(e) => {
                // Handle different error types
                match e {
                    ConnectError::Connection(_) => Ok("connecting".to_string()),
                    ConnectError::Database(_) => Ok("error_state".to_string()),
                    _ => Ok("completed".to_string()),
                }
            }
        }
    }
}
```

## Best Practices

### State Design

1. **Keep states focused**: Each state should have a single responsibility
2. **Use descriptive names**: State names should clearly indicate their purpose
3. **Handle errors gracefully**: Always handle potential errors in state handlers
4. **Use common states**: Leverage common states for standard workflow steps

### Handler Implementation

1. **Check prerequisites**: Always check if required resources are available
2. **Return appropriate states**: Return the correct next state based on conditions
3. **Use context effectively**: Store and retrieve data from context as needed
4. **Clean up resources**: Clean up resources in error cases

### Error Handling

1. **Classify errors**: Distinguish between transient and permanent errors
2. **Implement retry logic**: Retry transient errors with appropriate backoff
3. **Provide meaningful errors**: Include context in error messages
4. **Graceful degradation**: Handle partial failures gracefully

### Performance

1. **Minimize state transitions**: Avoid unnecessary state transitions
2. **Use async operations**: Leverage async/await for I/O operations
3. **Batch operations**: Batch database operations when possible
4. **Resource management**: Properly manage database connections and other resources

## Examples

### Complete Dynamic Workflow Example

```rust
use test_rig::{DynamicStateMachine, DynamicStateHandler, DynamicStateContext, ConnectError, common_states::*};
use async_trait::async_trait;

// Define custom states
dynamic_state!(setup_test);
dynamic_state!(run_test);
dynamic_state!(verify_results);
dynamic_state!(cleanup);

// Handler implementations
struct SetupTestHandler;

#[async_trait]
impl DynamicStateHandler for SetupTestHandler {
    async fn execute(&self, context: &mut DynamicStateContext) -> Result<String, ConnectError> {
        if let Some(conn) = &mut context.connection {
            conn.query_drop("CREATE TABLE IF NOT EXISTS test_data (id INT, value VARCHAR(50))").await?;
            context.set_data("table_created", true);
            Ok("run_test".to_string())
        } else {
            Ok("connecting".to_string())
        }
    }
}

struct RunTestHandler;

#[async_trait]
impl DynamicStateHandler for RunTestHandler {
    async fn execute(&self, context: &mut DynamicStateContext) -> Result<String, ConnectError> {
        if let Some(conn) = &mut context.connection {
            // Insert test data
            for i in 1..=10 {
                conn.query_drop(&format!(
                    "INSERT INTO test_data (id, value) VALUES ({}, 'test{}')",
                    i, i
                )).await?;
            }
            
            // Perform test operations
            conn.query_drop("UPDATE test_data SET value = CONCAT(value, '_updated') WHERE id % 2 = 0").await?;
            
            context.set_data("test_completed", true);
            Ok("verify_results".to_string())
        } else {
            Ok("connecting".to_string())
        }
    }
}

struct VerifyResultsHandler;

#[async_trait]
impl DynamicStateHandler for VerifyResultsHandler {
    async fn execute(&self, context: &mut DynamicStateContext) -> Result<String, ConnectError> {
        if let Some(conn) = &mut context.connection {
            let result: Vec<mysql::Row> = conn.query("SELECT COUNT(*) FROM test_data").await?;
            let count = result[0].get::<i32, _>(0).unwrap();
            
            println!("Test data contains {} rows", count);
            
            if count == 10 {
                println!("✓ Test verification passed");
                Ok("cleanup".to_string())
            } else {
                println!("✗ Test verification failed");
                Ok("error_state".to_string())
            }
        } else {
            Ok("connecting".to_string())
        }
    }
}

struct CleanupHandler;

#[async_trait]
impl DynamicStateHandler for CleanupHandler {
    async fn execute(&self, context: &mut DynamicStateContext) -> Result<String, ConnectError> {
        if let Some(conn) = &mut context.connection {
            conn.query_drop("DROP TABLE IF EXISTS test_data").await?;
            println!("✓ Cleanup completed");
        }
        Ok("completed".to_string())
    }
}

// Create and run the machine
async fn run_dynamic_workflow() -> Result<(), ConnectError> {
    let mut machine = DynamicStateMachine::new();
    
    // Register common state handlers
    machine.register_handler(parsing_config(), Box::new(ParsingConfigHandlerAdapter));
    machine.register_handler(connecting(), Box::new(ConnectingHandlerAdapter));
    machine.register_handler(testing_connection(), Box::new(TestingConnectionHandlerAdapter));
    
    // Register custom state handlers
    machine.register_handler(setup_test(), Box::new(SetupTestHandler));
    machine.register_handler(run_test(), Box::new(RunTestHandler));
    machine.register_handler(verify_results(), Box::new(VerifyResultsHandler));
    machine.register_handler(cleanup(), Box::new(CleanupHandler));
    
    machine.run().await
}
```

This dynamic states system provides a powerful and flexible way to create custom workflows while maintaining the benefits of the state machine architecture. 
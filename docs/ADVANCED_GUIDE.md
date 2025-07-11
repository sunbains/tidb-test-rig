# Advanced Guide

This guide covers advanced features and usage patterns for the test_rig framework.

## Advanced State Machine Usage

### Dynamic State Machine

The dynamic state machine allows you to define custom states at compile time for extensible workflows:

```rust
use test_rig::{DynamicStateMachine, DynamicStateHandler, DynamicStateContext, common_states::*};

// Define custom states
dynamic_state!(creating_table);
dynamic_state!(populating_data);
dynamic_state!(testing_isolation);

// Create dynamic state machine
let mut machine = DynamicStateMachine::new();

// Register handlers for common states
machine.register_handler(parsing_config(), Box::new(ParsingConfigHandlerAdapter));
machine.register_handler(connecting(), Box::new(ConnectingHandlerAdapter));
machine.register_handler(testing_connection(), Box::new(TestingConnectionHandlerAdapter));

// Register handlers for custom states
machine.register_handler(creating_table(), Box::new(CreatingTableHandler));
machine.register_handler(populating_data(), Box::new(PopulatingDataHandler));
machine.register_handler(testing_isolation(), Box::new(TestingIsolationHandler));

// Run the machine
machine.run().await?;
```

### Custom State Handlers

Create custom state handlers for your specific testing needs:

```rust
use test_rig::{DynamicStateHandler, DynamicStateContext, ConnectError};

pub struct CreatingTableHandler;

#[async_trait]
impl DynamicStateHandler for CreatingTableHandler {
    async fn execute(&self, context: &mut DynamicStateContext) -> Result<String, ConnectError> {
        if let Some(conn) = &mut context.connection {
            // Create test table
            conn.query_drop("DROP TABLE IF EXISTS test_table").await?;
            conn.query_drop("CREATE TABLE test_table (id INT, name VARCHAR(50))").await?;
            
            // Store table info in context for other handlers
            context.set_data("table_name", "test_table");
            
            Ok("populating_data".to_string())
        } else {
            Err(ConnectError::Connection("No database connection".into()))
        }
    }
}
```

## Multi-Connection Testing

### Simple Multi-Connection

For basic multi-connection testing:

```rust
use test_rig::{SimpleMultiConnectionCoordinator, ConnectionConfig};

let mut coordinator = SimpleMultiConnectionCoordinator::new();

// Add connections
coordinator.add_connection(ConnectionConfig {
    id: "primary".to_string(),
    host: "tidb-primary:4000".to_string(),
    port: 4000,
    username: "root".to_string(),
    password: "".to_string(),
    database: Some("test".to_string()),
});

coordinator.add_connection(ConnectionConfig {
    id: "secondary".to_string(),
    host: "tidb-secondary:4000".to_string(),
    port: 4000,
    username: "root".to_string(),
    password: "".to_string(),
    database: Some("test".to_string()),
});

// Run all connections concurrently
coordinator.run_all_connections().await?;

// Get results
coordinator.print_results();
```

### Advanced Multi-Connection

For complex coordination scenarios:

```rust
use test_rig::{MultiConnectionStateMachine, ConnectionCoordinator, SharedState};

let shared_state = Arc::new(Mutex::new(SharedState::new()));
let coordinator = Arc::new(ConnectionCoordinator::new(shared_state.clone()));

let mut machine = MultiConnectionStateMachine::new(coordinator);

// Add connections with custom handlers
machine.add_connection("primary", Box::new(PrimaryConnectionHandler));
machine.add_connection("secondary", Box::new(SecondaryConnectionHandler));

// Run with coordination
machine.run_with_coordination().await?;
```

## Python Plugin System

### Advanced Python Handlers

Create sophisticated Python handlers with custom logic:

```python
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState
import time
import random

class AdvancedTransactionHandler(PyStateHandler):
    def __init__(self):
        super().__init__()
        self.attempt_count = 0
        self.max_attempts = 3
    
    def enter(self, context: PyStateContext) -> str:
        print(f"Starting advanced transaction test on {context.host}")
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        if not context.connection:
            return PyState.connecting()
        
        self.attempt_count += 1
        
        try:
            # Create test environment
            context.connection.execute_query("DROP TABLE IF EXISTS advanced_test")
            context.connection.execute_query("""
                CREATE TABLE advanced_test (
                    id INT PRIMARY KEY,
                    name VARCHAR(50),
                    value DECIMAL(10,2),
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )
            """)
            
            # Simulate complex transaction scenario
            context.connection.start_transaction()
            
            # Insert initial data
            for i in range(5):
                context.connection.execute_query(
                    f"INSERT INTO advanced_test (id, name, value) VALUES ({i}, 'Item{i}', {i * 10.5})"
                )
            
            # Simulate some processing time
            time.sleep(0.1)
            
            # Update some records
            context.connection.execute_query("UPDATE advanced_test SET value = value * 1.1 WHERE id % 2 = 0")
            
            # Simulate potential conflict
            if random.random() < 0.3:  # 30% chance of rollback
                print("Simulating transaction rollback")
                context.connection.rollback()
                if self.attempt_count < self.max_attempts:
                    return PyState.connecting()  # Retry
            else:
                context.connection.commit()
            
            # Verify results
            results = context.connection.execute_query("SELECT COUNT(*) FROM advanced_test")
            print(f"Final record count: {results[0]['col_0']}")
            
            return PyState.completed()
            
        except Exception as e:
            print(f"Error in attempt {self.attempt_count}: {e}")
            if self.attempt_count < self.max_attempts:
                return PyState.connecting()  # Retry
            else:
                return PyState.completed()  # Give up
    
    def exit(self, context: PyStateContext) -> None:
        print(f"Advanced transaction test completed after {self.attempt_count} attempts")
```

### Multi-Connection Python Handlers

Create Python handlers that work with multiple concurrent connections:

```python
from src.common.test_rig_python import MultiConnectionTestHandler, PyStateContext, PyState
import threading
import time

class ConcurrentLoadTestHandler(MultiConnectionTestHandler):
    def __init__(self):
        super().__init__(connection_count=5)  # Use 5 concurrent connections
    
    def execute(self, context: PyStateContext) -> str:
        # Create test table
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS load_test")
            context.connection.execute_query("""
                CREATE TABLE load_test (
                    id INT PRIMARY KEY,
                    connection_id INT,
                    operation_type VARCHAR(20),
                    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )
            """)
        
        # Define concurrent operations
        operations = []
        
        # Each connection performs different operations
        for conn_id in range(self.connection_count):
            operations.extend([
                {'connection_id': conn_id, 'operation': 'start_transaction'},
                {'connection_id': conn_id, 'operation': 'query', 'query': f'INSERT INTO load_test (id, connection_id, operation_type) VALUES ({conn_id * 100 + 1}, {conn_id}, "INSERT")'},
                {'connection_id': conn_id, 'operation': 'query', 'query': f'INSERT INTO load_test (id, connection_id, operation_type) VALUES ({conn_id * 100 + 2}, {conn_id}, "INSERT")'},
                {'connection_id': conn_id, 'operation': 'query', 'query': f'UPDATE load_test SET operation_type = "UPDATED" WHERE connection_id = {conn_id}'},
                {'connection_id': conn_id, 'operation': 'query', 'query': f'SELECT COUNT(*) FROM load_test WHERE connection_id = {conn_id}'},
                {'connection_id': conn_id, 'operation': 'commit'},
            ])
        
        # Execute operations concurrently
        results = self.execute_concurrent_operations(operations)
        
        # Analyze results
        if context.connection:
            final_results = context.connection.execute_query("SELECT COUNT(*) FROM load_test")
            print(f"Total records created: {final_results[0]['col_0']}")
            
            # Check for conflicts or issues
            conflict_results = context.connection.execute_query("SELECT COUNT(*) FROM load_test WHERE operation_type = 'UPDATED'")
            print(f"Records updated: {conflict_results[0]['col_0']}")
        
        return PyState.completed()
```

## Error Handling and Resilience

### Retry Strategies

Implement custom retry logic in your handlers:

```rust
use test_rig::{retry::RetryConfig, error_utils::classify_error};

let retry_config = RetryConfig {
    max_attempts: 5,
    base_delay: Duration::from_secs(1),
    max_delay: Duration::from_secs(30),
    jitter: true,
};

let result = retry_config.execute("database_operation", || async {
    // Your database operation here
    perform_database_operation().await
}).await;
```

### Circuit Breaker Pattern

Use circuit breakers for system protection:

```rust
use test_rig::{retry::CircuitBreakerConfig, error_utils::classify_error};

let circuit_config = CircuitBreakerConfig {
    failure_threshold: 5,
    recovery_timeout: Duration::from_secs(60),
    success_threshold: 2,
};

let result = circuit_config.execute("critical_operation", || async {
    // Your critical operation here
    perform_critical_operation().await
}).await;
```

### Error Classification

Classify errors for appropriate handling:

```rust
use test_rig::error_utils::{classify_error, ErrorCategory};

let error = ConnectError::Connection(mysql::Error::server_disconnected());
match classify_error(&error) {
    ErrorCategory::Transient => {
        // Will be retried automatically
        println!("Transient error, retrying...");
    }
    ErrorCategory::Permanent => {
        // Fail fast
        println!("Permanent error, failing immediately");
    }
    ErrorCategory::Unknown => {
        // Retry with limits
        println!("Unknown error, retrying with limits");
    }
}
```

## Configuration Management

### Custom Configuration Extensions

Add test-specific configuration options:

```rust
use test_rig::{ConfigExtension, register_config_extension};
use clap::Command;

struct MyTestConfigExtension;

impl ConfigExtension for MyTestConfigExtension {
    fn add_cli_args(&self, app: Command) -> Command {
        app.arg(
            clap::Arg::new("test-rows")
                .long("test-rows")
                .help("Number of test rows to create")
                .default_value("1000")
        )
        .arg(
            clap::Arg::new("concurrent-users")
                .long("concurrent-users")
                .help("Number of concurrent users to simulate")
                .default_value("10")
        )
    }
    
    fn build_config(&self, args: &clap::ArgMatches, config: &mut test_rig::config::AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(test_rows) = args.get_one::<String>("test-rows") {
            if let Ok(rows) = test_rows.parse::<u32>() {
                config.test.rows = rows;
            }
        }
        
        if let Some(users) = args.get_one::<String>("concurrent-users") {
            if let Ok(user_count) = users.parse::<u32>() {
                config.test.concurrent_users = user_count;
            }
        }
        
        Ok(())
    }
    
    fn get_extension_name(&self) -> &'static str { "my_test" }
    fn get_help_text(&self) -> &'static str { "Adds test-specific options" }
}

// Register the extension
fn register_extensions() {
    register_config_extension(Box::new(MyTestConfigExtension));
}
```

### Dynamic Configuration

Load configuration dynamically based on environment:

```rust
use test_rig::config::AppConfig;

let config = AppConfig::load_with_priority(&[
    "config/production.toml",
    "config/development.toml",
    "config/default.toml",
])?;

// Override with environment variables
let config = config.with_env_overrides()?;
```

## Performance Optimization

### Connection Pooling

Optimize connection usage with pooling:

```rust
use test_rig::connection_manager::ConnectionPool;

let pool = ConnectionPool::new(
    pool_config,
    connection_config,
)?;

// Use pooled connections
let conn = pool.get_connection().await?;
// ... use connection
pool.return_connection(conn).await;
```

### Batch Operations

Optimize database operations with batching:

```rust
// Batch inserts
let mut batch = Vec::new();
for i in 0..1000 {
    batch.push(format!("({}, 'item{}', {})", i, i, i * 10.5));
}

let query = format!(
    "INSERT INTO test_table (id, name, value) VALUES {}",
    batch.join(",")
);

conn.query_drop(&query).await?;
```

### Async Operations

Leverage async/await for better performance:

```rust
use tokio::join;

// Execute operations concurrently
let (result1, result2, result3) = join!(
    async_operation_1(),
    async_operation_2(),
    async_operation_3(),
);
```

## Monitoring and Observability

### Structured Logging

Use structured logging for better debugging:

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(connection))]
async fn perform_operation(connection: &mut mysql::Conn) -> Result<(), ConnectError> {
    info!("Starting database operation");
    
    match connection.query_drop("SELECT 1").await {
        Ok(_) => {
            info!("Operation completed successfully");
            Ok(())
        }
        Err(e) => {
            error!(error = %e, "Operation failed");
            Err(ConnectError::Database(e))
        }
    }
}
```

### Metrics Collection

Collect metrics for performance monitoring:

```rust
use std::time::Instant;

let start_time = Instant::now();
let result = perform_operation().await;
let duration = start_time.elapsed();

info!(
    operation = "database_query",
    duration_ms = duration.as_millis(),
    success = result.is_ok(),
);
```

## Testing Strategies

### Test Isolation

Ensure tests are isolated and don't affect each other:

```rust
// Use unique table names
let table_name = format!("test_{}", uuid::Uuid::new_v4().simple());
conn.query_drop(&format!("CREATE TABLE {} (id INT)", table_name)).await?;

// Clean up after test
conn.query_drop(&format!("DROP TABLE {}", table_name)).await?;
```

### Test Data Management

Manage test data effectively:

```rust
// Create test data factory
struct TestDataFactory {
    connection: mysql::Conn,
}

impl TestDataFactory {
    async fn create_test_user(&mut self, name: &str) -> Result<i32, ConnectError> {
        self.connection.query_drop(&format!(
            "INSERT INTO users (name) VALUES ('{}')",
            name
        )).await?;
        
        let result: Vec<mysql::Row> = self.connection.query("SELECT LAST_INSERT_ID()").await?;
        Ok(result[0].get(0).unwrap())
    }
    
    async fn cleanup(&mut self) -> Result<(), ConnectError> {
        self.connection.query_drop("DELETE FROM users WHERE name LIKE 'test_%'").await?;
        Ok(())
    }
}
```

### Concurrent Testing

Test concurrent scenarios effectively:

```rust
use tokio::task::JoinSet;

let mut tasks = JoinSet::new();

// Spawn multiple concurrent operations
for i in 0..10 {
    let mut conn = pool.get_connection().await?;
    tasks.spawn(async move {
        perform_concurrent_operation(&mut conn, i).await
    });
}

// Wait for all tasks to complete
while let Some(result) = tasks.join_next().await {
    match result {
        Ok(Ok(_)) => println!("Task completed successfully"),
        Ok(Err(e)) => println!("Task failed: {}", e),
        Err(e) => println!("Task panicked: {}", e),
    }
}
```

## Best Practices

### Code Organization

1. **Separate concerns**: Keep state handlers focused on single responsibilities
2. **Use common states**: Leverage the `common_states` module for standard workflows
3. **Handler-local context**: Use handler-local context for state-specific data
4. **Error propagation**: Properly propagate errors through the state machine

### Performance

1. **Connection pooling**: Use connection pools for efficiency
2. **Batch operations**: Batch database operations when possible
3. **Async operations**: Use async/await for I/O operations
4. **Resource cleanup**: Clean up resources in the `exit` method

### Testing

1. **Test isolation**: Ensure tests don't interfere with each other
2. **Mock connections**: Use mock connections for unit testing
3. **Real database testing**: Test with real databases for integration testing
4. **Concurrent testing**: Test concurrent scenarios thoroughly

### Error Handling

1. **Error classification**: Classify errors appropriately
2. **Retry strategies**: Implement retry strategies for transient errors
3. **Circuit breakers**: Use circuit breakers for system protection
4. **Error context**: Preserve error context for debugging 
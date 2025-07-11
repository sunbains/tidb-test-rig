# Advanced Guide

This guide covers advanced features and usage patterns for the test_rig framework. For basic usage, see the other documentation files in this directory.

## Related Documentation

- **[Architecture](ARCHITECTURE.md)** - System architecture and design overview
- **[Running Python Tests](RUNNING_PYTHON_TESTS.md)** - How to run Python test suites
- **[Creating Python Test Directories](CREATING_PYTHON_TEST_DIRECTORIES.md)** - How to create new test suites
- **[Dynamic States](DYNAMIC_STATES.md)** - Using the dynamic state system

## Advanced State Machine Usage

### Custom State Handlers

Create custom state handlers for your specific testing needs:

```rust
use test_rig::{DynamicStateHandler, DynamicStateContext, ConnectError};

pub struct CustomTestHandler {
    test_config: TestConfig,
}

#[async_trait]
impl DynamicStateHandler for CustomTestHandler {
    async fn execute(&self, context: &mut DynamicStateContext) -> Result<String, ConnectError> {
        if let Some(conn) = &mut context.connection {
            // Your custom test logic here
            self.run_custom_test(conn).await?;
            
            // Store results in context for other handlers
            context.set_data("test_results", self.get_results());
            
            Ok("next_state".to_string())
        } else {
            Err(ConnectError::Connection("No database connection".into()))
        }
    }
}
```

### Advanced Context Management

Use the dynamic context for complex data sharing:

```rust
use test_rig::DynamicStateContext;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
struct TestResults {
    success_count: u32,
    failure_count: u32,
    duration: Duration,
    errors: Vec<String>,
}

// Store complex data in context
context.set_data("test_results", TestResults {
    success_count: 10,
    failure_count: 2,
    duration: Duration::from_secs(30),
    errors: vec!["Connection timeout".to_string()],
});

// Retrieve and use data
if let Some(results) = context.get_data::<TestResults>("test_results") {
    println!("Test completed: {} successes, {} failures", 
             results.success_count, results.failure_count);
}
```

## Advanced Multi-Connection Testing

### Custom Coordination Logic

Implement custom coordination between multiple connections:

```rust
use test_rig::{MultiConnectionStateMachine, ConnectionCoordinator, SharedState};
use tokio::sync::mpsc;

struct CustomCoordinator {
    shared_state: Arc<Mutex<SharedState>>,
    event_sender: mpsc::Sender<CoordinationEvent>,
    event_receiver: mpsc::Receiver<CoordinationEvent>,
}

impl CustomCoordinator {
    async fn coordinate_operations(&mut self) -> Result<(), ConnectError> {
        // Wait for all connections to be ready
        self.wait_for_all_connections().await?;
        
        // Send coordination signals
        self.broadcast_event(CoordinationEvent::StartTest).await?;
        
        // Monitor progress
        while let Some(event) = self.event_receiver.recv().await {
            match event {
                CoordinationEvent::TestCompleted { connection_id } => {
                    println!("Connection {} completed test", connection_id);
                }
                CoordinationEvent::ConnectionFailed { connection_id, error } => {
                    println!("Connection {} failed: {}", connection_id, error);
                }
                _ => {}
            }
        }
        
        Ok(())
    }
}
```

### Load Testing Patterns

Implement sophisticated load testing scenarios:

```rust
use test_rig::{DynamicStateMachine, DynamicStateHandler, DynamicStateContext};
use tokio::time::{sleep, Duration};

struct LoadTestHandler {
    concurrent_users: u32,
    test_duration: Duration,
    ramp_up_time: Duration,
}

#[async_trait]
impl DynamicStateHandler for LoadTestHandler {
    async fn execute(&self, context: &mut DynamicStateContext) -> Result<String, ConnectError> {
        let start_time = Instant::now();
        let mut tasks = JoinSet::new();
        
        // Ramp up connections gradually
        for user_id in 0..self.concurrent_users {
            let delay = self.ramp_up_time * user_id / self.concurrent_users;
            sleep(delay).await;
            
            let mut conn = context.connection_pool.get_connection().await?;
            tasks.spawn(async move {
                self.simulate_user_workload(&mut conn, user_id).await
            });
        }
        
        // Monitor tasks until test duration expires
        while start_time.elapsed() < self.test_duration {
            while let Some(result) = tasks.join_next().await {
                match result {
                    Ok(Ok(_)) => println!("User task completed successfully"),
                    Ok(Err(e)) => println!("User task failed: {}", e),
                    Err(e) => println!("User task panicked: {}", e),
                }
            }
        }
        
        Ok("completed".to_string())
    }
}
```

## Advanced Error Handling and Resilience

### Custom Retry Strategies

Implement domain-specific retry logic:

```rust
use test_rig::{retry::RetryConfig, error_utils::classify_error};
use std::time::Duration;

struct DatabaseRetryStrategy {
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
    backoff_multiplier: f64,
}

impl DatabaseRetryStrategy {
    async fn execute_with_retry<F, Fut, T>(&self, operation: F) -> Result<T, ConnectError>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, ConnectError>>,
    {
        let mut attempt = 0;
        let mut delay = self.base_delay;
        
        loop {
            attempt += 1;
            
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if attempt >= self.max_attempts {
                        return Err(error);
                    }
                    
                    match classify_error(&error) {
                        ErrorCategory::Transient => {
                            // Retry with exponential backoff
                            sleep(delay).await;
                            delay = Duration::from_secs_f64(
                                delay.as_secs_f64() * self.backoff_multiplier
                            ).min(self.max_delay);
                        }
                        ErrorCategory::Permanent => {
                            // Don't retry permanent errors
                            return Err(error);
                        }
                        ErrorCategory::Unknown => {
                            // Retry with limits for unknown errors
                            if attempt < 3 {
                                sleep(delay).await;
                            } else {
                                return Err(error);
                            }
                        }
                    }
                }
            }
        }
    }
}
```

### Circuit Breaker with Custom Logic

Implement domain-specific circuit breaker patterns:

```rust
use test_rig::retry::CircuitBreakerConfig;
use std::sync::atomic::{AtomicU32, Ordering};

struct DatabaseCircuitBreaker {
    failure_count: AtomicU32,
    success_count: AtomicU32,
    failure_threshold: u32,
    success_threshold: u32,
    recovery_timeout: Duration,
    last_failure_time: Mutex<Option<Instant>>,
}

impl DatabaseCircuitBreaker {
    async fn execute<F, Fut, T>(&self, operation: F) -> Result<T, ConnectError>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, ConnectError>>,
    {
        // Check if circuit is open
        if let Some(last_failure) = *self.last_failure_time.lock().unwrap() {
            if last_failure.elapsed() < self.recovery_timeout {
                return Err(ConnectError::Connection("Circuit breaker open".into()));
            }
        }
        
        // Execute operation
        match operation().await {
            Ok(result) => {
                self.success_count.fetch_add(1, Ordering::Relaxed);
                if self.success_count.load(Ordering::Relaxed) >= self.success_threshold {
                    self.reset_circuit();
                }
                Ok(result)
            }
            Err(error) => {
                self.failure_count.fetch_add(1, Ordering::Relaxed);
                *self.last_failure_time.lock().unwrap() = Some(Instant::now());
                
                if self.failure_count.load(Ordering::Relaxed) >= self.failure_threshold {
                    self.open_circuit();
                }
                
                Err(error)
            }
        }
    }
}
```

## Advanced Configuration Management

### Custom Configuration Extensions

Add test-specific configuration options:

```rust
use test_rig::{ConfigExtension, register_config_extension};
use clap::Command;

struct PerformanceTestConfigExtension;

impl ConfigExtension for PerformanceTestConfigExtension {
    fn add_cli_args(&self, app: Command) -> Command {
        app.arg(
            clap::Arg::new("concurrent-users")
                .long("concurrent-users")
                .help("Number of concurrent users to simulate")
                .default_value("10")
        )
        .arg(
            clap::Arg::new("test-duration")
                .long("test-duration")
                .help("Test duration in seconds")
                .default_value("300")
        )
        .arg(
            clap::Arg::new("ramp-up-time")
                .long("ramp-up-time")
                .help("Ramp up time in seconds")
                .default_value("60")
        )
        .arg(
            clap::Arg::new("think-time")
                .long("think-time")
                .help("Think time between requests in milliseconds")
                .default_value("1000")
        )
    }
    
    fn build_config(&self, args: &clap::ArgMatches, config: &mut test_rig::config::AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(users) = args.get_one::<String>("concurrent-users") {
            config.test.concurrent_users = users.parse::<u32>()?;
        }
        
        if let Some(duration) = args.get_one::<String>("test-duration") {
            config.test.duration_secs = duration.parse::<u32>()?;
        }
        
        if let Some(ramp_up) = args.get_one::<String>("ramp-up-time") {
            config.test.ramp_up_secs = ramp_up.parse::<u32>()?;
        }
        
        if let Some(think_time) = args.get_one::<String>("think-time") {
            config.test.think_time_ms = think_time.parse::<u32>()?;
        }
        
        Ok(())
    }
    
    fn get_extension_name(&self) -> &'static str { "performance_test" }
    fn get_help_text(&self) -> &'static str { "Adds performance testing options" }
}
```

### Dynamic Configuration Loading

Load configuration based on environment and conditions:

```rust
use test_rig::config::AppConfig;
use std::collections::HashMap;

struct DynamicConfigLoader {
    environment: String,
    config_overrides: HashMap<String, String>,
}

impl DynamicConfigLoader {
    async fn load_config(&self) -> Result<AppConfig, Box<dyn std::error::Error>> {
        let mut config = AppConfig::default();
        
        // Load base configuration
        match self.environment.as_str() {
            "development" => {
                config.merge_from_file("config/development.toml")?;
            }
            "staging" => {
                config.merge_from_file("config/staging.toml")?;
            }
            "production" => {
                config.merge_from_file("config/production.toml")?;
            }
            _ => {
                config.merge_from_file("config/default.toml")?;
            }
        }
        
        // Apply environment-specific overrides
        for (key, value) in &self.config_overrides {
            config.set_value(key, value)?;
        }
        
        // Apply environment variables
        config.with_env_overrides()?;
        
        Ok(config)
    }
}
```

## Performance Optimization

### Connection Pool Optimization

Optimize connection usage for high-performance scenarios:

```rust
use test_rig::connection_manager::ConnectionPool;
use std::time::Duration;

struct OptimizedConnectionPool {
    pool: ConnectionPool,
    metrics: PoolMetrics,
}

impl OptimizedConnectionPool {
    async fn get_optimized_connection(&mut self) -> Result<PooledConnection, ConnectError> {
        let start_time = Instant::now();
        
        // Try to get connection with timeout
        let conn = tokio::time::timeout(
            Duration::from_secs(5),
            self.pool.get_connection()
        ).await
        .map_err(|_| ConnectError::Connection("Connection timeout".into()))??;
        
        // Update metrics
        self.metrics.connection_wait_time = start_time.elapsed();
        self.metrics.total_connections += 1;
        
        Ok(conn)
    }
    
    async fn return_connection(&mut self, conn: PooledConnection) {
        // Update metrics before returning
        self.metrics.active_connections = self.metrics.active_connections.saturating_sub(1);
        
        self.pool.return_connection(conn).await;
    }
}
```

### Batch Operation Optimization

Optimize database operations with intelligent batching:

```rust
use std::collections::VecDeque;

struct BatchProcessor {
    batch_size: usize,
    batch_timeout: Duration,
    pending_operations: VecDeque<DatabaseOperation>,
    last_batch_time: Instant,
}

impl BatchProcessor {
    async fn add_operation(&mut self, operation: DatabaseOperation) -> Result<(), ConnectError> {
        self.pending_operations.push_back(operation);
        
        // Flush batch if it's full or timeout has elapsed
        if self.pending_operations.len() >= self.batch_size || 
           self.last_batch_time.elapsed() >= self.batch_timeout {
            self.flush_batch().await?;
        }
        
        Ok(())
    }
    
    async fn flush_batch(&mut self) -> Result<(), ConnectError> {
        if self.pending_operations.is_empty() {
            return Ok(());
        }
        
        let operations = std::mem::take(&mut self.pending_operations);
        let batch_query = self.build_batch_query(operations);
        
        // Execute batch operation
        self.execute_batch(batch_query).await?;
        
        self.last_batch_time = Instant::now();
        Ok(())
    }
}
```

## Monitoring and Observability

### Custom Metrics Collection

Implement domain-specific metrics:

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

#[derive(Default)]
struct TestMetrics {
    total_operations: AtomicU64,
    successful_operations: AtomicU64,
    failed_operations: AtomicU64,
    total_duration: AtomicU64, // in nanoseconds
    operation_durations: Mutex<Vec<Duration>>,
}

impl TestMetrics {
    fn record_operation(&self, duration: Duration, success: bool) {
        self.total_operations.fetch_add(1, Ordering::Relaxed);
        
        if success {
            self.successful_operations.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_operations.fetch_add(1, Ordering::Relaxed);
        }
        
        self.total_duration.fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
        
        self.operation_durations.lock().unwrap().push(duration);
    }
    
    fn get_statistics(&self) -> TestStatistics {
        let total = self.total_operations.load(Ordering::Relaxed);
        let successful = self.successful_operations.load(Ordering::Relaxed);
        let failed = self.failed_operations.load(Ordering::Relaxed);
        let total_duration = Duration::from_nanos(self.total_duration.load(Ordering::Relaxed));
        
        let durations = self.operation_durations.lock().unwrap();
        let avg_duration = if !durations.is_empty() {
            durations.iter().sum::<Duration>() / durations.len() as u32
        } else {
            Duration::ZERO
        };
        
        TestStatistics {
            total_operations: total,
            success_rate: if total > 0 { successful as f64 / total as f64 } else { 0.0 },
            average_duration: avg_duration,
            total_duration,
        }
    }
}
```

### Structured Logging with Context

Implement rich logging with context:

```rust
use tracing::{info, warn, error, instrument, Span};
use tracing_subscriber::fmt::format::FmtSpan;

#[instrument(skip(connection), fields(operation = %operation_name))]
async fn perform_operation_with_logging(
    connection: &mut mysql::Conn,
    operation_name: &str,
    operation_data: &str,
) -> Result<(), ConnectError> {
    let span = Span::current();
    span.record("operation_data", &operation_data);
    
    let start_time = Instant::now();
    
    info!("Starting database operation: {}", operation_name);
    
    match connection.query_drop(operation_data).await {
        Ok(_) => {
            let duration = start_time.elapsed();
            info!(
                operation = %operation_name,
                duration_ms = duration.as_millis(),
                status = "success",
                "Database operation completed successfully"
            );
            Ok(())
        }
        Err(e) => {
            let duration = start_time.elapsed();
            error!(
                operation = %operation_name,
                duration_ms = duration.as_millis(),
                error = %e,
                status = "failed",
                "Database operation failed"
            );
            Err(ConnectError::Database(e))
        }
    }
}
```

## Best Practices

### Performance Optimization

1. **Connection pooling**: Use connection pools for high-concurrency scenarios
2. **Batch operations**: Batch database operations when possible
3. **Async operations**: Leverage async/await for I/O operations
4. **Resource management**: Properly manage database connections and other resources
5. **Metrics collection**: Collect and monitor performance metrics

### Error Handling

1. **Error classification**: Distinguish between transient and permanent errors
2. **Retry strategies**: Implement retry strategies for transient errors
3. **Circuit breakers**: Use circuit breakers for system protection
4. **Error context**: Preserve error context for debugging

### Testing Strategy

1. **Test isolation**: Ensure tests don't interfere with each other
2. **Load testing**: Test with realistic load patterns
3. **Failure injection**: Test error handling and recovery
4. **Monitoring**: Monitor test execution and performance

### Code Organization

1. **Separation of concerns**: Keep handlers focused on single responsibilities
2. **Configuration management**: Use configuration extensions for test-specific options
3. **Metrics and logging**: Implement comprehensive monitoring
4. **Error handling**: Implement robust error handling throughout 
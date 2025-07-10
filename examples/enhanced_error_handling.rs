use mysql::{Opts, Pool};
use std::time::Duration;
use test_rig::error_utils::{
    ErrorContextBuilder, ResilientConnectionManager, classify_error,
    create_db_circuit_breaker_config, create_db_retry_config, get_recovery_strategy,
};
use test_rig::errors::{ConnectError, RetryConfig, RetryStrategy};
use test_rig::retry::CircuitBreaker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Basic retry configuration
    println!("=== Example 1: Basic Retry Configuration ===");
    let retry_config = create_db_retry_config();
    println!("Retry config: {:?}", retry_config);

    let circuit_config = create_db_circuit_breaker_config();
    println!("Circuit breaker config: {:?}", circuit_config);

    // Example 2: Error classification
    println!("\n=== Example 2: Error Classification ===");
    let connection_error = ConnectError::Connection(mysql::Error::server_disconnected());
    let auth_error = ConnectError::Authentication("invalid credentials".to_string());

    println!(
        "Connection error category: {:?}",
        classify_error(&connection_error)
    );
    println!("Auth error category: {:?}", classify_error(&auth_error));

    // Example 3: Recovery strategies
    println!("\n=== Example 3: Recovery Strategies ===");
    println!(
        "Connection error recovery: {:?}",
        get_recovery_strategy(&connection_error)
    );
    println!(
        "Auth error recovery: {:?}",
        get_recovery_strategy(&auth_error)
    );

    // Example 4: Error context building
    println!("\n=== Example 4: Error Context Building ===");
    let context = ErrorContextBuilder::new("database_query".to_string())
        .with_connection_info(
            "localhost".to_string(),
            "testdb".to_string(),
            "testuser".to_string(),
        )
        .with_query("SELECT * FROM users".to_string())
        .with_attempt(3)
        .with_duration(Duration::from_secs(5))
        .with_additional_info("table".to_string(), "users".to_string())
        .build();

    println!("Error context: {:?}", context);

    // Example 5: Resilient connection manager (if database is available)
    if let Ok(pool) = create_test_pool() {
        println!("\n=== Example 5: Resilient Connection Manager ===");
        let manager = ResilientConnectionManager::with_custom_config(
            pool,
            "localhost".to_string(),
            "testdb".to_string(),
            "testuser".to_string(),
            circuit_config.clone(),
            retry_config,
        );

        // Example operation with resilience
        let result = manager
            .execute_with_resilience("test_query", || {
                // Simulate a database operation
                if rand::random::<bool>() {
                    Ok("success")
                } else {
                    Err(ConnectError::Connection(mysql::Error::server_disconnected()))
                }
            })
            .await;

        match result {
            Ok(value) => println!("Operation succeeded: {}", value),
            Err(enhanced_error) => {
                println!("Operation failed with enhanced error: {:?}", enhanced_error);
            }
        }
    } else {
        println!("\n=== Example 5: Skipped (no database connection) ===");
        println!("Database connection not available, skipping resilient manager example");
    }

    // Example 6: Custom retry configuration
    println!("\n=== Example 6: Custom Retry Configuration ===");
    let custom_retry_config = RetryConfig {
        max_retries: 10,
        base_delay: Duration::from_millis(50),
        max_delay: Duration::from_secs(5),
        backoff_multiplier: 2.0,
    };
    println!("Custom retry config: {:?}", custom_retry_config);

    // Example 7: Circuit breaker states
    println!("\n=== Example 7: Circuit Breaker States ===");
    let circuit_breaker = CircuitBreaker::new(circuit_config);
    println!(
        "Initial circuit breaker state: {:?}",
        circuit_breaker.get_state()
    );

    // Example 8: Error handling patterns
    println!("\n=== Example 8: Error Handling Patterns ===");
    demonstrate_error_handling_patterns().await;

    Ok(())
}

fn create_test_pool() -> Result<Pool, mysql::Error> {
    // This is just for demonstration - in real usage, you'd use actual connection details
    let opts = Opts::from_url("mysql://user:pass@localhost:4000/test")?;
    Pool::new(opts)
}

async fn demonstrate_error_handling_patterns() {
    println!("Demonstrating different error handling patterns...");

    // Pattern 1: Simple retry with exponential backoff
    println!("Pattern 1: Simple retry with exponential backoff");
    let retry_config = create_db_retry_config();
    let retry_strategy = RetryStrategy::new(retry_config);

    let result = retry_strategy
        .retry(|| {
            Box::pin(async {
                // Simulate a failure that succeeds on the third attempt
                static mut ATTEMPT_COUNT: usize = 0;
                unsafe {
                    ATTEMPT_COUNT += 1;
                    if ATTEMPT_COUNT < 3 {
                        Err(ConnectError::Connection(mysql::Error::server_disconnected()))
                    } else {
                        Ok("success after retries")
                    }
                }
            })
        })
        .await;

    match result {
        Ok(value) => println!("  Success: {}", value),
        Err(error) => println!("  Failed: {:?}", error),
    }

    // Pattern 2: Circuit breaker with retry
    println!("Pattern 2: Circuit breaker with retry");
    let circuit_config = create_db_circuit_breaker_config();
    let _circuit_breaker = test_rig::CircuitBreaker::new(circuit_config);

    let result = retry_strategy
        .retry(|| {
            Box::pin(async {
                // Simulate intermittent failures with circuit breaker
                if rand::random::<f64>() < 0.7 {
                    Err(ConnectError::Connection(mysql::Error::server_disconnected()))
                } else {
                    Ok("circuit breaker success")
                }
            })
        })
        .await;

    match result {
        Ok(value) => println!("  Success: {}", value),
        Err(error) => println!("  Failed: {:?}", error),
    }

    // Pattern 3: Error context preservation
    println!("Pattern 3: Error context preservation");
    let context = ErrorContextBuilder::new("complex_operation".to_string())
        .with_connection_info(
            "prod-db.example.com".to_string(),
            "production".to_string(),
            "app_user".to_string(),
        )
        .with_query("UPDATE critical_table SET status = 'processing'".to_string())
        .with_additional_info("transaction_id".to_string(), "txn_12345".to_string())
        .with_additional_info("user_id".to_string(), "user_67890".to_string())
        .build();

    println!("  Preserved context: {:?}", context);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_error_classification() {
        let connection_error = ConnectError::Connection(mysql::Error::server_disconnected());
        assert_eq!(
            classify_error(&connection_error),
            test_rig::error_utils::ErrorCategory::Transient
        );

        let auth_error = ConnectError::Authentication("invalid".to_string());
        assert_eq!(
            classify_error(&auth_error),
            test_rig::error_utils::ErrorCategory::Permanent
        );
    }

    #[tokio::test]
    async fn test_recovery_strategies() {
        let connection_error = ConnectError::Connection(mysql::Error::server_disconnected());
        assert_eq!(
            get_recovery_strategy(&connection_error),
            test_rig::error_utils::RecoveryStrategy::Retry
        );

        let auth_error = ConnectError::Authentication("invalid".to_string());
        assert_eq!(
            get_recovery_strategy(&auth_error),
            test_rig::error_utils::RecoveryStrategy::FailFast
        );
    }

    #[tokio::test]
    async fn test_retry_with_backoff() {
        let config = create_db_retry_config();

        let result = test_rig::retry_with_backoff(&config, || {
            // Simulate a failure that succeeds on the third attempt
            static mut ATTEMPT_COUNT: usize = 0;
            unsafe {
                ATTEMPT_COUNT += 1;
                if ATTEMPT_COUNT < 3 {
                    Err(ConnectError::Connection(mysql::Error::server_disconnected()))
                } else {
                    Ok("success")
                }
            }
        })
        .await;

        assert!(result.is_ok());
    }
}

//! # Error Utilities
//!
//! Enhanced error utilities and context management for resilient database operations.
//! Provides error classification, recovery strategies, and enhanced error context building.

use crate::errors::RetryConfig;
use crate::errors::{ConnectError, EnhancedError, ErrorContext};
use crate::retry::{CircuitBreaker, CircuitBreakerConfig, retry_with_circuit_breaker};
use mysql::{Pool, PooledConn};
use std::time::Duration;

/// Enhanced database connection manager with retry and circuit breaker
pub struct ResilientConnectionManager {
    pool: Pool,
    circuit_breaker: CircuitBreaker,
    retry_config: RetryConfig,
    host: String,
    database: String,
    user: String,
}

impl ResilientConnectionManager {
    #[must_use]
    pub fn new(pool: Pool, host: String, database: String, user: String) -> Self {
        let circuit_config = CircuitBreakerConfig::default();
        let retry_config = RetryConfig::default();

        Self {
            pool,
            circuit_breaker: CircuitBreaker::new(circuit_config),
            retry_config,
            host,
            database,
            user,
        }
    }

    #[must_use]
    pub fn with_custom_config(
        pool: Pool,
        host: String,
        database: String,
        user: String,
        circuit_config: CircuitBreakerConfig,
        retry_config: RetryConfig,
    ) -> Self {
        Self {
            pool,
            circuit_breaker: CircuitBreaker::new(circuit_config),
            retry_config,
            host,
            database,
            user,
        }
    }

    /// Execute an operation with resilience and retry logic
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails after all retry attempts.
    pub async fn execute_with_resilience<F, T>(
        &self,
        operation: &str,
        f: F,
    ) -> Result<T, EnhancedError>
    where
        F: Fn() -> Result<T, ConnectError> + Send + Sync,
    {
        let context = ErrorContext::new(operation.to_string())
            .with_host(self.host.clone())
            .with_database(self.database.clone())
            .with_user(self.user.clone());

        let start_time = std::time::Instant::now();

        let result = retry_with_circuit_breaker(&self.circuit_breaker, &self.retry_config, f).await;

        let duration = start_time.elapsed();
        let context = context.with_duration(duration);

        match result {
            Ok(value) => Ok(value),
            Err(error) => {
                let enhanced_error = match error {
                    ConnectError::Connection(_) | ConnectError::Timeout(_) => {
                        EnhancedError::DatabaseOperation {
                            operation: operation.to_string(),
                            error: Box::new(error),
                            context,
                        }
                    }
                    ConnectError::Network(_) => EnhancedError::NetworkOperation {
                        operation: operation.to_string(),
                        error: Box::new(error),
                        context,
                    },
                    _ => EnhancedError::DatabaseOperation {
                        operation: operation.to_string(),
                        error: Box::new(error),
                        context,
                    },
                };
                Err(enhanced_error)
            }
        }
    }

    /// Get a connection from the pool
    ///
    /// # Errors
    ///
    /// Returns an error if a connection cannot be obtained from the pool.
    pub async fn get_connection(&self) -> Result<PooledConn, EnhancedError> {
        self.execute_with_resilience("get_connection", || {
            self.pool.get_conn().map_err(ConnectError::from)
        })
        .await
    }

    /// Execute a query with error handling
    ///
    /// # Errors
    ///
    /// Returns an error if the query execution fails.
    pub async fn execute_query<F, T>(&self, _query: &str, f: F) -> Result<T, EnhancedError>
    where
        F: Fn(&mut PooledConn) -> Result<T, ConnectError> + Send + Sync,
    {
        self.execute_with_resilience("execute_query", || {
            let mut conn = self.pool.get_conn()?;
            f(&mut conn)
        })
        .await
    }
}

/// Helper function to create a retry configuration for database operations
#[must_use]
pub fn create_db_retry_config() -> RetryConfig {
    RetryConfig {
        max_retries: 5,
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(10),
        backoff_multiplier: 2.0,
    }
}

/// Helper function to create a circuit breaker configuration for database operations
#[must_use]
pub fn create_db_circuit_breaker_config() -> CircuitBreakerConfig {
    CircuitBreakerConfig {
        failure_threshold: 3,
        failure_window: Duration::from_secs(30),
        recovery_timeout: Duration::from_secs(10),
        success_threshold: 2,
    }
}

/// Error classification for different types of failures
#[must_use]
pub fn classify_error(error: &ConnectError) -> ErrorCategory {
    match error {
        ConnectError::Authentication(_)
        | ConnectError::Configuration(_)
        | ConnectError::Validation(_)
        | ConnectError::Parse(_)
        | ConnectError::CliArgument(_) => ErrorCategory::Permanent,
        ConnectError::Connection(_)
        | ConnectError::Timeout(_)
        | ConnectError::Network(_)
        | ConnectError::Database(_)
        | ConnectError::IsolationTest(_)
        | ConnectError::StateMachine(_)
        | ConnectError::Logging(_)
        | ConnectError::Io(_)
        | ConnectError::Retry(_)
        | ConnectError::CircuitBreaker(_)
        | ConnectError::Protocol(_)
        | ConnectError::Resource(_)
        | ConnectError::Unknown(_) => ErrorCategory::Transient,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    Transient, // Can be retried
    Permanent, // Should not be retried
    Unknown,   // Unknown if retryable
}

/// Error recovery strategies
#[must_use]
pub fn get_recovery_strategy(error: &ConnectError) -> RecoveryStrategy {
    match classify_error(error) {
        ErrorCategory::Transient => RecoveryStrategy::Retry,
        ErrorCategory::Permanent => RecoveryStrategy::FailFast,
        ErrorCategory::Unknown => RecoveryStrategy::RetryWithLimit,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryStrategy {
    Retry,          // Retry with exponential backoff
    FailFast,       // Don't retry, fail immediately
    RetryWithLimit, // Retry with limited attempts
}

/// Error context builder for database operations
pub struct ErrorContextBuilder {
    context: ErrorContext,
}

impl ErrorContextBuilder {
    #[must_use]
    pub fn new(operation: String) -> Self {
        Self {
            context: ErrorContext::new(operation),
        }
    }

    #[must_use]
    pub fn with_connection_info(mut self, host: String, database: String, user: String) -> Self {
        self.context = self
            .context
            .with_host(host)
            .with_database(database)
            .with_user(user);
        self
    }

    #[must_use]
    pub fn with_query(mut self, query: String) -> Self {
        self.context = self.context.with_info("query".to_string(), query);
        self
    }

    #[must_use]
    pub fn with_attempt(mut self, attempt: usize) -> Self {
        self.context = self.context.with_attempt(attempt);
        self
    }

    #[must_use]
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.context = self.context.with_duration(duration);
        self
    }

    #[must_use]
    pub fn with_additional_info(mut self, key: String, value: String) -> Self {
        self.context = self.context.with_info(key, value);
        self
    }

    #[must_use]
    pub fn build(self) -> ErrorContext {
        self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_classification() {
        let connection_error = ConnectError::Connection(mysql::Error::server_disconnected());
        assert_eq!(classify_error(&connection_error), ErrorCategory::Transient);

        let auth_error = ConnectError::Authentication("invalid credentials".to_string());
        assert_eq!(classify_error(&auth_error), ErrorCategory::Permanent);
    }

    #[test]
    fn test_recovery_strategies() {
        let connection_error = ConnectError::Connection(mysql::Error::server_disconnected());
        assert_eq!(
            get_recovery_strategy(&connection_error),
            RecoveryStrategy::Retry
        );

        let auth_error = ConnectError::Authentication("invalid credentials".to_string());
        assert_eq!(
            get_recovery_strategy(&auth_error),
            RecoveryStrategy::FailFast
        );
    }

    #[test]
    fn test_error_context_builder() {
        let context = ErrorContextBuilder::new("test_operation".to_string())
            .with_connection_info(
                "localhost".to_string(),
                "testdb".to_string(),
                "testuser".to_string(),
            )
            .with_query("SELECT * FROM test".to_string())
            .with_attempt(3)
            .with_duration(Duration::from_secs(5))
            .with_additional_info("custom_key".to_string(), "custom_value".to_string())
            .build();

        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.host, Some("localhost".to_string()));
        assert_eq!(context.database, Some("testdb".to_string()));
        assert_eq!(context.user, Some("testuser".to_string()));
        assert_eq!(context.attempt, 3);
        assert_eq!(context.duration, Duration::from_secs(5));
        assert_eq!(
            context.additional_info.get("query"),
            Some(&"SELECT * FROM test".to_string())
        );
        assert_eq!(
            context.additional_info.get("custom_key"),
            Some(&"custom_value".to_string())
        );
    }
}

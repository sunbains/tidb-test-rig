use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use crate::errors::{ConnectError, RetryConfig};

// RetryConfig is now defined in errors.rs

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Circuit is open, failing fast
    HalfOpen, // Testing if service is back
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: usize,
    /// Time window for counting failures
    pub failure_window: Duration,
    /// Time to wait before trying half-open
    pub recovery_timeout: Duration,
    /// Number of successful calls to close circuit
    pub success_threshold: usize,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            failure_window: Duration::from_secs(60),
            recovery_timeout: Duration::from_secs(30),
            success_threshold: 3,
        }
    }
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<Mutex<CircuitState>>,
    failure_count: Arc<Mutex<usize>>,
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    success_count: Arc<Mutex<usize>>,
    last_state_change: Arc<Mutex<Instant>>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_count: Arc::new(Mutex::new(0)),
            last_failure_time: Arc::new(Mutex::new(None)),
            success_count: Arc::new(Mutex::new(0)),
            last_state_change: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn get_state(&self) -> CircuitState {
        *self.state.lock().unwrap()
    }

    pub fn call<F, T, E>(&self, f: F) -> Result<T, ConnectError>
    where
        F: FnOnce() -> Result<T, E>,
        E: Into<ConnectError>,
    {
        let state = self.get_state();
        
        match state {
            CircuitState::Open => {
                if self.should_attempt_reset() {
                    self.set_state(CircuitState::HalfOpen);
                    self.call_half_open(f)
                } else {
                    Err(ConnectError::Connection(
                        mysql::Error::server_disconnected()
                    ))
                }
            }
            CircuitState::HalfOpen => self.call_half_open(f),
            CircuitState::Closed => self.call_closed(f),
        }
    }

    fn call_closed<F, T, E>(&self, f: F) -> Result<T, ConnectError>
    where
        F: FnOnce() -> Result<T, E>,
        E: Into<ConnectError>,
    {
        match f() {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(error) => {
                self.record_failure();
                Err(error.into())
            }
        }
    }

    fn call_half_open<F, T, E>(&self, f: F) -> Result<T, ConnectError>
    where
        F: FnOnce() -> Result<T, E>,
        E: Into<ConnectError>,
    {
        match f() {
            Ok(result) => {
                self.record_success();
                if self.should_close_circuit() {
                    self.set_state(CircuitState::Closed);
                }
                Ok(result)
            }
            Err(error) => {
                self.record_failure();
                self.set_state(CircuitState::Open);
                Err(error.into())
            }
        }
    }

    fn set_state(&self, new_state: CircuitState) {
        let mut state = self.state.lock().unwrap();
        *state = new_state;
        *self.last_state_change.lock().unwrap() = Instant::now();
    }

    fn record_success(&self) {
        let mut success_count = self.success_count.lock().unwrap();
        *success_count += 1;
    }

    fn record_failure(&self) {
        let mut failure_count = self.failure_count.lock().unwrap();
        let mut last_failure_time = self.last_failure_time.lock().unwrap();
        
        *failure_count += 1;
        *last_failure_time = Some(Instant::now());

        if *failure_count >= self.config.failure_threshold {
            self.set_state(CircuitState::Open);
        }
    }

    fn should_attempt_reset(&self) -> bool {
        let last_change = *self.last_state_change.lock().unwrap();
        Instant::now().duration_since(last_change) >= self.config.recovery_timeout
    }

    fn should_close_circuit(&self) -> bool {
        let success_count = *self.success_count.lock().unwrap();
        success_count >= self.config.success_threshold
    }
}

/// Retry with exponential backoff
pub async fn retry_with_backoff<F, T, E>(
    config: &RetryConfig,
    operation: F,
) -> Result<T, ConnectError>
where
    F: Fn() -> Result<T, E>,
    E: Into<ConnectError>,
{
    let mut attempt = 0;
    let mut delay = config.base_delay;

    loop {
        attempt += 1;

        match operation() {
            Ok(result) => return Ok(result),
            Err(error) => {
                        if attempt >= config.max_retries {
            return Err(error.into());
        }

        // Simple exponential backoff without jitter for now
        tokio::time::sleep(delay).await;

        // Exponential backoff
        delay = Duration::from_millis(
            (delay.as_millis() as f64 * config.backoff_multiplier) as u64
        );
        delay = delay.min(config.max_delay);
            }
        }
    }
}

/// Retry with circuit breaker
pub async fn retry_with_circuit_breaker<F, T, E>(
    circuit_breaker: &CircuitBreaker,
    retry_config: &RetryConfig,
    operation: F,
) -> Result<T, ConnectError>
where
    F: Fn() -> Result<T, E>,
    E: Into<ConnectError>,
{
    let operation_with_circuit = || circuit_breaker.call(&operation);
    retry_with_backoff(retry_config, operation_with_circuit).await
}

/// Error context for better debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub attempt: usize,
    pub duration: Duration,
    pub circuit_state: Option<CircuitState>,
    pub additional_info: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(operation: String) -> Self {
        Self {
            operation,
            attempt: 0,
            duration: Duration::ZERO,
            circuit_state: None,
            additional_info: std::collections::HashMap::new(),
        }
    }

    pub fn with_attempt(mut self, attempt: usize) -> Self {
        self.attempt = attempt;
        self
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn with_circuit_state(mut self, state: CircuitState) -> Self {
        self.circuit_state = Some(state);
        self
    }

    pub fn with_info(mut self, key: String, value: String) -> Self {
        self.additional_info.insert(key, value);
        self
    }
}

/// Enhanced error with context
#[derive(Debug)]
pub struct ContextualError {
    pub error: ConnectError,
    pub context: ErrorContext,
}

impl std::fmt::Display for ContextualError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (context: {:?})", self.error, self.context)
    }
}

impl std::error::Error for ContextualError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_retry_with_backoff() {
        let config = RetryConfig {
            max_retries: 3,
            base_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
        };

        let counter = AtomicUsize::new(0);
        let operation = || {
            let count = counter.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                Err("temporary failure")
            } else {
                Ok("success")
            }
        };

        let result = retry_with_backoff(&config, operation).await;
        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_circuit_breaker() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            failure_window: Duration::from_secs(60),
            recovery_timeout: Duration::from_millis(100),
            success_threshold: 1,
        };

        let circuit_breaker = CircuitBreaker::new(config);

        // First failure
        let result = circuit_breaker.call(|| Err::<(), &str>("failure"));
        assert!(result.is_err());
        assert_eq!(circuit_breaker.get_state(), CircuitState::Closed);

        // Second failure - circuit should open
        let result = circuit_breaker.call(|| Err::<(), &str>("failure"));
        assert!(result.is_err());
        assert_eq!(circuit_breaker.get_state(), CircuitState::Open);

        // Wait for recovery timeout
        std::thread::sleep(Duration::from_millis(150));

        // Should be half-open now
        let result = circuit_breaker.call(|| Ok::<(), &str>(()));
        assert!(result.is_ok());
        assert_eq!(circuit_breaker.get_state(), CircuitState::Closed);
    }
} 
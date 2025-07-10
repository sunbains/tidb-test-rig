use thiserror::Error;

/// Main error type for the TiDB connection and testing framework
#[derive(Error, Debug)]
pub enum ConnectError {
    #[error("Connection error: {0}")]
    Connection(mysql::Error),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("State machine error: {0}")]
    StateMachine(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Import job error: {0}")]
    ImportJob(String),

    #[error("Isolation test error: {0}")]
    IsolationTest(String),

    #[error("CLI argument error: {0}")]
    CliArgument(String),

    #[error("Logging error: {0}")]
    Logging(String),

    #[error("IO error: {0}")]
    Io(std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Specific error types for different components
#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("Failed to connect to {host}:{port}: {message}")]
    ConnectFailed {
        host: String,
        port: u16,
        message: String,
    },

    #[error("Authentication failed for user {user}: {message}")]
    AuthFailed {
        user: String,
        message: String,
    },

    #[error("Database {database} does not exist or access denied")]
    DatabaseNotFound { database: String },

    #[error("Connection pool error: {0}")]
    PoolError(#[from] mysql::Error),

    #[error("Connection timeout after {timeout_secs} seconds")]
    Timeout { timeout_secs: u64 },
}

#[derive(Error, Debug)]
pub enum StateMachineError {
    #[error("No handler registered for state: {state}")]
    NoHandlerRegistered { state: String },

    #[error("Invalid state transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },

    #[error("State machine context error: {0}")]
    ContextError(String),

    #[error("Handler execution failed: {0}")]
    HandlerError(String),
}

#[derive(Error, Debug)]
pub enum ImportJobError {
    #[error("No active import jobs found")]
    NoActiveJobs,

    #[error("Import job {job_id} not found")]
    JobNotFound { job_id: String },

    #[error("Failed to monitor import job {job_id}: {message}")]
    MonitorFailed {
        job_id: String,
        message: String,
    },

    #[error("Import job monitoring timeout after {duration_secs} seconds")]
    MonitorTimeout { duration_secs: u64 },
}

#[derive(Error, Debug)]
pub enum IsolationTestError {
    #[error("Failed to create test table {table}: {message}")]
    TableCreationFailed {
        table: String,
        message: String,
    },

    #[error("Failed to populate test data: {0}")]
    DataPopulationFailed(String),

    #[error("Isolation test failed: {0}")]
    TestFailed(String),

    #[error("Failed to clean up test table {table}: {message}")]
    CleanupFailed {
        table: String,
        message: String,
    },
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Missing required argument: {arg}")]
    MissingArgument { arg: String },

    #[error("Invalid argument value for {arg}: {value}")]
    InvalidArgumentValue { arg: String, value: String },

    #[error("Conflicting arguments: {args}")]
    ConflictingArguments { args: String },

    #[error("Password required but not provided")]
    PasswordRequired,
}

// Type aliases for backward compatibility
pub type StateError = ConnectError;
pub type Result<T> = std::result::Result<T, ConnectError>;

// Conversion implementations
impl From<mysql::Error> for ConnectError {
    fn from(err: mysql::Error) -> Self {
        ConnectError::Connection(err)
    }
}

impl From<std::io::Error> for ConnectError {
    fn from(err: std::io::Error) -> Self {
        ConnectError::Io(err)
    }
}

impl From<String> for ConnectError {
    fn from(err: String) -> Self {
        ConnectError::Unknown(err)
    }
}

impl From<&str> for ConnectError {
    fn from(err: &str) -> Self {
        ConnectError::Unknown(err.to_string())
    }
}

impl From<ConnectionError> for ConnectError {
    fn from(_err: ConnectionError) -> Self {
        ConnectError::Connection(mysql::Error::server_disconnected())
    }
}

impl From<StateMachineError> for ConnectError {
    fn from(err: StateMachineError) -> Self {
        ConnectError::StateMachine(err.to_string())
    }
}

impl From<ImportJobError> for ConnectError {
    fn from(err: ImportJobError) -> Self {
        ConnectError::ImportJob(err.to_string())
    }
}

impl From<IsolationTestError> for ConnectError {
    fn from(err: IsolationTestError) -> Self {
        ConnectError::IsolationTest(err.to_string())
    }
}

impl From<CliError> for ConnectError {
    fn from(err: CliError) -> Self {
        ConnectError::CliArgument(err.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for ConnectError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        ConnectError::Unknown(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_connect_error_display() {
        let err = ConnectError::Configuration("bad config".to_string());
        assert!(format!("{}", err).contains("bad config"));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::Other, "io fail");
        let err: ConnectError = io_err.into();
        assert!(format!("{}", err).contains("io fail"));
    }

    #[test]
    fn test_result_alias() {
        fn returns_result() -> Result<u32> { Ok(42) }
        assert_eq!(returns_result().unwrap(), 42);
    }
} 
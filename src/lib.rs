pub mod cli;
pub mod config;
pub mod config_extensions;
pub mod connection;
pub mod connection_manager;
pub mod errors;
pub mod lib_utils;
pub mod logging;
pub mod multi_connection_state_machine;
pub mod state_handlers;
pub mod state_machine;
pub mod retry;
pub mod error_utils;

pub use cli::{CommonArgs, get_connection_info, parse_args};
pub use config::{AppConfig, ConfigBuilder, DatabaseConfig, LoggingConfig, TestConfig};
pub use config_extensions::{
    ConfigExtension, apply_extensions_to_command, apply_extensions_to_config,
    print_extensions_help, register_config_extension,
};
pub use connection_manager::{ConnectionCoordinator, ConnectionInfo, GlobalConfig, SharedState};
pub use errors::{
    CliError, ConnectError, ConnectionError, Result,
    StateError,
};
pub use lib_utils::register_standard_handlers;
pub use lib_utils::{
    CommonArgsSetup, TestSetup, print_error_and_exit, print_success, print_test_header,
};
pub use logging::{
    ErrorContext, LogConfig, init_default_logging, init_logging, init_logging_from_env,
    log_memory_usage, log_performance_metric,
};
pub use multi_connection_state_machine::{CoordinationHandler, MultiConnectionStateMachine};
pub use state_handlers::{
    ConnectingHandler, InitialHandler, ParsingConfigHandler, TestingConnectionHandler,
    VerifyingDatabaseHandler,
};

// Re-export retry types for convenience
pub use retry::{
    CircuitBreaker,
    CircuitBreakerConfig,
    CircuitState,
    retry_with_backoff,
    retry_with_circuit_breaker,
    ErrorContext as RetryErrorContext,
};
pub use errors::RetryConfig;

// Re-export error utility types
pub use error_utils::{
    ResilientConnectionManager,
    create_db_retry_config,
    create_db_circuit_breaker_config,
    classify_error,
    get_recovery_strategy,
    ErrorCategory,
    RecoveryStrategy,
    ErrorContextBuilder,
};

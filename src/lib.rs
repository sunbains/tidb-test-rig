pub mod cli;
pub mod config;
pub mod config_extensions;
pub mod connection;
pub mod connection_manager;
pub mod errors;
pub mod lib_utils;
pub mod logging;
pub mod multi_connection_state_machine;
pub mod retry;
pub mod state_handlers;
pub mod state_machine;
pub mod error_utils;

#[cfg(feature = "python_plugins")]
pub mod python_bindings;

pub use cli::{CommonArgs, get_connection_info, parse_args};
pub use config::{AppConfig, ConfigBuilder, DatabaseConfig, LoggingConfig, TestConfig};
pub use config_extensions::{ConfigExtension, apply_extensions_to_command, apply_extensions_to_config, print_extensions_help, register_config_extension};
pub use connection_manager::{ConnectionCoordinator, ConnectionInfo, GlobalConfig, SharedState};
pub use errors::{ConnectError, Result, StateError, RetryConfig};
pub use lib_utils::{print_error_and_exit, print_success, print_test_header};
pub use logging::init_logging;
pub use multi_connection_state_machine::MultiConnectionStateMachine;
pub use retry::{CircuitBreaker, CircuitBreakerConfig, CircuitState, retry_with_backoff, retry_with_circuit_breaker};
pub use state_handlers::*;
pub use state_machine::{State, StateContext, StateHandler, StateMachine};

#[cfg(feature = "python_plugins")]
pub use python_bindings::{load_python_handlers, register_python_handler, PyStateHandler};

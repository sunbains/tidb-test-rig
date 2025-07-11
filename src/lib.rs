//! # test_rig - TiDB Connection and Testing Framework
//!
//! A Rust framework for testing TiDB connections with state machine-based workflows,
//! Python plugin support, and comprehensive error handling.
//!
//! ## Core Features
//! - State machine for managing database connection workflows
//! - Python plugin system for custom state handlers
//! - Multi-connection support with load balancing
//! - Comprehensive error handling and retry mechanisms
//! - Configuration management with file and environment support
//! - CLI utilities for common operations

/// Command-line interface support and argument parsing
pub mod cli;

/// Configuration management with file, environment, and programmatic support
pub mod config;

/// Configuration extensions for dynamic configuration
pub mod config_extensions;

/// Low-level database connection utilities and parsing
pub mod connection;

/// High-level connection management and coordination
pub mod connection_manager;

/// Enhanced error utilities and context management
pub mod error_utils;

/// Comprehensive error types and retry mechanisms
pub mod errors;

/// Common utility functions and helpers
pub mod lib_utils;

/// Structured logging configuration and utilities
pub mod logging;

/// State machine for managing multiple database connections
pub mod multi_connection_state_machine;

/// Retry mechanisms with circuit breaker pattern
pub mod retry;

/// Built-in state handler implementations
pub mod state_handlers;

/// Core state machine implementation for TiDB connection workflows
pub mod state_machine;

/// Dynamic state machine implementation for test-defined states
pub mod state_machine_dynamic;

/// Common state definitions shared across binaries
pub mod common_states;

/// Python plugin system using PyO3 for writing state handlers in Python
#[cfg(feature = "python_plugins")]
pub mod python_bindings;

pub use cli::{CommonArgs, get_connection_info, parse_args};
pub use config::{AppConfig, ConfigBuilder, DatabaseConfig, LoggingConfig, TestConfig};
pub use config_extensions::{
    ConfigExtension, apply_extensions_to_command, apply_extensions_to_config,
    print_extensions_help, register_config_extension,
};
pub use connection_manager::{ConnectionCoordinator, ConnectionInfo, GlobalConfig, SharedState};
pub use errors::{ConnectError, Result, RetryConfig, StateError};
pub use lib_utils::{print_error_and_exit, print_success, print_test_header};
pub use logging::init_logging;
pub use multi_connection_state_machine::MultiConnectionStateMachine;
pub use retry::{
    CircuitBreaker, CircuitBreakerConfig, CircuitState, retry_with_backoff,
    retry_with_circuit_breaker,
};
pub use state_handlers::*;
pub use state_machine::{State, StateContext, StateHandler, StateMachine};
pub use state_machine_dynamic::{
    DynamicState, DynamicStateContext, DynamicStateHandler, DynamicStateMachine, states,
};

#[cfg(feature = "python_plugins")]
pub use python_bindings::{PyStateHandler, load_python_handlers, register_python_handler};

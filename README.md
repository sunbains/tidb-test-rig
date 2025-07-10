[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

# TiDB Testing Framework

A Rust library framework for testing TiDB database functionality. It should be possible to use it for MySQL too.

## Overview

This framework provides examples for different TiDB testing scenarios:
- Basic connection testing
- Multi-connection testing
- Transaction isolation testing
- Import job monitoring
- Configuration file generation


The multi-connection feature is for creating separate state machines and using a coordinator to manage and control
the different connections for sophisticated concurrent/parallel testing.

## Features

- **Connection Testing**: Test basic database connectivity and simple queries
- **Multi-Connection Testing**: Test multiple concurrent connections
- **Isolation Testing**: Test transaction isolation levels
- **Configuration Management**: JSON and TOML configuration files
- **Command Line Interface**: Flexible CLI options for all tests
- **Enhanced Error Handling**: Retry strategies, circuit breakers, and error context preservation

## Enhanced Error Handling

The framework includes sophisticated error handling capabilities for production-ready database operations:

### Retry Strategies
- **Exponential Backoff**: Configurable retry delays with exponential increase
- **Jitter**: Random delay variation to prevent thundering herd problems
- **Timeout Control**: Overall operation timeout with per-attempt limits
- **Configurable Attempts**: Set maximum retry attempts per operation

### Circuit Breaker Pattern
- **Failure Threshold**: Open circuit after specified number of failures
- **Recovery Timeout**: Wait period before attempting to close circuit
- **Half-Open State**: Test service health before fully closing circuit
- **Success Threshold**: Number of successful calls to close circuit

### Error Context Preservation
- **Rich Context**: Timestamp, operation name, attempt count, duration
- **Connection Info**: Host, database, user information
- **Custom Metadata**: Additional key-value pairs for debugging
- **Builder Pattern**: Fluent API for building error contexts

### Error Classification
- **Transient Errors**: Automatically retried (connection failures, timeouts)
- **Permanent Errors**: Fail fast (authentication, configuration errors)
- **Recovery Strategies**: Appropriate handling based on error type

### Usage Examples

```rust
use test_rig::{
    ResilientConnectionManager,
    create_db_retry_config,
    create_db_circuit_breaker_config,
    classify_error,
    get_recovery_strategy,
};

// Create resilient connection manager
let retry_config = create_db_retry_config();
let circuit_config = create_db_circuit_breaker_config();
let manager = ResilientConnectionManager::with_custom_config(
    pool,
    circuit_config,
    retry_config,
);

// Execute with automatic retry and circuit breaker
let result = manager.execute_with_resilience("critical_query", || {
    // Your database operation here
    perform_database_operation()
}).await;

// Error classification and recovery
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

## Python Plugin Support

The framework now supports Python plugins for state handlers, allowing you to write test logic in Python while leveraging the Rust state machine infrastructure.

### Features

- **Python State Handlers**: Write state handlers in Python using the `PyStateHandler` base class
- **Seamless Integration**: Python handlers integrate with the Rust state machine
- **Type Safety**: Full type hints and validation between Rust and Python
- **Standalone Testing**: Python handlers can be tested independently
- **Configuration Support**: Python scripts read from the same config files and environment variables

### Quick Start with Python

#### 1. Install Python Dependencies

```bash
# Install mysql-connector-python for database connections
pip install mysql-connector-python
```

#### 2. Run Python Isolation Test

```bash
# Using environment variables
export TIDB_HOST=tidb.qyruvz1u6xtd.clusters.dev.tidb-cloud.com:4000
export TIDB_USER=your_user
export TIDB_PASSWORD=your_password
export TIDB_DATABASE=your_database
python3 examples/run_isolation_test.py

# Using command line arguments
python3 examples/run_isolation_test.py \
  --host tidb.qyruvz1u6xtd.clusters.dev.tidb-cloud.com:4000 \
  --user your_user \
  --password your_password \
  --database your_database

# Using configuration file
python3 examples/run_isolation_test.py --config tidb_config.json
```

#### 3. Create Custom Python Handlers

```python
from test_rig_python import PyStateHandler, PyStateContext, PyState

class MyCustomHandler(PyStateHandler):
    def __init__(self):
        super().__init__()
        self.counter = 0
    
    def enter(self, context: PyStateContext) -> str:
        print(f"Entering state with host: {context.host}")
        return PyState.initial()
    
    def execute(self, context: PyStateContext) -> str:
        self.counter += 1
        print(f"Executing (attempt {self.counter})")
        
        if context.connection:
            # Execute database operations
            results = context.connection.execute_query("SELECT 1")
            return PyState.completed()
        else:
            return PyState.connecting()
    
    def exit(self, context: PyStateContext) -> None:
        print(f"Exiting state (total executions: {self.counter})")
```

### Python Handler Integration

Python handlers can be integrated with the Rust state machine:

```rust
use test_rig::python_bindings::register_python_handler;

// Register Python handler for a specific state
let py_handler = load_python_handler("my_module.MyCustomHandler")?;
register_python_handler(&mut state_machine, State::TestingConnection, py_handler)?;
```

### Available Python Examples

- **`examples/python_handlers.py`**: Collection of example Python handlers
- **`examples/run_isolation_test.py`**: Standalone Python isolation test
- **`examples/test_rig_python.pyi`**: Type stubs for IDE support

### Configuration Priority

Python scripts follow the same configuration priority as Rust binaries:
1. Command-line arguments (highest priority)
2. Environment variables (`TIDB_HOST`, `TIDB_USER`, `TIDB_PASSWORD`, `TIDB_DATABASE`)
3. Configuration file (`tidb_config.json` or `tidb_config.toml`)
4. Default values (lowest priority)

For more detailed information about the Python plugin system, see the [Python Plugin Documentation](docs/PYTHON_PLUGIN.md).

## Quick Start

### 1. Generate Configuration

Create a configuration file using the config generator:

```bash
# Generate default JSON configuration
cargo run --bin config_gen

# Generate TOML configuration with custom settings
cargo run --bin config_gen -- --format toml --host my-tidb:4000 --username myuser --database mydb
```

### 2. Run Tests

Use configuration files with any test:

```bash
# Basic connection test with config file
cargo run --bin basic -- -c tidb_config.json

# Import Job monitoring with config file
cargo run --bin job_monitor --features import_jobs -- -c tidb_config.json

# Isolation test with config file
cargo run --bin isolation --features isolation_test -- -c tidb_config.json
```

## Configuration

### Configuration File Format

The framework supports both JSON and TOML configuration files:

#### JSON Example (`tidb_config.json`)
```json
{
  "database": {
    "host": "localhost:4000",
    "username": "root",
    "password": null,
    "database": "test",
    "pool_size": 5,
    "timeout_secs": 30
  },
  "logging": {
    "level": "info",
    "format": "text",
    "file": null,
    "console": true
  },
  "test": {
    "rows": 10,
    "timeout_secs": 60,
    "verbose": false
  }
}
```

#### TOML Example (`tidb_config.toml`)
```toml
[database]
host = "localhost:4000"
username = "root"
# password = "your_password_here"
database = "test"
pool_size = 5
timeout_secs = 30

[logging]
level = "info"
format = "text"
console = true

[test]
rows = 10
timeout_secs = 60
verbose = false
```

### Environment Variables

You can override configuration settings using environment variables:

```bash
export TIDB_HOST="my-tidb:4000"
export TIDB_USERNAME="myuser"
export TIDB_PASSWORD="mypassword"
export TIDB_DATABASE="mydb"
export TIDB_LOG_LEVEL="debug"
export TIDB_TEST_ROWS="20"
export TIDB_MONITOR_DURATION="600"
```

### Configuration Priority

1. Command-line arguments (highest priority)
2. Environment variables
3. Configuration file
4. Default values (lowest priority)

## Available Tests

### Basic Connection Test
```bash
cargo run --bin basic
cargo run --bin basic -- -c config.json
```

### Multi-Connection Test
```bash
cargo run --bin simple_multi_connection --features multi_connection
cargo run --bin multi_connection --features multi_connection,import_jobs
```

### Isolation Test
```bash
cargo run --bin isolation --features isolation_test
cargo run --bin isolation --features isolation_test -- --test-rows 20
```

### Job Monitoring
```bash
cargo run --bin job_monitor --features import_jobs
cargo run --bin job_monitor --features import_jobs -- --monitor-duration 600
```

## Command Line Options

All binaries support these common options:

- `-c, --config <FILE>`: Configuration file path
- `-H, --host <HOST>`: Database host (hostname:port)
- `-u, --user <USER>`: Database username
- `-d, --database <DB>`: Database name
- `--password <PASSWORD>`: Database password
- `--no-password-prompt`: Skip password prompt
- `--log-level <LEVEL>`: Log level (debug, info, warn, error)
- `--log-file`: Enable file logging
- `--log-file-path <PATH>`: Log file path
- `-v, --verbose`: Enable verbose logging

### Test-Specific Options

#### Job Monitor Options
- `-t, --monitor-duration <SECONDS>`: Import job monitoring duration (default: 300)
- `--import-config <FILE>`: Import job config file path (JSON or TOML)
- `--update-interval <SECONDS>`: Status update interval
- `--show-details`: Show detailed job information

#### Isolation Test Options
- `--test-rows <ROWS>`: Number of test rows to use
- `--isolation-level <LEVEL>`: Database isolation level
- `--concurrent-connections <COUNT>`: Number of concurrent connections

## Project Structure

```
test_rig/
├── src/
│   ├── lib.rs              # Main library with shared functionality
│   ├── config.rs           # Configuration management
│   ├── cli.rs              # CLI argument parsing
│   ├── errors.rs           # Error types and handling
│   ├── retry.rs            # Retry strategies and circuit breakers
│   ├── error_utils.rs      # Error handling utilities
│   ├── state_machine.rs    # Basic state management for tests
│   ├── state_handlers.rs   # State handlers for different test phases
│   ├── import_job_handlers.rs # Import job specific handlers
│   ├── lib_utils.rs        # Utility functions
│   └── bin/                # Binary executables (separate workspace)
│       ├── Cargo.toml      # Binary package configuration
│       ├── basic.rs        # Basic connection test
│       ├── config_gen.rs   # Configuration file generator
│       ├── isolation.rs    # Transaction isolation test
│       ├── job_monitor.rs  # Import job monitoring
│       ├── multi_connection.rs # Multi-connection test
│       └── simple_multi_connection.rs # Simple multi-connection test
├── examples/               # Example usage
│   └── enhanced_error_handling.rs # Enhanced error handling examples
├── Cargo.toml              # Main package configuration (workspace)
└── README.md               # This file
```

## Development

### Building

```bash
# Build entire workspace (library + binaries)
cargo build --workspace

# Build only the library
cargo build

# Build specific binary
cargo build --bin basic

# Build with features
cargo build --bin job_monitor --features import_jobs
```

### Testing

```bash
# Run all tests in workspace
cargo test --workspace

# Run only library tests
cargo test

# Run with specific features
cargo test --features import_jobs
```

### Code Quality

```bash
# Check code
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

### Running Examples

```bash
# Run enhanced error handling example
cargo run --example enhanced_error_handling
```

## Configuration Extension System

The configuration generator (`config_gen.rs`) supports a plugin pattern for test-specific configuration options. This allows new tests to add their own CLI arguments and config logic without modifying the core generator.

### How It Works
- Each test binary can define a `ConfigExtension` implementing the `ConfigExtension` trait
- The extension registers itself with the config generator at runtime
- When you run `config_gen`, all registered extensions add their CLI options and config logic
- This keeps test-specific config code in the test binary, not in the core generator

### Example: Adding a Test-Specific Option
To add a `--test-rows` option for the isolation test:

**In `src/bin/isolation.rs`:**
```rust
use test_rig::{ConfigExtension, register_config_extension};
use clap::Command;

struct IsolationConfigExtension;

impl ConfigExtension for IsolationConfigExtension {
    fn add_cli_args(&self, app: Command) -> Command {
        app.arg(
            clap::Arg::new("test-rows")
                .long("test-rows")
                .help("Number of test rows to create for isolation testing")
                .default_value("10")
        )
    }
    fn build_config(&self, args: &clap::ArgMatches, config: &mut test_rig::config::AppConfig) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if let Some(test_rows) = args.get_one::<String>("test-rows") {
            if let Ok(rows) = test_rows.parse::<u32>() {
                config.test.rows = rows;
            }
        }
        Ok(())
    }
    fn get_extension_name(&self) -> &'static str { "isolation_test" }
    fn get_help_text(&self) -> &'static str { "Adds --test-rows option for isolation testing" }
}

fn register_extensions() {
    register_config_extension(Box::new(IsolationConfigExtension));
}

// In your main function, call register_extensions() before parsing args:
fn main() {
    register_extensions();
    // ... rest of main
}
```

### Using the Extension
Now, when you run the config generator, the `--test-rows` option will be available:

```bash
cargo run --bin config_gen -- --test-rows 25 --host my-tidb:4000 --username myuser
```

## License

[LICENSE file](LICENSE)


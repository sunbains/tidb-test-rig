[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)


# TiDB Connection and Testing Framework

A comprehensive Rust framework for testing TiDB connections, monitoring import jobs, and performing isolation tests.

## Features

- **Multiple Connection Types**: Basic, simple, multi-connection, and isolation testing
- **Import Job Monitoring**: Real-time monitoring of TiDB import jobs
- **State Machine Architecture**: Robust state-based testing framework
- **Comprehensive Logging**: Configurable logging with file and console output
- **External Configuration**: Support for JSON and TOML configuration files
- **Environment Variable Overrides**: Flexible configuration management
- **Error Handling**: Rich error types with `thiserror`
- **Modular Design**: Test-specific configurations are self-contained within their respective binaries

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

# Job monitoring with config file
cargo run --bin job_monitor --features import_jobs -- -c tidb_config.json

# Isolation test with config file
cargo run --bin isolation --features isolation_test -- -c tidb_config.json
```

## Configuration

### Core Configuration Structure

The framework uses a modular configuration approach:
- **Core Config** (`src/config.rs`): Contains shared database, logging, and test configurations
- **Test-Specific Configs**: Each binary contains its own test-specific configuration structs and parsing logic

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

### Test-Specific Configurations

Each test binary contains its own configuration structs for test-specific settings:

#### Job Monitor Config (`src/bin/job_monitor.rs`)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportJobConfig {
    pub monitor_duration: u64,
    pub update_interval: u64,
    pub show_details: bool,
}
```

#### Isolation Test Config (`src/bin/isolation.rs`)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationTestConfig {
    pub test_rows: usize,
    pub isolation_level: String,
    pub concurrent_connections: usize,
}
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

### Simple Connection Test
```bash
cargo run --bin simple_connection
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

### Logging Test
```bash
cargo run --bin logging
cargo run --bin logging -- --log-file --log-file-path logs/test.log
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
- `-t, --monitor-duration <SECONDS>`: Import job monitoring duration
- `--update-interval <SECONDS>`: Status update interval
- `--show-details`: Show detailed job information

#### Isolation Test Options
- `--test-rows <ROWS>`: Number of test rows to use
- `--isolation-level <LEVEL>`: Database isolation level
- `--concurrent-connections <COUNT>`: Number of concurrent connections

## Project Structure

```
src/
├── bin/                    # Binary executables with test-specific configs
│   ├── basic.rs           # Basic connection test
│   ├── simple_connection.rs
│   ├── simple_multi_connection.rs
│   ├── multi_connection.rs
│   ├── isolation.rs       # Isolation testing with IsolationTestConfig
│   ├── job_monitor.rs     # Import job monitoring with ImportJobConfig
│   ├── logging.rs         # Logging test
│   └── config_gen.rs      # Configuration generator
├── cli.rs                 # Command-line interface utilities
├── config.rs              # Core configuration management (AppConfig, DatabaseConfig, etc.)
├── connection.rs          # Database connection utilities
├── errors.rs              # Error types and handling
├── import_job_handlers.rs # Import job state handlers
├── import_job_monitor.rs  # Import job monitoring
├── lib_utils.rs           # Common utilities
├── logging.rs             # Logging configuration
├── state_machine.rs       # State machine framework
├── state_handlers.rs      # State handlers
└── multi_connection_state_machine.rs
```

## Configuration Architecture

### Design Principles

1. **Separation of Concerns**: Core configs are shared, test-specific configs are isolated
2. **Self-Contained Tests**: Each binary contains its own configuration parsing and validation
3. **Reduced Coupling**: Test binaries don't depend on each other's config structures
4. **Maintainability**: Changes to test-specific configs don't affect other tests

### Benefits

- **Better Organization**: Test-specific configs are co-located with their tests
- **Reduced Dependencies**: No cross-dependencies between test configs
- **Easier Maintenance**: Changes to one test's config don't affect others
- **Clear Boundaries**: Clear separation between core and test-specific functionality

## Error Handling

The framework uses `thiserror` for comprehensive error handling:

- `ConnectError`: Main error type for all operations
- `ConnectionError`: Database connection specific errors
- `StateMachineError`: State machine operation errors
- `ImportJobError`: Import job monitoring errors
- `IsolationTestError`: Isolation testing errors
- `CliError`: Command-line interface errors

## Development

### Building

```bash
# Build all binaries
cargo build

# Build specific binary
cargo build --bin basic

# Build with features
cargo build --bin job_monitor --features import_jobs
```

### Testing

```bash
# Run unit tests
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

## Examples

### Example Configuration Files

The framework generates sample configuration files:
- `tidb_config.json` - JSON configuration example
- `tidb_config.toml` - TOML configuration example

### Example Usage

```bash
# Generate configuration
cargo run --bin config_gen -- --host my-tidb:4000 --username myuser --database mydb

# Run basic test with configuration
cargo run --bin basic -- -c tidb_config.json

# Run job monitoring with custom duration
cargo run --bin job_monitor --features import_jobs -- -c tidb_config.json --monitor-duration 600

# Run isolation test with verbose logging
cargo run --bin isolation --features isolation_test -- -c tidb_config.json --verbose
```

## License

[LICENSE file](LICENSE)


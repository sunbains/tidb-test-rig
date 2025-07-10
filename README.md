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
  },
  "import_jobs": {
    "monitor_duration": 300,
    "update_interval": 5,
    "show_details": true
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

[import_jobs]
monitor_duration = 300
update_interval = 5
show_details = true
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
- `-t, --monitor-duration <SECONDS>`: Import job monitoring duration
- `--password <PASSWORD>`: Database password
- `--no-password-prompt`: Skip password prompt
- `--log-level <LEVEL>`: Log level (debug, info, warn, error)
- `--log-file`: Enable file logging
- `--log-file-path <PATH>`: Log file path
- `-v, --verbose`: Enable verbose logging

## Project Structure

```
src/
├── bin/                    # Binary executables
│   ├── basic.rs           # Basic connection test
│   ├── simple_connection.rs
│   ├── simple_multi_connection.rs
│   ├── multi_connection.rs
│   ├── isolation.rs       # Isolation testing
│   ├── job_monitor.rs     # Import job monitoring
│   ├── logging.rs         # Logging test
│   └── config_gen.rs      # Configuration generator
├── cli.rs                 # Command-line interface
├── config.rs              # Configuration management
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

See `examples/` directory for sample configuration files:
- `examples/tidb_config.json` - JSON configuration example
- `examples/tidb_config.toml` - TOML configuration example

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


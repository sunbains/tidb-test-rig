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
│   ├── config.rs           # Core configuration management (AppConfig, DatabaseConfig, etc.)
│   ├── cli.rs              # CLI argument parsing
│   ├── errors.rs           # Error types and handling
│   ├── state_machine.rs    # State machine framework
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
├── Cargo.toml              # Main package configuration (workspace)
└── README.md               # This file
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

## Configuration Plugin Extension System

The configuration generator (`config_gen.rs`) supports a **plugin pattern** for test-specific configuration options. This allows new tests to add their own CLI arguments and config logic without modifying the core generator.

### How It Works
- Each test binary can define a `ConfigExtension` implementing the `ConfigExtension` trait.
- The extension registers itself with the config generator at runtime.
- When you run `config_gen`, all registered extensions add their CLI options and config logic.
- This keeps test-specific config code in the test binary, not in the core generator.

### Example: Adding a Test-Specific Option
Suppose you want to add a `--test-rows` option for the isolation test:

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

The generated config will include the test-specific setting, and the isolation test will use it automatically.

### Benefits
- **No core changes needed** for new test options
- **Test-specific config stays with the test**
- **Easy to extend**: just implement and register a new extension
- **All CLI options are available in `config_gen --help`**

## License

[LICENSE file](LICENSE)


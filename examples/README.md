# TiDB Connection Test Tool - Examples

This directory contains example programs demonstrating how to use the TiDB connection test tool with a common CLI library and robust logging.

## Common CLI Library

All examples use a shared command-line interface library that provides:

- **Standardized Arguments**: Common host, user, database, and monitoring parameters
- **Environment Variable Support**: Configuration via `TIDB_HOST`, `TIDB_USER`, `TIDB_DATABASE`, `TIDB_PASSWORD`
- **Flexible Password Input**: Command line, environment variable, or interactive prompt
- **Parameter Validation**: Automatic validation of connection parameters
- **Help and Usage**: Built-in help with `--help` flag
- **Integrated Logging**: Control log level, file output, and verbosity from the CLI

### CLI Usage

```bash
# Basic usage with interactive password prompt
cargo run --example basic_example -- -H localhost:4000 -u root -d test

# Using environment variables
export TIDB_HOST=localhost:4000
export TIDB_USER=root
export TIDB_PASSWORD=mypassword
cargo run --example basic_example

# Using command line password (less secure)
cargo run --example basic_example -- -H localhost:4000 -u root --password mypassword

# Skip password prompt (for automated testing)
cargo run --example basic_example -- -H localhost:4000 -u root --no-password-prompt

# Enable debug logging to console
cargo run --example basic_example -- --log-level debug

# Log to a file
cargo run --example basic_example -- --log-file --log-file-path logs/mylog.log
```

### Available Arguments

**Common Arguments (all examples):**
- `-H, --host`: Hostname and port (default: localhost:4000)
- `-u, --user`: Username (default: root)
- `-d, --database`: Database name (optional)
- `--password`: Password from command line (alternative to prompt)
- `--no-password-prompt`: Skip password prompt (for automated testing)
- `--log-level`: Log level (`debug`, `info`, `warn`, `error`; default: `info`)
- `--log-file`: Enable file logging
- `--log-file-path`: Path to log file (default: logs/tidb_connect.log)
- `-v, --verbose`: Shortcut for debug logging

**Example-specific Arguments:**
- `-t, --monitor-duration`: Duration to monitor import jobs in seconds (default: 60) - *multi-connection examples*
- `--test-rows`: Number of test rows to create for isolation testing (default: 10) - *isolation test examples*
- `--connection-count`: Number of connections to create for multi-connection tests (default: 2) - *multi-connection examples*

## Logging Facility

The project uses the [`tracing`](https://docs.rs/tracing) ecosystem for structured, high-performance logging.

- **Log to console and/or file**
- **Control log level at runtime** (`--log-level debug`)
- **Structured logs for connection, query, state transitions, and errors**
- **Performance and memory usage metrics**
- **Error context for troubleshooting**

### Example Logging Usage

```rust
use tracing::{info, debug, warn, error};

info!("Connected to TiDB");
debug!("Query executed: {}", query);
warn!("Slow query detected");
error!("Failed to connect: {}", err);
```

You can also use provided macros for common events:

```rust
log_connection_attempt!(host, user);
log_query!(query);
log_state_transition!(from, to);
```

### Example: Logging to File

```bash
cargo run --example basic_example -- --log-level debug --log-file --log-file-path logs/mylog.log
```

## Available Examples

### 1. Basic Example (`basic_example.rs`)
A comprehensive example showing how to connect to TiDB and perform basic operations.

**Features:**
- Uses the common CLI library for argument parsing
- Demonstrates basic connection testing
- Shows version checking and database verification
- Includes import job monitoring capabilities
- Minimal example for getting started

### 2. Isolation Test Example (`isolation_test_example.rs`)
A comprehensive example testing TiDB's repeatable read isolation.

**Features:**
- Creates test tables and populates data
- Tests transaction isolation with multiple connections
- Demonstrates repeatable read behavior
- Uses the common CLI library for configuration

### 3. Simple Multi-Connection Example (`simple_multi_connection.rs`)
A basic example showing how to create and manage multiple TiDB connections with the state machine infrastructure.

**Features:**
- Creates multiple connections to different TiDB instances
- Demonstrates basic connection coordination
- Shows how to handle connection states and errors

### 4. Advanced Multi-Connection Example (`multi_connection_example.rs`)
A comprehensive example showing advanced multi-connection scenarios with import job monitoring.

**Features:**
- Complex connection management with shared state
- Import job monitoring across multiple connections
- Advanced error handling and recovery
- Coordination between multiple state machines

### 5. Logging Example (`logging_example.rs`)
A demonstration of the logging facility, including log levels, file output, performance metrics, and error context.

**Features:**
- Shows how to use the logging macros and error context
- Demonstrates logging to both console and file
- Logs performance and memory usage metrics
- Integrates with the state machine and CLI

## Building and Running Examples

### Using Cargo Directly

```bash
# Build all examples
cargo build --examples

# Run basic example
cargo run --example basic_example -- -H localhost:4000 -u root -d test

# Run isolation test example
cargo run --example isolation_test_example -- -H localhost:4000 -u root -d test

# Run simple multi-connection example
cargo run --example simple_multi_connection

# Run advanced multi-connection example
cargo run --example multi_connection_example

# Run logging example
cargo run --example logging_example -- --log-level debug --log-file --log-file-path logs/mylog.log

# Check if examples compile
cargo check --examples
```

### Using Make

```bash
# Build all examples
make examples

# Run basic example
make run-basic

# Run isolation test example
make run-isolation-test

# Run simple multi-connection example
make run-simple-multi-connection

# Run advanced multi-connection example
make run-advanced

# Run logging example
make run-logging-example

# Check compilation
make check

# Clean build artifacts
make clean
```

## Configuration

Before running the examples, you may need to configure:

1. **Database Connection Details**: Use CLI arguments or environment variables
2. **Authentication**: Ensure you have valid TiDB credentials
3. **Network Access**: Verify connectivity to your TiDB instances

## Feature-based Configuration

The project supports different features at compile time:

```bash
# Build with import job monitoring support
cargo build --features import_jobs

# Build with isolation test support
cargo build --features isolation_test

# Build with multi-connection support
cargo build --features multi_connection

# Build with multiple features
cargo build --features "import_jobs,multi_connection"
```

## Example Output

### Basic Example
```
TiDB Basic Connection Test
===========================
Connection Info:
  Host: localhost:4000
  User: root
  Database: test
  Monitor Duration: 60s
✓ Connected to TiDB!
TiDB version: 6.5.0
✓ Database 'test' exists

✅ Basic connection test completed successfully!
```

### Isolation Test Example
```
TiDB Repeatable Read Isolation Test
===================================
Connection Info:
  Host: localhost:4000
  User: root
  Database: test
  Monitor Duration: 60s
[TEST] ✓ Created test table: isolation_test_...
[TEST] ✓ Inserted 10 rows into test table
[TEST] ✓ Created second connection for isolation testing
...
🎉 All isolation tests passed! Repeatable Read isolation is working correctly.

✅ Isolation test completed successfully!
```

### Logging Example
```
TiDB Logging Example
====================
Connection Info:
  Host: localhost:4000
  User: root
  Database: test
  Monitor Duration: 60s
[INFO] Starting TiDB logging example
[DEBUG] Connection parameters: host=localhost:4000, user=root, database=Some("test")
[INFO] Starting operation: database_connection
[INFO] Completed operation: database_connection
[INFO] Performance metric: operation=database_connection, duration_ms=100
...
✅ Logging example completed successfully!
Check the logs for detailed information.
```

## Troubleshooting

### Common Issues

1. **Connection Refused**: Check if TiDB instances are running and accessible
2. **Authentication Failed**: Verify username and password
3. **Compilation Errors**: Ensure all dependencies are installed
4. **Runtime Errors**: Check the connection parameters and network connectivity

### Debug Mode

To run examples with debug output:

```bash
RUST_LOG=debug cargo run --example basic_example
```

## Contributing

When adding new examples:

1. Follow the naming convention: `descriptive_name.rs`
2. Add the example to `Cargo.toml` in the `[[example]]` section
3. Update this README with documentation
4. Include proper error handling and logging
5. Test the example thoroughly before committing

## Architecture Notes

The examples demonstrate the following architectural patterns:

- **State Machine Pattern**: Each connection uses a state machine for lifecycle management
- **Coordinator Pattern**: Multiple connections are coordinated through a central coordinator
- **Message Passing**: Asynchronous communication between components
- **Shared State Management**: Thread-safe shared state with proper synchronization
- **Error Handling**: Comprehensive error handling with graceful degradation 
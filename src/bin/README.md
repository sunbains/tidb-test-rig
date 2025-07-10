# TiDB Connection Test Tool - Tests

This directory contains test programs demonstrating how to use the TiDB connection test tool with a common CLI library and robust logging.

## Common Setup and Utilities

All tests use shared utilities that provide:

- **Standardized Arguments**: Common host, user, database, and monitoring parameters
- **Environment Variable Support**: Configuration via `TIDB_HOST`, `TIDB_USER`, `TIDB_DATABASE`, `TIDB_PASSWORD`
- **Flexible Password Input**: Command line, environment variable, or interactive prompt
- **Parameter Validation**: Automatic validation of connection parameters
- **Help and Usage**: Built-in help with `--help` flag
- **Integrated Logging**: Control log level, file output, and verbosity from the CLI
- **Shared State Machine Setup**: Common state machine configuration and handler registration
- **Standardized Error Handling**: Consistent error handling and user-friendly error messages

### CLI Usage

```bash
# Basic usage with interactive password prompt
cargo run --example basic_test -- -H localhost:4000 -u root -d test

# Using environment variables
export TIDB_HOST=localhost:4000
export TIDB_USER=root
export TIDB_PASSWORD=mypassword
cargo run --example basic_test

# Using command line password (less secure)
cargo run --example basic_test -- -H localhost:4000 -u root --password mypassword

# Skip password prompt (for automated testing)
cargo run --example basic_test -- -H localhost:4000 -u root --no-password-prompt

# Enable debug logging to console
cargo run --example basic_test -- --log-level debug

# Log to a file
cargo run --example basic_test -- --log-file --log-file-path logs/mylog.log
```

### Available Arguments

**Common Arguments (all tests):**
- `-H, --host`: Hostname and port (default: localhost:4000)
- `-u, --user`: Username (default: root)
- `-d, --database`: Database name (optional)
- `--password`: Password from command line (alternative to prompt)
- `--no-password-prompt`: Skip password prompt (for automated testing)
- `--log-level`: Log level (`debug`, `info`, `warn`, `error`; default: `info`)
- `--log-file`: Enable file logging
- `--log-file-path`: Path to log file (default: logs/tidb_connect.log)
- `-v, --verbose`: Shortcut for debug logging

**Test-specific Arguments:**
- `-t, --monitor-duration`: Duration to monitor import jobs in seconds (default: 60) - *multi-connection tests*
- `--test-rows`: Number of test rows to create for isolation testing (default: 10) - *isolation test tests*
- `--connection-count`: Number of connections to create for multi-connection tests (default: 2) - *multi-connection tests*

## Logging Facility

The project uses the [`tracing`](https://docs.rs/tracing) ecosystem for structured, high-performance logging.

- **Log to console and/or file**
- **Control log level at runtime** (`--log-level debug`)
- **Structured logs for connection, query, state transitions, and errors**
- **Performance and memory usage metrics**
- **Error context for troubleshooting**

### Test Logging Usage

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

### Test: Logging to File

```bash
cargo run --example basic_test -- --log-level debug --log-file --log-file-path logs/mylog.log
```

## Shared Test Utilities

The project provides shared utilities in `src/lib_utils.rs` that eliminate code duplication across tests:

### TestSetup
For tests using the legacy `parse_args()` approach:
```rust
use connect::{TestSetup, print_test_header, print_success};

#[tokio::main]
async fn main() {
    print_test_header("My Test");
    
    let mut setup = TestSetup::new()?;
    setup.run_with_job_monitoring().await?;
    
    print_success("Test completed!");
}
```

### CommonArgsSetup
For tests using the new `CommonArgs::parse()` approach:
```rust
use connect::{CommonArgsSetup, print_test_header, print_success};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_test_header("My Test");
    
    let mut setup = CommonArgsSetup::new()?;
    setup.run_with_error_handling().await?;
    
    print_success("Test completed!");
    Ok(())
}
```

### Helper Functions
- `print_test_header(title)`: Print a formatted test header
- `print_success(message)`: Print a success message
- `print_error_and_exit(message, error)`: Print error and exit
- `create_state_machine_with_handlers(...)`: Create state machine with standard handlers

## Available Tests

### 1. Basic Test (`basic_test.rs`)
A comprehensive test showing how to connect to TiDB and perform basic operations.

**Features:**
- Uses the shared test utilities for setup and error handling
- Demonstrates basic connection testing
- Shows version checking and database verification
- Includes import job monitoring capabilities
- Minimal test for getting started

### 2. Isolation Test (`isolation_test.rs`)
A comprehensive test testing TiDB's repeatable read isolation.

**Features:**
- Creates test tables and populates data
- Tests transaction isolation with multiple connections
- Demonstrates repeatable read behavior
- Uses the shared test utilities for setup and error handling

### 3. Simple Multi-Connection Test (`simple_multi_connection.rs`)
A basic test showing how to create and manage multiple TiDB connections with the state machine infrastructure.

**Features:**
- Creates multiple connections to different TiDB instances
- Demonstrates basic connection coordination
- Shows how to handle connection states and errors

### 4. Advanced Multi-Connection Test (`multi_connection_test.rs`)
A comprehensive test showing advanced multi-connection scenarios with import job monitoring.

**Features:**
- Complex connection management with shared state
- Import job monitoring across multiple connections
- Advanced error handling and recovery
- Coordination between multiple state machines

### 5. Logging Test (`logging_test.rs`)
A demonstration of the logging facility, including log levels, file output, performance metrics, and error context.

**Features:**
- Shows how to use the logging macros and error context
- Demonstrates logging to both console and file
- Logs performance and memory usage metrics
- Integrates with the state machine and shared utilities

## Building and Running Tests

### Using Cargo Directly

```bash
# Build all tests
cargo build --examples

# Run basic test
cargo run --example basic_test -- -H localhost:4000 -u root -d test

# Run isolation test
cargo run --example isolation_test -- -H localhost:4000 -u root -d test

# Run simple multi-connection test
cargo run --example simple_multi_connection

# Run advanced multi-connection test
cargo run --example multi_connection_test

# Run logging test
cargo run --example logging_test -- --log-level debug --log-file --log-file-path logs/mylog.log

# Check if tests compile
cargo check --examples
```

### Using Make

```bash
# Build all tests
make tests

# Run basic test
make run-basic

# Run isolation test
make run-isolation-test

# Run simple multi-connection test
make run-simple-multi-connection

# Run advanced multi-connection test
make run-advanced

# Run logging test
make run-logging-test

# Check compilation
make check

# Clean build artifacts
make clean
```

## Configuration

Before running the tests, you may need to configure:

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

## Test Output

### Basic Test
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

### Isolation Test
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

### Logging Test
```
TiDB Logging Test
====================
Connection Info:
  Host: localhost:4000
  User: root
  Database: test
  Monitor Duration: 60s
[INFO] Starting TiDB logging test
[DEBUG] Connection parameters: host=localhost:4000, user=root, database=Some("test")
[INFO] Starting operation: database_connection
[INFO] Completed operation: database_connection
[INFO] Performance metric: operation=database_connection, duration_ms=100
...
✅ Logging test completed successfully!
Check the logs for detailed information.
```

## Troubleshooting

### Common Issues

1. **Connection Refused**: Check if TiDB instances are running and accessible
2. **Authentication Failed**: Verify username and password
3. **Compilation Errors**: Ensure all dependencies are installed
4. **Runtime Errors**: Check the connection parameters and network connectivity

### Debug Mode

To run tests with debug output:

```bash
RUST_LOG=debug cargo run --example basic_test
```

## Contributing

When adding new tests:

1. Follow the naming convention: `descriptive_name_test.rs`
2. Add the test to `Cargo.toml` in the `[[example]]` section
3. Update this README with documentation
4. Include proper error handling and logging
5. Test the test thoroughly before committing

## Architecture Notes

The tests demonstrate the following architectural patterns:

- **State Machine Pattern**: Each connection uses a state machine for lifecycle management
- **Coordinator Pattern**: Multiple connections are coordinated through a central coordinator
- **Message Passing**: Asynchronous communication between components
- **Shared State Management**: Thread-safe shared state with proper synchronization
- **Error Handling**: Comprehensive error handling with graceful degradation 

# TiDB Connection Test Tool - Examples

This directory contains example programs demonstrating how to use the TiDB connection test tool with a common CLI library.

## Common CLI Library

All examples use a shared command-line interface library that provides:

- **Standardized Arguments**: Common host, user, database, and monitoring parameters
- **Environment Variable Support**: Configuration via `TIDB_HOST`, `TIDB_USER`, `TIDB_DATABASE`, `TIDB_PASSWORD`
- **Flexible Password Input**: Command line, environment variable, or interactive prompt
- **Parameter Validation**: Automatic validation of connection parameters
- **Help and Usage**: Built-in help with `--help` flag
- **Compile-time Options**: Different CLI options based on example type using features and macros

### CLI Usage

```bash
# Basic usage with interactive password prompt
cargo run --example simple_connection -- -H localhost:4000 -u root -d test

# Using environment variables
export TIDB_HOST=localhost:4000
export TIDB_USER=root
export TIDB_PASSWORD=mypassword
cargo run --example simple_connection

# Using command line password (less secure)
cargo run --example simple_connection -- -H localhost:4000 -u root --password mypassword

# Skip password prompt (for automated testing)
cargo run --example simple_connection -- -H localhost:4000 -u root --no-password-prompt
```

### Available Arguments

**Common Arguments (all examples):**
- `-H, --host`: Hostname and port (default: localhost:4000)
- `-u, --user`: Username (default: root)
- `-d, --database`: Database name (optional)
- `--password`: Password from command line (alternative to prompt)
- `--no-password-prompt`: Skip password prompt (for automated testing)

**Example-specific Arguments:**
- `-t, --monitor-duration`: Duration to monitor import jobs in seconds (default: 60) - *multi-connection examples*
- `--test-rows`: Number of test rows to create for isolation testing (default: 10) - *isolation test examples*
- `--connection-count`: Number of connections to create for multi-connection tests (default: 2) - *multi-connection examples*

## Available Examples

### 1. Simple Connection Example (`simple_connection_example.rs`)
A basic example showing how to connect to TiDB and perform basic operations.

**Features:**
- Uses the common CLI library for argument parsing
- Demonstrates basic connection testing
- Shows version checking and database verification
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

### 5. Macro-based CLI Example (`macro_cli_example.rs`)
A demonstration of compile-time CLI configuration using macros.

**Features:**
- Uses macro-generated CLI arguments specific to isolation testing
- Shows how to customize CLI options at compile time
- Demonstrates example-specific argument handling
- Includes test rows configuration for isolation testing

## Building and Running Examples

### Using Cargo Directly

```bash
# Build all examples
cargo build --examples

# Run simple connection example
cargo run --example simple_connection -- -H localhost:4000 -u root -d test

# Run isolation test example
cargo run --example isolation_test_example -- -H localhost:4000 -u root -d test

# Run macro-based CLI example
cargo run --example macro_cli_example -- -H localhost:4000 -u root -d test --test-rows 20

# Run simple multi-connection example
cargo run --example simple_multi_connection

# Run advanced multi-connection example
cargo run --example multi_connection_example

# Check if examples compile
cargo check --examples
```

### Using Make

```bash
# Build all examples
make examples

# Run simple connection example
make run-simple-connection

# Run isolation test example
make run-isolation-test

# Run simple multi-connection example
make run-simple-multi-connection

# Run advanced multi-connection example
make run-advanced

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

## Compile-time CLI Configuration

The project supports different CLI configurations at compile time:

### Feature-based Configuration

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

### Macro-based Configuration

Examples can use macros to generate custom CLI arguments:

```rust
use connect::{generate_cli_args, generate_cli_impl};

// Generate CLI arguments specific to isolation testing
generate_cli_args!(isolation_test);

// Generate CLI implementation specific to isolation testing
generate_cli_impl!(isolation_test);
```

This approach allows each example to have its own CLI interface while sharing common functionality.

## Example Output

### Simple Connection Example
```
TiDB Simple Connection Test
===========================
Connection Info:
  Host: localhost:4000
  User: root
  Database: test
  Monitor Duration: 60s
✓ Connected to TiDB!
TiDB version: 6.5.0
✓ Database 'test' exists

✅ Simple connection test completed successfully!
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

## Troubleshooting

### Common Issues

1. **Connection Refused**: Check if TiDB instances are running and accessible
2. **Authentication Failed**: Verify username and password
3. **Compilation Errors**: Ensure all dependencies are installed
4. **Runtime Errors**: Check the connection parameters and network connectivity

### Debug Mode

To run examples with debug output:

```bash
RUST_LOG=debug cargo run --example simple_connection
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
# TiDB Multi-Connection Test Tool - Examples

This directory contains example programs demonstrating how to use the TiDB multi-connection test tool.

## Available Examples

### 1. Simple Multi-Connection Example (`simple_multi_connection.rs`)
A basic example showing how to create and manage multiple TiDB connections with the state machine infrastructure.

**Features:**
- Creates multiple connections to different TiDB instances
- Demonstrates basic connection coordination
- Shows how to handle connection states and errors

### 2. Advanced Multi-Connection Example (`multi_connection_example.rs`)
A comprehensive example showing advanced multi-connection scenarios with import job monitoring.

**Features:**
- Complex connection management with shared state
- Import job monitoring across multiple connections
- Advanced error handling and recovery
- Coordination between multiple state machines

## Building and Running Examples

### Using Cargo Directly

```bash
# Build all examples
cargo build --examples

# Run simple multi-connection example
cargo run --example simple_multi_connection

# Run advanced multi-connection example
cargo run --example multi_connection_example

# Check if examples compile
cargo check --examples
```

### Using the Build Script

```bash
# Make the script executable (if not already)
chmod +x build_examples.sh

# Build all examples
./build_examples.sh build

# Run simple example
./build_examples.sh run-simple

# Run advanced example
./build_examples.sh run-advanced

# Check compilation
./build_examples.sh check

# Clean build artifacts
./build_examples.sh clean
```

### Using Make

```bash
# Build all examples
make examples

# Run simple example
make run-simple

# Run advanced example
make run-advanced

# Check compilation
make check

# Clean build artifacts
make clean
```

## Configuration

Before running the examples, you may need to configure:

1. **Database Connection Details**: Update the connection parameters in the example files
2. **Authentication**: Ensure you have valid TiDB credentials
3. **Network Access**: Verify connectivity to your TiDB instances

## Example Output

### Simple Multi-Connection Example
```
[INFO] Starting 2 connection state machines...
[INFO] Connection conn1: Connecting to tidb1.example.com:4000
[INFO] Connection conn2: Connecting to tidb2.example.com:4000
[SUCCESS] All connections established successfully
[INFO] Starting coordination phase...
[SUCCESS] Multi-connection test completed
```

### Advanced Multi-Connection Example
```
[INFO] Starting 3 connection state machines...
[INFO] Connection primary: Connecting to primary.tidb.com:4000
[INFO] Connection replica1: Connecting to replica1.tidb.com:4000
[INFO] Connection replica2: Connecting to replica2.tidb.com:4000
[INFO] Monitoring import jobs across all connections...
[INFO] Found 2 active import jobs on primary
[INFO] Job monitoring completed after 60 seconds
[SUCCESS] Advanced multi-connection test completed
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
RUST_LOG=debug cargo run --example simple_multi_connection
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
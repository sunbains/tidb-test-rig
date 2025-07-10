# TiDB Multi-Connection Test Tool

A sophisticated Rust-based testing framework for TiDB databases that provides multi-connection coordination, state machine-driven workflows, and comprehensive monitoring capabilities.

## Overview

This tool is designed to test and monitor TiDB database connections with advanced features including:

- **Multi-Connection Management**: Coordinate multiple TiDB connections simultaneously
- **State Machine Architecture**: Robust, extensible state-driven workflows
- **Import Job Monitoring**: Real-time monitoring of TiDB import jobs
- **Secure Authentication**: Password input with hidden terminal input
- **Async Operations**: Full async/await support with Tokio runtime
- **Error Handling**: Comprehensive error handling with graceful degradation

## Features

### Core Capabilities
- ✅ **Connection Testing**: Verify TiDB connectivity and authentication
- ✅ **Database Verification**: Check database existence and permissions
- ✅ **Version Detection**: Retrieve and display TiDB version information
- ✅ **Import Job Monitoring**: Monitor active import jobs with real-time updates
- ✅ **Multi-Connection Coordination**: Manage multiple connections with shared state
- ✅ **State Machine Workflows**: Extensible state-driven architecture
- ✅ **Secure Password Input**: Hidden password prompts for security
- ✅ **Async Operations**: Non-blocking operations with proper concurrency

### Advanced Features
- 🔄 **State Transitions**: Automatic state progression with error handling
- 📊 **Job Monitoring**: Real-time import job status with elapsed time tracking
- 🔗 **Connection Pooling**: Efficient connection management and reuse
- 📝 **Structured Logging**: Comprehensive logging with different verbosity levels
- 🛡️ **Error Recovery**: Graceful error handling and recovery mechanisms
- 🔧 **Extensible Architecture**: Easy to add new states and handlers

## Architecture

### State Machine Core
The tool uses a state machine pattern for managing complex workflows:

```
Initial → ParsingConfig → Connecting → TestingConnection → 
VerifyingDatabase → GettingVersion → CheckingImportJobs → 
ShowingImportJobDetails → Completed
```

Each state has dedicated handlers that implement:
- `enter()`: State initialization
- `execute()`: Main state logic
- `exit()`: State cleanup

### Multi-Connection Coordination
For scenarios requiring multiple connections:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Coordinator   │    │  State Machine  │    │  State Machine  │
│                 │◄──►│   Connection 1   │    │   Connection 2   │
│  Shared State   │    │                 │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 ▼
                    ┌─────────────────┐
                    │  Message Queue  │
                    │   (Tokio MPSC)  │
                    └─────────────────┘
```

### Key Components

1. **StateMachine**: Core state machine implementation
2. **StateHandler**: Trait for implementing state-specific logic
3. **StateContext**: Shared context with connection and handler-specific data
4. **ConnectionCoordinator**: Manages multiple connections with shared state
5. **ImportJobHandlers**: Specialized handlers for import job monitoring

## Installation

### Prerequisites
- Rust 1.70+ with Cargo
- Access to TiDB database instances
- Network connectivity to TiDB hosts

### Building from Source
```bash
# Clone the repository
git clone <repository-url>
cd connect

# Build the project
cargo build --release

# Run tests
cargo test
```

## Usage

### Basic Usage
```bash
# Run with default settings
cargo run -- -u your_username

# Specify custom host and database
cargo run -- -H your-tidb-host:4000 -u your_username -d your_database

# Monitor import jobs for 120 seconds
cargo run -- -u your_username -t 120
```

### Command Line Options
```bash
tidb-client [OPTIONS]

Options:
  -H, --host <HOST>                    Hostname and port [default: tidb.qyruvz1u6xtd.clusters.dev.tidb-cloud.com:4000]
  -u, --user <USER>                    Username for database authentication
  -d, --database <DATABASE>            Database name (optional)
  -t, --monitor-duration <DURATION>    Duration to monitor import jobs in seconds [default: 60]
  -h, --help                           Print help
```

### Example Workflows

#### 1. Basic Connection Test
```bash
cargo run -- -u admin
# Prompts for password, then tests connection and shows TiDB version
```

#### 2. Import Job Monitoring
```bash
cargo run -- -u admin -t 300
# Monitors import jobs for 5 minutes with real-time updates
```

#### 3. Multi-Database Testing
```bash
# Test multiple databases in sequence
for db in db1 db2 db3; do
    cargo run -- -u admin -d $db
done
```

## Examples

The project includes comprehensive examples demonstrating various use cases:

### Simple Multi-Connection Example
```bash
cargo run --example simple_multi_connection
```
Demonstrates basic multi-connection management with state machine coordination.

### Advanced Multi-Connection Example
```bash
cargo run --example multi_connection_example
```
Shows advanced scenarios with import job monitoring across multiple connections.

### Building Examples
```bash
# Build all examples
cargo build --examples

# Check example compilation
cargo check --examples

# Using Make
make examples
make run-simple
make run-advanced
```

See [examples/README.md](examples/README.md) for detailed example documentation.

## Configuration

### Environment Variables
```bash
# Enable debug logging
RUST_LOG=debug cargo run -- -u admin

# Set custom log level
RUST_LOG=info cargo run -- -u admin
```

### Connection Parameters
- **Host**: TiDB server hostname and port (default: 4000)
- **Username**: Database username (required)
- **Password**: Prompted securely at runtime
- **Database**: Optional database name for testing

## Development

### Project Structure
```
src/
├── main.rs                 # CLI entry point
├── lib.rs                  # Library exports
├── connection.rs           # Connection management
├── state_machine.rs        # State machine core
├── state_handlers.rs       # State handler implementations
├── import_job_handlers.rs  # Import job monitoring
├── connection_manager.rs   # Multi-connection coordination
└── multi_connection_state_machine.rs  # Multi-connection state machines

examples/
├── simple_multi_connection.rs    # Basic multi-connection example
├── multi_connection_example.rs   # Advanced multi-connection example
└── README.md                     # Example documentation

docs/
└── ARCHITECTURE.md              # Detailed architecture documentation
```

### Adding New States
1. Define the state in `state_machine.rs`
2. Implement the handler in `state_handlers.rs`
3. Register the handler in `main.rs`
4. Update state transitions as needed

### Adding New Features
1. Create new modules as needed
2. Implement async traits for handlers
3. Add proper error handling
4. Update documentation
5. Add tests

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Code Quality
```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check for issues
cargo check
```

## Troubleshooting

### Common Issues

#### Connection Problems
```bash
# Error: Connection refused
# Solution: Check if TiDB is running and accessible
telnet your-tidb-host 4000

# Error: Access denied
# Solution: Verify username and password
```

#### Compilation Issues
```bash
# Error: Let chains require Rust 2024
# Solution: Ensure edition = "2024" in Cargo.toml
```

#### Runtime Issues
```bash
# Error: No connection available
# Solution: Check connection parameters and network connectivity

# Error: Import job monitoring fails
# Solution: Verify user has SHOW IMPORT JOBS permission
```

### Debug Mode
```bash
# Enable debug logging
RUST_LOG=debug cargo run -- -u admin

# Run with verbose output
cargo run -- -u admin --verbose
```

### Performance Tuning
- Adjust connection pool sizes for high-load scenarios
- Use appropriate monitor durations for import jobs
- Consider connection timeouts for network issues

## Contributing

### Development Setup
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Update documentation
6. Submit a pull request

### Code Style
- Follow Rust conventions
- Use async/await for I/O operations
- Implement proper error handling
- Add comprehensive documentation
- Include tests for new features

### Testing Guidelines
- Unit tests for individual components
- Integration tests for workflows
- Example tests for user scenarios
- Performance tests for critical paths

## License

[Add your license information here]

## Support

For issues and questions:
- Check the troubleshooting section
- Review the architecture documentation
- Open an issue on the repository
- Check example implementations


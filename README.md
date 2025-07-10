# TiDB Test Rig Library

A sophisticated Rust-based testing framework for TiDB databases that provides multi-connection coordination, state machine-driven workflows, and comprehensive monitoring capabilities.

## Overview

This tool is designed to test and monitor TiDB database connections with advanced features including:

- **Multi-Connection Management**: Coordinate multiple TiDB connections simultaneously
- **State Machine Architecture**: Robust, extensible state-driven workflows
- **Secure Authentication**: Password input with hidden terminal input
- **Async Operations**: Full async/await support with Tokio runtime
- **Error Handling**: Comprehensive error handling with graceful degradation

## Library Structure

This project is now a **reusable Rust library** for TiDB connection and import job testing. The main CLI application previously in `src/main.rs` has been moved to `tests/basic_test.rs`.

- **Library usage:** Import the `connect` crate in your own Rust projects and use the state machine, handlers, and coordination logic directly.
- **CLI usage:** Run the main CLI as a test:
  ```bash
  cargo run --example basic_test -- -H localhost:4000 -u root -d test
  ```
  or
  ```bash
  make run-basic
  ```

All other tests (multi-connection, isolation, CLI, etc.) are also available in the `tests/` directory and use the library API.

### Modular CLI Architecture

The project uses a modular CLI argument structure where each test defines its own argument struct while sharing common arguments:

- **CommonArgs**: Contains truly common arguments (host, user, database, monitor-duration)
- **Test-specific Args**: Each test defines its own `Args` struct with `#[command(flatten)]` for `CommonArgs` plus test-specific arguments
- **Shared Utilities**: Common setup and error handling utilities in `lib_utils.rs`

---

## Features

### Core Capabilities
- ✅ **Connection Testing**: Verify TiDB connectivity and authentication
- ✅ **Database Verification**: Check database existence and permissions
- ✅ **Version Detection**: Retrieve and display TiDB version information
- ✅ **Multi-Connection Coordination**: Manage multiple connections with shared state
- ✅ **State Machine Workflows**: Extensible state-driven architecture
- ✅ **Secure Password Input**: Hidden password prompts for security
- ✅ **Async Operations**: Non-blocking operations with proper concurrency

### Advanced Features
- 🔄 **State Transitions**: Automatic state progression with error handling
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
│                 │◄──►│   Connection 1  │    │   Connection 2  │
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

Each test has its own CLI arguments. Here are the common options shared across tests:

```bash
Common Options:
  -H, --host <HOST>                    Hostname and port [default: tidb.qyruvz1u6xtd.clusters.dev.tidb-cloud.com:4000]
  -u, --user <USER>                    Username for database authentication
  -d, --database <DATABASE>            Database name (optional)
  -t, --monitor-duration <DURATION>    Duration to monitor import jobs in seconds [default: 60]
  -h, --help                           Print help

Test-specific options vary by test. Run any test with --help to see its specific options:
  cargo test --test basic_test -- -- --help
  cargo test --test isolation_test -- -- --help
  cargo test --test cli_test -- -- --help
```

### Test Workflows

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

## Tests

The project includes comprehensive tests demonstrating various use cases:

### Basic Test (Main CLI)
```bash
cargo test --test basic_test -- -- -H localhost:4000 -u root -d test
```
This is the main CLI entry point for single-connection and import job monitoring workflows.

### CLI Test
```bash
cargo test --test cli_test -- -- -H localhost:4000 -u root -d test
```
Demonstrates CLI argument parsing and basic connection testing with modular argument structure.

### Isolation Test
```bash
cargo test --test isolation_test -- -- -H localhost:4000 -u root -d test --test-rows 20
```
Tests transaction isolation with configurable test data.

### Logging Test
```bash
cargo test --test logging_test -- -- -H localhost:4000 -u root -d test
```
Demonstrates structured logging with different verbosity levels.

### Simple Multi-Connection Test
```bash
cargo test --test simple_multi_connection --
```
Demonstrates basic multi-connection management with state machine coordination.

### Advanced Multi-Connection Test
```bash
cargo test --test multi_connection_test --
```
Shows advanced scenarios with import job monitoring across multiple connections.

### Building Tests
```bash
# Build all test binaries
cargo test --no-run

# Check test compilation for all test binaries
cargo check --tests

# Using Make
make tests
make run-simple
make run-advanced
```

### Using Makefile

The project includes a comprehensive Makefile for common development tasks:

```bash
# Build the main application
make build

# Run tests
make test

# Clean build artifacts
make clean

# Build all tests
make tests

# Run specific tests
make run-simple-connection
make run-isolation-test
make run-cli-test
make run-logging-test

# Code quality
make format
make lint
make check

# Show help
make help
```

#### Makefile Targets

| Target | Description | Test |
|--------|-------------|---------|
| `run-simple-connection` | Basic connection test | `make run-simple-connection` |
| `run-isolation-test` | Transaction isolation testing | `make run-isolation-test` |
| `run-cli-test` | CLI test with modular arguments | `make run-cli-test` |
| `run-logging-test` | Logging demonstration | `make run-logging-test` |
| `run-simple` | Simple multi-connection test | `make run-simple` |
| `run-advanced` | Advanced multi-connection test | `make run-advanced` |

**Note:** Each test has its own CLI arguments. Use `--help` with any test to see its specific options.

See [tests/README.md](tests/README.md) for detailed test documentation.

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
├── lib.rs                  # Library exports (consolidated imports)
├── connection.rs           # Connection management
├── state_machine.rs        # State machine core
├── state_handlers.rs       # Common state handler implementations
├── import_job_handlers.rs  # Import job monitoring
├── connection_manager.rs   # Multi-connection coordination
├── multi_connection_state_machine.rs  # Multi-connection state machines
├── cli.rs                  # Common CLI argument handling
├── lib_utils.rs            # Shared utilities for tests
└── logging.rs              # Logging infrastructure

tests/
├── simple_connection_test.rs     # Basic connection test
├── isolation_test.rs             # Transaction isolation testing
├── logging_test.rs               # Logging demonstration
├── cli_test.rs                   # CLI test
├── simple_multi_connection.rs    # Basic multi-connection test
├── multi_connection_test.rs      # Advanced multi-connection test
└── README.md                     # Test documentation

docs/
└── ARCHITECTURE.md              # Detailed architecture documentation
```

### Import Structure

#### Common Imports (from lib.rs)
```rust
// All tests can import common functionality
use connect::{InitialHandler, ParsingConfigHandler, ConnectingHandler, 
              TestingConnectionHandler, VerifyingDatabaseHandler, GettingVersionHandler};
use connect::{CommonArgs, TestSetup, CommonArgsSetup};
use connect::lib_utils::{print_test_header, print_success, print_error_and_exit};
```

#### Test-Specific Imports
```rust
// Test-specific imports
use connect::state_machine::{StateMachine, State, StateContext, StateHandler, StateError};
use clap::Parser;

// Each test defines its own Args struct
#[derive(Parser)]
#[command(flatten)]
struct Args {
    #[command(flatten)]
    common: CommonArgs,
    // Test-specific arguments here
}
```

### Adding New Tests

#### Test Structure
Each test follows a consistent pattern:

1. **Define Args struct** with `#[command(flatten)]` for `CommonArgs`
2. **Use shared utilities** from `lib_utils.rs` for setup and error handling
3. **Implement test-specific logic** using the state machine

```rust
#[derive(Parser)]
#[command(about = "Test description")]
struct Args {
    #[command(flatten)]
    common: CommonArgs,
    // Test-specific arguments
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let setup = TestSetup::new(&args.common);
    
    // Test-specific logic here
}
```

#### Adding New States
1. For common states: Define in `state_machine.rs` and export in `lib.rs`
2. For test-specific states: Use existing states in `State` enum or add new ones
3. Implement the handler in the appropriate module
4. Register the handler in the test
5. Update state transitions as needed

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
- Test scenarios for user workflows
- Performance tests for critical paths

## License

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.


## Support

For issues and questions:
- Check the troubleshooting section
- Review the architecture documentation
- Open an issue on the repository
- Check test implementations


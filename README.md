[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

# TiDB Testing Framework

A Rust library framework for testing TiDB database functionality with Python plugin support. The framework provides comprehensive testing capabilities for database operations, transactions, and DDL operations.

## Overview

This framework provides a comprehensive testing solution for TiDB with the following capabilities:

- **Core Testing**: Basic connection testing and database operations
- **Multi-Connection Testing**: Concurrent connection testing with coordination
- **Transaction Testing**: Comprehensive transaction isolation and concurrency testing
- **DDL Testing**: Data Definition Language operation testing
- **Scale Testing**: Performance and scalability testing
- **Python Plugin System**: Write test handlers in Python for rapid development
- **Configuration Management**: JSON and TOML configuration files
- **Command Line Interface**: Flexible CLI options for all tests

## Features

### Core Framework
- **State Machine Architecture**: Clean, extensible state-based testing workflows
- **Connection Management**: Robust database connection handling with pooling
- **Error Handling**: Comprehensive error handling with retry strategies and circuit breakers
- **Configuration**: Flexible configuration via files, environment variables, and CLI
- **Logging**: Structured logging with configurable levels and outputs

### Python Plugin System
- **Python State Handlers**: Write test logic in Python using `PyStateHandler` base class
- **Seamless Integration**: Python handlers integrate with Rust state machine
- **Type Safety**: Full type hints and validation between Rust and Python
- **Standalone Testing**: Python handlers can be tested independently
- **Real Database Support**: Connect to real TiDB/MySQL servers or use mock connections

### Test Suites
- **DDL Tests**: Data Definition Language operations (CREATE, ALTER, DROP)
- **Transaction Tests**: Transaction isolation, concurrency, deadlocks, savepoints
- **Scale Tests**: Performance and scalability testing
- **Multi-Connection Tests**: Concurrent connection testing with coordination

## Quick Start

### Prerequisites

1. **Rust**: Install Rust and Cargo
2. **Python**: Python 3.8+ with development headers
3. **Database**: TiDB or MySQL server (optional for mock testing)

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd tidb_tests

# Build the project
cargo build

# Install Python dependencies (optional, for real database connections)
pip install mysql-connector-python
```

### Basic Usage

#### Run Basic Connection Test
```bash
cargo run --bin basic -- -H localhost:4000 -u root
```

#### Run Python Test Suites
```bash
# Run all Python test suites
make run-python-tests

# Run specific test suite
make run-ddl-tests
make run-txn-tests
make run-scale-tests

# Run with real database connection
TIDB_HOST=your-tidb-host:4000 TIDB_USER=your-user TIDB_PASSWORD=your-password make run-txn-tests REAL_DB=1

# Show SQL queries and output
make run-txn-tests SHOW_SQL=1 SHOW_OUTPUT=1
```

#### Run Multi-Connection Tests
```bash
cargo run --bin simple_multi_connection --features multi_connection
```

#### Run Isolation Tests
```bash
cargo run --bin isolation --features isolation_test
```

## Python Plugin System

The framework supports writing test handlers in Python, allowing rapid development and testing of database operations.

### Writing Python Handlers

```python
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState

class BasicTransactionHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        print(f"Connecting to {context.host}:{context.port}")
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Execute database operations
            context.connection.execute_query("DROP TABLE IF EXISTS txn_test")
            context.connection.execute_query("CREATE TABLE txn_test (id INT, name VARCHAR(50), balance DECIMAL(10,2))")
            
            # Test transactions
            context.connection.start_transaction()
            context.connection.execute_query("INSERT INTO txn_test (id, name, balance) VALUES (1, 'Alice', 100.00)")
            context.connection.execute_query("INSERT INTO txn_test (id, name, balance) VALUES (2, 'Bob', 200.00)")
            context.connection.rollback()
            
            return PyState.completed()
        return PyState.connecting()
    
    def exit(self, context: PyStateContext) -> None:
        print("Transaction test completed")
```

### Running Python Tests

```bash
# Run with mock connections (default)
make run-txn-tests

# Run with real database
TIDB_HOST=your-host:4000 TIDB_USER=your-user TIDB_PASSWORD=your-password make run-txn-tests REAL_DB=1

# Show SQL queries
make run-txn-tests SHOW_SQL=1

# Show all output
make run-txn-tests SHOW_OUTPUT=1
```

## Configuration

### Environment Variables
```bash
export TIDB_HOST="localhost:4000"
export TIDB_USER="root"
export TIDB_PASSWORD="your-password"
export TIDB_DATABASE="test"
```

### Configuration Files

#### JSON Configuration (`tidb_config.json`)
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
    "console": true
  }
}
```

#### TOML Configuration (`tidb_config.toml`)
```toml
[database]
host = "localhost:4000"
username = "root"
database = "test"
pool_size = 5
timeout_secs = 30

[logging]
level = "info"
format = "text"
console = true
```

## Available Binaries

- **`basic`**: Basic connection testing
- **`simple_multi_connection`**: Multi-connection testing (requires `multi_connection` feature)
- **`isolation`**: Transaction isolation testing (requires `isolation_test` feature)
- **`job_monitor`**: Import job monitoring (requires `import_jobs` feature)
- **`config_gen`**: Configuration file generation
- **`python_demo`**: Python plugin demonstration (requires `python_plugins` feature)
- **`python_test_runner`**: Python test suite runner (requires `python_plugins` feature)

## Test Suites

### DDL Tests (`src/ddl/`)
- Basic DDL operations (CREATE, ALTER, DROP)
- Concurrent DDL operations
- Specialized object testing
- User and index management

### Transaction Tests (`src/txn/`)
- Basic transaction operations
- Transaction isolation levels
- Deadlock detection and handling
- MVCC version chain testing
- Transaction concurrency
- Savepoints and rollbacks
- Transaction error handling
- Performance testing

### Scale Tests (`src/scale/`)
- Basic scalability testing
- Performance benchmarks

## Makefile Targets

```bash
# Python test suites
make run-python-tests          # Run all Python test suites
make run-ddl-tests            # Run DDL test suite
make run-txn-tests            # Run transaction test suite
make run-scale-tests          # Run scale test suite

# With options
make run-txn-tests REAL_DB=1 SHOW_SQL=1 SHOW_OUTPUT=1

# Build and check
make build                    # Build the project
make check                    # Check compilation
make format                   # Format code
make lint                     # Run linter
```

## Error Handling

The framework includes sophisticated error handling with:

- **Retry Strategies**: Exponential backoff with jitter
- **Circuit Breaker Pattern**: Automatic failure detection and recovery
- **Error Classification**: Transient vs permanent error handling
- **Context Preservation**: Rich error context for debugging

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.


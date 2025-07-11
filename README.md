[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

# TiDB Testing Framework

A Rust library framework for testing TiDB database functionality with Python plugin support. The framework provides comprehensive testing capabilities for database operations, transactions, and DDL operations.

## Overview

This framework provides a testing framework for TiDB, it's easy to extend and write tests in Rust and Python.

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

#### Run Rust Binaries
```bash
# Basic connection test
cargo run --bin basic -- -H localhost:4000 -u root

# Multi-connection test
cargo run --bin simple_multi_connection --features multi_connection

# Isolation test
cargo run --bin isolation --features isolation_test
```

## Documentation

For detailed information about using the framework, see the documentation in the `docs/` directory:

- **[Architecture](docs/ARCHITECTURE.md)** - System architecture and design
- **[Running Python Tests](docs/RUNNING_PYTHON_TESTS.md)** - How to run Python test suites
- **[Creating Python Test Directories](docs/CREATING_PYTHON_TEST_DIRECTORIES.md)** - How to create new test suites
- **[Advanced Guide](docs/ADVANCED_GUIDE.md)** - Advanced features and usage patterns
- **[Dynamic States](docs/DYNAMIC_STATES.md)** - Using the dynamic state system

The framework supports JSON and TOML configuration files. See the [Advanced Guide](docs/ADVANCED_GUIDE.md) for detailed configuration options.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.


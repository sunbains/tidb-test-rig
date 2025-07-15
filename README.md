[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

# TiDB Testing Framework

A Rust library framework for testing TiDB database functionality with Python plugin support. The framework provides comprehensive testing capabilities for database operations, transactions, DDL operations, and data import functionality.

## Overview

This framework provides a testing framework for TiDB, it's easy to extend and write tests in Rust and Python. It includes specialized test suites for various TiDB features including DDL operations, transactions, scaling scenarios, and data import operations.

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
make run-import-tests

# Run with real database connection
TIDB_HOST=your-tidb-host:4000 TIDB_USER=your-user TIDB_PASSWORD=your-password make run-txn-tests REAL_DB=1

# Show SQL queries and output
make run-txn-tests SHOW_SQL=1 SHOW_OUTPUT=1

# Run a specific test file within a suite
cargo run --bin python_test_runner --features="python_plugins" -- --suite import --show-sql --real-db --test-file test_import_large.py
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

## Test Suites

### Available Test Suites

1. **DDL Tests** (`ddl/`) - Database schema operations, table creation, modifications
2. **Transaction Tests** (`txn/`) - ACID properties, isolation levels, concurrent transactions
3. **Scale Tests** (`scale/`) - Performance and scalability testing
4. **Import Tests** (`import/`) - Data import functionality, CSV/TSV imports, large datasets

### Import Test Suite

The import test suite provides comprehensive testing for TiDB's data import capabilities:

#### Test Files
- **`test_import.py`** - Basic import functionality testing
  - Null value handling
  - Duplicate key scenarios
  - Column mapping
  - Character set handling
  - Partitioned table imports
  - Constraint validation
  - Auto-increment behavior
  - Error handling
  - TSV format support
  - Import options

- **`test_import_large.py`** - Large dataset and performance testing
  - Large dataset imports (100k+ rows)
  - Complex data types
  - TSV format with special characters
  - Partitioned table performance
  - Duplicate key handling at scale
  - Performance benchmarking
  - Error handling under load

- **`test_import_and_monitor.py`** - Multi-connection import scenarios
  - Import with real-time monitoring
  - Concurrent imports
  - Large dataset monitoring
  - Job status tracking

#### Data Generation

The suite includes a data generator (`create_import.py`) for creating test datasets:

```bash
# Generate default dataset (100k rows, CSV format)
python create_import.py

# Generate custom dataset
python create_import.py --rows 50000 --format tsv --complex

# Generate simple dataset
python create_import.py --rows 10000 --format csv --simple
```

**Options:**
- `--rows`: Number of rows to generate (default: 100000)
- `--format`: Output format - csv or tsv (default: csv)
- `--complex`: Generate complex data with special characters
- `--simple`: Generate simple alphanumeric data
- `--output`: Output file path (default: generated_data.csv/tsv)

### Test Runner Options

The Python test runner supports several options:

```bash
cargo run --bin python_test_runner --features="python_plugins" -- [OPTIONS]

Options:
  --suite <SUITE>              Test suite to run (ddl, txn, scale, import)
  --show-sql                   Show SQL queries being executed
  --show-output                Show test output
  --real-db                    Use real database connection
  --test-file <TEST_FILE>      Run only the specified test file within the suite
  --help                       Show help information
```

**Examples:**
```bash
# Run all import tests
cargo run --bin python_test_runner --features="python_plugins" -- --suite import --show-sql --real-db

# Run only the large import test
cargo run --bin python_test_runner --features="python_plugins" -- --suite import --show-sql --real-db --test-file test_import_large.py

# Run only the multi-connection test
cargo run --bin python_test_runner --features="python_plugins" -- --suite import --show-sql --real-db --test-file test_import_and_monitor.py
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


# Python Test System and Test Directory Creation

This guide covers the Python test system for the test_rig framework and how to create new Python test directories.

## Python Test System Overview

The test_rig framework supports writing test handlers in Python, allowing you to leverage Python's rich ecosystem while maintaining the performance and reliability of the Rust-based test runner.

### Key Features

- ✅ **Python State Handlers**: Write test logic in Python using `PyStateHandler` base class
- ✅ **Mock and Real Connections**: Test with mock connections or connect to real TiDB/MySQL servers
- ✅ **Multi-Connection Support**: Test concurrent operations across multiple connections
- ✅ **Type Safety**: Full type hints and validation
- ✅ **Automated Test Discovery**: Automatic discovery and execution of Python test files
- ✅ **Configurable Output**: Control SQL logging and test output

### Architecture

The Python test system consists of:

- **Rust Test Runner**: High-performance test execution and coordination
- **Python Handlers**: Easy-to-write test handlers with access to Python libraries
- **Test Infrastructure**: Mock and real connection support
- **Automated Discovery**: Python test files are automatically discovered and executed

## Quick Start with Python Tests

### 1. Enable Python Support

Build with the `python_plugins` feature:

```bash
cargo build --features python_plugins
```

### 2. Write a Python Handler

Create a Python file with your handlers:

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

### 3. Run Python Tests

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

## System Dependencies

### Debian/Ubuntu

Before using Python tests, you need to install system dependencies on Debian/Ubuntu systems:

#### 1. Install Python Development Headers

```bash
# For Python 3.11 (default on Ubuntu 22.04+)
sudo apt update
sudo apt install python3.11-dev python3.11-venv

# For Python 3.10 (Ubuntu 20.04)
sudo apt install python3.10-dev python3.10-venv

# For Python 3.9 (older Ubuntu versions)
sudo apt install python3.9-dev python3.9-venv
```

#### 2. Install Build Dependencies

```bash
# Essential build tools
sudo apt install build-essential pkg-config

# Additional dependencies for PyO3
sudo apt install libssl-dev libffi-dev
```

#### 3. Install Python Package Dependencies

If you're using Python handlers that require external packages (like `mysql-connector-python`):

```bash
# Install pip if not already available
sudo apt install python3-pip

# For systems with externally managed Python environments (Ubuntu 23.04+)
# Use user-level installation to avoid conflicts
pip3 install --user --break-system-packages mysql-connector-python

# For older systems without external environment management
pip3 install --user mysql-connector-python
```

#### 4. Verify Installation

Test that Python and pip are working correctly:

```bash
# Check Python version
python3 --version

# Check pip installation
pip3 --version

# Test importing required packages
python3 -c "import mysql.connector; print('MySQL connector installed successfully')"
```

#### 5. Environment Setup (Optional but Recommended)

For better dependency management, consider using a virtual environment:

```bash
# Create a virtual environment
python3 -m venv test_rig_env

# Activate the environment
source test_rig_env/bin/activate

# Install packages in the virtual environment
pip install mysql-connector-python

# When running test_rig, activate the environment first
source test_rig_env/bin/activate
make run-txn-tests
```

### Troubleshooting Debian/Ubuntu Issues

#### Permission Errors

If you encounter permission errors when installing Python packages:

```bash
# Use user-level installation
pip3 install --user --break-system-packages package_name

# Or create a virtual environment
python3 -m venv ~/test_rig_env
source ~/test_rig_env/bin/activate
pip install package_name
```

#### Missing Python Headers

If you get errors about missing Python headers:

```bash
# Install the correct Python development package for your version
python3 --version  # Check your Python version first
sudo apt install python3.X-dev  # Replace X with your version
```

#### PyO3 Compilation Issues

If PyO3 fails to compile:

```bash
# Ensure you have the latest Rust toolchain
rustup update

# Install additional build dependencies
sudo apt install cmake libclang-dev
```

## Handler Interface

### Base Class

All Python handlers must inherit from `PyStateHandler`:

```python
from src.common.test_rig_python import PyStateHandler

class MyHandler(PyStateHandler):
    def __init__(self):
        super().__init__()
        # Your initialization code
```

### Required Methods

#### `enter(context: PyStateContext) -> str`
Called when entering the state. Should return the next state.

```python
def enter(self, context: PyStateContext) -> str:
    print(f"Entering state with host: {context.host}")
    return PyState.connecting()
```

#### `execute(context: PyStateContext) -> str`
Called during state execution. This is where your main test logic goes.

```python
def execute(self, context: PyStateContext) -> str:
    if context.connection:
        # Execute database operations
        results = context.connection.execute_query("SELECT 1")
        print(f"Query returned {len(results)} results")
        return PyState.completed()
    else:
        return PyState.connecting()
```

#### `exit(context: PyStateContext) -> None`
Called when exiting the state. Use for cleanup.

```python
def exit(self, context: PyStateContext) -> None:
    print("Exiting state")
```

## Connection Management

### Mock Connections (Default)

By default, tests use mock connections that simulate database operations:

```python
def execute(self, context: PyStateContext) -> str:
    if context.connection:
        # These operations are mocked
        context.connection.execute_query("CREATE TABLE test (id INT)")
        context.connection.start_transaction()
        context.connection.execute_query("INSERT INTO test VALUES (1)")
        context.connection.commit()
    return PyState.completed()
```

### Real Database Connections

To use real database connections, set the `REAL_DB=1` environment variable:

```bash
TIDB_HOST=your-host:4000 TIDB_USER=your-user TIDB_PASSWORD=your-password make run-txn-tests REAL_DB=1
```

The framework will automatically:
- Parse hostname and port from the connection string
- Connect to the real database using `mysql-connector-python`
- Detect the server type (TiDB or MySQL)
- Execute real SQL queries

### Connection Methods

#### `execute_query(query: str) -> List[Dict[str, Any]]`
Execute a SQL query and return results as a list of dictionaries.

```python
results = context.connection.execute_query("SELECT * FROM users WHERE age > 18")
for row in results:
    print(f"User: {row['name']}, Age: {row['age']}")
```

#### `start_transaction() -> None`
Start a new transaction.

```python
context.connection.start_transaction()
```

#### `commit() -> None`
Commit the current transaction.

```python
context.connection.commit()
```

#### `rollback() -> None`
Rollback the current transaction.

```python
context.connection.rollback()
```

## Multi-Connection Testing

For tests that require multiple concurrent connections, inherit from `MultiConnectionTestHandler`:

```python
from src.common.test_rig_python import MultiConnectionTestHandler, PyStateContext, PyState

class ConcurrentTransactionHandler(MultiConnectionTestHandler):
    def __init__(self):
        super().__init__(connection_count=3)  # Use 3 concurrent connections
    
    def enter(self, context: PyStateContext) -> str:
        # This creates multiple connections automatically
        return super().enter(context)
    
    def execute(self, context: PyStateContext) -> str:
        # Execute operations concurrently across multiple connections
        operations = [
            {'connection_id': 0, 'operation': 'start_transaction'},
            {'connection_id': 1, 'operation': 'start_transaction'},
            {'connection_id': 0, 'operation': 'query', 'query': 'INSERT INTO test VALUES (1)'},
            {'connection_id': 1, 'operation': 'query', 'query': 'INSERT INTO test VALUES (2)'},
            {'connection_id': 0, 'operation': 'commit'},
            {'connection_id': 1, 'operation': 'rollback'},
        ]
        
        results = self.execute_concurrent_operations(operations)
        return PyState.completed()
```

## State Management

### Available States

Use the `PyState` class to return state transitions:

```python
from src.common.test_rig_python import PyState

# Available states
PyState.initial()           # Initial state
PyState.parsing_config()    # Parsing configuration
PyState.connecting()        # Connecting to database
PyState.testing_connection() # Testing connection
PyState.verifying_database() # Verifying database
PyState.getting_version()   # Getting server version
PyState.completed()         # Test completed
```

### Context Information

The `PyStateContext` provides access to connection information:

```python
def enter(self, context: PyStateContext) -> str:
    print(f"Host: {context.host}")
    print(f"Port: {context.port}")
    print(f"Username: {context.username}")
    print(f"Database: {context.database}")
    print(f"Connection available: {context.connection is not None}")
    return PyState.connecting()
```

## Creating New Test Directories

### 1. Create the Directory Structure

Create a new test directory in `src/`:

```bash
mkdir src/my_test_suite
cd src/my_test_suite
```

### 2. Create Required Files

#### `Cargo.toml`
```toml
[package]
name = "my_test_suite"
version = "0.1.0"
edition = "2024"

[dependencies]
test_rig = { path = "../.." }
```

#### `lib.rs`
```rust
//! My Test Suite
//! 
//! This module contains Python tests for my specific testing needs.

pub fn main() {
    println!("My test suite module");
}
```

#### `__init__.py`
```python
# My Test Suite Python Package
# This makes the directory a Python package for importing test modules
```

### 3. Add Python Test Files

Create Python test files with your handlers:

#### `test_basic_operations.py`
```python
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState

class BasicOperationsHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        print("Starting basic operations test")
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Your test logic here
            context.connection.execute_query("SELECT 1")
            return PyState.completed()
        return PyState.connecting()
    
    def exit(self, context: PyStateContext) -> None:
        print("Basic operations test completed")
```

### 4. Update the Main Cargo.toml

Add your new test suite to the workspace in the main `Cargo.toml`:

```toml
[workspace]
members = ["src/bin", "src/ddl", "src/scale", "src/txn", "src/my_test_suite"]
```

### 5. Update the Makefile

Add a new target to the Makefile:

```makefile
run-my-test-suite:
	@echo "Running My Test Suite..."
	RUST_LOG=$(RUST_LOG) cargo run --bin python_test_runner --features="python_plugins" -- --suite my_test_suite $(if $(SHOW_OUTPUT),--show-output) $(if $(SHOW_SQL),--show-sql) $(if $(REAL_DB),--real-db)
```

### 6. Create a README

Create a `README.md` file for your test suite:

```markdown
# My Test Suite

This test suite contains Python tests for my specific testing needs.

## Tests

- `test_basic_operations.py`: Basic operations testing
- `test_advanced_operations.py`: Advanced operations testing

## Running Tests

```bash
# Run with mock connections
make run-my-test-suite

# Run with real database
TIDB_HOST=your-host:4000 TIDB_USER=your-user TIDB_PASSWORD=your-password make run-my-test-suite REAL_DB=1

# Show SQL queries
make run-my-test-suite SHOW_SQL=1
```
```

## Configuration

### Environment Variables

Python tests read configuration from environment variables:

```bash
export TIDB_HOST="localhost:4000"
export TIDB_USER="root"
export TIDB_PASSWORD="your-password"
export TIDB_DATABASE="test"
```

### Configuration Priority

1. **Command-line arguments** (highest priority)
2. **Environment variables** (`TIDB_HOST`, `TIDB_USER`, `TIDB_PASSWORD`, `TIDB_DATABASE`)
3. **Default values** (lowest priority)

## Output Control

### SQL Logging

Enable SQL logging to see all queries being executed:

```bash
make run-txn-tests SHOW_SQL=1
```

This will show output like:
```
🔍 SQL [test_conn]: CREATE TABLE test (id INT)
🔍 SQL [test_conn]: INSERT INTO test VALUES (1)
🔍 SQL [test_conn]: SELECT * FROM test
```

### Output Control

Show all test output (including print statements):

```bash
make run-txn-tests SHOW_OUTPUT=1
```

### Combined Options

Use both options together:

```bash
make run-txn-tests SHOW_SQL=1 SHOW_OUTPUT=1 REAL_DB=1
```

## Best Practices

### Handler Design

1. **Keep handlers focused**: Each handler should test one specific aspect
2. **Use descriptive names**: Handler class names should clearly indicate what they test
3. **Handle connection availability**: Always check if `context.connection` is available
4. **Return appropriate states**: Use the correct state transitions

### Error Handling

1. **Catch and handle exceptions**: Wrap database operations in try-catch blocks
2. **Provide meaningful error messages**: Include context in error messages
3. **Clean up resources**: Use the `exit` method for cleanup

### Testing Strategy

1. **Start with mock connections**: Develop and test with mock connections first
2. **Test with real database**: Once working, test with real database connections
3. **Use multiple connections**: For concurrency testing, use `MultiConnectionTestHandler`
4. **Isolate tests**: Each test should be independent and not affect others

## Examples

### Basic Transaction Test

```python
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState

class TransactionTestHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        print("Starting transaction test")
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Create test table
            context.connection.execute_query("DROP TABLE IF EXISTS transaction_test")
            context.connection.execute_query("""
                CREATE TABLE transaction_test (
                    id INT PRIMARY KEY,
                    name VARCHAR(50),
                    balance DECIMAL(10,2)
                )
            """)
            
            # Test transaction
            context.connection.start_transaction()
            context.connection.execute_query("INSERT INTO transaction_test VALUES (1, 'Alice', 100.00)")
            context.connection.execute_query("INSERT INTO transaction_test VALUES (2, 'Bob', 200.00)")
            
            # Check data before commit
            results = context.connection.execute_query("SELECT COUNT(*) FROM transaction_test")
            print(f"Records before commit: {results[0]['col_0']}")
            
            context.connection.commit()
            
            # Check data after commit
            results = context.connection.execute_query("SELECT COUNT(*) FROM transaction_test")
            print(f"Records after commit: {results[0]['col_0']}")
            
            return PyState.completed()
        return PyState.connecting()
    
    def exit(self, context: PyStateContext) -> None:
        print("Transaction test completed")
```

### Concurrent Operations Test

```python
from src.common.test_rig_python import MultiConnectionTestHandler, PyStateContext, PyState

class ConcurrentOperationsHandler(MultiConnectionTestHandler):
    def __init__(self):
        super().__init__(connection_count=3)
    
    def execute(self, context: PyStateContext) -> str:
        # Create test table
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS concurrent_test")
            context.connection.execute_query("CREATE TABLE concurrent_test (id INT, value INT)")
        
        # Execute concurrent operations
        operations = [
            {'connection_id': 0, 'operation': 'start_transaction'},
            {'connection_id': 1, 'operation': 'start_transaction'},
            {'connection_id': 2, 'operation': 'start_transaction'},
            
            {'connection_id': 0, 'operation': 'query', 'query': 'INSERT INTO concurrent_test VALUES (1, 100)'},
            {'connection_id': 1, 'operation': 'query', 'query': 'INSERT INTO concurrent_test VALUES (2, 200)'},
            {'connection_id': 2, 'operation': 'query', 'query': 'INSERT INTO concurrent_test VALUES (3, 300)'},
            
            {'connection_id': 0, 'operation': 'commit'},
            {'connection_id': 1, 'operation': 'commit'},
            {'connection_id': 2, 'operation': 'rollback'},
        ]
        
        results = self.execute_concurrent_operations(operations)
        
        # Check final state
        if context.connection:
            final_results = context.connection.execute_query("SELECT COUNT(*) FROM concurrent_test")
            print(f"Final record count: {final_results[0]['col_0']}")
        
        return PyState.completed()
```

## Troubleshooting

### Common Issues

1. **Import errors**: Ensure you're importing from `src.common.test_rig_python`
2. **Connection errors**: Check that `REAL_DB=1` is set for real database connections
3. **Permission errors**: Use virtual environments or user-level Python package installation
4. **Compilation errors**: Ensure Python development headers are installed

### Debugging

1. **Enable verbose output**: Use `SHOW_OUTPUT=1` to see all print statements
2. **Enable SQL logging**: Use `SHOW_SQL=1` to see all SQL queries
3. **Check environment variables**: Verify that database connection variables are set correctly
4. **Test with mock connections first**: Use mock connections to isolate Python logic issues 
# Running Python Tests

This guide covers how to run Python tests in the test_rig framework, including configuration options and troubleshooting.

## Overview

The test_rig framework includes a comprehensive Python test system that allows you to write and run database tests using Python handlers. The system supports both mock connections (for development and testing) and real database connections (for actual testing against TiDB/MySQL servers).

## Quick Start

### Basic Test Execution

```bash
# Run all Python test suites
make run-python-tests

# Run specific test suite
make run-ddl-tests
make run-txn-tests
make run-scale-tests
```

### With Real Database Connection

```bash
# Set environment variables for real database
export TIDB_HOST="your-tidb-host:4000"
export TIDB_USER="your-username"
export TIDB_PASSWORD="your-password"
export TIDB_DATABASE="test"

# Run with real database
make run-txn-tests REAL_DB=1
```

### Output Control

```bash
# Show SQL queries being executed
make run-txn-tests SHOW_SQL=1

# Show all test output (including print statements)
make run-txn-tests SHOW_OUTPUT=1

# Combine options
make run-txn-tests REAL_DB=1 SHOW_SQL=1 SHOW_OUTPUT=1
```

## Available Test Suites

### DDL Tests (`src/ddl/`)
Tests for Data Definition Language operations:
- Basic DDL operations (CREATE, ALTER, DROP)
- Concurrent DDL operations
- Specialized object testing
- User and index management

### Transaction Tests (`src/txn/`)
Tests for transaction behavior:
- Basic transaction operations
- Transaction isolation levels
- Deadlock detection and handling
- MVCC version chain testing
- Transaction concurrency
- Savepoints and rollbacks
- Transaction error handling
- Performance testing

### Scale Tests (`src/scale/`)
Tests for performance and scalability:
- Basic scalability testing
- Performance benchmarks

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

### Connection Types

#### Mock Connections (Default)
By default, tests use mock connections that simulate database operations. This is useful for:
- Development and testing without a real database
- Isolating Python logic from database issues
- Fast execution for debugging

#### Real Database Connections
To use real database connections, set `REAL_DB=1`:

```bash
make run-txn-tests REAL_DB=1
```

The framework will:
- Parse hostname and port from the connection string
- Connect to the real database using `mysql-connector-python`
- Detect the server type (TiDB or MySQL)
- Execute real SQL queries

## Makefile Targets

### Python Test Suites
```bash
make run-python-tests          # Run all Python test suites
make run-ddl-tests            # Run DDL test suite
make run-txn-tests            # Run transaction test suite
make run-scale-tests          # Run scale test suite
```

### Build and Development
```bash
make build                    # Build the project
make check                    # Check compilation
make format                   # Format code
make lint                     # Run linter
```

### Options
All test targets support these options:
- `REAL_DB=1`: Use real database connections
- `SHOW_SQL=1`: Show SQL queries being executed
- `SHOW_OUTPUT=1`: Show all test output

## Output and Logging

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

### Test Output

Show all test output (including print statements from Python handlers):

```bash
make run-txn-tests SHOW_OUTPUT=1
```

### Combined Output

Use both options together for maximum visibility:

```bash
make run-txn-tests SHOW_SQL=1 SHOW_OUTPUT=1 REAL_DB=1
```

## Test Structure

### Python Test Files

Each test suite contains Python files with test handlers:

```python
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState

class BasicTransactionHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        print("Starting transaction test")
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Test logic here
            context.connection.execute_query("SELECT 1")
            return PyState.completed()
        return PyState.connecting()
    
    def exit(self, context: PyStateContext) -> None:
        print("Transaction test completed")
```

### Test Discovery

The test runner automatically discovers Python test files in each test suite directory. Test files should:
- Be named `test_*.py`
- Contain classes that inherit from `PyStateHandler`
- Be located in the appropriate test suite directory

## Multi-Connection Testing

For tests that require multiple concurrent connections:

```python
from src.common.test_rig_python import MultiConnectionTestHandler, PyStateContext, PyState

class ConcurrentTransactionHandler(MultiConnectionTestHandler):
    def __init__(self):
        super().__init__(connection_count=3)  # Use 3 concurrent connections
    
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

## Troubleshooting

### Common Issues

#### Import Errors
```
ModuleNotFoundError: No module named 'src.common.test_rig_python'
```
**Solution**: Ensure you're running tests through the Makefile, not directly with Python.

#### Connection Errors
```
Can't connect to MySQL server on 'localhost:4000' (111)
```
**Solution**: 
- Check that your database server is running
- Verify connection parameters in environment variables
- Use mock connections for development: `make run-txn-tests` (without `REAL_DB=1`)

#### Permission Errors
```
Permission denied when installing Python packages
```
**Solution**: Use virtual environments or user-level installation:
```bash
pip3 install --user --break-system-packages mysql-connector-python
```

#### Compilation Errors
```
error: failed to run custom build command for `pyo3`
```
**Solution**: Install Python development headers:
```bash
sudo apt install python3.11-dev  # Replace with your Python version
```

### Debugging

#### Enable Verbose Output
```bash
make run-txn-tests SHOW_OUTPUT=1
```

#### Enable SQL Logging
```bash
make run-txn-tests SHOW_SQL=1
```

#### Check Environment Variables
```bash
echo "TIDB_HOST: $TIDB_HOST"
echo "TIDB_USER: $TIDB_USER"
echo "TIDB_DATABASE: $TIDB_DATABASE"
```

#### Test with Mock Connections First
```bash
# Use mock connections to isolate Python logic issues
make run-txn-tests
```

### Error Messages

#### "Failed to execute handler"
This usually indicates an error in the Python handler code. Check:
- Python syntax errors
- Missing imports
- Logic errors in the handler

#### "Unknown MySQL server host"
This indicates a connection issue. Check:
- Hostname and port format (should be `hostname:port`)
- Network connectivity
- Database server status

#### "Access denied for user"
This indicates an authentication issue. Check:
- Username and password
- User permissions
- Database name

## Best Practices

### Development Workflow

1. **Start with mock connections**: Develop and test with mock connections first
2. **Test with real database**: Once working, test with real database connections
3. **Use SQL logging**: Enable SQL logging to see what queries are being executed
4. **Isolate tests**: Each test should be independent and not affect others

### Handler Design

1. **Keep handlers focused**: Each handler should test one specific aspect
2. **Use descriptive names**: Handler class names should clearly indicate what they test
3. **Handle connection availability**: Always check if `context.connection` is available
4. **Return appropriate states**: Use the correct state transitions

### Error Handling

1. **Catch and handle exceptions**: Wrap database operations in try-catch blocks
2. **Provide meaningful error messages**: Include context in error messages
3. **Clean up resources**: Use the `exit` method for cleanup

## Examples

### Basic Test Execution
```bash
# Run all tests with mock connections
make run-python-tests

# Run specific test suite with real database
TIDB_HOST=localhost:4000 TIDB_USER=root TIDB_PASSWORD= make run-txn-tests REAL_DB=1

# Run with full output and SQL logging
make run-txn-tests SHOW_SQL=1 SHOW_OUTPUT=1 REAL_DB=1
```

### Environment Setup
```bash
# Set up environment variables
export TIDB_HOST="your-tidb-host:4000"
export TIDB_USER="your-username"
export TIDB_PASSWORD="your-password"
export TIDB_DATABASE="test"

# Run tests
make run-txn-tests REAL_DB=1 SHOW_SQL=1
```

### Development Testing
```bash
# Quick test with mock connections
make run-txn-tests SHOW_OUTPUT=1

# Test specific functionality
make run-ddl-tests SHOW_SQL=1
``` 
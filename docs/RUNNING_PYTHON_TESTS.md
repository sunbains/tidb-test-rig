# Running Python tests

This guide covers how to run and develop Python-based DDL tests for TiDB using the Rust `test_rig` framework.

## Overview

The DDL testing module provides a comprehensive suite of Python tests for TiDB Data Definition Language operations. These tests are designed to work seamlessly with the Rust library's Python plugin system, allowing you to test database schema modifications, table operations, and concurrent DDL scenarios.

## Quick Start

### Running DDL Tests with Cargo

The easiest way to run all DDL tests is using Cargo:

```bash
# Run all DDL tests with Python plugins enabled
cargo test -p ddl --features python_plugins

# Run a specific test
cargo test -p ddl --features python_plugins test_create_table
```

### Standalone Python Testing

You can also run tests independently using the shared Python stub module:

```bash
cd src/ddl
python3 -c "
import sys
sys.path.append('../..')
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState
from test_create_table import CreateTableHandler

# Create a mock context
context = PyStateContext(
    host='localhost',
    port=4000,
    username='root',
    password='',
    database='test',
    connection=None
)

# Test the handler
handler = CreateTableHandler()
result = handler.execute(context)
print(f'Test result: {result}')
"
```

## Test Structure

Each DDL test follows a consistent pattern using the `PyStateHandler` interface:

```python
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState

class MyDDLHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test data, drop existing objects"""
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS test_table")
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Main test logic - perform DDL operations"""
        if context.connection:
            # Perform your DDL operations here
            context.connection.execute_query("CREATE TABLE test_table (id INT)")
            context.connection.execute_query("ALTER TABLE test_table ADD COLUMN name VARCHAR(100)")
            return PyState.completed()
        return PyState.error("No connection available")
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase - remove test objects"""
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS test_table")
```

## Available Test Categories

### Basic DDL Operations

#### Table Operations
- **`test_create_table.py`** - CREATE TABLE with various data types and constraints
- **`test_alter_table.py`** - Comprehensive ALTER TABLE operations including:
  - ADD/MODIFY/DROP columns
  - Index operations
  - Foreign key constraints
  - Table attributes (engine, charset, etc.)
- **`test_drop_table.py`** - DROP TABLE operations
- **`test_rename_table.py`** - RENAME TABLE operations
- **`test_truncate_table.py`** - TRUNCATE TABLE operations

#### Database Operations
- **`test_create_database.py`** - CREATE DATABASE operations
- **`test_alter_database.py`** - ALTER DATABASE operations

#### Index Operations
- **`test_create_index.py`** - CREATE INDEX operations
- **`test_drop_index.py`** - DROP INDEX operations

### Advanced Features
- **`test_views.py`** - CREATE/DROP VIEW operations
- **`test_temp_tables.py`** - Temporary table operations

### Concurrent Operations
- **`test_concurrent_ddl_dml.py`** - Concurrent DDL and DML operations
- **`test_concurrent_alter_table_conflict.py`** - Concurrent ALTER TABLE conflicts
- **`test_concurrent_create_drop_table.py`** - Concurrent CREATE/DROP operations
- **`test_concurrent_insert_alter.py`** - Concurrent INSERT and ALTER operations

### System and Information
- **`test_information_schema.py`** - Information schema queries
- **`test_error_conditions.py`** - Error handling and edge cases
- **`test_permissions.py`** - Permission-related tests

## Example Test Walkthrough

Let's examine a typical DDL test - `test_create_table.py`:

```python
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState

class CreateTableHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        # Clean up any existing test table
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Create a test table with various data types
            context.connection.execute_query("""
                CREATE TABLE ddl_test (
                    id INT PRIMARY KEY AUTO_INCREMENT,
                    name VARCHAR(100) NOT NULL,
                    email VARCHAR(255) UNIQUE,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )
            """)
            
            # Verify the table was created
            result = context.connection.execute_query("SHOW TABLES LIKE 'ddl_test'")
            if result and any('ddl_test' in str(row) for row in result):
                return PyState.completed()
        
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        # Clean up the test table
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")
```

## Advanced Example: ALTER TABLE Operations

The `test_alter_table.py` demonstrates comprehensive ALTER TABLE testing:

```python
class AlterTableHandler(PyStateHandler):
    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            conn = context.connection
            
            # Add columns
            conn.execute_query("ALTER TABLE ddl_test ADD COLUMN phone VARCHAR(20)")
            conn.execute_query("ALTER TABLE ddl_test ADD COLUMN address TEXT AFTER name")
            
            # Modify columns
            conn.execute_query("ALTER TABLE ddl_test MODIFY COLUMN name VARCHAR(200)")
            conn.execute_query("ALTER TABLE ddl_test MODIFY COLUMN age INT NOT NULL")
            
            # Add indexes
            conn.execute_query("ALTER TABLE ddl_test ADD INDEX idx_name (name)")
            conn.execute_query("ALTER TABLE ddl_test ADD UNIQUE INDEX idx_email (email)")
            
            # Table attributes
            conn.execute_query("ALTER TABLE ddl_test ENGINE = InnoDB")
            conn.execute_query("ALTER TABLE ddl_test CONVERT TO CHARACTER SET utf8mb4")
            
            return PyState.completed()
        
        return PyState.completed()
```

## State Management

DDL tests use a simple state management system:

- **`PyState.connecting()`** - Move to connecting state (typically used in `enter()`)
- **`PyState.completed()`** - Complete successfully
- **`PyState.error(message)`** - Complete with error

## Shared Mock Implementation

The `src/common/test_rig_python.py` stub module provides a shared mock database implementation that:

- Simulates SQL operations without requiring a real database
- Returns appropriate mock results based on query patterns
- Provides debugging output for executed queries
- Allows standalone testing and development
- Is shared across all Python test suites (DDL, Scale, etc.)

## Integration with Rust Framework

When integrated with the Rust `test_rig` framework:

```rust
use test_rig::load_python_handlers;
use test_rig::StateMachine;

let mut state_machine = StateMachine::new();
load_python_handlers(&mut state_machine, "src.ddl")?;

// The state machine will now include all DDL test handlers
```

## Development Guidelines

### Creating New DDL Tests

1. **Follow the naming convention**: `test_<operation>.py`
2. **Inherit from `PyStateHandler`**: All tests must implement the interface
3. **Use proper cleanup**: Always clean up test objects in the `exit()` method
4. **Handle errors gracefully**: Use try-catch blocks for operations that might fail
5. **Test edge cases**: Include tests for error conditions and boundary cases

### Best Practices

- **Isolation**: Each test should be independent and not rely on other tests
- **Cleanup**: Always clean up test objects to avoid interference
- **Verification**: Verify that operations completed successfully
- **Documentation**: Include docstrings explaining what the test covers
- **Error handling**: Test both success and failure scenarios

## Troubleshooting

### Common Issues

1. **Import errors**: Ensure you're importing from `src.common.test_rig_python`
2. **Path issues**: Add the project root to `sys.path` when running standalone
3. **Connection issues**: The mock implementation doesn't require real connections
4. **State transition errors**: Use the correct `PyState` methods
5. **Cleanup failures**: Always check if connection exists before operations

### Debugging

Enable debug output by setting environment variables:

```bash
export RUST_LOG=debug
cargo test -p ddl --features python_plugins
```

## Performance Considerations

- DDL tests are designed to be lightweight and fast
- Mock implementation allows for rapid iteration
- Real database tests will be slower but more comprehensive
- Consider running tests in parallel when possible

## Contributing

When adding new DDL tests:

1. Follow the existing patterns and conventions
2. Include comprehensive test coverage for the operation
3. Add appropriate error handling and edge cases
4. Update this documentation with new test descriptions
5. Ensure tests pass both with mock and real database connections

## Related Documentation

- [Python Plugins Guide](../PYTHON_PLUGINS.md) - General Python plugin system
- [Architecture Guide](../ARCHITECTURE.md) - Overall system architecture
- [Advanced Guide](../ADVANCED_GUIDE.md) - Advanced usage patterns 
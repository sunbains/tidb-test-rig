# DDL (Data Definition Language) Testing Module

This module contains Python tests for TiDB DDL operations that are compatible with the Rust `test_rig` framework.

## Overview

The DDL tests are designed to work with the Rust library's Python plugin system. Each test is implemented as a Python class that inherits from `PyStateHandler` and implements the required `enter()`, `execute()`, and `exit()` methods.

## Structure

- `test_rig_python.py` - Python stub module providing the necessary classes and methods
- `__init__.py` - Makes this directory a Python package
- `test_*.py` - Individual DDL test files

## Available Tests

### Basic DDL Operations
- `test_create_table.py` - CREATE TABLE operations
- `test_alter_table.py` - ALTER TABLE operations (ADD, MODIFY, DROP columns)
- `test_drop_table.py` - DROP TABLE operations
- `test_create_database.py` - CREATE DATABASE operations
- `test_alter_database.py` - ALTER DATABASE operations
- `test_rename_table.py` - RENAME TABLE operations

### Index Operations
- `test_create_index.py` - CREATE INDEX operations
- `test_drop_index.py` - DROP INDEX operations

### Advanced DDL Features
- `test_views.py` - CREATE/DROP VIEW operations
- `test_procedures.py` - CREATE/DROP PROCEDURE operations
- `test_temp_tables.py` - Temporary table operations
- `test_truncate_table.py` - TRUNCATE TABLE operations

### Concurrent Operations
- `test_concurrent_ddl_dml.py` - Concurrent DDL and DML operations
- `test_concurrent_alter_table_conflict.py` - Concurrent ALTER TABLE operations
- `test_concurrent_create_drop_table.py` - Concurrent CREATE/DROP TABLE operations
- `test_concurrent_insert_alter.py` - Concurrent INSERT and ALTER operations

### System and Information
- `test_information_schema.py` - Information schema queries
- `test_error_conditions.py` - Error handling tests
- `test_permissions.py` - Permission-related tests (placeholder)

## Usage

### Standalone Testing

The tests can be run independently using the Python stub module:

```python
from test_rig_python import PyStateHandler, PyStateContext, PyState, PyConnection
from test_create_table import CreateTableHandler

# Create a mock context
context = PyStateContext(
    host="localhost",
    port=4000,
    username="root",
    password="",
    database="test",
    connection=PyConnection()
)

# Create and test a handler
handler = CreateTableHandler()
result = handler.execute(context)
print(f"Result: {result}")
```

### Integration with Rust Framework

When integrated with the Rust `test_rig` framework (with `python_plugins` feature enabled):

```rust
use test_rig::load_python_handlers;

let mut state_machine = StateMachine::new();
load_python_handlers(&mut state_machine, "src.ddl")?;
```

## Handler Interface

All DDL test handlers implement the `PyStateHandler` interface:

```python
class MyHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        """Called when entering the state"""
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Called during state execution"""
        # Perform DDL operations using context.connection.execute_query()
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Called when exiting the state"""
        # Cleanup operations
        pass
```

## Mock Implementation

The `test_rig_python.py` stub module provides a mock implementation that:

- Simulates database operations without requiring a real database connection
- Returns appropriate mock results based on query patterns
- Allows testing of handler logic and state transitions
- Provides debugging output for executed queries

## State Transitions

Handlers use the following state transitions:
- `PyState.connecting()` - Move to connecting state
- `PyState.completed()` - Complete successfully
- `PyState.error(message)` - Complete with error

## Testing

To test all DDL modules:

```bash
cd src/ddl
python3 -c "from test_create_table import CreateTableHandler; print('Import successful')"
```

## Integration Notes

- The Python handlers are designed to work with the Rust state machine
- When used with the real Rust extension, `context.connection` will be a real database connection
- The mock implementation allows for standalone testing and development
- All handlers follow the same pattern for consistency and maintainability

# Running Python tests

This guide covers how to run and develop Python-based tests for TiDB using the Rust `test_rig` framework.

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

## Creating and Customizing Python Test Suites

For instructions on how to create new Python test directories, set up test files, and customize your test infrastructure, see:

**[How to Create Python Test Directories and Suites](./CREATING_PYTHON_TEST_DIRECTORIES.md)**

This includes:
- Directory structure and setup
- Sample `Cargo.toml` and `lib.rs`
- Test file and handler conventions
- Customization guidelines
- Integration with the Rust framework

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

- [How to Create Python Test Directories and Suites](./CREATING_PYTHON_TEST_DIRECTORIES.md)
- [Architecture Guide](./ARCHITECTURE.md)
- [Advanced Guide](./ADVANCED_GUIDE.md) 
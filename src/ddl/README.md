# DDL Test Suite

This directory contains comprehensive tests for TiDB Data Definition Language (DDL) operations. The test suite is organized into logical categories to provide clear separation of concerns and comprehensive coverage of DDL functionality.

## Test Organization

### Basic Operations (`test_basic_operations.py`)
Fundamental DDL operations that form the foundation of database schema management:

- **CreateTableHandler**: Tests basic table creation with various data types and constraints
- **DropTableHandler**: Tests table dropping and cleanup operations
- **CreateIndexHandler**: Tests index creation on table columns
- **DropIndexHandler**: Tests index removal operations
- **CreateDatabaseHandler**: Tests database creation and management
- **RenameTableHandler**: Tests table renaming operations
- **TruncateTableHandler**: Tests table truncation (removing all data while preserving structure)

### Concurrent Operations (`test_concurrent_operations.py`)
Tests that verify TiDB's handling of multiple DDL operations running simultaneously:

- **ConcurrentCreateDropTableHandler**: Tests concurrent CREATE and DROP TABLE operations
- **ConcurrentInsertAlterHandler**: Tests concurrent INSERT and ALTER TABLE operations
- **ConcurrentDdlDmlHandler**: Tests concurrent DDL and DML operations
- **ConcurrentAlterTableConflictHandler**: Tests concurrent ALTER TABLE operations that might conflict

### Specialized Objects (`test_specialized_objects.py`)
Tests for specialized database objects with specific behaviors and requirements:

- **ViewsHandler**: Tests view creation, management, and behavior
- **TempTablesHandler**: Tests temporary table creation and session-scoped behavior
- **ProceduresHandler**: Tests stored procedure creation and management
- **PermissionsHandler**: Tests basic permission and grant operations
- **InformationSchemaHandler**: Tests information schema access and queries

### Comprehensive ALTER Operations

#### ALTER TABLE (`test_alter_table.py`)
Comprehensive test covering all ALTER TABLE operations including:
- Column operations (ADD, MODIFY, CHANGE, DROP)
- Index operations (ADD, DROP)
- Constraint operations (PRIMARY KEY, FOREIGN KEY, UNIQUE)
- Table properties (ENGINE, CHARACTER SET, COLLATION, COMMENT)
- Partition operations
- Storage and compression options

#### ALTER INDEX (`test_alter_index.py`)
Comprehensive test covering ALTER INDEX operations including:
- Visibility controls (VISIBLE, INVISIBLE)
- Renaming operations
- Algorithm and lock options
- Storage options (DISK, MEMORY)
- Performance optimizations (REBUILD, REORGANIZE)
- Validation and compression options

#### ALTER DATABASE (`test_alter_database.py`)
Comprehensive test covering ALTER DATABASE operations including:
- Character set and collation changes
- Database properties and options
- Storage and performance settings

#### ALTER USER (`test_alter_user.py`)
Comprehensive test covering ALTER USER operations including:
- Password changes and authentication
- Account locking and unlocking
- Resource limits and quotas
- SSL and connection requirements
- Role and privilege management

#### ALTER VIEW (`test_alter_view.py`)
Comprehensive test covering ALTER VIEW operations including:
- View definition modifications
- Column changes and additions
- Security and access controls
- Performance optimizations

## Test Structure

Each test file follows the standard Python test handler pattern:

1. **enter()**: Setup phase - creates necessary objects and prepares the test environment
2. **execute()**: Test execution - performs the actual DDL operations and verifies results
3. **exit()**: Cleanup phase - removes test objects and restores the environment

## Running Tests

### Run All DDL Tests
```bash
make run-ddl-tests
```

### Run Specific Test Categories
```bash
# Basic operations only
cargo run --bin python_test_runner --features python_plugins -- --test-dir src/ddl --filter basic

# Concurrent operations only
cargo run --bin python_test_runner --features python_plugins -- --test-dir src/ddl --filter concurrent

# Specialized objects only
cargo run --bin python_test_runner --features python_plugins -- --test-dir src/ddl --filter specialized
```

### Run Individual Test Files
```bash
# Run only basic operations
cargo run --bin python_test_runner --features python_plugins -- --test-dir src/ddl --test-file test_basic_operations.py

# Run only ALTER TABLE tests
cargo run --bin python_test_runner --features python_plugins -- --test-dir src/ddl --test-file test_alter_table.py
```

## Test Coverage

The DDL test suite provides comprehensive coverage of:

- **Basic DDL Operations**: CREATE, DROP, RENAME, TRUNCATE for tables, indexes, and databases
- **Concurrent Operations**: Multi-threaded DDL operations and conflict resolution
- **Specialized Objects**: Views, temporary tables, stored procedures, permissions
- **Comprehensive ALTER Operations**: All aspects of ALTER statements for tables, indexes, databases, users, and views
- **Error Handling**: Invalid operations and edge cases
- **Performance**: Large-scale operations and optimization scenarios

## Integration with Rust

The DDL tests are integrated with the Rust test infrastructure through the `PyStateHandler` interface, allowing them to be executed as part of the larger test suite while maintaining the flexibility and expressiveness of Python for complex DDL scenarios. 
# TiDB Transaction Test Suite

This directory contains comprehensive transaction tests for TiDB, organized into meaningful categories to avoid duplication and provide clear test coverage.

## Test Structure

### 1. Basic Transactions (`test_basic_transactions.py`)
- **Purpose**: Tests fundamental transaction operations
- **Coverage**: Basic START TRANSACTION, COMMIT, ROLLBACK operations
- **Handlers**: `BasicTransactionHandler`

### 2. Basic Isolation (`test_isolation.py`)
- **Purpose**: Tests basic transaction isolation behavior
- **Coverage**: Transaction visibility, uncommitted changes isolation
- **Handlers**: `IsolationTestHandler`, `ConcurrentIsolationTestHandler`
- **Note**: For comprehensive isolation level testing, see `test_transaction_isolation_levels.py`

### 3. Isolation Levels (`test_transaction_isolation_levels.py`)
- **Purpose**: Comprehensive testing of all transaction isolation levels
- **Coverage**: READ UNCOMMITTED, READ COMMITTED, REPEATABLE READ, SERIALIZABLE
- **Handlers**: `ReadUncommittedTestHandler`, `ReadCommittedTestHandler`, `RepeatableReadTestHandler`, `SerializableTestHandler`, `IsolationLevelComparisonTestHandler`

### 4. Concurrency (`test_transaction_concurrency.py`)
- **Purpose**: Tests concurrent transaction behavior
- **Coverage**: Concurrent read/write operations, lock contention, race conditions
- **Handlers**: `ConcurrentReadWriteTestHandler`, `LockContentionTestHandler`, `RaceConditionTestHandler`, `ConcurrentTransactionConflictTestHandler`, `TransactionRollbackTestHandler`

### 5. MVCC and Version Chain (`test_mvcc_version_chain.py`)
- **Purpose**: Tests MVCC behavior and version chain management
- **Coverage**: MVCC deadlocks, phantom reads, version chain management, optimistic locking
- **Handlers**: `MVCCDeadlockTestHandler`, `PhantomReadDeadlockTestHandler`, `VersionChainDeadlockTestHandler`

### 6. Deadlock Detection (`test_deadlock_detection.py`)
- **Purpose**: Tests deadlock detection and isolation level behavior
- **Coverage**: Deadlock detection, automatic rollback, isolation level interactions
- **Handlers**: `DeadlockDetectionTestHandler`, `IsolationLevelDeadlockTestHandler`

### 7. Savepoints (`test_savepoints.py`)
- **Purpose**: Comprehensive savepoint testing
- **Coverage**: Basic savepoints, nested savepoints, savepoint release, error conditions, isolation interactions, performance
- **Handlers**: `BasicSavepointTestHandler`, `NestedSavepointTestHandler`, `SavepointReleaseTestHandler`, `SavepointErrorTestHandler`, `SavepointIsolationTestHandler`, `SavepointPerformanceTestHandler`

### 8. Error Handling (`test_transaction_errors.py`)
- **Purpose**: Tests transaction error scenarios and recovery
- **Coverage**: Constraint violations, deadlock recovery, timeouts, rollback recovery, error propagation
- **Handlers**: `ConstraintViolationTestHandler`, `DeadlockRecoveryTestHandler`, `TransactionTimeoutTestHandler`, `RollbackRecoveryTestHandler`, `ErrorPropagationTestHandler`, `RecoveryMechanismTestHandler`

### 9. Performance (`test_transaction_performance.py`)
- **Purpose**: Tests transaction performance characteristics
- **Coverage**: Bulk operations, large transactions, performance monitoring, size limits
- **Handlers**: `BulkInsertTestHandler`, `BulkUpdateTestHandler`, `LargeTransactionTestHandler`, `TransactionPerformanceMonitoringTestHandler`, `TransactionSizeLimitTestHandler`

### 10. Edge Cases (`test_transaction_edge_cases.py`)
- **Purpose**: Tests edge cases requiring multiple concurrent connections
- **Coverage**: Real deadlock scenarios, nested transactions with multiple connections
- **Handlers**: `DeadlockTestHandler`, `NestedTransactionTestHandler`
- **Note**: Uses `MultiConnectionTestHandler` infrastructure for true concurrency

## Key Features

### Multi-Connection Infrastructure
The `test_transaction_edge_cases.py` file uses the `MultiConnectionTestHandler` base class which provides:
- Multiple concurrent database connections
- Thread-safe operations
- Concurrent execution of operations across connections
- Real-world scenario testing

### Test Categories
Tests are organized to avoid duplication:
- **Basic operations** are in dedicated files
- **Complex scenarios** are split by domain (isolation, concurrency, etc.)
- **Edge cases** that require multiple connections are isolated
- **Performance tests** are separate from functional tests

### Comprehensive Coverage
The test suite covers:
- ✅ Basic transaction operations
- ✅ All isolation levels
- ✅ Concurrent operations
- ✅ Deadlock scenarios
- ✅ Savepoint functionality
- ✅ Error handling and recovery
- ✅ Performance characteristics
- ✅ Edge cases with multiple connections

## Running Tests

### Run All Transaction Tests
```bash
make run-txn-tests
```

### Run Specific Test Categories
```bash
# Run only basic transactions
python3 test_basic_transactions.py

# Run only isolation tests
python3 test_isolation.py

# Run only savepoint tests
python3 test_savepoints.py
```

### Using the Test Runner
The test runner automatically discovers and runs all test files:
```bash
cargo run --bin python_test_runner --features="python_plugins" -- --suite txn
```

## Test Infrastructure

### Handler Base Classes
- `PyStateHandler`: Base class for single-connection tests
- `MultiConnectionTestHandler`: Base class for multi-connection tests

### Common Utilities
- `PyStateContext`: Context for test execution
- `PyConnection`: Mock database connection with thread safety
- `PyState`: State management for test workflows

### Test Patterns
Each test follows a consistent pattern:
1. **Enter**: Setup test environment (tables, data)
2. **Execute**: Run the actual test logic
3. **Exit**: Cleanup test environment

## Contributing

When adding new tests:
1. **Check for duplicates** in existing files
2. **Choose the appropriate category** based on test purpose
3. **Use the right base class** (single vs multi-connection)
4. **Follow the established patterns** for consistency
5. **Update this README** if adding new categories 
# Transaction Tests Module

This directory contains Python-based tests for TiDB transaction operations.

## Quick Start

```bash
# Run all transaction tests
cargo test -p txn --features python_plugins

# Run standalone Python tests
cd src/txn
python3 -c "
import sys
sys.path.append('../..')
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState
from test_basic_transactions import BasicTransactionHandler
print('✅ Transaction tests import correctly')
"
```

## Documentation

For comprehensive documentation, examples, and usage instructions, see:

**[📖 Running Python Tests Guide](../../docs/RUNNING_PYTHON_TESTS.md)**

## Test Files

- `test_*.py` - Individual test files
- `__init__.py` - Makes this a Python package

## Available Tests

- **Basic Transactions**: Test fundamental transaction operations (START, COMMIT, ROLLBACK)
- **Concurrent Transactions**: Test transaction isolation and concurrency
- **Transaction Isolation**: Test different isolation levels
- **Deadlock Detection**: Test deadlock scenarios and resolution
- **Savepoints**: Test savepoint functionality
- **Error Handling**: Test transaction error conditions

## Shared Infrastructure

All Python tests use the shared `test_rig_python.py` stub module located in `src/common/`. This provides:
- Consistent mock database interface across all test suites
- Unified state management and error handling
- Easy maintenance and updates

See the [main documentation](../../docs/RUNNING_PYTHON_TESTS.md) for detailed descriptions of each test. 
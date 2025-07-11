# DDL Testing Module

This directory contains Python-based DDL (Data Definition Language) tests for TiDB.

## Quick Start

```bash
# Run all DDL tests
cargo test -p ddl --features python_plugins

# Run standalone Python tests
cd src/ddl
python3 -c "
import sys
sys.path.append('../..')
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState
from test_create_table import CreateTableHandler
print('✅ DDL tests import correctly')
"
```

## Documentation

For comprehensive documentation, examples, and usage instructions, see:

**[📖 Running Python Tests Guide](../../docs/RUNNING_PYTHON_TESTS.md)**

## Test Files

- `test_*.py` - Individual DDL test files
- `__init__.py` - Makes this a Python package

## Available Tests

- **Basic Operations**: CREATE/DROP/ALTER tables, databases, indexes
- **Advanced Features**: Views, temporary tables, concurrent operations
- **System Tests**: Information schema, error conditions, permissions

## Shared Infrastructure

All Python tests use the shared `test_rig_python.py` stub module located in `src/common/`. This provides:
- Consistent mock database interface across all test suites
- Unified state management and error handling
- Easy maintenance and updates

See the [main documentation](../../docs/RUNNING_PYTHON_TESTS.md) for detailed descriptions of each test. 
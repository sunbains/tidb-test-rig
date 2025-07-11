# Python Plugin System and Test Directory Creation

This guide covers both the Python plugin system for the test_rig framework and how to create new Python test directories by copying the necessary files from `src/ddl`.

## Python Plugin System Overview

The test_rig framework supports writing state handlers in Python, allowing you to leverage Python's rich ecosystem while maintaining the performance and reliability of the Rust-based state machine.

### Key Features

- ✅ **Async Support**: Python handlers support async/await operations
- ✅ **Database Access**: Full access to database connections and queries
- ✅ **State Management**: Complete access to state machine context
- ✅ **Error Handling**: Proper error propagation between Rust and Python
- ✅ **Hot Reloading**: Python handlers can be modified without restarting
- ✅ **Type Safety**: Type-safe bindings between Rust and Python

### Architecture

The Python plugin system uses PyO3 to provide seamless integration between Rust and Python:

- **Rust Core**: High-performance state machine and database operations
- **Python Handlers**: Easy-to-write state handlers with access to Python libraries
- **Automatic Loading**: Python handlers are automatically discovered and loaded
- **Mixed Execution**: Rust and Python handlers can be used together

## Quick Start with Python Plugins

### 1. Enable Python Plugins

Build with the `python_plugins` feature:

```bash
cargo build --features python_plugins
```

### 2. Write a Python Handler

Create a Python file with your handlers:

```python
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState

class MyPythonHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        print("Entering state...")
        return PyState.initial()
    
    def execute(self, context: PyStateContext) -> str:
        print("Executing state...")
        if context.connection:
            results = context.connection.execute_query("SELECT 1")
            print(f"Query returned {len(results)} results")
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        print("Exiting state...")
```

### 3. Run with Python Handlers

Use the `python_demo` binary to test your handlers:

```bash
cargo run --bin python_demo --features python_plugins -- --python-module my_handlers
```

## System Dependencies

### Debian/Ubuntu

Before using Python plugins, you need to install system dependencies on Debian/Ubuntu systems:

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
cargo run --bin python_demo --features python_plugins
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

Called when entering the state. Return the next state as a string.

```python
def enter(self, context: PyStateContext) -> str:
    print(f"Connecting to {context.host}")
    return PyState.connecting()
```

#### `execute(context: PyStateContext) -> str`

Called during state execution. This is where your main logic goes.

```python
def execute(self, context: PyStateContext) -> str:
    if context.connection:
        results = context.connection.execute_query("SHOW TABLES")
        if results:
            return PyState.completed()
        else:
            return PyState.testing_connection()
    return PyState.connecting()
```

#### `exit(context: PyStateContext) -> None`

Called when exiting the state. Use for cleanup.

```python
def exit(self, context: PyStateContext) -> None:
    print("State execution completed")
```

## Available States

Use `PyState` to return state names:

```python
from src.common.test_rig_python import PyState

# Available states:
PyState.initial()                    # Initial state
PyState.parsing_config()            # Parsing configuration
PyState.connecting()                # Connecting to database
PyState.testing_connection()        # Testing connection
PyState.verifying_database()        # Verifying database
PyState.getting_version()           # Getting database version
PyState.checking_import_jobs()      # Checking import jobs
PyState.showing_import_job_details() # Showing import job details
PyState.completed()                 # Completed state
```

## Context Access

### Connection Information

```python
def execute(self, context: PyStateContext) -> str:
    print(f"Host: {context.host}")
    print(f"Port: {context.port}")
    print(f"User: {context.username}")
    print(f"Database: {context.database}")
```

### Database Operations

```python
def execute(self, context: PyStateContext) -> str:
    if context.connection:
        # Execute a query
        results = context.connection.execute_query("SELECT * FROM users")
        
        # Process results
        for row in results:
            for key, value in row.items():
                print(f"{key}: {value}")
    
    return PyState.completed()
```

## Example Handlers

### Basic Connection Test

```python
class ConnectionTestHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        print(f"Testing connection to {context.host}")
        return PyState.testing_connection()
    
    def execute(self, context: PyStateContext) -> str:
        if not context.connection:
            return PyState.connecting()
        
        try:
            results = context.connection.execute_query("SELECT 1 as test")
            if results and len(results) > 0:
                print("✓ Connection test successful")
                return PyState.completed()
            else:
                print("✗ Connection test failed")
                return PyState.connecting()
        except Exception as e:
            print(f"✗ Connection error: {e}")
            return PyState.connecting()
    
    def exit(self, context: PyStateContext) -> None:
        print("Connection test completed")
```

### Import Job Monitor

```python
import time

class ImportJobMonitorHandler(PyStateHandler):
    def __init__(self, duration: int = 30):
        super().__init__()
        self.duration = duration
        self.start_time = None
    
    def enter(self, context: PyStateContext) -> str:
        print(f"Starting import job monitoring for {self.duration}s")
        self.start_time = time.time()
        return PyState.checking_import_jobs()
    
    def execute(self, context: PyStateContext) -> str:
        if not context.connection:
            return PyState.completed()
        
        # Check if monitoring time has elapsed
        if self.start_time and (time.time() - self.start_time) >= self.duration:
            print("Monitoring completed")
            return PyState.completed()
        
        try:
            results = context.connection.execute_query("SHOW IMPORT JOBS")
            active_jobs = [row for row in results if self._is_active_job(row)]
            
            if active_jobs:
                print(f"Found {len(active_jobs)} active import jobs")
                for job in active_jobs:
                    job_id = job.get('Job_ID', 'Unknown')
                    phase = job.get('Phase', 'Unknown')
                    print(f"  Job {job_id}: {phase}")
            
            time.sleep(5)  # Wait before next check
            return PyState.showing_import_job_details()
            
        except Exception as e:
            print(f"Error monitoring jobs: {e}")
            return PyState.completed()
    
    def _is_active_job(self, job_row):
        """Check if a job is active (End_Time is NULL)"""
        for key, value in job_row.items():
            if 'end_time' in key.lower() and value is not None and value != 'NULL':
                return False
        return True
    
    def exit(self, context: PyStateContext) -> None:
        if self.start_time:
            duration = time.time() - self.start_time
            print(f"Monitoring completed in {duration:.2f}s")
```

## Advanced Usage

### Custom Configuration

```python
class ConfigurableHandler(PyStateHandler):
    def __init__(self, config: dict):
        super().__init__()
        self.timeout = config.get('timeout', 30)
        self.retries = config.get('retries', 3)
        self.attempt = 0
    
    def execute(self, context: PyStateContext) -> str:
        self.attempt += 1
        
        if self.attempt > self.retries:
            print(f"Max retries ({self.retries}) exceeded")
            return PyState.completed()
        
        # Your logic here
        return PyState.testing_connection()
```

### Async Operations

```python
import asyncio

class AsyncHandler(PyStateHandler):
    async def execute(self, context: PyStateContext) -> str:
        # Simulate async work
        await asyncio.sleep(1)
        
        if context.connection:
            results = context.connection.execute_query("SELECT 1")
            return PyState.completed()
        
        return PyState.connecting()
```

## Error Handling

### Exception Handling

```python
class RobustHandler(PyStateHandler):
    def execute(self, context: PyStateContext) -> str:
        try:
            if not context.connection:
                raise ConnectionError("No database connection")
            
            results = context.connection.execute_query("SELECT 1")
            return PyState.completed()
            
        except ConnectionError as e:
            print(f"Connection error: {e}")
            return PyState.connecting()
        except Exception as e:
            print(f"Unexpected error: {e}")
            return PyState.completed()
```

### State Validation

```python
def execute(self, context: PyStateContext) -> str:
    # Validate context
    if not context.host:
        print("No host specified")
        return PyState.completed()
    
    if not context.connection:
        print("No database connection")
        return PyState.connecting()
    
    # Proceed with logic
    return PyState.completed()
```

## Best Practices

### 1. Handler Naming

Use descriptive names ending with "Handler":

```python
class DatabaseConnectionHandler(PyStateHandler): pass
class ImportJobMonitoringHandler(PyStateHandler): pass
class PerformanceTestHandler(PyStateHandler): pass
```

### 2. State Management

Keep state transitions clear and predictable:

```python
def execute(self, context: PyStateContext) -> str:
    # Always check prerequisites first
    if not self._can_proceed(context):
        return PyState.connecting()
    
    # Execute main logic
    result = self._do_work(context)
    
    # Return appropriate next state
    if result.success:
        return PyState.completed()
    else:
        return PyState.testing_connection()
```

### 3. Resource Management

Clean up resources in the `exit` method:

```python
def exit(self, context: PyStateContext) -> None:
    # Clean up any resources
    if hasattr(self, 'temp_files'):
        for file in self.temp_files:
            try:
                os.remove(file)
            except OSError:
                pass
```

### 4. Logging

Use Python's logging for better debugging:

```python
import logging

class LoggingHandler(PyStateHandler):
    def __init__(self):
        super().__init__()
        self.logger = logging.getLogger(__name__)
    
    def execute(self, context: PyStateContext) -> str:
        self.logger.info("Executing handler")
        # Your logic here
        self.logger.info("Handler completed")
        return PyState.completed()
```

## Integration with Existing Code

### Mixed Rust and Python Handlers

You can use both Rust and Python handlers in the same state machine:

#### With Core State Machine
```rust
// Register Rust handlers
state_machine.register_handler(State::Initial, Box::new(InitialHandler));
state_machine.register_handler(State::Connecting, Box::new(ConnectingHandler));

// Load Python handlers
load_python_handlers(&mut state_machine, "my_python_handlers")?;
```

#### With Dynamic State Machine
```rust
use test_rig::{DynamicStateMachine, common_states::*};

let mut machine = DynamicStateMachine::new();

// Register Rust handlers for common states
machine.register_handler(parsing_config(), Box::new(ParsingConfigHandlerAdapter));
machine.register_handler(connecting(), Box::new(ConnectingHandlerAdapter));

// Register test-specific states
machine.register_handler(creating_table(), Box::new(CreatingTableHandler));
machine.register_handler(populating_data(), Box::new(PopulatingDataHandler));

// Load Python handlers for custom states
load_python_handlers(&mut machine, "my_python_handlers")?;
```

## Performance Considerations

### When to Use Python Handlers

✅ **Good for Python handlers:**
- Complex business logic
- Data processing and analysis
- Integration with Python libraries
- Prototyping and experimentation
- Scripting and automation

❌ **Better in Rust:**
- High-frequency operations
- Memory-intensive tasks
- Low-level system operations
- Performance-critical code paths

### Optimization Tips

1. **Minimize Python-Rust calls**: Batch operations when possible
2. **Use async**: Leverage Python's async capabilities
3. **Cache results**: Store frequently accessed data
4. **Profile**: Monitor performance and optimize bottlenecks

---

# Creating New Python Test Directories

This section shows how to create a new Python test directory by copying the necessary files from `src/ddl` and customizing them for your specific test suite.

## Overview

The `src/ddl` directory serves as a template for creating new Python test suites. This guide walks through the process of creating a new test directory (e.g., `src/my_tests`) with all the necessary infrastructure.

## Step 1: Create the Directory Structure

```bash
# Create the new test directory
mkdir -p src/my_tests

# Copy the essential files from DDL
cp src/ddl/Cargo.toml src/my_tests/
cp src/ddl/lib.rs src/my_tests/
cp src/ddl/__init__.py src/my_tests/
cp src/ddl/README.md src/my_tests/
```

## Step 2: Customize Cargo.toml

Edit `src/my_tests/Cargo.toml` to update the package name. The test workspaces only need minimal configuration since they inherit dependencies from the main `test_rig` crate:

```toml
[package]
name = "my_tests"  # Change this to your test suite name
version = "0.1.0"
edition = "2024"

[lib]
name = "my_tests"  # Change this to match package name
path = "lib.rs"

[features]
python_plugins = ["test_rig/python_plugins"]

[dependencies]
test_rig = { path = "../.." }
```

**Note**: Test workspaces are intentionally minimal. They inherit all necessary dependencies (tokio, serde, tracing, etc.) from the main `test_rig` crate, so you only need to specify the core `test_rig` dependency.

## Step 3: Update lib.rs

Edit `src/my_tests/lib.rs` to reflect your test suite:

```rust
// Minimal lib.rs for the my_tests crate

// Re-export common test infrastructure if needed
// pub use crate::common::python_tests;

// Add any public API or test registration here as needed
```

## Step 4: Update __init__.py

Edit `src/my_tests/__init__.py` to make it a Python package:

```python
# My Tests Python Package
# This makes the directory a Python package for importing test modules
```

## Step 5: Create Your First Test File

Create a sample test file `src/my_tests/test_my_feature.py`:

```python
"""
Sample test for my feature.
"""

from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState

class MyFeatureHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test data"""
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS my_test")
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Main test logic"""
        if context.connection:
            # Create test table
            context.connection.execute_query("""
                CREATE TABLE my_test (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT
                )
            """)
            
            # Insert test data
            context.connection.execute_query("INSERT INTO my_test (id, name, value) VALUES (1, 'test', 42)")
            
            # Verify the data
            result = context.connection.execute_query("SELECT COUNT(*) FROM my_test")
            if result and result[0].get('col_0', 0) > 0:
                return PyState.completed()
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS my_test")
```

## Step 6: Update README.md

Edit `src/my_tests/README.md` to document your test suite:

```markdown
# My Tests Module

This directory contains Python-based tests for [describe your test suite].

## Quick Start

```bash
# Run all tests
cargo test -p my_tests --features python_plugins

# Run standalone Python tests
cd src/my_tests
python3 -c "
import sys
sys.path.append('../..')
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState
from test_my_feature import MyFeatureHandler
print('✅ My tests import correctly')
"
```

## Documentation

For comprehensive documentation, examples, and usage instructions, see:

**[📖 Running Python Tests Guide](../../docs/RUNNING_PYTHON_TESTS.md)**

## Test Files

- `test_*.py` - Individual test files
- `__init__.py` - Makes this a Python package

## Available Tests

- **Feature Tests**: [List your test categories]
- **Integration Tests**: [List integration tests]
- **Edge Cases**: [List edge case tests]

## Shared Infrastructure

All Python tests use the shared `test_rig_python.py` stub module located in `src/common/`. This provides:
- Consistent mock database interface across all test suites
- Unified state management and error handling
- Easy maintenance and updates

See the [main documentation](../../docs/RUNNING_PYTHON_TESTS.md) for detailed descriptions of each test.
```

## Step 7: Add to Workspace

Add your new test suite to the root `Cargo.toml` workspace members:

```toml
[workspace]
members = [
    "src/ddl",
    "src/scale",
    "src/my_tests",  # Add your new test suite here
]
```

## Step 8: Register in Common Infrastructure

Add your test suite to `src/common/python_tests.rs`:

```rust
/// List all Python test suites here
pub static PYTHON_SUITES: &[PythonSuiteConfig] = &[
    PythonSuiteConfig {
        name: "DDL",
        test_dir: "src/ddl",
        module_prefix: "src.ddl",
    },
    PythonSuiteConfig {
        name: "Scale",
        test_dir: "src/scale",
        module_prefix: "src.scale",
    },
    PythonSuiteConfig {
        name: "MyTests",  // Add your test suite here
        test_dir: "src/my_tests",
        module_prefix: "src.my_tests",
    },
    // Add more suites here as needed
];
```

## Step 9: Test Your Setup

Verify that your new test suite works:

```bash
# Test that the crate builds
cargo check -p my_tests --features python_plugins

# Test that Python imports work
cd src/my_tests
python3 -c "
import sys
sys.path.append('../..')
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState
from test_my_feature import MyFeatureHandler
print('✅ My tests setup is working!')
"

# Test with cargo
cargo test -p my_tests --features python_plugins
```

## Step 10: Create Additional Tests

Follow the pattern established in `src/ddl` to create more test files:

```bash
# Create additional test files
touch src/my_tests/test_another_feature.py
touch src/my_tests/test_edge_cases.py
touch src/my_tests/test_integration.py
```

## Customization Guidelines

### Test File Naming Convention

- Use `test_<feature>.py` for feature-specific tests
- Use `test_<operation>_<detail>.py` for detailed operation tests
- Use `test_concurrent_<scenario>.py` for concurrent operation tests
- Use `test_error_<condition>.py` for error condition tests

### Handler Class Naming Convention

- Use `<Feature>Handler` for feature-specific handlers
- Use `<Operation>Handler` for operation-specific handlers
- Use `<Scenario>Handler` for scenario-specific handlers

### Import Structure

Always import from the shared module:

```python
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState
```

### Test Structure

Follow the established pattern:

```python
class MyTestHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        # Setup - create test data, drop existing objects
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        # Main test logic - perform operations
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        # Cleanup - remove test objects
        pass
```

## Advanced Customization

### Custom Mock Responses

If your tests need specific mock responses, you can extend the shared `PyConnection` class:

```python
from src.common.test_rig_python import PyConnection

class CustomPyConnection(PyConnection):
    def execute_query(self, query: str):
        # Add custom mock responses for your specific queries
        if "MY_CUSTOM_QUERY" in query:
            return [{"custom_result": "expected_value"}]
        # Fall back to parent implementation
        return super().execute_query(query)
```

### Custom State Transitions

If you need additional state transitions, you can extend the `PyState` class:

```python
from src.common.test_rig_python import PyState

class CustomPyState(PyState):
    @staticmethod
    def my_custom_state() -> str:
        return "MyCustomState"
```

### Integration with Rust Framework

To integrate your Python tests with the Rust state machine:

```rust
use test_rig::load_python_handlers;

let mut state_machine = StateMachine::new();
load_python_handlers(&mut state_machine, "src.my_tests")?;
```

## Troubleshooting

### Common Issues

1. **Import errors**: Ensure you're importing from `src.common.test_rig_python`
2. **Path issues**: Add the project root to `sys.path` when running standalone
3. **Build errors**: Check that your `Cargo.toml` has the correct package name
4. **Missing dependencies**: Ensure all required dependencies are listed in `Cargo.toml`
5. **Module Not Found**: Ensure your Python module is in the Python path
6. **Handler Not Loaded**: Check that your handler class name ends with "Handler"
7. **State Machine Errors**: Check that your handler returns valid state names

### Debug Mode

Enable debug logging to see detailed information:

```bash
RUST_LOG=debug cargo run --bin python_demo --features python_plugins
```

### Python Debugging

Add debug prints to your Python handlers:

```python
def execute(self, context: PyStateContext) -> str:
    print(f"DEBUG: Context host = {context.host}")
    print(f"DEBUG: Context connection = {context.connection is not None}")
    # Your logic here
```

### Verification Checklist

- [ ] Directory structure created
- [ ] `Cargo.toml` customized with correct package name
- [ ] `lib.rs` created and minimal
- [ ] `__init__.py` created
- [ ] First test file created and working
- [ ] `README.md` updated
- [ ] Added to workspace members in root `Cargo.toml`
- [ ] Registered in `src/common/python_tests.rs`
- [ ] Tests build successfully with `cargo check`
- [ ] Python imports work correctly
- [ ] Tests run successfully with `cargo test`

## Example: Complete Test Suite

Here's a complete example of a new test suite called `src/validation`:

### Directory Structure
```
src/validation/
├── Cargo.toml
├── lib.rs
├── __init__.py
├── README.md
├── test_data_validation.py
├── test_constraint_validation.py
└── test_schema_validation.py
```

### Cargo.toml
```toml
[package]
name = "validation"
version = "0.1.0"
edition = "2024"

[lib]
name = "validation"
path = "lib.rs"

[features]
python_plugins = ["test_rig/python_plugins"]

[dependencies]
test_rig = { path = "../.." }
```

### test_data_validation.py
```python
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState

class DataValidationHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS validation_test")
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Test data validation logic here
            context.connection.execute_query("CREATE TABLE validation_test (id INT CHECK (id > 0))")
            return PyState.completed()
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS validation_test")
```

This template provides everything you need to create a new Python test suite that integrates seamlessly with the existing infrastructure.

## Support

For issues and questions:

1. Check the troubleshooting section above
2. Review the example handlers in `examples/python_handlers.py`
3. Enable debug logging for detailed error information
4. Consult the PyO3 documentation for advanced usage 
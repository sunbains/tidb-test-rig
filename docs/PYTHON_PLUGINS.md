# Python Plugin System for test_rig

The test_rig framework supports writing state handlers in Python, allowing you to leverage Python's rich ecosystem while maintaining the performance and reliability of the Rust-based state machine.

## Overview

The Python plugin system uses PyO3 to provide seamless integration between Rust and Python:

- **Rust Core**: High-performance state machine and database operations
- **Python Handlers**: Easy-to-write state handlers with access to Python libraries
- **Automatic Loading**: Python handlers are automatically discovered and loaded
- **Mixed Execution**: Rust and Python handlers can be used together

## Features

- ✅ **Async Support**: Python handlers support async/await operations
- ✅ **Database Access**: Full access to database connections and queries
- ✅ **State Management**: Complete access to state machine context
- ✅ **Error Handling**: Proper error propagation between Rust and Python
- ✅ **Hot Reloading**: Python handlers can be modified without restarting
- ✅ **Type Safety**: Type-safe bindings between Rust and Python

## Quick Start

### 1. Enable Python Plugins

Build with the `python_plugins` feature:

```bash
cargo build --features python_plugins
```

### 2. Write a Python Handler

Create a Python file with your handlers:

```python
from test_rig_python import PyStateHandler, PyStateContext, PyState

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
from test_rig_python import PyStateHandler

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
from test_rig_python import PyState

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
    print(f"Version: {context.version}")
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

## Troubleshooting

### Common Issues

1. **Module Not Found**: Ensure your Python module is in the Python path
2. **Handler Not Loaded**: Check that your handler class name ends with "Handler"
3. **Import Errors**: Verify all required Python packages are installed
4. **State Machine Errors**: Check that your handler returns valid state names

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

// Load Python handlers for custom states
load_python_handlers(&mut machine, "my_python_handlers")?;
```

### Gradual Migration

Start with simple handlers and gradually migrate more complex logic:

1. **Phase 1**: Add Python handlers for simple operations
2. **Phase 2**: Migrate complex business logic to Python
3. **Phase 3**: Keep only performance-critical code in Rust

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

## Support

For issues and questions:

1. Check the troubleshooting section above
2. Review the example handlers in `examples/python_handlers.py`
3. Enable debug logging for detailed error information
4. Consult the PyO3 documentation for advanced usage 
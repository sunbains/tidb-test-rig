# Advanced Guide: Extending, Architecture, Performance, and Troubleshooting

This guide covers advanced usage, design decisions, performance notes, and troubleshooting for the TiDB Test Rig framework.

---

## 1. Extending with Custom States and Handlers

The framework uses a dual state machine pattern with both core and dynamic state systems. You can add custom states and handlers for new test phases or logic. Handlers can be written in **Rust** or **Python** (see Python Plugin Support below).

### State Machine Types

#### Core State Machine (for standard workflows)
- **Use case**: Basic connection tests, standard workflows
- **States**: Predefined enum (`Initial`, `ParsingConfig`, `Connecting`, etc.)
- **Benefits**: Type-safe, compile-time validation

#### Dynamic State Machine (for extensible workflows)
- **Use case**: Test-specific workflows, custom test phases
- **States**: String-based, defined at compile time for each test
- **Benefits**: Flexible, extensible, supports custom data storage, no core library changes needed

### Example: Adding a Custom State (Rust)

#### For Core State Machine
```rust
use test_rig::state_machine::{State, StateContext, StateHandler};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct ReportingHandler;

#[async_trait]
impl StateHandler for ReportingHandler {
    async fn enter(&self, _context: &mut StateContext) -> test_rig::errors::Result<State> {
        println!("Generating summary report...");
        Ok(State::Reporting)
    }
    async fn execute(&self, context: &mut StateContext) -> test_rig::errors::Result<State> {
        // Access test results from context, print or save as needed
        println!("✓ Report generated");
        Ok(State::Completed)
    }
    async fn exit(&self, _context: &mut StateContext) -> test_rig::errors::Result<()> {
        Ok(())
    }
}

// Register your custom state and handler:
state_machine.register_handler(State::Reporting, Box::new(ReportingHandler));
```

#### For Dynamic State Machine
```rust
use test_rig::{
    DynamicState, DynamicStateContext, DynamicStateHandler, DynamicStateMachine,
    dynamic_state, common_states::*,
};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct CustomTestHandler;

#[async_trait]
impl DynamicStateHandler for CustomTestHandler {
    async fn enter(&self, _context: &mut DynamicStateContext) -> test_rig::errors::Result<DynamicState> {
        println!("Starting custom test...");
        Ok(dynamic_state!("custom_test", "Custom Test"))
    }
    async fn execute(&self, context: &mut DynamicStateContext) -> test_rig::errors::Result<DynamicState> {
        // Store custom data in context
        context.set_custom_data("test_results".to_string(), vec!["result1", "result2"]);
        println!("✓ Custom test completed");
        Ok(completed())
    }
    async fn exit(&self, _context: &mut DynamicStateContext) -> test_rig::errors::Result<()> {
        Ok(())
    }
}

// Define your state module
mod my_test_states {
    use super::*;
    
    // Re-export common states
    pub use test_rig::common_states::*;
    
    // Define custom states
    pub fn custom_test() -> DynamicState {
        dynamic_state!("custom_test", "Custom Test")
    }
}

// Register your custom state and handler:
let mut machine = DynamicStateMachine::new();
machine.register_handler(my_test_states::custom_test(), Box::new(CustomTestHandler));
```

### Example: Adding a Custom State (Python)

You can also write state handlers in Python and register them with the Rust state machine:

```python
from test_rig_python import PyStateHandler, PyStateContext, PyState

class ReportingHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        print("Generating summary report...")
        return PyState.completed()
    def execute(self, context: PyStateContext) -> str:
        print("✓ Report generated")
        return PyState.completed()
    def exit(self, context: PyStateContext) -> None:
        pass
```

Register your Python handler using the Rust API:
```rust
use test_rig::python_bindings::register_python_handler;
register_python_handler(&mut state_machine, State::Reporting, py_handler)?;
```

### Using Common States

When creating custom states, always use the common states module to avoid duplication:

```rust
mod my_test_states {
    use super::*;
    
    // Re-export common states to avoid duplication
    pub use test_rig::common_states::{
        parsing_config, connecting, testing_connection, 
        verifying_database, getting_version, completed,
    };
    
    // Define only test-specific states
    pub fn my_custom_phase() -> DynamicState {
        dynamic_state!("my_custom_phase", "My Custom Phase")
    }
}
```

---

## 2. Python Plugin Support and Cross-Language Integration

The framework supports Python plugins for state handlers, enabling rapid prototyping and cross-language test logic. Python handlers can be used alongside Rust handlers in the same state machine.

- Write handlers in Python using the `PyStateHandler` interface
- Register Python handlers for any state
- Use the standalone Python test runner for quick iteration
- See [docs/PYTHON_PLUGIN.md](PYTHON_PLUGIN.md) for full details and examples

### Configuration Compatibility
- Python scripts use the same config files (`tidb_config.json`/`.toml`), environment variables, and CLI arguments as Rust binaries
- Configuration priority: **CLI > Env > Config > Default** (for both Rust and Python)

### Example: Running a Python Isolation Test
```bash
export TIDB_HOST=tidb.example.com:4000
export TIDB_USER=myuser
export TIDB_PASSWORD=mypass
export TIDB_DATABASE=mydb
python3 examples/run_isolation_test.py
```

---

## 3. Architecture Decision Records (ADRs)

### Why a State Machine?
- **Explicit test phases**: Each phase (connect, test, verify, report) is a state.
- **Extensibility**: New states/handlers can be added in Rust or Python.
- **Error isolation**: Failures in one phase don't corrupt others.

### Why Plugin-Based Config Extensions?
- **Test-specific options**: Each test binary can add its own CLI/config logic.
- **No core changes needed**: New options don't require editing the main config generator.

### Error Handling and Retry Design
- **Rich error types**: Each failure mode has a specific error variant.
- **Retry/circuit breaker**: Resilience for transient DB/network issues.
- **Error context**: All errors can carry operation, attempt, and connection info.

### CLI/Environment/Config Precedence
- **CLI > Env > Config > Default**: Most explicit wins, for user control and predictability.
- **Python and Rust are now fully compatible in config precedence**

---

## 4. Performance Characteristics

- **Throughput**: The framework is async and can run many connections in parallel (see multi-connection tests).
- **Latency**: Each state transition adds a small overhead, but most time is spent in DB/network I/O.
- **Logging**: Structured logging is efficient, but file logging can add I/O overhead if enabled.
- **Scaling**: For high concurrency, increase connection pool size and use async handlers.
- **Resource usage**: Memory usage is low unless you store large test results in context.
- **Python plugin overhead**: Calling into Python from Rust adds some FFI overhead, but is negligible for most test logic.

**Tip:** For large-scale tests, run with `--release` and tune connection pool and async runtime settings.

---

## 5. Troubleshooting Guide

### Common Issues
- **Connection refused**: Check host/port, DB is running, firewall rules.
- **Authentication failed**: Check username/password, user privileges.
- **Unknown database**: Ensure the database exists and is accessible.
- **Timeouts**: Increase `timeout_secs` in config, check network latency.
- **State machine deadlock**: Ensure all states have handlers and transitions.
- **Test hangs**: Enable verbose logging (`-v`), check for stuck async tasks.
- **Python import errors**: Ensure `test_rig_python` is built and in your PYTHONPATH, or use the standalone Python runner for pure Python tests.
- **Python environment issues**: Make sure dependencies (e.g., `mysql-connector-python`) are installed in the correct environment.

### Debugging Tips
- **Enable verbose logging**: Use `-v` or `--log-level debug` for more output.
- **Check error context**: Error messages include operation, attempt, and connection info.
- **Use error context builder**: Add custom info to errors for easier debugging.
- **Run with RUST_BACKTRACE=1**: Get full stack traces for panics.
- **Use debug prints in Python handlers**: Print context and state transitions for troubleshooting.

### Interpreting Error Messages
- **Transient errors**: Will be retried automatically (connection, timeout, network).
- **Permanent errors**: Fail fast (auth, config, validation).
- **Circuit breaker open**: Too many failures, wait for recovery timeout.
- **Python handler errors**: Check Python stack trace and handler logic.

### What to Do If a Test Fails
- Check the error message and context.
- Try running with increased logging.
- Validate your config and CLI arguments.
- If using custom states, ensure all transitions are registered.
- For persistent issues, file a bug with logs and error context.
- For Python handler issues, check the [Python Plugin Documentation](PYTHON_PLUGIN.md).

---

For more, see the main README and [docs/PYTHON_PLUGIN.md](PYTHON_PLUGIN.md) for Python plugin details. 
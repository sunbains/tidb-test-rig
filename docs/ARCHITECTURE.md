# TiDB Testing Framework Architecture

## Overview
This project is a modular, extensible Rust tool for testing TiDB database functionality with Python plugin support. It supports both single and multiple concurrent connections, with a focus on clean state management, handler modularity, and robust error handling.

---

## Key Components

### 1. State Machine Core
The framework provides two complementary state management systems:

#### Core State Machine (`state_machine.rs`)
- **StateMachine**: Drives standard workflows for single connections, transitioning through predefined states.
- **State**: Enum representing each logical step in the workflow (`Initial`, `ParsingConfig`, `Connecting`, `TestingConnection`, `VerifyingDatabase`, `GettingVersion`, `Completed`, `Error`).
- **StateHandler**: Trait implemented by each handler, encapsulating logic for a specific state. Handlers are async and can maintain their own local context.
- **StateContext**: Minimal global context (connection, credentials, etc.), with support for handler-local context via a type-erased map.

#### Dynamic State Machine (`state_machine_dynamic.rs`)
- **DynamicStateMachine**: Drives extensible workflows with custom states defined at compile time.
- **DynamicState**: String-based state representation allowing tests to define their own states.
- **DynamicStateHandler**: Trait for handlers that work with dynamic states.
- **DynamicStateContext**: Extended context with custom data storage for test-specific information.

#### Common States (`common_states.rs`)
- **Shared Definitions**: Common workflow states used across multiple binaries to eliminate code duplication.
- **Available States**: `parsing_config()`, `connecting()`, `testing_connection()`, `verifying_database()`, `getting_version()`, `completed()`.
- **Benefits**: Single source of truth, consistent behavior, easier maintenance.

### 2. Handler Modularity
- Each state has its own handler struct implementing `StateHandler` or `DynamicStateHandler`.
- Handlers can store and retrieve their own context, reducing global state bloat and improving maintainability.
- New states/handlers can be added with minimal impact on the rest of the system.

### 3. Python Plugin System
- **PyO3 Integration**: Seamless integration between Rust and Python using PyO3.
- **Python State Handlers**: Write test logic in Python using `PyStateHandler` base class.
- **Type Safety**: Full type hints and validation between Rust and Python.
- **Mock and Real Connections**: Support for both mock connections (for testing) and real database connections.

### 4. Secure & Flexible CLI
- Uses `clap` for argument parsing.
- Secure password input via `rpassword`.
- Supports runtime configuration of host, user, database, and test-specific options.

### 5. Multi-Connection Coordination
- **MultiConnectionStateMachine**: Manages multiple `StateMachine` instances, each for a separate connection.
- **ConnectionCoordinator**: Shared state and event/message passing for coordination between connections.
- **SharedState**: Tracks global test status, connection results, and coordination events.
- **Tokio Tasks**: Each connection runs in its own async task for true concurrency.
- **Examples**: See `src/bin/simple_multi_connection.rs` and `src/bin/multi_connection.rs` for usage patterns.

### 6. Test Suites
- **DDL Tests** (`src/ddl/`): Data Definition Language operations testing
- **Transaction Tests** (`src/txn/`): Transaction isolation, concurrency, and error handling
- **Scale Tests** (`src/scale/`): Performance and scalability testing
- **Python Test Runner**: Automated execution of Python test suites

## Multi-Connection Architecture

### Recommended Approach: Multiple State Machines with Shared State

**Why this approach?**
- Better separation of concerns
- Easier to scale horizontally
- Each state machine can be simpler and focused
- Better for concurrent operations
- Easier to test individual connections

### Architecture Components

#### 1. Individual State Machines
Each connection gets its own state machine instance:
```rust
let mut state_machine = StateMachine::new();
// Register handlers for this specific connection
state_machine.register_handler(State::Initial, Box::new(InitialHandler));
// ... other handlers
```

#### 2. Shared State Coordinator
A coordinator manages shared state and coordinates between state machines:
```rust
pub struct SimpleMultiConnectionCoordinator {
    shared_state: Arc<Mutex<SharedTestState>>,
    connections: Vec<ConnectionConfig>,
}
```

#### 3. Concurrent Execution
Use Tokio tasks to run state machines concurrently:
```rust
let handle = tokio::spawn(async move {
    state_machine.run().await
});
```

### Implementation Examples

#### Simple Approach (Recommended for most use cases)
See `src/bin/simple_multi_connection.rs` for a straightforward implementation that:
- Creates multiple state machines
- Runs them concurrently
- Shares results through a simple coordinator
- Provides easy status tracking

#### Advanced Approach (For complex coordination)
See `src/bin/multi_connection.rs` for a more sophisticated implementation with:
- Message-passing coordination
- Event-driven architecture
- Complex state synchronization

## Python Plugin Architecture

### Core Components

#### 1. Python Bindings (`python_bindings.rs`)
- **PyStateContext**: Python wrapper for state context
- **PyConnection**: Python wrapper for database connections
- **PyState**: Python wrapper for state enums
- **PyStateHandler**: Base class for Python handlers

#### 2. Test Infrastructure (`common/test_rig_python.py`)
- **Mock Connections**: `PyConnection` for testing without real database
- **Real Connections**: `RealPyConnection` for actual database operations
- **Multi-Connection Support**: `MultiConnectionTestHandler` for concurrent testing

#### 3. Test Runner (`python_test_runner.rs`)
- **Test Discovery**: Automatically finds and runs Python test files
- **Environment Support**: Handles configuration via environment variables
- **Output Control**: Configurable output and SQL logging

### Python Handler Interface

```python
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState

class MyHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        # Called when entering the state
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        # Called during state execution
        if context.connection:
            results = context.connection.execute_query("SELECT 1")
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        # Called when exiting the state
        pass
```

## Error Handling Architecture

### Error Types
- **ConnectError**: Unified error type for all database operations
- **StateError**: State machine specific errors
- **ConfigError**: Configuration related errors
- **EnhancedError**: Rich error context with metadata

### Error Handling Features
- **Retry Strategies**: Exponential backoff with jitter
- **Circuit Breaker Pattern**: Automatic failure detection and recovery
- **Error Classification**: Transient vs permanent error handling
- **Context Preservation**: Rich error context for debugging

## Configuration Architecture

### Configuration Sources
1. **Command-line arguments** (highest priority)
2. **Environment variables** (`TIDB_HOST`, `TIDB_USER`, etc.)
3. **Configuration files** (JSON or TOML)
4. **Default values** (lowest priority)

### Configuration Extensions
- **ConfigExtension trait**: Allows test binaries to add custom configuration options
- **Runtime registration**: Extensions register themselves with the config system
- **Plugin pattern**: Keeps test-specific config in test binaries

## Example Workflows

### Core State Machine Workflow (Single Connection)
1. **Initial**: Start the workflow.
2. **ParsingConfig**: Parse CLI/config input.
3. **Connecting**: Establish DB connection.
4. **TestingConnection**: Run a test query.
5. **VerifyingDatabase**: Check DB existence.
6. **GettingVersion**: Query server version.
7. **Completed/Error**: End state.

### Dynamic State Machine Workflow (Extensible)
1. **Initial**: Start the workflow.
2. **ParsingConfig**: Parse CLI/config input (from common states).
3. **Connecting**: Establish DB connection (from common states).
4. **TestingConnection**: Run a test query (from common states).
5. **VerifyingDatabase**: Check DB existence (from common states).
6. **GettingVersion**: Query server version (from common states).
7. **Custom States**: Test-specific states (e.g., `creating_table`, `populating_data`, `testing_isolation`).
8. **Completed**: End state (from common states).

### Python Test Workflow
1. **Test Discovery**: Find Python test files in test directories
2. **Handler Loading**: Load and instantiate Python handlers
3. **Context Creation**: Create test context with connection info
4. **Handler Execution**: Execute enter, execute, and exit methods
5. **Result Reporting**: Report test results and any errors

## Best Practices

### State Machine Design
- Keep handlers focused on single responsibilities
- Use handler-local context for state-specific data
- Minimize global state in StateContext
- Implement proper error handling in all handlers

### Python Handler Development
- Inherit from `PyStateHandler` for all handlers
- Use type hints for better IDE support
- Handle connection availability gracefully
- Implement proper error handling

### Multi-Connection Testing
- Use connection pools for efficiency
- Implement proper error isolation
- Use shared state for coordination
- Monitor connection health

### Error Handling
- Classify errors appropriately (transient vs permanent)
- Implement retry strategies for transient errors
- Preserve error context for debugging
- Use circuit breakers for system protection

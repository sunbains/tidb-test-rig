# TiDB Multi-Connection Test Tool Architecture

## Overview
This project is a modular, extensible Rust tool for testing TiDB, database state, and import job monitoring. It supports both single and multiple concurrent connections, with a focus on clean state management, handler modularity, and robust error handling.

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
- Each state (e.g., Connecting, CheckingImportJobs) has its own handler struct implementing `StateHandler`.
- Handlers can store and retrieve their own context, reducing global state bloat and improving maintainability.
- New states/handlers can be added with minimal impact on the rest of the system.

### 3. Secure & Flexible CLI
- Uses `clap` for argument parsing.
- Secure password input via `rpassword`.
- Supports runtime configuration of host, user, database, and import job monitoring duration.

### 4. Multi-Connection Coordination
- **MultiConnectionStateMachine**: Manages multiple `StateMachine` instances, each for a separate connection.
- **ConnectionCoordinator**: Shared state and event/message passing for coordination between connections.
- **SharedState**: Tracks global test status, connection results, import jobs, and coordination events.
- **Tokio Tasks**: Each connection runs in its own async task for true concurrency.
- **Examples**: See `examples/simple_multi_connection.rs` and `examples/multi_connection_example.rs` for usage patterns.

## Multi-Connection Architecture (In-Depth)

# Multi-Connection Architecture for TiDB Testing

## Overview

When extending the testing to multiple connections that need to coordinate with each other, there are several architectural approaches. This document outlines the recommended approach and provides implementation examples.

## Recommended Approach: Multiple State Machines with Shared State

**Why this approach?**
- Better separation of concerns
- Easier to scale horizontally
- Each state machine can be simpler and focused
- Better for concurrent operations
- Easier to test individual connections

## Architecture Components

### 1. Individual State Machines
Each connection gets its own state machine instance:
```rust
let mut state_machine = StateMachine::new();
// Register handlers for this specific connection
state_machine.register_handler(State::Initial, Box::new(InitialHandler));
// ... other handlers
```

### 2. Shared State Coordinator
A coordinator manages shared state and coordinates between state machines:
```rust
pub struct SimpleMultiConnectionCoordinator {
    shared_state: Arc<Mutex<SharedTestState>>,
    connections: Vec<ConnectionConfig>,
}
```

### 3. Concurrent Execution
Use Tokio tasks to run state machines concurrently:
```rust
let handle = tokio::spawn(async move {
    state_machine.run().await
});
```

## Implementation Examples

### Simple Approach (Recommended for most use cases)
See `examples/simple_multi_connection.rs` for a straightforward implementation that:
- Creates multiple state machines
- Runs them concurrently
- Shares results through a simple coordinator
- Provides easy status tracking

### Advanced Approach (For complex coordination)
See `examples/multi_connection_example.rs` for a more sophisticated implementation with:
- Message-passing coordination
- Event-driven architecture
- Complex state synchronization
- Import job monitoring across connections

## Key Benefits

1. **Scalability**: Easy to add more connections
2. **Isolation**: Each connection's failures don't affect others
3. **Concurrency**: True parallel execution
4. **Maintainability**: Simple, focused components
5. **Testability**: Each component can be tested independently

## When to Use Each Approach

### Use Simple Approach When:
- You have 2-10 connections
- Basic coordination is sufficient
- You want minimal complexity
- Quick implementation is priority

### Use Advanced Approach When:
- You have many connections (>10)
- Complex coordination logic is needed
- Real-time status updates are required
- Event-driven architecture is preferred

## Migration Path

1. Start with the simple approach
2. Add complexity only when needed
3. Extract common patterns into shared libraries
4. Consider the advanced approach for large-scale deployments

## Example Usage

```rust
// Create coordinator
let mut coordinator = SimpleMultiConnectionCoordinator::new();

// Add connections
coordinator.add_connection(ConnectionConfig {
    id: "primary".to_string(),
    host: "tidb-primary.tidbcloud.com".to_string(),
    port: 4000,
    username: "user".to_string(),
    password: "password".to_string(),
    database: Some("test".to_string()),
});

// Run all connections concurrently
coordinator.run_all_connections().await?;

// Get results
coordinator.print_results();
```

## Best Practices

1. **Error Handling**: Each connection should handle its own errors
2. **Resource Management**: Use connection pools for efficiency
3. **Monitoring**: Implement health checks and status reporting
4. **Configuration**: Use environment variables or config files
5. **Logging**: Implement structured logging for debugging

## Conclusion

The multiple state machines with shared state approach provides the best balance of simplicity, scalability, and maintainability for most multi-connection testing scenarios. Start simple and add complexity only when the requirements demand it.

### 5. Extensibility & Best Practices
- Add new states by implementing `StateHandler` and registering in the state machine.
- Add new connection types or workflows by composing new state machines and/or coordinators.
- Use handler-local context for state-specific data, keeping `StateContext` minimal.
- All error types are unified as `StateError` for async compatibility.
- Clippy and Rustfmt clean for code quality.

---

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

### Multi-Connection Workflow
- Each connection runs either the core or dynamic workflow in parallel.
- Shared state tracks results, errors, and coordination events.
- Coordinators can implement custom logic (e.g., wait for all connections to be ready before proceeding).



---

## File Structure
- `src/lib.rs`: Main library with shared functionality and exports.
- `src/state_machine.rs`: Core state machine for standard workflows.
- `src/state_machine_dynamic.rs`: Dynamic state machine for extensible workflows.
- `src/common_states.rs`: Shared state definitions for common workflows.
- `src/state_handlers.rs`: Handlers for core state machine states.
- `src/import_job_handlers.rs`: Handlers and context for import job monitoring.
- `src/connection.rs`: Connection utilities.
- `src/connection_manager.rs`: Shared state and coordination for multi-connection mode.
- `src/multi_connection_state_machine.rs`: Multi-connection state machine logic.
- `src/bin/`: Binary executables for different test scenarios.
  - `basic.rs`: Basic connection test using core state machine.
  - `isolation.rs`: Isolation test using dynamic state machine.
  - `job_monitor.rs`: Job monitoring using dynamic state machine.
  - `simple_multi_connection.rs`: Simple multi-connection test.
  - `python_demo.rs`: Python plugin demonstration.
- `examples/`: Example usage for both single and multi-connection workflows.

---

## Extending the System

### Adding New States

#### For Core State Machine
- **Add a new state**: Implement `StateHandler`, add to `State` enum, and register in the state machine.

#### For Dynamic State Machine
- **Add a new state**: Use `dynamic_state!` macro to create states, implement `DynamicStateHandler`, and register in the dynamic state machine.
- **Use common states**: Import from `common_states` module to avoid duplication.

### Adding New Workflows
- **Add a new connection workflow**: Compose new state machines and/or coordinators.
- **Add test-specific states**: Define custom states in your binary's state module, re-exporting common states as needed.

### Adding New CLI Options
- **Add new CLI options**: Extend the `Args` struct in your binary and pass to handlers as needed.
- **Use configuration extensions**: Implement `ConfigExtension` trait for reusable CLI options.

### Best Practices
- **Use common states**: Always use the `common_states` module for standard workflow states.
- **Keep binaries focused**: Each binary should only define states specific to its test scenario.
- **Re-export common states**: Use `pub use test_rig::common_states::*` in your state modules.
- **Document custom states**: Provide clear documentation for any test-specific states you create.
- **Add new coordination logic**: Extend `ConnectionCoordinator` and `SharedState`.

---

## Best Practices
- Keep handler-local context for state-specific data.
- Use async/await for all I/O and state transitions.
- Handle errors at each state and propagate meaningful messages.
- Use shared state and message passing for coordination, not global mutable state.
- Keep the main state machine and context minimal and focused.

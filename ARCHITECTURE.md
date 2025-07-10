# TiDB Multi-Connection Test Tool Architecture

## Overview
This project is a modular, extensible Rust tool for testing TiDB, database state, and import job monitoring. It supports both single and multiple concurrent connections, with a focus on clean state management, handler modularity, and robust error handling.

---

## Key Components

### 1. State Machine Core
- **StateMachine**: Drives the workflow for a single connection, transitioning through states such as Initial, ParsingConfig, Connecting, TestingConnection, VerifyingDatabase, GettingVersion, CheckingImportJobs, and ShowingImportJobDetails.
- **State**: Enum representing each logical step in the workflow.
- **StateHandler**: Trait implemented by each handler, encapsulating logic for a specific state. Handlers are async and can maintain their own local context.
- **StateContext**: Minimal global context (connection, credentials, etc.), with support for handler-local context via a type-erased map.

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

### 5. Extensibility & Best Practices
- Add new states by implementing `StateHandler` and registering in the state machine.
- Add new connection types or workflows by composing new state machines and/or coordinators.
- Use handler-local context for state-specific data, keeping `StateContext` minimal.
- All error types are unified as `StateError` for async compatibility.
- Clippy and Rustfmt clean for code quality.

---

## Example Workflow (Single Connection)
1. **Initial**: Start the workflow.
2. **ParsingConfig**: Parse CLI/config input.
3. **Connecting**: Establish DB connection.
4. **TestingConnection**: Run a test query.
5. **VerifyingDatabase**: Check DB existence.
6. **GettingVersion**: Query server version.
7. **CheckingImportJobs**: List active import jobs.
8. **ShowingImportJobDetails**: Monitor job progress for N seconds.
9. **Completed/Error**: End state.

## Example Workflow (Multiple Connections)
- Each connection runs the above workflow in parallel.
- Shared state tracks results, errors, and coordination events.
- Coordinators can implement custom logic (e.g., wait for all connections to be ready before proceeding).

---

## File Structure
- `src/main.rs`: CLI entry point for single-connection mode.
- `src/state_machine.rs`: State machine core, context, and trait definitions.
- `src/state_handlers.rs`: Handlers for each state.
- `src/import_job_handlers.rs`: Handlers and context for import job monitoring.
- `src/connection.rs`: Connection utilities.
- `src/connection_manager.rs`: Shared state and coordination for multi-connection mode.
- `src/multi_connection_state_machine.rs`: Multi-connection state machine logic.
- `examples/`: Example usage for both single and multi-connection workflows.

---

## Extending the System
- **Add a new state**: Implement `StateHandler`, add to `State` enum, and register in the state machine.
- **Add a new connection workflow**: Compose new state machines and/or coordinators.
- **Add new CLI options**: Extend the `Args` struct in `main.rs` and pass to handlers as needed.
- **Add new coordination logic**: Extend `ConnectionCoordinator` and `SharedState`.

---

## Best Practices
- Keep handler-local context for state-specific data.
- Use async/await for all I/O and state transitions.
- Handle errors at each state and propagate meaningful messages.
- Use shared state and message passing for coordination, not global mutable state.
- Keep the main state machine and context minimal and focused.

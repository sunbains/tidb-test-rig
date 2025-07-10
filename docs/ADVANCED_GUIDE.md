# Advanced Guide: Extending, Architecture, Performance, and Troubleshooting

This guide covers advanced usage, design decisions, performance notes, and troubleshooting for the TiDB Test Rig framework.

---

## 1. Extending with Custom States

The framework uses a state machine pattern. You can add custom states and handlers for new test phases or logic.

### Example: Adding a Custom State

Suppose you want to add a `Reporting` state that generates a summary report after all tests.

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

You can define new states in your own enum or use the provided `State` enum with custom variants.

---

## 2. Architecture Decision Records (ADRs)

### Why a State Machine?
- **Explicit test phases**: Each phase (connect, test, verify, report) is a state.
- **Extensibility**: New states/handlers can be added without changing core logic.
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

---

## 3. Performance Characteristics

- **Throughput**: The framework is async and can run many connections in parallel (see multi-connection tests).
- **Latency**: Each state transition adds a small overhead, but most time is spent in DB/network I/O.
- **Logging**: Structured logging is efficient, but file logging can add I/O overhead if enabled.
- **Scaling**: For high concurrency, increase connection pool size and use async handlers.
- **Resource usage**: Memory usage is low unless you store large test results in context.

**Tip:** For large-scale tests, run with `--release` and tune connection pool and async runtime settings.

---

## 4. Troubleshooting Guide

### Common Issues
- **Connection refused**: Check host/port, DB is running, firewall rules.
- **Authentication failed**: Check username/password, user privileges.
- **Unknown database**: Ensure the database exists and is accessible.
- **Timeouts**: Increase `timeout_secs` in config, check network latency.
- **State machine deadlock**: Ensure all states have handlers and transitions.
- **Test hangs**: Enable verbose logging (`-v`), check for stuck async tasks.

### Debugging Tips
- **Enable verbose logging**: Use `-v` or `--log-level debug` for more output.
- **Check error context**: Error messages include operation, attempt, and connection info.
- **Use error context builder**: Add custom info to errors for easier debugging.
- **Run with RUST_BACKTRACE=1**: Get full stack traces for panics.

### Interpreting Error Messages
- **Transient errors**: Will be retried automatically (connection, timeout, network).
- **Permanent errors**: Fail fast (auth, config, validation).
- **Circuit breaker open**: Too many failures, wait for recovery timeout.

### What to Do If a Test Fails
- Check the error message and context.
- Try running with increased logging.
- Validate your config and CLI arguments.
- If using custom states, ensure all transitions are registered.
- For persistent issues, file a bug with logs and error context.

---

For more, see the main README and code comments throughout the framework. 
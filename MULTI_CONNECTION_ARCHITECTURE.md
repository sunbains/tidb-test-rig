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
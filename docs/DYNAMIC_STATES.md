# Dynamic States

This document explains how to make the state system dynamic, allowing tests to define their own states without modifying the core library.

## Approaches

### 1. String-Based Dynamic States (Recommended)

The `state_machine_dynamic` module provides a flexible string-based state system:

**Key Features:**
- ✅ **Maximum Flexibility**: Tests can define any states they need
- ✅ **No Core Library Changes**: States are defined in test code
- ✅ **Type Safety**: Still maintains type safety through the `DynamicState` struct
- ✅ **Validation**: Optional state transition validation
- ✅ **Custom Data**: Test-specific data storage in context
- ✅ **Backward Compatibility**: Can coexist with the enum-based system

**Usage Example:**

```rust
use test_rig::{
    dynamic_state, register_transitions, DynamicState, DynamicStateContext, DynamicStateHandler,
    DynamicStateMachine, states,
};

// Define custom states
let setup_state = dynamic_state!("setup_test_data", "Setting Up Test Data");
let test_state = dynamic_state!("run_performance_test", "Running Performance Test");
let cleanup_state = dynamic_state!("cleanup_test_data", "Cleaning Up Test Data");

// Create state machine
let mut machine = DynamicStateMachine::new();

// Register handlers
machine.register_handler(setup_state.clone(), Box::new(SetupHandler));
machine.register_handler(test_state.clone(), Box::new(TestHandler));
machine.register_handler(cleanup_state.clone(), Box::new(CleanupHandler));

// Register valid transitions
register_transitions!(machine, states::initial(), [setup_state.clone()]);
register_transitions!(machine, setup_state, [test_state.clone()]);
register_transitions!(machine, test_state, [cleanup_state.clone()]);
register_transitions!(machine, cleanup_state, [states::completed()]);

// Run the machine
machine.run().await?;
```

### 2. Enum Extension with Macros

Extend the existing enum system with macros:

```rust
// In test code
macro_rules! define_test_states {
    ($($name:ident),*) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum TestState {
            $($name),*
        }
        
        impl From<TestState> for State {
            fn from(test_state: TestState) -> Self {
                match test_state {
                    $(TestState::$name => State::Custom(stringify!($name))),*
                }
            }
        }
    };
}

define_test_states!(SetupData, RunTest, ValidateResults, Cleanup);
```

### 3. Plugin-Based State Registration

Register states at runtime:

```rust
// In core library
pub struct StateRegistry {
    states: HashMap<String, Box<dyn StateHandler + Send + Sync>>,
}

impl StateRegistry {
    pub fn register_state(&mut self, name: &str, handler: Box<dyn StateHandler + Send + Sync>) {
        self.states.insert(name.to_string(), handler);
    }
}
```

## Comparison

| Approach | Flexibility | Type Safety | Performance | Complexity |
|----------|-------------|-------------|-------------|------------|
| String-Based | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐ |
| Enum Extension | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| Plugin Registration | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |

## Recommended Approach: String-Based Dynamic States

The string-based approach is recommended because it provides:

1. **Maximum Flexibility**: Tests can define any states they need
2. **No Breaking Changes**: Doesn't require modifying the core library
3. **Good Performance**: Minimal overhead compared to enum-based system
4. **Type Safety**: Still maintains compile-time safety where possible
5. **Validation**: Optional state transition validation prevents invalid flows

## Implementation Details

### DynamicState Structure

```rust
pub struct DynamicState {
    name: String,           // Internal identifier
    display_name: Option<String>, // Human-readable name
}
```

### Context Extensions

The `DynamicStateContext` extends the base context with:

```rust
pub struct DynamicStateContext {
    // ... base fields ...
    custom_data: HashMap<String, Box<dyn Any + Send + Sync>>,
}
```

### Helper Macros

```rust
// Create states easily
let state = dynamic_state!("my_state", "My Custom State");

// Register transitions
register_transitions!(machine, from_state, [to_state1, to_state2]);
```

## Migration Guide

### From Enum-Based to Dynamic

1. **Replace State enum usage:**
   ```rust
   // Old
   State::Connecting
   
   // New
   states::connecting()
   ```

2. **Update handlers:**
   ```rust
   // Old
   impl StateHandler for MyHandler {
       async fn execute(&self, context: &mut StateContext) -> Result<State> {
           Ok(State::Completed)
       }
   }
   
   // New
   impl DynamicStateHandler for MyHandler {
       async fn execute(&self, context: &mut DynamicStateContext) -> Result<DynamicState> {
           Ok(states::completed())
       }
   }
   ```

3. **Register transitions (optional):**
   ```rust
   machine.register_transitions(
       states::initial(),
       vec![states::connecting(), custom_states::my_state()]
   );
   ```

## Best Practices

1. **Use Descriptive Names**: Make state names self-documenting
2. **Group Related States**: Use modules to organize custom states
3. **Validate Transitions**: Register valid transitions to catch errors early
4. **Store Test Data**: Use `set_custom_data()` for test-specific information
5. **Provide Display Names**: Use `with_display_name()` for better logging

## Common States Module

The framework provides a `common_states` module that eliminates code duplication by sharing common workflow states across multiple binaries.

### Available Common States

```rust
use test_rig::common_states::*;

// Core workflow states available to all binaries
parsing_config()      // "Parsing Configuration"
connecting()          // "Connecting to TiDB"
testing_connection()  // "Testing Connection"
verifying_database()  // "Verifying Database"
getting_version()     // "Getting Server Version"
completed()           // "Completed"
```

### Using Common States in Your Binary

```rust
mod my_test_states {
    use super::*;
    use test_rig::common_states::*;

    // Re-export common states
    pub use test_rig::common_states::{
        parsing_config, connecting, testing_connection, 
        verifying_database, getting_version, completed,
    };

    // Define test-specific states
    pub fn my_custom_state() -> DynamicState {
        dynamic_state!("my_custom", "My Custom State")
    }
    
    pub fn another_custom_state() -> DynamicState {
        dynamic_state!("another_custom", "Another Custom State")
    }
}
```

### Benefits of Common States

1. **Eliminates Duplication**: No need to redefine common workflow states in each binary
2. **Single Source of Truth**: Changes to common states only need to be made in one place
3. **Consistency**: All binaries use the same state definitions for common operations
4. **Maintainability**: Easier to maintain and update common workflow logic

## Example: Performance Test

```rust
mod performance_states {
    use super::*;
    use test_rig::common_states::*;
    
    // Re-export common states
    pub use test_rig::common_states::{
        parsing_config, connecting, testing_connection, 
        verifying_database, getting_version, completed,
    };
    
    // Performance test specific states
    pub fn setup_data() -> DynamicState {
        dynamic_state!("perf_setup", "Setting Up Performance Test Data")
    }
    
    pub fn run_benchmark() -> DynamicState {
        dynamic_state!("perf_benchmark", "Running Performance Benchmark")
    }
    
    pub fn analyze_results() -> DynamicState {
        dynamic_state!("perf_analyze", "Analyzing Performance Results")
    }
}

// Usage in test
let mut machine = DynamicStateMachine::new();
machine.register_handler(performance_states::setup_data(), Box::new(SetupHandler));
machine.register_handler(performance_states::run_benchmark(), Box::new(BenchmarkHandler));
machine.register_handler(performance_states::analyze_results(), Box::new(AnalyzeHandler));

// Store test configuration
context.set_custom_data("benchmark_iterations".to_string(), 1000u32);
context.set_custom_data("benchmark_timeout".to_string(), Duration::from_secs(60));
```

This approach gives you complete flexibility while maintaining the benefits of the state machine pattern. 
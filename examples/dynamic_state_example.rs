//! Example demonstrating dynamic state machine usage
//!
//! This example shows how tests can define their own states without
//! modifying the core library.

use async_trait::async_trait;
use test_rig::errors::Result;
use test_rig::{
    DynamicState, DynamicStateContext, DynamicStateHandler, DynamicStateMachine, dynamic_state,
    register_transitions, states,
};

// Custom states for a specific test
mod custom_states {
    use super::*;

    pub fn setup_test_data() -> DynamicState {
        dynamic_state!("setup_test_data", "Setting Up Test Data")
    }

    pub fn run_performance_test() -> DynamicState {
        dynamic_state!("run_performance_test", "Running Performance Test")
    }

    pub fn validate_results() -> DynamicState {
        dynamic_state!("validate_results", "Validating Test Results")
    }

    pub fn cleanup_test_data() -> DynamicState {
        dynamic_state!("cleanup_test_data", "Cleaning Up Test Data")
    }
}

// Custom handler for setting up test data
struct SetupTestDataHandler;

#[async_trait]
impl DynamicStateHandler for SetupTestDataHandler {
    async fn enter(&self, _context: &mut DynamicStateContext) -> Result<DynamicState> {
        println!("Entering setup test data state...");
        Ok(custom_states::setup_test_data())
    }

    async fn execute(&self, context: &mut DynamicStateContext) -> Result<DynamicState> {
        println!("Setting up test data...");

        // Store test-specific data in context
        context.set_custom_data("test_rows".to_string(), 1000u32);
        context.set_custom_data("test_table".to_string(), "performance_test".to_string());

        // Simulate some work
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        println!("✓ Test data setup completed");
        Ok(custom_states::run_performance_test())
    }

    async fn exit(&self, _context: &mut DynamicStateContext) -> Result<()> {
        println!("Exiting setup test data state...");
        Ok(())
    }
}

// Custom handler for running performance tests
struct PerformanceTestHandler;

#[async_trait]
impl DynamicStateHandler for PerformanceTestHandler {
    async fn enter(&self, context: &mut DynamicStateContext) -> Result<DynamicState> {
        println!("Entering performance test state...");

        // Retrieve test-specific data
        if let Some(test_rows) = context.get_custom_data::<u32>("test_rows") {
            println!("Will test with {} rows", test_rows);
        }

        Ok(custom_states::run_performance_test())
    }

    async fn execute(&self, context: &mut DynamicStateContext) -> Result<DynamicState> {
        println!("Running performance test...");

        // Simulate performance test
        let start = std::time::Instant::now();
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let duration = start.elapsed();

        // Store results
        context.set_custom_data("test_duration".to_string(), duration);
        context.set_custom_data("test_success".to_string(), true);

        println!("✓ Performance test completed in {:?}", duration);
        Ok(custom_states::validate_results())
    }

    async fn exit(&self, _context: &mut DynamicStateContext) -> Result<()> {
        println!("Exiting performance test state...");
        Ok(())
    }
}

// Custom handler for validating results
struct ValidateResultsHandler;

#[async_trait]
impl DynamicStateHandler for ValidateResultsHandler {
    async fn enter(&self, _context: &mut DynamicStateContext) -> Result<DynamicState> {
        println!("Entering validation state...");
        Ok(custom_states::validate_results())
    }

    async fn execute(&self, context: &mut DynamicStateContext) -> Result<DynamicState> {
        println!("Validating test results...");

        // Retrieve and validate results
        if let Some(success) = context.get_custom_data::<bool>("test_success") {
            if let Some(duration) = context.get_custom_data::<std::time::Duration>("test_duration")
            {
                if *success && duration.as_millis() < 1000 {
                    println!("✓ Test validation passed");
                    Ok(custom_states::cleanup_test_data())
                } else {
                    println!("✗ Test validation failed");
                    Ok(states::error("Performance test failed validation"))
                }
            } else {
                Ok(states::error("Missing test duration"))
            }
        } else {
            Ok(states::error("Missing test success status"))
        }
    }

    async fn exit(&self, _context: &mut DynamicStateContext) -> Result<()> {
        println!("Exiting validation state...");
        Ok(())
    }
}

// Custom handler for cleanup
struct CleanupHandler;

#[async_trait]
impl DynamicStateHandler for CleanupHandler {
    async fn enter(&self, _context: &mut DynamicStateContext) -> Result<DynamicState> {
        println!("Entering cleanup state...");
        Ok(custom_states::cleanup_test_data())
    }

    async fn execute(&self, context: &mut DynamicStateContext) -> Result<DynamicState> {
        println!("Cleaning up test data...");

        // Retrieve test table name
        if let Some(table_name) = context.get_custom_data::<String>("test_table") {
            println!("Cleaning up table: {}", table_name);
        }

        // Simulate cleanup
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        println!("✓ Cleanup completed");
        Ok(states::completed())
    }

    async fn exit(&self, _context: &mut DynamicStateContext) -> Result<()> {
        println!("Exiting cleanup state...");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Dynamic State Machine Example ===\n");

    // Create dynamic state machine
    let mut machine = DynamicStateMachine::new();

    // Register custom handlers
    machine.register_handler(
        custom_states::setup_test_data(),
        Box::new(SetupTestDataHandler),
    );
    machine.register_handler(
        custom_states::run_performance_test(),
        Box::new(PerformanceTestHandler),
    );
    machine.register_handler(
        custom_states::validate_results(),
        Box::new(ValidateResultsHandler),
    );
    machine.register_handler(custom_states::cleanup_test_data(), Box::new(CleanupHandler));

    // Register valid state transitions
    register_transitions!(
        machine,
        states::initial(),
        [custom_states::setup_test_data()]
    );
    register_transitions!(
        machine,
        custom_states::setup_test_data(),
        [custom_states::run_performance_test()]
    );
    register_transitions!(
        machine,
        custom_states::run_performance_test(),
        [custom_states::validate_results()]
    );
    register_transitions!(
        machine,
        custom_states::validate_results(),
        [custom_states::cleanup_test_data(), states::error("")]
    );
    register_transitions!(
        machine,
        custom_states::cleanup_test_data(),
        [states::completed()]
    );

    // Run the state machine
    match machine.run().await {
        Ok(_) => {
            println!("\n✅ Dynamic state machine completed successfully!");

            // Print final context data
            let context = machine.get_context();
            if let Some(duration) = context.get_custom_data::<std::time::Duration>("test_duration")
            {
                println!("Final test duration: {:?}", duration);
            }
        }
        Err(e) => {
            println!("\n❌ Dynamic state machine failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple handler for initial state
    struct InitialHandler;

    #[async_trait]
    impl DynamicStateHandler for InitialHandler {
        async fn enter(&self, _context: &mut DynamicStateContext) -> Result<DynamicState> {
            Ok(states::initial())
        }

        async fn execute(&self, _context: &mut DynamicStateContext) -> Result<DynamicState> {
            Ok(custom_states::setup_test_data())
        }

        async fn exit(&self, _context: &mut DynamicStateContext) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_dynamic_state_machine_flow() {
        let mut machine = DynamicStateMachine::new();

        // Register handlers for all states
        machine.register_handler(states::initial(), Box::new(InitialHandler));
        machine.register_handler(
            custom_states::setup_test_data(),
            Box::new(SetupTestDataHandler),
        );
        machine.register_handler(
            custom_states::run_performance_test(),
            Box::new(PerformanceTestHandler),
        );
        machine.register_handler(
            custom_states::validate_results(),
            Box::new(ValidateResultsHandler),
        );
        machine.register_handler(custom_states::cleanup_test_data(), Box::new(CleanupHandler));

        // Register transitions
        register_transitions!(
            machine,
            states::initial(),
            [custom_states::setup_test_data()]
        );
        register_transitions!(
            machine,
            custom_states::setup_test_data(),
            [custom_states::run_performance_test()]
        );
        register_transitions!(
            machine,
            custom_states::run_performance_test(),
            [custom_states::validate_results()]
        );
        register_transitions!(
            machine,
            custom_states::validate_results(),
            [custom_states::cleanup_test_data()]
        );
        register_transitions!(
            machine,
            custom_states::cleanup_test_data(),
            [states::completed()]
        );

        // Run the machine
        let result = machine.run().await;
        assert!(result.is_ok());

        // Verify custom data was stored
        let context = machine.get_context();
        assert!(context.get_custom_data::<u32>("test_rows").is_some());
        assert!(context.get_custom_data::<String>("test_table").is_some());
        assert!(context.get_custom_data::<bool>("test_success").is_some());
        assert!(
            context
                .get_custom_data::<std::time::Duration>("test_duration")
                .is_some()
        );
    }
}

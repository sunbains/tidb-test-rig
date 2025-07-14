//! # Dynamic State Machine
//!
//! Dynamic state machine implementation that allows tests to define their own states.
//! Uses string-based states instead of enums for maximum flexibility.

use crate::errors::ConnectError;
use mysql::PooledConn;
use std::any::Any;
use std::collections::HashMap;
use std::fmt;

/// Dynamic state representation using strings
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DynamicState {
    name: String,
    display_name: Option<String>,
}

impl DynamicState {
    /// Create a new dynamic state
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: None,
        }
    }

    /// Create a new dynamic state with a custom display name
    pub fn with_display_name(name: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: Some(display_name.into()),
        }
    }

    /// Get the state name
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the display name (falls back to name if not set)
    #[must_use]
    pub fn display_name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.name)
    }
}

impl fmt::Display for DynamicState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl From<&str> for DynamicState {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl From<String> for DynamicState {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

/// Predefined states for common operations
pub mod states {
    use super::DynamicState;

    #[must_use]
    pub fn initial() -> DynamicState {
        DynamicState::with_display_name("initial", "Initial")
    }

    #[must_use]
    pub fn parsing_config() -> DynamicState {
        DynamicState::with_display_name("parsing_config", "Parsing Configuration")
    }

    #[must_use]
    pub fn connecting() -> DynamicState {
        DynamicState::with_display_name("connecting", "Connecting to TiDB")
    }

    #[must_use]
    pub fn testing_connection() -> DynamicState {
        DynamicState::with_display_name("testing_connection", "Testing Connection")
    }

    #[must_use]
    pub fn verifying_database() -> DynamicState {
        DynamicState::with_display_name("verifying_database", "Verifying Database")
    }

    #[must_use]
    pub fn getting_version() -> DynamicState {
        DynamicState::with_display_name("getting_version", "Getting Server Version")
    }

    #[must_use]
    pub fn completed() -> DynamicState {
        DynamicState::with_display_name("completed", "Completed")
    }

    pub fn error(msg: impl Into<String>) -> DynamicState {
        DynamicState::with_display_name(format!("error:{}", msg.into()), "Error")
    }
}

/// Context data that flows through the dynamic state machine
pub struct DynamicStateContext {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: Option<String>,
    pub connection: Option<PooledConn>,
    pub server_version: Option<String>,
    pub error_message: Option<String>,
    // Handler-specific context storage
    handler_contexts: HashMap<DynamicState, Box<dyn Any + Send + Sync>>,
    // Custom data storage for test-specific data
    custom_data: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl Default for DynamicStateContext {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicStateContext {
    #[must_use]
    pub fn new() -> Self {
        Self {
            host: String::new(),
            port: 0,
            username: String::new(),
            password: String::new(),
            database: None,
            connection: None,
            server_version: None,
            error_message: None,
            handler_contexts: HashMap::new(),
            custom_data: HashMap::new(),
        }
    }

    pub fn set_error(&mut self, error: String) {
        self.error_message = Some(error);
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Store handler-specific context
    pub fn set_handler_context<T: Any + Send + Sync>(&mut self, state: DynamicState, context: T) {
        self.handler_contexts.insert(state, Box::new(context));
    }

    /// Retrieve handler-specific context
    #[must_use]
    pub fn get_handler_context<T: Any + Send + Sync>(&self, state: &DynamicState) -> Option<&T> {
        self.handler_contexts
            .get(state)
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Retrieve mutable handler-specific context
    pub fn get_handler_context_mut<T: Any + Send + Sync>(
        &mut self,
        state: &DynamicState,
    ) -> Option<&mut T> {
        self.handler_contexts
            .get_mut(state)
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Store custom data for tests
    pub fn set_custom_data<T: Any + Send + Sync>(&mut self, key: String, data: T) {
        self.custom_data.insert(key, Box::new(data));
    }

    /// Retrieve custom data
    #[must_use]
    pub fn get_custom_data<T: Any + Send + Sync>(&self, key: &str) -> Option<&T> {
        self.custom_data
            .get(key)
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Retrieve mutable custom data
    pub fn get_custom_data_mut<T: Any + Send + Sync>(&mut self, key: &str) -> Option<&mut T> {
        self.custom_data
            .get_mut(key)
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }
}

/// Trait for dynamic state handlers
#[async_trait::async_trait]
pub trait DynamicStateHandler {
    async fn enter(&self, context: &mut DynamicStateContext) -> Result<DynamicState, ConnectError>;
    async fn execute(
        &self,
        context: &mut DynamicStateContext,
    ) -> Result<DynamicState, ConnectError>;
    async fn exit(&self, context: &mut DynamicStateContext) -> Result<(), ConnectError>;
}

/// Dynamic state machine that manages the flow between states
pub struct DynamicStateMachine {
    current_state: DynamicState,
    context: DynamicStateContext,
    handlers: HashMap<DynamicState, Box<dyn DynamicStateHandler + Send + Sync>>,
    // State transitions for validation
    valid_transitions: HashMap<DynamicState, Vec<DynamicState>>,
}

impl Default for DynamicStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicStateMachine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_state: states::initial(),
            context: DynamicStateContext::new(),
            handlers: HashMap::new(),
            valid_transitions: HashMap::new(),
        }
    }

    /// Register a handler for a state
    pub fn register_handler(
        &mut self,
        state: DynamicState,
        handler: Box<dyn DynamicStateHandler + Send + Sync>,
    ) {
        self.handlers.insert(state, handler);
    }

    /// Register valid transitions from a state
    pub fn register_transitions(&mut self, from_state: DynamicState, to_states: Vec<DynamicState>) {
        self.valid_transitions.insert(from_state, to_states);
    }

    /// Set the context
    pub fn set_context(&mut self, context: DynamicStateContext) {
        self.context = context;
    }

    /// Get the context
    #[must_use]
    pub fn get_context(&self) -> &DynamicStateContext {
        &self.context
    }

    /// Get mutable context
    pub fn get_context_mut(&mut self) -> &mut DynamicStateContext {
        &mut self.context
    }

    /// Get current state
    #[must_use]
    pub fn get_current_state(&self) -> &DynamicState {
        &self.current_state
    }

    /// Check if a transition is valid
    #[must_use]
    pub fn is_valid_transition(&self, from: &DynamicState, to: &DynamicState) -> bool {
        if let Some(valid_to_states) = self.valid_transitions.get(from) {
            valid_to_states.contains(to)
        } else {
            // If no transitions are registered, allow all transitions
            true
        }
    }

    /// Run the dynamic state machine
    ///
    /// # Errors
    ///
    /// Returns an error if the state machine execution fails.
    pub async fn run(&mut self) -> Result<(), ConnectError> {
        println!("Starting dynamic TiDB connection state machine...");

        while self.current_state != states::completed()
            && !self.current_state.name().starts_with("error:")
        {
            if let Some(handler) = self.handlers.get(&self.current_state) {
                // Enter state
                let _next_state = handler.enter(&mut self.context).await?;

                // Execute state logic
                let next_state = handler.execute(&mut self.context).await?;

                // Validate transition
                if !self.is_valid_transition(&self.current_state, &next_state) {
                    return Err(ConnectError::StateMachine(format!(
                        "Invalid transition from {} to {}",
                        self.current_state, next_state
                    )));
                }

                // Exit current state
                handler.exit(&mut self.context).await?;

                // Update current state
                self.current_state = next_state;
            } else {
                return Err(ConnectError::StateMachine(format!(
                    "No handler registered for state: {}",
                    self.current_state
                )));
            }
        }

        println!("Dynamic state machine completed.");
        Ok(())
    }
}

/// Helper macro to create dynamic states easily
#[macro_export]
macro_rules! dynamic_state {
    ($name:expr) => {
        DynamicState::new($name)
    };
    ($name:expr, $display:expr) => {
        DynamicState::with_display_name($name, $display)
    };
}

/// Helper macro to register multiple transitions
#[macro_export]
macro_rules! register_transitions {
    ($machine:expr, $from:expr, [$($to:expr),*]) => {
        $machine.register_transitions($from, vec![$($to),*]);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::Result;

    struct TestHandler {
        next_state: DynamicState,
    }

    #[async_trait::async_trait]
    impl DynamicStateHandler for TestHandler {
        async fn enter(&self, _context: &mut DynamicStateContext) -> Result<DynamicState> {
            Ok(self.next_state.clone())
        }

        async fn execute(&self, _context: &mut DynamicStateContext) -> Result<DynamicState> {
            Ok(self.next_state.clone())
        }

        async fn exit(&self, _context: &mut DynamicStateContext) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_dynamic_state_machine() {
        let mut machine = DynamicStateMachine::new();

        // Register handlers
        machine.register_handler(
            states::initial(),
            Box::new(TestHandler {
                next_state: states::connecting(),
            }),
        );

        machine.register_handler(
            states::connecting(),
            Box::new(TestHandler {
                next_state: states::completed(),
            }),
        );

        // Register transitions
        machine.register_transitions(states::initial(), vec![states::connecting()]);
        machine.register_transitions(states::connecting(), vec![states::completed()]);

        // Run the machine
        let result = machine.run().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_dynamic_state_creation() {
        let state = dynamic_state!("custom_test_state", "Custom Test State");
        assert_eq!(state.name(), "custom_test_state");
        assert_eq!(state.display_name(), "Custom Test State");
    }
}

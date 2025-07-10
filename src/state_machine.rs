use crate::errors::{StateError, StateMachineError};
use mysql::PooledConn;
use std::any::Any;
use std::fmt;

/// Represents the different states in the TiDB connection process
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum State {
    Initial,
    ParsingConfig,
    Connecting,
    TestingConnection,
    VerifyingDatabase,
    GettingVersion,
    CheckingImportJobs,
    ShowingImportJobDetails,
    CreatingTable,
    PopulatingData,
    TestingIsolation,
    VerifyingResults,
    Completed,
    Error(String),
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::Initial => write!(f, "Initial"),
            State::ParsingConfig => write!(f, "Parsing Configuration"),
            State::Connecting => write!(f, "Connecting to TiDB"),
            State::TestingConnection => write!(f, "Testing Connection"),
            State::VerifyingDatabase => write!(f, "Verifying Database"),
            State::GettingVersion => write!(f, "Getting Server Version"),
            State::CheckingImportJobs => write!(f, "Checking Import Jobs"),
            State::ShowingImportJobDetails => write!(f, "Showing Import Job Details"),
            State::CreatingTable => write!(f, "Creating Test Table"),
            State::PopulatingData => write!(f, "Populating Test Data"),
            State::TestingIsolation => write!(f, "Testing Isolation"),
            State::VerifyingResults => write!(f, "Verifying Results"),
            State::Completed => write!(f, "Completed"),
            State::Error(msg) => write!(f, "Error: {msg}"),
        }
    }
}

/// Minimal context data that flows through the state machine
pub struct StateContext {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: Option<String>,
    pub connection: Option<PooledConn>,
    pub server_version: Option<String>,
    pub error_message: Option<String>,
    // Handler-specific context storage
    handler_contexts: std::collections::HashMap<State, Box<dyn Any + Send + Sync>>,
}

impl Default for StateContext {
    fn default() -> Self {
        Self::new()
    }
}

impl StateContext {
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
            handler_contexts: std::collections::HashMap::new(),
        }
    }

    pub fn set_error(&mut self, error: String) {
        self.error_message = Some(error);
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Store handler-specific context
    pub fn set_handler_context<T: Any + Send + Sync>(&mut self, state: State, context: T) {
        self.handler_contexts.insert(state, Box::new(context));
    }

    /// Retrieve handler-specific context
    pub fn get_handler_context<T: Any + Send + Sync>(&self, state: &State) -> Option<&T> {
        self.handler_contexts
            .get(state)
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Retrieve mutable handler-specific context
    pub fn get_handler_context_mut<T: Any + Send + Sync>(
        &mut self,
        state: &State,
    ) -> Option<&mut T> {
        self.handler_contexts
            .get_mut(state)
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Remove handler-specific context
    pub fn remove_handler_context(&mut self, state: &State) {
        self.handler_contexts.remove(state);
    }

    /// Move handler context from one state to another
    pub fn move_handler_context<T: Any + Send + Sync + Clone>(
        &mut self,
        from_state: &State,
        to_state: State,
    ) -> Option<T> {
        if let Some(boxed) = self.handler_contexts.remove(from_state) {
            match boxed.downcast::<T>() {
                Ok(context) => {
                    let context = *context;
                    self.handler_contexts
                        .insert(to_state, Box::new(context.clone()));
                    Some(context)
                }
                Err(boxed) => {
                    // If downcast fails, put it back
                    self.handler_contexts.insert(from_state.clone(), boxed);
                    None
                }
            }
        } else {
            None
        }
    }
}

/// Trait for state handlers
#[async_trait::async_trait]
pub trait StateHandler {
    async fn enter(&self, context: &mut StateContext) -> Result<State, StateError>;
    async fn execute(&self, context: &mut StateContext) -> Result<State, StateError>;
    async fn exit(&self, context: &mut StateContext) -> Result<(), StateError>;
}

/// State machine that manages the flow between states
pub struct StateMachine {
    current_state: State,
    context: StateContext,
    handlers: std::collections::HashMap<State, Box<dyn StateHandler + Send + Sync>>,
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            current_state: State::Initial,
            context: StateContext::new(),
            handlers: std::collections::HashMap::new(),
        }
    }

    pub fn register_handler(&mut self, state: State, handler: Box<dyn StateHandler + Send + Sync>) {
        self.handlers.insert(state, handler);
    }

    pub fn set_context(&mut self, context: StateContext) {
        self.context = context;
    }

    pub fn get_context(&self) -> &StateContext {
        &self.context
    }

    pub fn get_context_mut(&mut self) -> &mut StateContext {
        &mut self.context
    }

    pub async fn run(&mut self) -> Result<(), StateError> {
        println!("Starting TiDB connection state machine...");

        while self.current_state != State::Completed
            && self.current_state != State::Error("".to_string())
        {
            if let Some(handler) = self.handlers.get(&self.current_state) {
                // Enter state
                let _next_state = handler.enter(&mut self.context).await?;

                // Execute state logic
                let next_state = handler.execute(&mut self.context).await?;

                // Exit current state
                handler.exit(&mut self.context).await?;

                // Transition to next state
                self.current_state = next_state;
            } else {
                return Err(StateMachineError::NoHandlerRegistered {
                    state: format!("{:?}", self.current_state),
                }
                .into());
            }
        }

        match &self.current_state {
            State::Completed => {
                println!("✓ State machine completed successfully!");
                Ok(())
            }
            State::Error(msg) => {
                eprintln!("✗ State machine failed: {msg}");
                Err(StateMachineError::HandlerError(msg.clone()).into())
            }
            _ => Err(StateMachineError::HandlerError("Unexpected final state".to_string()).into()),
        }
    }

    pub fn get_current_state(&self) -> &State {
        &self.current_state
    }
}

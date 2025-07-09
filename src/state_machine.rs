use std::fmt;
use mysql::PooledConn;

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
            State::Completed => write!(f, "Completed"),
            State::Error(msg) => write!(f, "Error: {}", msg),
        }
    }
}

/// Context data that flows through the state machine
pub struct StateContext {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: Option<String>,
    pub connection: Option<PooledConn>,
    pub server_version: Option<String>,
    pub error_message: Option<String>,
    pub active_import_jobs: Vec<String>, // Store job IDs of active import jobs
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
            active_import_jobs: Vec::new(),
        }
    }

    pub fn set_error(&mut self, error: String) {
        self.error_message = Some(error);
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
    }
}

/// Trait for state handlers
pub trait StateHandler {
    fn enter(&self, context: &mut StateContext) -> Result<State, Box<dyn std::error::Error>>;
    fn execute(&self, context: &mut StateContext) -> Result<State, Box<dyn std::error::Error>>;
    fn exit(&self, context: &mut StateContext) -> Result<(), Box<dyn std::error::Error>>;
}

/// State machine that manages the flow between states
pub struct StateMachine {
    current_state: State,
    context: StateContext,
    handlers: std::collections::HashMap<State, Box<dyn StateHandler>>,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            current_state: State::Initial,
            context: StateContext::new(),
            handlers: std::collections::HashMap::new(),
        }
    }

    pub fn register_handler(&mut self, state: State, handler: Box<dyn StateHandler>) {
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

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting TiDB connection state machine...");
        
        while self.current_state != State::Completed && self.current_state != State::Error("".to_string()) {
            if let Some(handler) = self.handlers.get(&self.current_state) {
                // Enter state
                let _next_state = handler.enter(&mut self.context)?;
                
                // Execute state logic
                let next_state = handler.execute(&mut self.context)?;
                
                // Exit current state
                handler.exit(&mut self.context)?;
                
                // Transition to next state
                self.current_state = next_state;
            } else {
                return Err(format!("No handler registered for state: {:?}", self.current_state).into());
            }
        }

        match &self.current_state {
            State::Completed => {
                println!("✓ State machine completed successfully!");
                Ok(())
            }
            State::Error(msg) => {
                eprintln!("✗ State machine failed: {}", msg);
                Err(msg.clone().into())
            }
            _ => Err("Unexpected final state".into()),
        }
    }

    pub fn get_current_state(&self) -> &State {
        &self.current_state
    }
} 
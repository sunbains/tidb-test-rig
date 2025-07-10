use crate::state_machine::{State, StateContext, StateHandler};
use crate::connection_manager::{ConnectionCoordinator, ConnectionInfo, CoordinationMessage, ConnectionStatus, ConnectionState};
use crate::errors::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// State machine for managing multiple connections
pub struct MultiConnectionStateMachine {
    coordinator: ConnectionCoordinator,
    state_machines: Vec<ConnectionStateMachine>,
    coordination_handle: Option<JoinHandle<()>>,
}

/// Individual state machine for a single connection
pub struct ConnectionStateMachine {
    connection_id: String,
    state_machine: crate::state_machine::StateMachine,
    coordinator_sender: mpsc::Sender<CoordinationMessage>,
}

impl MultiConnectionStateMachine {
    pub fn new(coordinator: ConnectionCoordinator) -> Self {
        Self {
            coordinator,
            state_machines: Vec::new(),
            coordination_handle: None,
        }
    }

    /// Add a new connection state machine
    pub fn add_connection(&mut self, connection_id: String, connection_info: ConnectionInfo) {
        // Add to coordinator
        self.coordinator.add_connection(connection_id.clone(), connection_info);
        
        // Create state machine for this connection
        let mut state_machine = crate::state_machine::StateMachine::new();
        let coordinator_sender = self.coordinator.get_sender();
        
        // Register handlers for this connection
        self.register_connection_handlers(&mut state_machine, &connection_id, coordinator_sender.clone());
        
        self.state_machines.push(ConnectionStateMachine {
            connection_id,
            state_machine,
            coordinator_sender,
        });
    }

    fn register_connection_handlers(
        &self,
        state_machine: &mut crate::state_machine::StateMachine,
        _connection_id: &str,
        _coordinator_sender: mpsc::Sender<CoordinationMessage>,
    ) {
        use crate::state_handlers::{InitialHandler, ParsingConfigHandler, ConnectingHandler, TestingConnectionHandler, VerifyingDatabaseHandler, GettingVersionHandler};

        state_machine.register_handler(State::Initial, Box::new(InitialHandler));
        state_machine.register_handler(State::ParsingConfig, Box::new(ParsingConfigHandler::new(
            "".to_string(), // Will be set from coordinator
            "".to_string(),
            "".to_string(),
            None,
        )));
        state_machine.register_handler(State::Connecting, Box::new(ConnectingHandler));
        state_machine.register_handler(State::TestingConnection, Box::new(TestingConnectionHandler));
        state_machine.register_handler(State::VerifyingDatabase, Box::new(VerifyingDatabaseHandler));
        state_machine.register_handler(State::GettingVersion, Box::new(GettingVersionHandler));
    }

    /// Start coordination processing
    pub fn start_coordination(&mut self) {
        let mut coordinator = ConnectionCoordinator::new(self.coordinator.get_shared_state().lock().unwrap().global_config.clone());
        let handle = tokio::spawn(async move {
            coordinator.process_messages().await;
        });
        self.coordination_handle = Some(handle);
    }

    /// Run all state machines concurrently
    pub async fn run_all(&mut self) -> Result<()> {
        println!("Starting {} connection state machines...", self.state_machines.len());
        
        // Start coordination
        self.start_coordination();
        
        // Run all state machines concurrently
        let mut handles = Vec::new();
        for mut state_machine in self.state_machines.drain(..) {
            let handle = tokio::spawn(async move {
                state_machine.state_machine.run().await
            });
            handles.push(handle);
        }
        
        // Wait for all to complete
        for handle in handles {
            match handle.await {
                Ok(Ok(_)) => println!("✓ State machine completed successfully"),
                Ok(Err(e)) => eprintln!("✗ State machine failed: {e}"),
                Err(e) => eprintln!("✗ State machine task failed: {e}"),
            }
        }
        
        // Stop coordination
        if let Some(handle) = self.coordination_handle.take() {
            handle.abort();
        }
        
        Ok(())
    }

    /// Get shared state
    pub fn get_shared_state(&self) -> Arc<std::sync::Mutex<crate::connection_manager::SharedState>> {
        self.coordinator.get_shared_state()
    }

    /// Check if all connections are ready
    pub fn all_connections_ready(&self) -> bool {
        self.coordinator.all_connections_ready()
    }

    /// Get all active import jobs across all connections
    pub fn get_active_import_jobs(&self) -> Vec<crate::import_job_monitor::ImportJobInfo> {
        self.coordinator.get_active_import_jobs()
    }
}

impl ConnectionStateMachine {
    /// Update connection status in coordinator
    pub async fn update_status(&self, status: ConnectionState, error_message: Option<String>) {
        let status_update = ConnectionStatus {
            connection_id: self.connection_id.clone(),
            host: "".to_string(), // Will be filled by coordinator
            port: 0,
            username: "".to_string(),
            status,
            last_activity: chrono::Utc::now(),
            error_message,
        };
        
        if let Err(e) = self.coordinator_sender.send(CoordinationMessage::UpdateConnectionStatus(status_update)).await {
            eprintln!("Failed to send status update: {e}");
        }
    }
}

/// Handler for coordinating multiple connections
pub struct CoordinationHandler {
    coordinator_sender: mpsc::Sender<CoordinationMessage>,
}

impl CoordinationHandler {
    pub fn new(coordinator_sender: mpsc::Sender<CoordinationMessage>) -> Self {
        Self { coordinator_sender }
    }
}

#[async_trait]
impl StateHandler for CoordinationHandler {
    async fn enter(&self, _context: &mut StateContext) -> Result<State> {
        println!("Starting coordination phase...");
        Ok(State::Initial)
    }

    async fn execute(&self, _context: &mut StateContext) -> Result<State> {
        // Broadcast that coordination is starting
        let event = crate::connection_manager::CoordinationEvent::AllConnectionsReady;
        if let Err(e) = self.coordinator_sender.send(CoordinationMessage::BroadcastEvent(event)).await {
            eprintln!("Failed to send coordination event: {e}");
        }
        
        Ok(State::Completed)
    }

    async fn exit(&self, _context: &mut StateContext) -> Result<()> {
        Ok(())
    }
} 
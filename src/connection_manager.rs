use crate::import_job_monitor;
use mysql::PooledConn;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Shared state that can be accessed by multiple state machines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedState {
    pub global_config: GlobalConfig,
    pub connection_status: HashMap<String, ConnectionStatus>,
    pub import_jobs: Vec<import_job_monitor::ImportJobInfo>,
    pub coordination_events: Vec<CoordinationEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub test_duration: u64,
    pub coordination_timeout: u64,
    pub max_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub connection_id: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub status: ConnectionState,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Testing,
    Monitoring,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationEvent {
    ConnectionEstablished {
        connection_id: String,
    },
    ConnectionFailed {
        connection_id: String,
        error: String,
    },
    ImportJobStarted {
        job_id: String,
        connection_id: String,
    },
    ImportJobCompleted {
        job_id: String,
        connection_id: String,
    },
    AllConnectionsReady,
    TestCompleted,
}

/// Message types for inter-state-machine communication
#[derive(Debug)]
pub enum CoordinationMessage {
    UpdateConnectionStatus(ConnectionStatus),
    AddImportJob(import_job_monitor::ImportJobInfo),
    UpdateImportJob(String, import_job_monitor::ImportJobInfo),
    BroadcastEvent(CoordinationEvent),
    RequestGlobalState,
    ResponseGlobalState(SharedState),
}

/// Manager for coordinating multiple state machines
pub struct ConnectionCoordinator {
    shared_state: Arc<Mutex<SharedState>>,
    connections: HashMap<String, ConnectionInfo>,
    tx: mpsc::Sender<CoordinationMessage>,
    rx: mpsc::Receiver<CoordinationMessage>,
}

pub struct ConnectionInfo {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: Option<String>,
    pub connection: Option<PooledConn>,
}

impl ConnectionCoordinator {
    pub fn new(config: GlobalConfig) -> Self {
        let (tx, rx) = mpsc::channel(100);
        let shared_state = Arc::new(Mutex::new(SharedState {
            global_config: config,
            connection_status: HashMap::new(),
            import_jobs: Vec::new(),
            coordination_events: Vec::new(),
        }));

        Self {
            shared_state,
            connections: HashMap::new(),
            tx,
            rx,
        }
    }

    /// Add a new connection to be managed
    pub fn add_connection(&mut self, connection_id: String, info: ConnectionInfo) {
        let host = info.host.clone();
        let port = info.port;
        let username = info.username.clone();

        self.connections.insert(connection_id.clone(), info);

        // Initialize connection status
        if let Ok(mut state) = self.shared_state.lock() {
            state.connection_status.insert(
                connection_id.clone(),
                ConnectionStatus {
                    connection_id,
                    host,
                    port,
                    username,
                    status: ConnectionState::Disconnected,
                    last_activity: chrono::Utc::now(),
                    error_message: None,
                },
            );
        }
    }

    /// Get a connection by ID
    pub fn get_connection(&mut self, connection_id: &str) -> Option<&mut ConnectionInfo> {
        self.connections.get_mut(connection_id)
    }

    /// Get shared state reference
    pub fn get_shared_state(&self) -> Arc<Mutex<SharedState>> {
        Arc::clone(&self.shared_state)
    }

    /// Get sender for coordination messages
    pub fn get_sender(&self) -> mpsc::Sender<CoordinationMessage> {
        self.tx.clone()
    }

    /// Process coordination messages
    pub async fn process_messages(&mut self) {
        while let Some(message) = self.rx.recv().await {
            match message {
                CoordinationMessage::UpdateConnectionStatus(status) => {
                    if let Ok(mut state) = self.shared_state.lock() {
                        state
                            .connection_status
                            .insert(status.connection_id.clone(), status);
                    }
                }
                CoordinationMessage::AddImportJob(job) => {
                    if let Ok(mut state) = self.shared_state.lock() {
                        state.import_jobs.push(job);
                    }
                }
                CoordinationMessage::UpdateImportJob(job_id, updated_job) => {
                    if let Ok(mut state) = self.shared_state.lock()
                        && let Some(job) = state.import_jobs.iter_mut().find(|j| j.job_id == job_id)
                    {
                        *job = updated_job;
                    }
                }
                CoordinationMessage::BroadcastEvent(event) => {
                    if let Ok(mut state) = self.shared_state.lock() {
                        state.coordination_events.push(event);
                    }
                }
                _ => {}
            }
        }
    }

    /// Check if all connections are ready
    pub fn all_connections_ready(&self) -> bool {
        if let Ok(state) = self.shared_state.lock() {
            state.connection_status.values().all(|status| {
                matches!(
                    status.status,
                    ConnectionState::Connected
                        | ConnectionState::Testing
                        | ConnectionState::Monitoring
                )
            })
        } else {
            false
        }
    }

    /// Get all active import jobs across all connections
    pub fn get_active_import_jobs(&self) -> Vec<import_job_monitor::ImportJobInfo> {
        if let Ok(state) = self.shared_state.lock() {
            state
                .import_jobs
                .iter()
                .filter(|job| job.end_time.is_none())
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            global_config: GlobalConfig {
                test_duration: 60,
                coordination_timeout: 30,
                max_connections: 5,
            },
            connection_status: HashMap::new(),
            import_jobs: Vec::new(),
            coordination_events: Vec::new(),
        }
    }
}

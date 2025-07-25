//! # State Handlers
//!
//! Built-in state handler implementations for common TiDB operations.
//! Provides handlers for connection, testing, verification, and monitoring workflows.
//!
//! Note: For extensible state handling, use the dynamic state machine system.
//! The core StateMachine now only supports Initial, Completed, and Error states.

use crate::connection::{create_connection_pool, parse_connection_string};
use crate::errors::Result;
use crate::state_machine::{State, StateContext, StateHandler};
use async_trait::async_trait;
use mysql::prelude::*;
use mysql::{Error, Row};
use tracing::{debug, error, info};

/// Handler for the initial state
pub struct InitialHandler;

#[async_trait]
impl StateHandler for InitialHandler {
    async fn enter(&self, _context: &mut StateContext) -> Result<State> {
        info!("Starting TiDB connection test...");
        println!("Starting TiDB connection test...");
        Ok(State::Initial)
    }

    async fn execute(&self, _context: &mut StateContext) -> Result<State> {
        // For extensible workflows, use DynamicStateMachine
        // This core handler just completes immediately
        Ok(State::Completed)
    }

    async fn exit(&self, _context: &mut StateContext) -> Result<()> {
        Ok(())
    }
}

/// Handler for parsing configuration
pub struct ParsingConfigHandler {
    host: String,
    username: String,
    password: String,
    database: Option<String>,
}

impl ParsingConfigHandler {
    #[must_use]
    pub fn new(host: String, username: String, password: String, database: Option<String>) -> Self {
        Self {
            host,
            username,
            password,
            database,
        }
    }
}

#[async_trait]
impl StateHandler for ParsingConfigHandler {
    async fn enter(&self, _context: &mut StateContext) -> Result<State> {
        println!("Parsing connection configuration...");
        Ok(State::Initial)
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State> {
        // Parse host and port from connection string
        let (host, port) = parse_connection_string(&self.host)?;

        // Store configuration in context
        context.host = host;
        context.port = port;
        context.username.clone_from(&self.username);
        context.password.clone_from(&self.password);
        context.database.clone_from(&self.database);

        info!("Configuration parsed: {}:{}", context.host, context.port);
        println!("✓ Configuration parsed: {}:{}", context.host, context.port);

        // For extensible workflows, use DynamicStateMachine
        // This handler completes the basic workflow
        Ok(State::Completed)
    }

    async fn exit(&self, _context: &mut StateContext) -> Result<()> {
        Ok(())
    }
}

/// Handler for establishing connection
pub struct ConnectingHandler;

#[async_trait]
impl StateHandler for ConnectingHandler {
    async fn enter(&self, _context: &mut StateContext) -> Result<State> {
        println!("Establishing connection to TiDB...");
        Ok(State::Initial)
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State> {
        info!(
            "Attempting connection to {}:{} as user {}",
            context.host, context.port, context.username
        );

        // Create connection pool
        let pool = create_connection_pool(
            &context.host,
            context.port,
            &context.username,
            &context.password,
            context.database.as_deref(),
        )?;

        // Get a connection from the pool
        let conn = pool.get_conn()?;
        context.connection = Some(conn);

        info!(
            "Connection established successfully to {}:{}",
            context.host, context.port
        );
        println!("✓ Connection established successfully");

        // For extensible workflows, use DynamicStateMachine
        // This handler completes the basic workflow
        Ok(State::Completed)
    }

    async fn exit(&self, _context: &mut StateContext) -> Result<()> {
        Ok(())
    }
}

/// Handler for testing the connection
pub struct TestingConnectionHandler;

#[async_trait]
impl StateHandler for TestingConnectionHandler {
    async fn enter(&self, _context: &mut StateContext) -> Result<State> {
        println!("Testing connection...");
        Ok(State::Initial)
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State> {
        if let Some(ref mut conn) = context.connection {
            debug!("Testing connection with SELECT 1");
            // Simple ping test
            let result: std::result::Result<Vec<Row>, Error> = conn.exec("SELECT 1", ());
            match result {
                Ok(_) => {
                    info!("Connection test passed");
                    println!("✓ Connection test passed");
                    Ok(State::Completed)
                }
                Err(e) => {
                    error!("Connection test failed: {}", e);
                    context.set_error(format!("Connection test failed: {e}"));
                    Err(format!("Connection test failed: {e}").into())
                }
            }
        } else {
            let error_msg = "No connection available for testing";
            error!("{}", error_msg);
            context.set_error(error_msg.to_string());
            Err(error_msg.into())
        }
    }

    async fn exit(&self, _context: &mut StateContext) -> Result<()> {
        Ok(())
    }
}

/// Handler for verifying database
pub struct VerifyingDatabaseHandler;

#[async_trait]
impl StateHandler for VerifyingDatabaseHandler {
    async fn enter(&self, _context: &mut StateContext) -> Result<State> {
        println!("Verifying database...");
        Ok(State::Initial)
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State> {
        if let Some(ref mut conn) = context.connection {
            if let Some(ref db_name) = context.database {
                // Test if we can access the specified database
                let query = format!("USE `{db_name}`");
                match conn.query_drop(query) {
                    Ok(()) => {
                        println!("✓ Database '{db_name}' verified");
                        Ok(State::Completed)
                    }
                    Err(e) => {
                        context.set_error(format!("Database verification failed: {e}"));
                        Err(format!("Database verification failed: {e}").into())
                    }
                }
            } else {
                // No specific database specified, just proceed
                println!("✓ No specific database specified, proceeding...");
                Ok(State::Completed)
            }
        } else {
            let error_msg = "No connection available for database verification";
            context.set_error(error_msg.to_string());
            Err(error_msg.into())
        }
    }

    async fn exit(&self, _context: &mut StateContext) -> Result<()> {
        Ok(())
    }
}

/// Handler for getting server version
pub struct GettingVersionHandler;

#[async_trait]
impl StateHandler for GettingVersionHandler {
    async fn enter(&self, _context: &mut StateContext) -> Result<State> {
        println!("Getting server version...");
        Ok(State::Initial)
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State> {
        if let Some(ref mut conn) = context.connection {
            let version_query = "SELECT VERSION()";
            match conn.query_first::<String, _>(version_query) {
                Ok(Some(version)) => {
                    context.server_version = Some(version.clone());
                    info!("Server version: {}", version);
                    println!("✓ Server version: {version}");
                    Ok(State::Completed)
                }
                Ok(None) => {
                    let error_msg = "No version returned from server";
                    context.set_error(error_msg.to_string());
                    Err(error_msg.into())
                }
                Err(e) => {
                    let error_msg = format!("Failed to get server version: {e}");
                    context.set_error(error_msg.clone());
                    Err(error_msg.into())
                }
            }
        } else {
            let error_msg = "No connection available for getting version";
            context.set_error(error_msg.to_string());
            Err(error_msg.into())
        }
    }

    async fn exit(&self, _context: &mut StateContext) -> Result<()> {
        Ok(())
    }
}

/// Handler that transitions to a specified next state
pub struct NextStateVersionHandler {
    pub next_state: State,
}

impl NextStateVersionHandler {
    #[must_use]
    pub fn new(next_state: State) -> Self {
        Self { next_state }
    }
}

#[async_trait]
impl StateHandler for NextStateVersionHandler {
    async fn enter(&self, _context: &mut StateContext) -> Result<State> {
        Ok(State::Initial)
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State> {
        if let Some(ref mut conn) = context.connection {
            let version_query = "SELECT VERSION()";
            match conn.query_first::<String, _>(version_query) {
                Ok(Some(version)) => {
                    context.server_version = Some(version.clone());
                    info!("Server version: {}", version);
                    println!("✓ Server version: {version}");
                    Ok(self.next_state.clone())
                }
                Ok(None) => {
                    let error_msg = "No version returned from server";
                    context.set_error(error_msg.to_string());
                    Err(error_msg.into())
                }
                Err(e) => {
                    let error_msg = format!("Failed to get server version: {e}");
                    context.set_error(error_msg.clone());
                    Err(error_msg.into())
                }
            }
        } else {
            let error_msg = "No connection available for getting version";
            context.set_error(error_msg.to_string());
            Err(error_msg.into())
        }
    }

    async fn exit(&self, _context: &mut StateContext) -> Result<()> {
        Ok(())
    }
}

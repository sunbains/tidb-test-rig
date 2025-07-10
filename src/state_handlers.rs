use crate::state_machine::{State, StateContext, StateHandler};
use crate::connection::{parse_connection_string, create_connection_pool};
use mysql::prelude::*;
use mysql::*;
use async_trait::async_trait;

/// Handler for the initial state
pub struct InitialHandler;

#[async_trait]
impl StateHandler for InitialHandler {
    async fn enter(&self, _context: &mut StateContext) -> Result<State, Box<dyn std::error::Error + Send + Sync>> {
        println!("Starting TiDB connection test...");
        Ok(State::Initial)
    }

    async fn execute(&self, _context: &mut StateContext) -> Result<State, Box<dyn std::error::Error + Send + Sync>> {
        Ok(State::ParsingConfig)
    }

    async fn exit(&self, _context: &mut StateContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    async fn enter(&self, _context: &mut StateContext) -> Result<State, crate::state_machine::StateError> {
        println!("Parsing connection configuration...");
        Ok(State::ParsingConfig)
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State, crate::state_machine::StateError> {
        // Parse host and port from connection string
        let (host, port) = parse_connection_string(&self.host)?;
        
        // Store configuration in context
        context.host = host;
        context.port = port;
        context.username = self.username.clone();
        context.password = self.password.clone();
        context.database = self.database.clone();
        
        println!("✓ Configuration parsed: {}:{}", context.host, context.port);
        Ok(State::Connecting)
    }

    async fn exit(&self, _context: &mut StateContext) -> Result<(), crate::state_machine::StateError> {
        Ok(())
    }
}

/// Handler for establishing connection
pub struct ConnectingHandler;

#[async_trait]
impl StateHandler for ConnectingHandler {
    async fn enter(&self, _context: &mut StateContext) -> Result<State, crate::state_machine::StateError> {
        println!("Establishing connection to TiDB...");
        Ok(State::Connecting)
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State, crate::state_machine::StateError> {
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
        
        println!("✓ Connection established successfully");
        Ok(State::TestingConnection)
    }

    async fn exit(&self, _context: &mut StateContext) -> Result<(), crate::state_machine::StateError> {
        Ok(())
    }
}

/// Handler for testing the connection
pub struct TestingConnectionHandler;

#[async_trait]
impl StateHandler for TestingConnectionHandler {
    async fn enter(&self, _context: &mut StateContext) -> Result<State, crate::state_machine::StateError> {
        println!("Testing connection...");
        Ok(State::TestingConnection)
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State, crate::state_machine::StateError> {
        if let Some(ref mut conn) = context.connection {
            // Simple ping test
            let result: Result<Vec<Row>, Error> = conn.exec("SELECT 1", ());
            match result {
                Ok(_) => {
                    println!("✓ Connection test passed");
                    Ok(State::VerifyingDatabase)
                }
                Err(e) => {
                    context.set_error(format!("Connection test failed: {e}"));
                    Err(format!("Connection test failed: {e}").into())
                }
            }
        } else {
            let error_msg = "No connection available for testing";
            context.set_error(error_msg.to_string());
            Err(error_msg.into())
        }
    }

    async fn exit(&self, _context: &mut StateContext) -> Result<(), crate::state_machine::StateError> {
        Ok(())
    }
}

/// Handler for verifying database
pub struct VerifyingDatabaseHandler;

#[async_trait]
impl StateHandler for VerifyingDatabaseHandler {
    async fn enter(&self, _context: &mut StateContext) -> Result<State, crate::state_machine::StateError> {
        println!("Verifying database...");
        Ok(State::VerifyingDatabase)
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State, crate::state_machine::StateError> {
        if let Some(ref mut conn) = context.connection {
            if let Some(ref db_name) = context.database {
                // Test if we can access the specified database
                let query = format!("USE `{db_name}`");
                match conn.query_drop(query) {
                    Ok(_) => {
                        println!("✓ Database '{db_name}' verified");
                        Ok(State::GettingVersion)
                    }
                    Err(e) => {
                        context.set_error(format!("Database verification failed: {e}"));
                        Err(format!("Database verification failed: {e}").into())
                    }
                }
            } else {
                // No specific database specified, just proceed
                println!("✓ No specific database specified, proceeding...");
                Ok(State::GettingVersion)
            }
        } else {
            let error_msg = "No connection available for database verification";
            context.set_error(error_msg.to_string());
            Err(error_msg.into())
        }
    }

    async fn exit(&self, _context: &mut StateContext) -> Result<(), crate::state_machine::StateError> {
        Ok(())
    }
}

/// Handler for getting server version
pub struct GettingVersionHandler;

#[async_trait]
impl StateHandler for GettingVersionHandler {
    async fn enter(&self, _context: &mut StateContext) -> Result<State, crate::state_machine::StateError> {
        println!("Getting server version...");
        Ok(State::GettingVersion)
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State, crate::state_machine::StateError> {
        if let Some(ref mut conn) = context.connection {
            let query = "SELECT VERSION() as version";
            let result: Result<Vec<Row>, Error> = conn.exec(query, ());
            
            match result {
                Ok(rows) => {
                    if let Some(row) = rows.first() {
                        if let Some(version) = row.get::<String, _>("version") {
                            context.server_version = Some(version.clone());
                            println!("✓ Server version: {version}");
                            Ok(State::CheckingImportJobs)
                        } else {
                            context.set_error("Could not extract version from result".to_string());
                            Err("Could not extract version from result".into())
                        }
                    } else {
                        context.set_error("No version information returned".to_string());
                        Err("No version information returned".into())
                    }
                }
                Err(e) => {
                    context.set_error(format!("Failed to get server version: {e}"));
                    Err(format!("Failed to get server version: {e}").into())
                }
            }
        } else {
            let error_msg = "No connection available for getting version";
            context.set_error(error_msg.to_string());
            Err(error_msg.into())
        }
    }

    async fn exit(&self, _context: &mut StateContext) -> Result<(), crate::state_machine::StateError> {
        Ok(())
    }
} 
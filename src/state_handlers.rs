use crate::state_machine::{State, StateContext, StateHandler};
use crate::connection::{parse_host_port, create_connection, test_connection, verify_database_exists, get_server_version};

/// Handler for the initial state
pub struct InitialHandler;

impl StateHandler for InitialHandler {
    fn enter(&self, _context: &mut StateContext) -> Result<State, Box<dyn std::error::Error>> {
        println!("Initializing TiDB connection test...");
        Ok(State::Initial)
    }

    fn execute(&self, _context: &mut StateContext) -> Result<State, Box<dyn std::error::Error>> {
        // Initial state just transitions to parsing configuration
        Ok(State::ParsingConfig)
    }

    fn exit(&self, _context: &mut StateContext) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// Handler for parsing configuration from command line arguments
pub struct ParsingConfigHandler {
    host_port: String,
    username: String,
    password: String,
    database: Option<String>,
}

impl ParsingConfigHandler {
    pub fn new(host_port: String, username: String, password: String, database: Option<String>) -> Self {
        Self {
            host_port,
            username,
            password,
            database,
        }
    }
}

impl StateHandler for ParsingConfigHandler {
    fn enter(&self, _context: &mut StateContext) -> Result<State, Box<dyn std::error::Error>> {
        println!("Parsing configuration...");
        Ok(State::ParsingConfig)
    }

    fn execute(&self, context: &mut StateContext) -> Result<State, Box<dyn std::error::Error>> {
        // Parse host:port
        let (host, port) = parse_host_port(&self.host_port)?;
        context.host = host;
        context.port = port;

        // Set username and password
        context.username = self.username.clone();
        context.password = self.password.clone();

        // Set database if provided
        context.database = self.database.clone();

        println!("✓ Configuration parsed successfully");
        Ok(State::Connecting)
    }

    fn exit(&self, _context: &mut StateContext) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// Handler for establishing connection to TiDB
pub struct ConnectingHandler;

impl StateHandler for ConnectingHandler {
    fn enter(&self, context: &mut StateContext) -> Result<State, Box<dyn std::error::Error>> {
        println!("Connecting to TiDB at {}:{} as user '{}'", 
                context.host, context.port, context.username);
        Ok(State::Connecting)
    }

    fn execute(&self, context: &mut StateContext) -> Result<State, Box<dyn std::error::Error>> {
        let conn = create_connection(
            &context.host,
            context.port,
            &context.username,
            &context.password,
            context.database.as_deref()
        )?;

        context.connection = Some(conn);
        println!("✓ Successfully connected to TiDB");
        Ok(State::TestingConnection)
    }

    fn exit(&self, _context: &mut StateContext) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// Handler for testing the connection
pub struct TestingConnectionHandler;

impl StateHandler for TestingConnectionHandler {
    fn enter(&self, _context: &mut StateContext) -> Result<State, Box<dyn std::error::Error>> {
        println!("Testing connection...");
        Ok(State::TestingConnection)
    }

    fn execute(&self, context: &mut StateContext) -> Result<State, Box<dyn std::error::Error>> {
        if let Some(ref mut conn) = context.connection {
            test_connection(conn)?;
            println!("✓ Connection test successful");
        } else {
            return Err("No connection available for testing".into());
        }

        // Determine next state based on whether database was specified
        if context.database.is_some() {
            Ok(State::VerifyingDatabase)
        } else {
            Ok(State::GettingVersion)
        }
    }

    fn exit(&self, _context: &mut StateContext) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// Handler for verifying database existence
pub struct VerifyingDatabaseHandler;

impl StateHandler for VerifyingDatabaseHandler {
    fn enter(&self, context: &mut StateContext) -> Result<State, Box<dyn std::error::Error>> {
        if let Some(ref db_name) = context.database {
            println!("Verifying database '{}' exists...", db_name);
        }
        Ok(State::VerifyingDatabase)
    }

    fn execute(&self, context: &mut StateContext) -> Result<State, Box<dyn std::error::Error>> {
        if let Some(ref db_name) = context.database {
            if let Some(ref mut conn) = context.connection {
                let exists = verify_database_exists(conn, db_name)?;
                if exists {
                    println!("✓ Database '{}' exists and is accessible", db_name);
                } else {
                    context.set_error(format!("Database '{}' does not exist", db_name));
                    return Ok(State::Error(context.error_message.clone().unwrap()));
                }
            } else {
                return Err("No connection available for database verification".into());
            }
        }

        Ok(State::GettingVersion)
    }

    fn exit(&self, _context: &mut StateContext) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// Handler for getting server version
pub struct GettingVersionHandler;

impl StateHandler for GettingVersionHandler {
    fn enter(&self, _context: &mut StateContext) -> Result<State, Box<dyn std::error::Error>> {
        println!("Getting server version...");
        Ok(State::GettingVersion)
    }

    fn execute(&self, context: &mut StateContext) -> Result<State, Box<dyn std::error::Error>> {
        if let Some(ref mut conn) = context.connection {
            let version = get_server_version(conn)?;
            context.server_version = version.clone();
            
            match version {
                Some(ver) => println!("✓ TiDB version: {}", ver),
                None => println!("! Could not retrieve TiDB version"),
            }
        } else {
            return Err("No connection available for version retrieval".into());
        }

        Ok(State::Completed)
    }

    fn exit(&self, _context: &mut StateContext) -> Result<(), Box<dyn std::error::Error>> {
        println!("✓ Disconnecting from TiDB");
        // Connection will be dropped when context goes out of scope
        Ok(())
    }
} 
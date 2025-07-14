use async_trait::async_trait;
use clap::Parser;
use mysql::prelude::*;
use test_rig::{
    CommonArgs, State, StateContext, StateHandler, StateMachine, print_error_and_exit,
    print_success, print_test_header,
};

#[cfg(feature = "python_plugins")]
use test_rig::load_python_handlers;

#[derive(Parser)]
#[command(name = "python-demo")]
#[command(about = "Demo of Python handlers with test_rig framework")]
struct Args {
    #[command(flatten)]
    common: CommonArgs,

    /// Python module path containing handlers
    #[arg(long, default_value = "examples.python_handlers")]
    python_module: String,
}

impl Args {
    pub fn init_logging(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.common.init_logging()
    }

    pub fn get_connection_info(&self) -> test_rig::cli::ConnInfoResult {
        self.common.get_connection_info()
    }
}

// Adapter for InitialHandler to StateHandler
struct InitialHandlerAdapter;
#[async_trait]
impl StateHandler for InitialHandlerAdapter {
    async fn enter(&self, _context: &mut StateContext) -> test_rig::Result<State> {
        Ok(State::Initial)
    }
    async fn execute(&self, _context: &mut StateContext) -> test_rig::Result<State> {
        Ok(State::ParsingConfig)
    }
    async fn exit(&self, _context: &mut StateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

// Adapter for ParsingConfigHandler to StateHandler
struct ParsingConfigHandlerAdapter {
    host: String,
    user: String,
    password: String,
    database: Option<String>,
}
#[async_trait]
impl StateHandler for ParsingConfigHandlerAdapter {
    async fn enter(&self, _context: &mut StateContext) -> test_rig::Result<State> {
        Ok(State::ParsingConfig)
    }
    async fn execute(&self, context: &mut StateContext) -> test_rig::Result<State> {
        let (host, port) = test_rig::connection::parse_connection_string(&self.host)?;
        context.host = host;
        context.port = port;
        context.username.clone_from(&self.user);
        context.password.clone_from(&self.password);
        context.database.clone_from(&self.database);
        Ok(State::Connecting)
    }
    async fn exit(&self, _context: &mut StateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

// Adapter for ConnectingHandler to StateHandler
struct ConnectingHandlerAdapter;
#[async_trait]
impl StateHandler for ConnectingHandlerAdapter {
    async fn enter(&self, _context: &mut StateContext) -> test_rig::Result<State> {
        Ok(State::Connecting)
    }
    async fn execute(&self, context: &mut StateContext) -> test_rig::Result<State> {
        let pool = test_rig::connection::create_connection_pool(
            &context.host,
            context.port,
            &context.username,
            &context.password,
            context.database.as_deref(),
        )?;
        let conn = pool.get_conn()?;
        context.connection = Some(conn);
        Ok(State::TestingConnection)
    }
    async fn exit(&self, _context: &mut StateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

// Adapter for TestingConnectionHandler to StateHandler
struct TestingConnectionHandlerAdapter;
#[async_trait]
impl StateHandler for TestingConnectionHandlerAdapter {
    async fn enter(&self, _context: &mut StateContext) -> test_rig::Result<State> {
        Ok(State::TestingConnection)
    }
    async fn execute(&self, context: &mut StateContext) -> test_rig::Result<State> {
        if let Some(ref mut conn) = context.connection {
            let result: std::result::Result<Vec<mysql::Row>, mysql::Error> =
                conn.exec("SELECT 1", ());
            match result {
                Ok(_) => Ok(State::VerifyingDatabase),
                Err(e) => Err(format!("Connection test failed: {e}").into()),
            }
        } else {
            Err("No connection available for testing".into())
        }
    }
    async fn exit(&self, _context: &mut StateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

// Adapter for VerifyingDatabaseHandler to StateHandler
struct VerifyingDatabaseHandlerAdapter;
#[async_trait]
impl StateHandler for VerifyingDatabaseHandlerAdapter {
    async fn enter(&self, _context: &mut StateContext) -> test_rig::Result<State> {
        Ok(State::VerifyingDatabase)
    }
    async fn execute(&self, context: &mut StateContext) -> test_rig::Result<State> {
        if let Some(ref mut conn) = context.connection {
            if let Some(ref db_name) = context.database {
                let query = format!("USE `{db_name}`");
                match conn.query_drop(query) {
                    Ok(()) => Ok(State::GettingVersion),
                    Err(e) => Err(format!("Database verification failed: {e}").into()),
                }
            } else {
                Ok(State::GettingVersion)
            }
        } else {
            Err("No connection available for database verification".into())
        }
    }
    async fn exit(&self, _context: &mut StateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

// Adapter for GettingVersionHandler to StateHandler
struct GettingVersionHandlerAdapter;
#[async_trait]
impl StateHandler for GettingVersionHandlerAdapter {
    async fn enter(&self, _context: &mut StateContext) -> test_rig::Result<State> {
        Ok(State::GettingVersion)
    }
    async fn execute(&self, context: &mut StateContext) -> test_rig::Result<State> {
        if let Some(ref mut conn) = context.connection {
            let version_query = "SELECT VERSION()";
            match conn.query_first::<String, _>(version_query) {
                Ok(Some(version)) => {
                    context.server_version = Some(version.clone());
                    Ok(State::Completed)
                }
                Ok(None) => Err("No version returned from server".into()),
                Err(e) => Err(format!("Failed to get server version: {e}").into()),
            }
        } else {
            Err("No connection available for getting version".into())
        }
    }
    async fn exit(&self, _context: &mut StateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_test_header("Python Handlers Demo");

    let args = Args::parse();
    args.init_logging().expect("Failed to initialize logging");

    // Get connection info
    let (host, user, password, database) = args
        .get_connection_info()
        .expect("Failed to get connection info");

    println!("Python Module: {}", args.python_module);
    println!("Connection Info:");
    println!("  Host: {host}");
    println!("  User: {user}");
    println!("  Database: {database:?}");

    // Create and configure the state machine
    let mut machine = StateMachine::new();
    machine.register_handler(State::Initial, Box::new(InitialHandlerAdapter));
    machine.register_handler(
        State::ParsingConfig,
        Box::new(ParsingConfigHandlerAdapter {
            host,
            user,
            password,
            database,
        }),
    );
    machine.register_handler(State::Connecting, Box::new(ConnectingHandlerAdapter));
    machine.register_handler(
        State::TestingConnection,
        Box::new(TestingConnectionHandlerAdapter),
    );
    machine.register_handler(
        State::VerifyingDatabase,
        Box::new(VerifyingDatabaseHandlerAdapter),
    );
    machine.register_handler(
        State::GettingVersion,
        Box::new(GettingVersionHandlerAdapter),
    );

    // Load Python handlers (if enabled)
    #[cfg(feature = "python_plugins")]
    {
        println!(
            "\nLoading Python handlers from module: {}",
            args.python_module
        );
        match load_python_handlers(&mut machine, &args.python_module) {
            Ok(_) => println!("✓ Python handlers loaded successfully"),
            Err(e) => {
                eprintln!("✗ Failed to load Python handlers: {}", e);
                print_error_and_exit("Python handler loading failed", &e);
            }
        }
    }
    #[cfg(not(feature = "python_plugins"))]
    {
        println!("Python plugins are not enabled. Skipping Python handler loading.");
    }

    // Run the state machine
    println!("\nStarting state machine with mixed Rust and Python handlers...");
    match machine.run().await {
        Ok(()) => {
            println!("\n✓ State machine completed successfully");
            print_success("Python handlers demo completed!");
        }
        Err(e) => {
            eprintln!("\n✗ State machine failed: {e}");
            print_error_and_exit("State machine execution failed", &e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from([
            "test-bin",
            "--python-module",
            "test.handlers",
            "-H",
            "localhost:4000",
            "-u",
            "testuser",
        ]);
        assert_eq!(args.python_module, "test.handlers");
        assert_eq!(args.common.host, "localhost:4000");
        assert_eq!(args.common.user, "testuser");
    }

    #[test]
    fn test_args_defaults() {
        let args = Args::parse_from(["test-bin"]);
        assert_eq!(args.python_module, "examples.python_handlers");
        assert_eq!(args.common.host, "localhost:4000");
        assert_eq!(args.common.user, "root");
    }
}

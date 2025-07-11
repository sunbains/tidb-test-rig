use clap::Parser;
use test_rig::{
    CommonArgs, print_error_and_exit, print_success, print_test_header,
    dynamic_state, register_transitions, DynamicState, DynamicStateContext, DynamicStateHandler, DynamicStateMachine,
};
use async_trait::async_trait;
use mysql::prelude::*;

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

// Define custom states for the workflow
mod demo_states {
    // Re-export common states
    pub use test_rig::common_states::{
        parsing_config, connecting, testing_connection, verifying_database, getting_version, completed,
    };
}

// Adapter for InitialHandler to DynamicStateHandler
struct InitialHandlerAdapter;
#[async_trait]
impl DynamicStateHandler for InitialHandlerAdapter {
    async fn enter(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        Ok(dynamic_state!("initial", "Initial"))
    }
    async fn execute(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        Ok(demo_states::parsing_config())
    }
    async fn exit(&self, _context: &mut DynamicStateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

// Adapter for ParsingConfigHandler to DynamicStateHandler
struct ParsingConfigHandlerAdapter {
    host: String,
    user: String,
    password: String,
    database: Option<String>,
}
#[async_trait]
impl DynamicStateHandler for ParsingConfigHandlerAdapter {
    async fn enter(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        Ok(demo_states::parsing_config())
    }
    async fn execute(&self, context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        let (host, port) = test_rig::connection::parse_connection_string(&self.host)?;
        context.host = host;
        context.port = port;
        context.username = self.user.clone();
        context.password = self.password.clone();
        context.database = self.database.clone();
        Ok(demo_states::connecting())
    }
    async fn exit(&self, _context: &mut DynamicStateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

// Adapter for ConnectingHandler to DynamicStateHandler
struct ConnectingHandlerAdapter;
#[async_trait]
impl DynamicStateHandler for ConnectingHandlerAdapter {
    async fn enter(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        Ok(demo_states::connecting())
    }
    async fn execute(&self, context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        let pool = test_rig::connection::create_connection_pool(
            &context.host,
            context.port,
            &context.username,
            &context.password,
            context.database.as_deref(),
        )?;
        let conn = pool.get_conn()?;
        context.connection = Some(conn);
        Ok(demo_states::testing_connection())
    }
    async fn exit(&self, _context: &mut DynamicStateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

// Adapter for TestingConnectionHandler to DynamicStateHandler
struct TestingConnectionHandlerAdapter;
#[async_trait]
impl DynamicStateHandler for TestingConnectionHandlerAdapter {
    async fn enter(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        Ok(demo_states::testing_connection())
    }
    async fn execute(&self, context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        if let Some(ref mut conn) = context.connection {
            let result: std::result::Result<Vec<mysql::Row>, mysql::Error> = conn.exec("SELECT 1", ());
            match result {
                Ok(_) => Ok(demo_states::verifying_database()),
                Err(e) => Err(format!("Connection test failed: {e}").into()),
            }
        } else {
            Err("No connection available for testing".into())
        }
    }
    async fn exit(&self, _context: &mut DynamicStateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

// Adapter for VerifyingDatabaseHandler to DynamicStateHandler
struct VerifyingDatabaseHandlerAdapter;
#[async_trait]
impl DynamicStateHandler for VerifyingDatabaseHandlerAdapter {
    async fn enter(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        Ok(demo_states::verifying_database())
    }
    async fn execute(&self, context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        if let Some(ref mut conn) = context.connection {
            if let Some(ref db_name) = context.database {
                let query = format!("USE `{db_name}`");
                match conn.query_drop(query) {
                    Ok(_) => Ok(demo_states::getting_version()),
                    Err(e) => Err(format!("Database verification failed: {e}").into()),
                }
            } else {
                Ok(demo_states::getting_version())
            }
        } else {
            Err("No connection available for database verification".into())
        }
    }
    async fn exit(&self, _context: &mut DynamicStateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

// Adapter for GettingVersionHandler to DynamicStateHandler
struct GettingVersionHandlerAdapter;
#[async_trait]
impl DynamicStateHandler for GettingVersionHandlerAdapter {
    async fn enter(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        Ok(demo_states::getting_version())
    }
    async fn execute(&self, context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        if let Some(ref mut conn) = context.connection {
            let version_query = "SELECT VERSION()";
            match conn.query_first::<String, _>(version_query) {
                Ok(Some(version)) => {
                    context.server_version = Some(version.clone());
                    Ok(demo_states::completed())
                }
                Ok(None) => Err("No version returned from server".into()),
                Err(e) => Err(format!("Failed to get server version: {e}").into()),
            }
        } else {
            Err("No connection available for getting version".into())
        }
    }
    async fn exit(&self, _context: &mut DynamicStateContext) -> test_rig::Result<()> {
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

    // Create and configure the dynamic state machine
    let mut machine = DynamicStateMachine::new();
    machine.register_handler(dynamic_state!("initial", "Initial"), Box::new(InitialHandlerAdapter));
    machine.register_handler(demo_states::parsing_config(), Box::new(ParsingConfigHandlerAdapter {
        host, user, password, database,
    }));
    machine.register_handler(demo_states::connecting(), Box::new(ConnectingHandlerAdapter));
    machine.register_handler(demo_states::testing_connection(), Box::new(TestingConnectionHandlerAdapter));
    machine.register_handler(demo_states::verifying_database(), Box::new(VerifyingDatabaseHandlerAdapter));
    machine.register_handler(demo_states::getting_version(), Box::new(GettingVersionHandlerAdapter));

    // Register valid transitions
    register_transitions!(machine, dynamic_state!("initial", "Initial"), [demo_states::parsing_config()]);
    register_transitions!(machine, demo_states::parsing_config(), [demo_states::connecting()]);
    register_transitions!(machine, demo_states::connecting(), [demo_states::testing_connection()]);
    register_transitions!(machine, demo_states::testing_connection(), [demo_states::verifying_database()]);
    register_transitions!(machine, demo_states::verifying_database(), [demo_states::getting_version()]);
    register_transitions!(machine, demo_states::getting_version(), [demo_states::completed()]);

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
        Ok(_) => {
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

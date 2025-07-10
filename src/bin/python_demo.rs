use clap::Parser;
use test_rig::{
    CommonArgs, StateMachine, print_error_and_exit, print_success, print_test_header,
    state_handlers::{
        ConnectingHandler, InitialHandler, NextStateVersionHandler, ParsingConfigHandler,
        TestingConnectionHandler, VerifyingDatabaseHandler,
    },
    state_machine::State,
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
    let mut state_machine = StateMachine::new();

    // Register standard Rust handlers
    register_rust_handlers(&mut state_machine, host, user, password, database);

    // Load Python handlers
    println!(
        "\nLoading Python handlers from module: {}",
        args.python_module
    );
    #[cfg(feature = "python_plugins")]
    match load_python_handlers(&mut state_machine, &args.python_module) {
        Ok(_) => println!("✓ Python handlers loaded successfully"),
        Err(e) => {
            eprintln!("✗ Failed to load Python handlers: {}", e);
            print_error_and_exit("Python handler loading failed", &e);
        }
    }
    #[cfg(not(feature = "python_plugins"))]
    {
        println!("Python plugins are not enabled. Skipping Python handler loading.");
    }

    // Run the state machine
    println!("\nStarting state machine with mixed Rust and Python handlers...");
    match state_machine.run().await {
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

fn register_rust_handlers(
    state_machine: &mut StateMachine,
    host: String,
    user: String,
    password: String,
    database: Option<String>,
) {
    // Register standard handlers
    state_machine.register_handler(State::Initial, Box::new(InitialHandler));
    state_machine.register_handler(
        State::ParsingConfig,
        Box::new(ParsingConfigHandler::new(
            host.clone(),
            user.clone(),
            password.clone(),
            database.clone(),
        )),
    );
    state_machine.register_handler(State::TestingConnection, Box::new(TestingConnectionHandler));
    state_machine.register_handler(State::Connecting, Box::new(ConnectingHandler));
    state_machine.register_handler(State::VerifyingDatabase, Box::new(VerifyingDatabaseHandler));
    state_machine.register_handler(
        State::GettingVersion,
        Box::new(NextStateVersionHandler {
            next_state: State::Completed,
        }),
    );

    // Set up connection info in the state machine context
    let context = state_machine.get_context_mut();
    context.host = host;
    context.username = user;
    context.password = password;
    context.database = database;
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

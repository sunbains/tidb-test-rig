use connect::{CommonArgs, print_example_header, print_success, print_error_and_exit};
use connect::state_machine::{StateMachine, State};
use connect::{InitialHandler, ParsingConfigHandler, ConnectingHandler, TestingConnectionHandler, VerifyingDatabaseHandler, GettingVersionHandler};
use tracing::{info, debug, warn, error, instrument};
use std::time::Instant;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "logging-example")]
#[command(about = "Logging example with only common arguments")]
pub struct Args {
    #[command(flatten)]
    pub common: CommonArgs,
}

impl Args {
    pub fn print_connection_info(&self) {
        self.common.print_connection_info();
    }
    pub fn init_logging(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.common.init_logging()
    }
    pub fn get_connection_info(&self) -> connect::cli::ConnInfoResult {
        self.common.get_connection_info()
    }
}

#[instrument(skip_all)]
async fn perform_logged_operation(operation_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    info!("Starting operation: {}", operation_name);
    debug!("Operation details: {}", operation_name);
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    connect::log_performance_metric(operation_name, start_time.elapsed());
    connect::log_memory_usage(operation_name, 1024 * 1024);
    info!("Completed operation: {}", operation_name);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_example_header("TiDB Logging Example");
    let args = Args::parse();
    args.init_logging()?;
    args.print_connection_info();
    let (host, user, password, database) = args.get_connection_info()?;
    info!("Starting TiDB logging example");
    debug!("Connection parameters: host={}, user={}, database={:?}", host, user, database);
    warn!("This is a warning message");
    error!("This is an error message (for demonstration)");
    perform_logged_operation("database_connection").await?;
    perform_logged_operation("query_execution").await?;
    perform_logged_operation("data_processing").await?;
    let mut state_machine = StateMachine::new();
    state_machine.register_handler(State::Initial, Box::new(InitialHandler));
    state_machine.register_handler(
        State::ParsingConfig,
        Box::new(ParsingConfigHandler::new(host, user, password, database))
    );
    state_machine.register_handler(State::Connecting, Box::new(ConnectingHandler));
    state_machine.register_handler(State::TestingConnection, Box::new(TestingConnectionHandler));
    state_machine.register_handler(State::VerifyingDatabase, Box::new(VerifyingDatabaseHandler));
    state_machine.register_handler(State::GettingVersion, Box::new(GettingVersionHandler));
    let start_time = Instant::now();
    match state_machine.run().await {
        Ok(_) => {
            let duration = start_time.elapsed();
            connect::log_performance_metric("state_machine_execution", duration);
            info!("Logging example completed successfully");
            print_success("Logging example completed successfully!");
            println!("Check the logs for detailed information.");
        }
        Err(e) => {
            print_error_and_exit("Logging example failed", &*e);
        }
    }
    Ok(())
} 
use connect::{CommonArgs, print_test_header, print_success, print_error_and_exit};
use connect::state_machine::{StateMachine, State};
use connect::{InitialHandler, ParsingConfigHandler, ConnectingHandler, TestingConnectionHandler, VerifyingDatabaseHandler, GettingVersionHandler};
use tracing::{info, debug, warn, error, instrument};
use std::time::Instant;
use clap::Parser;

#[derive(Parser)]
#[command(name = "logging-test")]
#[command(about = "Logging test with only common arguments")]
struct Args {
    #[command(flatten)]
    common: CommonArgs,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_test_header("TiDB Logging Test");
    let args = Args::parse();
    args.init_logging()?;
    
    info!("Starting TiDB logging test");
    
    // Use the shared test setup
    let setup = TestSetup::new(&args.common);
    
    // Run the basic connection workflow
    let result = setup.run_basic_workflow().await;
    
    match result {
        Ok(_) => {
            info!("Logging test completed successfully");
            print_success("Logging test completed successfully!");
            Ok(())
        }
        Err(e) => {
            print_error_and_exit("Logging test failed", &*e);
        }
    }
} 
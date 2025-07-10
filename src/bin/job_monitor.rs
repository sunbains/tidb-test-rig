use connect::{CommonArgs, print_test_header, print_success, print_error_and_exit};
use connect::state_machine::{StateMachine, State};
use connect::import_job_handlers::{CheckingImportJobsHandler, ShowingImportJobDetailsHandler};
use connect::state_handlers::{NextStateVersionHandler, InitialHandler, ParsingConfigHandler, ConnectingHandler, TestingConnectionHandler, VerifyingDatabaseHandler};
use clap::Parser;
use mysql::*;

#[derive(Parser)]
#[command(name = "job-monitor-test")]
#[command(about = "TiDB Import Job Monitoring Test")]
struct Args {
    #[command(flatten)]
    common: CommonArgs,
}

impl Args {
    pub fn init_logging(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.common.init_logging()
    }
    
    pub fn get_connection_info(&self) -> connect::cli::ConnInfoResult {
        self.common.get_connection_info()
    }
}

#[tokio::main]
async fn main() {
    print_test_header("TiDB Import Job Monitoring Test");
    
    let args = Args::parse();
    args.init_logging().expect("Failed to initialize logging");
    
    // Get connection info
    let (host, user, password, database) = args.get_connection_info().expect("Failed to get connection info");
    
    // Create and configure the state machine
    let mut state_machine = StateMachine::new();
    
    // Register handlers manually to include generic version handler
    register_job_monitor_handlers(&mut state_machine, host, user, password, database, args.common.monitor_duration);
    
    // Run the state machine
    match state_machine.run().await {
        Ok(_) => {
            print_success("Job monitoring test completed successfully!");
        }
        Err(e) => {
            print_error_and_exit("Job monitoring test failed", &e);
        }
    }
}

/// Register all handlers for job monitoring test
fn register_job_monitor_handlers(
    state_machine: &mut StateMachine,
    host: String,
    user: String,
    password: String,
    database: Option<String>,
    monitor_duration: u64,
) {
    // Register standard connection handlers
    state_machine.register_handler(State::Initial, Box::new(InitialHandler));
    state_machine.register_handler(
        State::ParsingConfig,
        Box::new(ParsingConfigHandler::new(host, user, password, database))
    );
    state_machine.register_handler(State::Connecting, Box::new(ConnectingHandler));
    state_machine.register_handler(State::TestingConnection, Box::new(TestingConnectionHandler));
    state_machine.register_handler(State::VerifyingDatabase, Box::new(VerifyingDatabaseHandler));
    
    // Register generic version handler that transitions to job monitoring
    state_machine.register_handler(State::GettingVersion, Box::new(NextStateVersionHandler::new(State::CheckingImportJobs)));
    
    // Register job monitoring handlers
    state_machine.register_handler(State::CheckingImportJobs, Box::new(CheckingImportJobsHandler));
    state_machine.register_handler(
        State::ShowingImportJobDetails, 
        Box::new(ShowingImportJobDetailsHandler::new(monitor_duration))
    );
} 
use connect::{CommonArgs, print_test_header, print_success, print_error_and_exit, register_standard_handlers};
use connect::state_machine::{StateMachine, State};
use connect::import_job_handlers::{CheckingImportJobsHandler, ShowingImportJobDetailsHandler};
use clap::Parser;

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
    
    // Register basic connection handlers using the shared function
    register_standard_handlers(&mut state_machine, host, user, password, database);
    
    // Register job monitoring handlers
    register_job_monitoring_handlers(&mut state_machine, args.common.monitor_duration);
    
    // Run the state machine
    match state_machine.run().await {
        Ok(_) => {
            print_success("Job monitoring test completed successfully!");
        }
        Err(e) => {
            print_error_and_exit("Job monitoring test failed", &*e);
        }
    }
}

/// Register the job monitoring handlers
fn register_job_monitoring_handlers(
    state_machine: &mut StateMachine,
    monitor_duration: u64,
) {
    state_machine.register_handler(State::CheckingImportJobs, Box::new(CheckingImportJobsHandler));
    state_machine.register_handler(
        State::ShowingImportJobDetails, 
        Box::new(ShowingImportJobDetailsHandler::new(monitor_duration))
    );
} 
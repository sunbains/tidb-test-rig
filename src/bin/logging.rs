use connect::{CommonArgs, print_test_header, print_success, print_error_and_exit, TestSetup};
use tracing::info;
use clap::Parser;

#[derive(Parser)]
#[command(name = "logging-test")]
#[command(about = "Logging test with only common arguments")]
struct Args {
    #[command(flatten)]
    common: CommonArgs,
}

impl Args {
    pub fn init_logging(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.common.init_logging()
    }
}

#[tokio::main]
async fn main() {
    print_test_header("TiDB Logging Test");
    let args = Args::parse();
    args.init_logging().expect("Failed to initialize logging");
    
    info!("Starting TiDB logging test");
    
    // Use the shared test setup
    let setup = TestSetup::new(&args.common);
    
    // Run the basic connection workflow
    let result = setup.run_basic_workflow().await;
    
    match result {
        Ok(_) => {
            info!("Logging test completed successfully");
            print_success("Logging test completed successfully!");
        }
        Err(e) => {
            print_error_and_exit("Logging test failed", &*e);
        }
    }
} 
use connect::{CommonArgs, print_example_header, print_success, print_error_and_exit, create_setup};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "cli-example")]
#[command(about = "TiDB CLI example with test-specific arguments")]
pub struct IsolationTestArgs {
    #[command(flatten)]
    pub common: CommonArgs,
    /// Number of test rows to create for isolation testing
    #[arg(long, default_value = "10")]
    pub test_rows: u32,
}

impl IsolationTestArgs {
    pub fn print_connection_info(&self) {
        self.common.print_connection_info();
        println!("  Test Rows: {}", self.test_rows);
    }
    pub fn init_logging(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.common.init_logging()
    }
    pub fn get_connection_info(&self) -> connect::cli::ConnInfoResult {
        self.common.get_connection_info()
    }
    pub fn get_database(&self) -> Option<String> {
        self.common.get_database()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_example_header("TiDB CLI Example");
    
    // Parse command line arguments using the specific args type
    let args = IsolationTestArgs::parse();
    
    // Use the shared example setup with the specific args
    args.init_logging()?;
    args.print_connection_info();
    let (host, user, password, database) = args.get_connection_info()?;
    
    // Access example-specific arguments
    println!("  Test Rows: {}", args.test_rows);
    
    // Run the state machine with standard error handling
    match state_machine.run().await {
        Ok(_) => {
            print_success("CLI example completed successfully!");
            println!("Used {} test rows configuration", args.test_rows);
        }
        Err(e) => {
            print_error_and_exit("CLI example failed", &*e);
        }
    }
    
    Ok(())
} 

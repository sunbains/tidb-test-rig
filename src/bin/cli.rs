use test_rig::{CommonArgs, print_test_header, print_success, print_error_and_exit, create_setup};
use clap::Parser;

#[derive(Parser)]
#[command(name = "cli-test")]
#[command(about = "TiDB CLI test with test-specific arguments")]
struct Args {
    #[command(flatten)]
    common: CommonArgs,
    
    /// Test-specific argument: custom message
    #[arg(long, default_value = "Hello from CLI test!")]
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_test_header("TiDB CLI Test");
    
    // Parse command line arguments using the specific args type
    let args = Args::parse();
    args.init_logging()?;
    
    // Use the shared test setup with the specific args
    let setup = TestSetup::new(&args.common);
    
    // Access test-specific arguments
    println!("Custom message: {}", args.message);
    
    // Run the basic connection workflow
    let result = setup.run_basic_workflow().await;
    
    match result {
        Ok(_) => {
            print_success("CLI test completed successfully!");
            Ok(())
        }
        Err(e) => {
            print_error_and_exit("CLI test failed", &*e);
        }
    }
} 

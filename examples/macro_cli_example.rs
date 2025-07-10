use connect::{CommonArgsSetup, print_example_header, print_success, print_error_and_exit};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_example_header("TiDB CLI Example");
    
    // Use the shared example setup
    let mut setup = CommonArgsSetup::new()?;
    
    // Access example-specific arguments
    println!("  Test Rows: {}", setup.args.test_rows);
    
    // Run the state machine with standard error handling
    match setup.state_machine.run().await {
        Ok(_) => {
            print_success("CLI example completed successfully!");
            println!("Used {} test rows configuration", setup.args.test_rows);
        }
        Err(e) => {
            print_error_and_exit("CLI example failed", &*e);
        }
    }
    
    Ok(())
} 

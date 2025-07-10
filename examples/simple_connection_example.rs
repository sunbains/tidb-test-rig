use connect::{CommonArgsSetup, print_example_header, print_success, print_error_and_exit};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_example_header("TiDB Simple Connection Test");
    
    // Use the shared example setup
    let mut setup = CommonArgsSetup::new()?;
    
    // Run the state machine with standard error handling
    match setup.state_machine.run().await {
        Ok(_) => {
            print_success("Simple connection test completed successfully!");
        }
        Err(e) => {
            print_error_and_exit("Simple connection test failed", &*e);
        }
    }
    
    Ok(())
} 
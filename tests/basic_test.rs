use connect::{CommonArgs, print_test_header, print_success, print_error_and_exit};
use connect::state_machine::{StateMachine, State};
use connect::{InitialHandler, ParsingConfigHandler, ConnectingHandler, TestingConnectionHandler, VerifyingDatabaseHandler, GettingVersionHandler};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "basic-test")]
#[command(about = "Basic TiDB connection test with only common arguments")]
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

#[tokio::main]
async fn main() {
    print_test_header("TiDB Basic Connection Test");
    let args = Args::parse();
    args.init_logging().expect("Failed to initialize logging");
    args.print_connection_info();
    let (host, user, password, database) = args.get_connection_info().expect("Failed to get connection info");

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

    match state_machine.run().await {
        Ok(_) => print_success("Basic connection test completed successfully!"),
        Err(e) => print_error_and_exit("Basic connection test failed", &*e),
    }
} 
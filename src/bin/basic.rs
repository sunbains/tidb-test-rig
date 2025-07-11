use clap::Parser;
// No specific handlers needed for basic binary
use test_rig::state_handlers::{
    ConnectingHandler, GettingVersionHandler, InitialHandler, ParsingConfigHandler,
    TestingConnectionHandler, VerifyingDatabaseHandler,
};
use test_rig::{CommonArgs, print_error_and_exit, print_success, print_test_header};
use test_rig::{State, StateMachine};

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
    pub fn get_connection_info(&self) -> test_rig::cli::ConnInfoResult {
        self.common.get_connection_info()
    }
}

// Use core state handlers for the basic workflow

#[tokio::main]
async fn main() {
    print_test_header("TiDB Basic Connection Test");
    let args = Args::parse();
    args.init_logging().expect("Failed to initialize logging");
    args.print_connection_info();
    let (host, user, password, database) = args
        .get_connection_info()
        .expect("Failed to get connection info");

    let mut machine = StateMachine::new();

    // Register core state handlers
    machine.register_handler(State::Initial, Box::new(InitialHandler));
    machine.register_handler(
        State::ParsingConfig,
        Box::new(ParsingConfigHandler::new(host, user, password, database)),
    );
    machine.register_handler(State::Connecting, Box::new(ConnectingHandler));
    machine.register_handler(State::TestingConnection, Box::new(TestingConnectionHandler));
    machine.register_handler(State::VerifyingDatabase, Box::new(VerifyingDatabaseHandler));
    machine.register_handler(State::GettingVersion, Box::new(GettingVersionHandler));

    match machine.run().await {
        Ok(_) => print_success("Basic connection test completed successfully!"),
        Err(e) => print_error_and_exit("Basic connection test failed", &e),
    }
}

use clap::Parser;
use test_rig::common::python_tests::{PYTHON_SUITES, PythonSuiteConfig};
use test_rig::logging::{LogConfig, init_logging};
use tracing::Level;

#[derive(Parser, Debug)]
#[command(name = "python-test-runner")]
#[command(about = "Unified runner for all Python test suites")]
pub struct Args {
    /// Name of the suite to run (case-insensitive, e.g. DDL, Scale). If omitted, runs all suites.
    #[arg(long)]
    suite: Option<String>,
    /// Run all suites
    #[arg(long)]
    all: bool,
    /// Print all output from test runs (stdout/stderr)
    #[arg(long)]
    show_output: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = LogConfig {
        level: Level::INFO,
        console: true,
        file: false,
        file_path: std::path::PathBuf::from("logs/python_test_runner.log"),
        max_file_size: 10,
        max_files: 5,
        include_timestamps: true,
        include_thread_ids: false,
        include_file_line: true,
    };
    init_logging(config)?;

    let suites: Vec<&PythonSuiteConfig> = if args.all || args.suite.is_none() {
        PYTHON_SUITES.iter().collect()
    } else {
        let name = args.suite.as_ref().unwrap().to_lowercase();
        let found: Vec<_> = PYTHON_SUITES
            .iter()
            .filter(|s| s.name.to_lowercase() == name)
            .collect();
        if found.is_empty() {
            eprintln!(
                "No suite found with name '{}'. Available: {:?}",
                name,
                PYTHON_SUITES.iter().map(|s| s.name).collect::<Vec<_>>()
            );
            std::process::exit(1);
        }
        found
    };

    let mut any_failed = false;
    for suite in suites {
        println!("\n=== Running Python test suite: {} ===", suite.name);
        match suite.run_suite_with_output(args.show_output).await {
            Ok(_) => println!("✅ Suite '{}' completed successfully", suite.name),
            Err(e) => {
                eprintln!("❌ Suite '{}' failed: {}", suite.name, e);
                any_failed = true;
            }
        }
    }
    if any_failed {
        std::process::exit(1);
    }
    Ok(())
}

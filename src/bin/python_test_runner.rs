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
    /// Output verbosity level
    #[arg(long, default_value = "normal")]
    output_level: OutputLevel,
    /// Database connection type
    #[arg(long, default_value = "mock")]
    db_type: DatabaseType,
    /// Show SQL queries in output
    #[arg(long)]
    show_sql: bool,
    /// Use real database connection instead of mock
    #[arg(long)]
    real_db: bool,
    /// Only run a specific test file (e.g. test_import_large.py)
    #[arg(long)]
    test_file: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputLevel {
    Normal,
    Verbose,
    Debug,
}

impl std::str::FromStr for OutputLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "normal" => Ok(OutputLevel::Normal),
            "verbose" => Ok(OutputLevel::Verbose),
            "debug" => Ok(OutputLevel::Debug),
            _ => Err(format!("Unknown output level: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    Mock,
    Real,
}

impl std::str::FromStr for DatabaseType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mock" => Ok(DatabaseType::Mock),
            "real" => Ok(DatabaseType::Real),
            _ => Err(format!("Unknown database type: {s}")),
        }
    }
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
    init_logging(&config)?;

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
        let show_output =
            args.output_level == OutputLevel::Verbose || args.output_level == OutputLevel::Debug;
        let show_sql = args.show_sql || args.output_level == OutputLevel::Debug;
        let real_db = args.real_db || args.db_type == DatabaseType::Real;

        match suite
            .run_suite_with_output_filtered(show_output, show_sql, real_db, args.test_file.as_deref())
            .await
        {
            Ok(()) => println!("✅ Suite '{}' completed successfully", suite.name),
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

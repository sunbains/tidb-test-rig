#![allow(non_snake_case)]
use async_trait::async_trait;
use chrono::{NaiveDateTime, Utc};
use clap::Parser;
use mysql::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use test_rig::errors::{ConnectError, Result};
use test_rig::{
    CommonArgs, DynamicState, DynamicStateContext, DynamicStateHandler, DynamicStateMachine,
    dynamic_state, print_error_and_exit, print_success, print_test_header, register_transitions,
};
use tokio::time::sleep;

// Import job types needed for the binary
#[derive(Debug, Clone, FromRow)]
pub struct ImportJob {
    #[allow(non_snake_case)]
    pub Job_ID: i32,
    #[allow(non_snake_case)]
    pub Data_Source: String,
    #[allow(non_snake_case)]
    pub Target_Table: String,
    #[allow(non_snake_case)]
    pub Table_ID: i32,
    #[allow(non_snake_case)]
    pub Phase: String,
    #[allow(non_snake_case)]
    pub Status: String,
    #[allow(non_snake_case)]
    pub Source_File_Size: String,
    #[allow(non_snake_case)]
    pub Imported_Rows: Option<i64>,
    #[allow(non_snake_case)]
    pub Result_Message: String,
    #[allow(non_snake_case)]
    pub Create_Time: Option<NaiveDateTime>,
    #[allow(non_snake_case)]
    pub Start_Time: Option<NaiveDateTime>,
    #[allow(non_snake_case)]
    pub End_Time: Option<NaiveDateTime>,
    #[allow(non_snake_case)]
    pub Created_By: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImportJobInfo {
    pub job_id: String,
    pub connection_id: String,
    pub phase: String,
    pub status: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Context specific to import job handlers
#[derive(Clone)]
pub struct ImportJobContext {
    pub active_import_jobs: Vec<String>,
    pub monitor_duration: u64,
}

impl ImportJobContext {
    pub fn new(monitor_duration: u64) -> Self {
        Self {
            active_import_jobs: Vec::new(),
            monitor_duration,
        }
    }
}

// Define custom states for the workflow
mod job_monitor_states {
    use super::*;

    // Re-export common states
    pub use test_rig::common_states::{
        completed, connecting, getting_version, parsing_config, testing_connection,
        verifying_database,
    };

    // Test-specific states
    pub fn checking_import_jobs() -> DynamicState {
        dynamic_state!("checking_import_jobs", "Checking Import Jobs")
    }
    pub fn showing_import_job_details() -> DynamicState {
        dynamic_state!("showing_import_job_details", "Showing Import Job Details")
    }
}

// Adapter for InitialHandler to DynamicStateHandler
struct InitialHandlerAdapter;
#[async_trait]
impl DynamicStateHandler for InitialHandlerAdapter {
    async fn enter(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        Ok(dynamic_state!("initial", "Initial"))
    }
    async fn execute(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        Ok(job_monitor_states::parsing_config())
    }
    async fn exit(&self, _context: &mut DynamicStateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

// Adapter for ParsingConfigHandler to DynamicStateHandler
struct ParsingConfigHandlerAdapter {
    host: String,
    user: String,
    password: String,
    database: Option<String>,
}
#[async_trait]
impl DynamicStateHandler for ParsingConfigHandlerAdapter {
    async fn enter(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        Ok(job_monitor_states::parsing_config())
    }
    async fn execute(&self, context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        let (host, port) = test_rig::connection::parse_connection_string(&self.host)?;
        context.host = host;
        context.port = port;
        context.username = self.user.clone();
        context.password = self.password.clone();
        context.database = self.database.clone();
        Ok(job_monitor_states::connecting())
    }
    async fn exit(&self, _context: &mut DynamicStateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

// Adapter for ConnectingHandler to DynamicStateHandler
struct ConnectingHandlerAdapter;
#[async_trait]
impl DynamicStateHandler for ConnectingHandlerAdapter {
    async fn enter(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        Ok(job_monitor_states::connecting())
    }
    async fn execute(&self, context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        let pool = test_rig::connection::create_connection_pool(
            &context.host,
            context.port,
            &context.username,
            &context.password,
            context.database.as_deref(),
        )?;
        let conn = pool.get_conn()?;
        context.connection = Some(conn);
        Ok(job_monitor_states::testing_connection())
    }
    async fn exit(&self, _context: &mut DynamicStateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

// Adapter for TestingConnectionHandler to DynamicStateHandler
struct TestingConnectionHandlerAdapter;
#[async_trait]
impl DynamicStateHandler for TestingConnectionHandlerAdapter {
    async fn enter(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        Ok(job_monitor_states::testing_connection())
    }
    async fn execute(&self, context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        if let Some(ref mut conn) = context.connection {
            let result: std::result::Result<Vec<mysql::Row>, mysql::Error> =
                conn.exec("SELECT 1", ());
            match result {
                Ok(_) => Ok(job_monitor_states::verifying_database()),
                Err(e) => Err(format!("Connection test failed: {e}").into()),
            }
        } else {
            Err("No connection available for testing".into())
        }
    }
    async fn exit(&self, _context: &mut DynamicStateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

// Adapter for VerifyingDatabaseHandler to DynamicStateHandler
struct VerifyingDatabaseHandlerAdapter;
#[async_trait]
impl DynamicStateHandler for VerifyingDatabaseHandlerAdapter {
    async fn enter(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        Ok(job_monitor_states::verifying_database())
    }
    async fn execute(&self, context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        if let Some(ref mut conn) = context.connection {
            if let Some(ref db_name) = context.database {
                let query = format!("USE `{db_name}`");
                match conn.query_drop(query) {
                    Ok(_) => Ok(job_monitor_states::getting_version()),
                    Err(e) => Err(format!("Database verification failed: {e}").into()),
                }
            } else {
                Ok(job_monitor_states::getting_version())
            }
        } else {
            Err("No connection available for database verification".into())
        }
    }
    async fn exit(&self, _context: &mut DynamicStateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

// Adapter for GettingVersionHandler to DynamicStateHandler
struct GettingVersionHandlerAdapter;
#[async_trait]
impl DynamicStateHandler for GettingVersionHandlerAdapter {
    async fn enter(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        Ok(job_monitor_states::getting_version())
    }
    async fn execute(&self, context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        if let Some(ref mut conn) = context.connection {
            let version_query = "SELECT VERSION()";
            match conn.query_first::<String, _>(version_query) {
                Ok(Some(version)) => {
                    context.server_version = Some(version.clone());
                    Ok(job_monitor_states::checking_import_jobs())
                }
                Ok(None) => Err("No version returned from server".into()),
                Err(e) => Err(format!("Failed to get server version: {e}").into()),
            }
        } else {
            Err("No connection available for getting version".into())
        }
    }
    async fn exit(&self, _context: &mut DynamicStateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

/// Handler for checking import jobs
pub struct CheckingImportJobsHandler;

#[async_trait]
impl DynamicStateHandler for CheckingImportJobsHandler {
    async fn enter(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        println!("Checking for active import jobs...");
        Ok(job_monitor_states::checking_import_jobs())
    }

    async fn execute(&self, context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        if let Some(ref mut conn) = context.connection {
            // Execute SHOW IMPORT JOBS
            let query = "SHOW IMPORT JOBS";
            let results: Vec<ImportJob> = conn.exec(query, ())?;

            // Extract job IDs where End_Time is NULL
            let mut active_jobs = Vec::new();
            for job in results {
                if job.End_Time.is_none() {
                    active_jobs.push(job.Job_ID.to_string());
                }
            }

            // Store active jobs in context for next state
            context.set_custom_data("active_import_jobs".to_string(), active_jobs.clone());

            // Check if we have active jobs
            if active_jobs.is_empty() {
                println!("✓ No active import jobs found");
                Ok(job_monitor_states::completed())
            } else {
                println!("✓ Found {} active import job(s)", active_jobs.len());
                Ok(job_monitor_states::showing_import_job_details())
            }
        } else {
            return Err(ConnectError::StateMachine(
                "No connection available for checking import jobs".to_string(),
            ));
        }
    }

    async fn exit(&self, _context: &mut DynamicStateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

/// Handler for showing import job details
pub struct ShowingImportJobDetailsHandler {
    monitor_duration: u64,
}

impl ShowingImportJobDetailsHandler {
    pub fn new(monitor_duration: u64) -> Self {
        Self { monitor_duration }
    }
}

#[async_trait]
impl DynamicStateHandler for ShowingImportJobDetailsHandler {
    async fn enter(&self, _context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        println!(
            "Monitoring active import job(s) for {} seconds...",
            self.monitor_duration
        );
        Ok(job_monitor_states::showing_import_job_details())
    }

    async fn execute(&self, context: &mut DynamicStateContext) -> test_rig::Result<DynamicState> {
        // Extract active jobs from context
        let active_jobs: Vec<String> =
            if let Some(jobs) = context.get_custom_data::<Vec<String>>("active_import_jobs") {
                jobs.clone()
            } else {
                return Err("No active import jobs found in context".into());
            };

        if let Some(ref mut conn) = context.connection {
            let start_time = std::time::Instant::now();
            let duration = Duration::from_secs(self.monitor_duration);

            while start_time.elapsed() < duration {
                println!(
                    "\n--- Import Job Status Update ({}s remaining) ---",
                    (duration - start_time.elapsed()).as_secs()
                );

                for job_id in &active_jobs {
                    let query = format!("SHOW IMPORT JOB {job_id}");
                    let results: Vec<ImportJob> = conn.exec(&query, ())?;
                    for job in results {
                        if job.End_Time.is_none() {
                            // Calculate time elapsed using UTC for consistency
                            let now = Utc::now().naive_utc();
                            let start_time = job.Start_Time.unwrap_or(now);
                            let elapsed = now - start_time;
                            let elapsed_h = elapsed.num_seconds() / 3600;
                            let elapsed_m = (elapsed.num_seconds() % 3600) / 60;
                            let elapsed_s = elapsed.num_seconds() % 60;
                            println!(
                                "Job_ID: {} | Phase: {} | Start_Time: {} | Source_File_Size: {} | Imported_Rows: {} | Time elapsed: {:02}:{:02}:{:02}",
                                job.Job_ID,
                                job.Phase,
                                job.Start_Time
                                    .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                                    .unwrap_or_else(|| "N/A".to_string()),
                                job.Source_File_Size,
                                job.Imported_Rows.unwrap_or(0),
                                elapsed_h,
                                elapsed_m,
                                elapsed_s
                            );
                        } else {
                            println!(
                                "Job_ID: {} | Status: Completed | End_Time: {}",
                                job.Job_ID,
                                job.End_Time
                                    .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                                    .unwrap_or_else(|| "N/A".to_string())
                            );
                        }
                    }
                }

                // Sleep before next update
                sleep(Duration::from_secs(5)).await;
            }

            println!("✓ Import job monitoring completed");
            Ok(job_monitor_states::completed())
        } else {
            Err("No connection available for showing import job details".into())
        }
    }

    async fn exit(&self, _context: &mut DynamicStateContext) -> test_rig::Result<()> {
        Ok(())
    }
}

/// Import job monitoring configuration specific to the job monitor test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportJobConfig {
    /// Default monitoring duration in seconds
    #[serde(default = "default_monitor_duration")]
    pub monitor_duration: u64,

    /// Update interval in seconds
    #[serde(default = "default_update_interval")]
    pub update_interval: u64,

    /// Show detailed job information
    #[serde(default = "default_show_details")]
    pub show_details: bool,
}

impl Default for ImportJobConfig {
    fn default() -> Self {
        Self {
            monitor_duration: default_monitor_duration(),
            update_interval: default_update_interval(),
            show_details: default_show_details(),
        }
    }
}

// Default value functions for ImportJobConfig
fn default_monitor_duration() -> u64 {
    300
}
fn default_update_interval() -> u64 {
    5
}
fn default_show_details() -> bool {
    true
}

impl ImportJobConfig {
    /// Load configuration from a file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("json");

        let config = match extension {
            "json" => {
                let content = std::fs::read_to_string(path)?;
                serde_json::from_str(&content).map_err(|e| ConnectError::from(e.to_string()))?
            }
            "toml" => {
                let content = std::fs::read_to_string(path)?;
                toml::from_str(&content).map_err(|e| ConnectError::from(e.to_string()))?
            }
            _ => return Err(format!("Unsupported config file format: {extension}").into()),
        };

        Ok(config)
    }

    /// Save configuration to a file
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("json");

        let content = match extension {
            "json" => {
                serde_json::to_string_pretty(self).map_err(|e| ConnectError::from(e.to_string()))?
            }
            "toml" => {
                toml::to_string_pretty(self).map_err(|e| ConnectError::from(e.to_string()))?
            }
            _ => return Err(format!("Unsupported config file format: {extension}").into()),
        };

        std::fs::write(path, content)?;
        Ok(())
    }

    /// Apply environment variable overrides
    pub fn apply_environment_overrides(&mut self) {
        if let Ok(duration) = std::env::var("TIDB_MONITOR_DURATION")
            && let Ok(duration) = duration.parse()
        {
            self.monitor_duration = duration;
        }
        if let Ok(interval) = std::env::var("TIDB_UPDATE_INTERVAL")
            && let Ok(interval) = interval.parse()
        {
            self.update_interval = interval;
        }
        if let Ok(show_details) = std::env::var("TIDB_SHOW_DETAILS") {
            self.show_details = show_details.to_lowercase() == "true";
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.monitor_duration == 0 {
            return Err("Monitor duration must be greater than 0".into());
        }
        if self.update_interval == 0 {
            return Err("Update interval must be greater than 0".into());
        }
        if self.update_interval > self.monitor_duration {
            return Err("Update interval cannot be greater than monitor duration".into());
        }
        Ok(())
    }
}

#[derive(Parser)]
#[command(name = "job-monitor-test")]
#[command(about = "TiDB Import Job Monitoring Test")]
struct Args {
    #[command(flatten)]
    common: CommonArgs,

    /// Import job config file path (JSON or TOML)
    #[arg(long)]
    import_config: Option<String>,

    /// Duration to monitor import jobs in seconds (default: 300)
    #[arg(short = 't', long, default_value = "300")]
    monitor_duration: u64,
}

impl Args {
    pub fn init_logging(&self) -> Result<()> {
        self.common
            .init_logging()
            .map_err(|e| ConnectError::from(e.to_string()))
    }

    pub fn get_connection_info(&self) -> test_rig::cli::ConnInfoResult {
        self.common.get_connection_info()
    }

    /// Load import job configuration, merging CLI args and config file
    pub fn get_import_config(&self) -> Result<ImportJobConfig> {
        let mut config = if let Some(ref config_path) = self.import_config {
            ImportJobConfig::from_file(config_path)
                .map_err(|e| ConnectError::from(e.to_string()))?
        } else {
            ImportJobConfig::default()
        };

        // Apply environment overrides
        config.apply_environment_overrides();

        // Override with CLI arguments if provided
        config.monitor_duration = self.monitor_duration;

        // Validate the configuration
        config
            .validate()
            .map_err(|e| ConnectError::from(e.to_string()))?;

        Ok(config)
    }
}

#[tokio::main]
async fn main() {
    print_test_header("TiDB Import Job Monitoring Test");

    let args = Args::parse();
    args.init_logging().expect("Failed to initialize logging");

    // Get connection info
    let (host, user, password, database) = args
        .get_connection_info()
        .expect("Failed to get connection info");

    // Get import job configuration
    let import_config = args
        .get_import_config()
        .expect("Failed to load import job configuration");

    println!("Import Job Configuration:");
    println!("  Monitor Duration: {}s", import_config.monitor_duration);
    println!("  Update Interval: {}s", import_config.update_interval);
    println!("  Show Details: {}", import_config.show_details);

    // Create and configure the dynamic state machine
    let mut machine = DynamicStateMachine::new();

    // Register handlers manually to include generic version handler
    register_job_monitor_handlers(
        &mut machine,
        host,
        user,
        password,
        database,
        import_config.monitor_duration,
    );

    // Register valid transitions
    register_transitions!(
        machine,
        dynamic_state!("initial", "Initial"),
        [job_monitor_states::parsing_config()]
    );
    register_transitions!(
        machine,
        job_monitor_states::parsing_config(),
        [job_monitor_states::connecting()]
    );
    register_transitions!(
        machine,
        job_monitor_states::connecting(),
        [job_monitor_states::testing_connection()]
    );
    register_transitions!(
        machine,
        job_monitor_states::testing_connection(),
        [job_monitor_states::verifying_database()]
    );
    register_transitions!(
        machine,
        job_monitor_states::verifying_database(),
        [job_monitor_states::getting_version()]
    );
    register_transitions!(
        machine,
        job_monitor_states::getting_version(),
        [job_monitor_states::checking_import_jobs()]
    );
    register_transitions!(
        machine,
        job_monitor_states::checking_import_jobs(),
        [
            job_monitor_states::showing_import_job_details(),
            job_monitor_states::completed()
        ]
    );
    register_transitions!(
        machine,
        job_monitor_states::showing_import_job_details(),
        [job_monitor_states::completed()]
    );

    // Run the state machine
    match machine.run().await {
        Ok(_) => {
            print_success("Job monitoring test completed successfully!");
        }
        Err(e) => {
            print_error_and_exit("Job monitoring test failed", &e);
        }
    }
}

/// Register all handlers for job monitoring test
fn register_job_monitor_handlers(
    state_machine: &mut DynamicStateMachine,
    host: String,
    user: String,
    password: String,
    database: Option<String>,
    monitor_duration: u64,
) {
    // Register standard connection handlers
    state_machine.register_handler(
        dynamic_state!("initial", "Initial"),
        Box::new(InitialHandlerAdapter),
    );
    state_machine.register_handler(
        job_monitor_states::parsing_config(),
        Box::new(ParsingConfigHandlerAdapter {
            host,
            user,
            password,
            database,
        }),
    );
    state_machine.register_handler(
        job_monitor_states::connecting(),
        Box::new(ConnectingHandlerAdapter),
    );
    state_machine.register_handler(
        job_monitor_states::testing_connection(),
        Box::new(TestingConnectionHandlerAdapter),
    );
    state_machine.register_handler(
        job_monitor_states::verifying_database(),
        Box::new(VerifyingDatabaseHandlerAdapter),
    );

    // Register generic version handler that transitions to job monitoring
    state_machine.register_handler(
        job_monitor_states::getting_version(),
        Box::new(GettingVersionHandlerAdapter),
    );

    // Register job monitoring handlers
    state_machine.register_handler(
        job_monitor_states::checking_import_jobs(),
        Box::new(CheckingImportJobsHandler),
    );
    state_machine.register_handler(
        job_monitor_states::showing_import_job_details(),
        Box::new(ShowingImportJobDetailsHandler::new(monitor_duration)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from([
            "test-bin",
            "-H",
            "localhost:4000",
            "-u",
            "testuser",
            "--password",
            "testpass",
        ]);
        assert_eq!(args.common.host, "localhost:4000");
        assert_eq!(args.common.user, "testuser");
        assert_eq!(args.common.password, Some("testpass".to_string()));
    }

    #[test]
    fn test_args_defaults() {
        let args = Args::parse_from(["test-bin"]);
        assert_eq!(args.common.host, "localhost:4000"); // default value
        assert_eq!(args.common.user, "root"); // default value
        assert_eq!(args.common.password, None); // default value
    }

    #[test]
    fn test_connection_info_validation() {
        let args = Args::parse_from([
            "test-bin",
            "-H",
            "testhost:4000",
            "-u",
            "testuser",
            "--password",
            "testpass",
            "-d",
            "testdb",
        ]);

        let result = args.get_connection_info();
        assert!(result.is_ok());

        let (host, user, password, database) = result.unwrap();
        assert_eq!(host, "testhost:4000");
        assert_eq!(user, "testuser");
        assert_eq!(password, "testpass");
        assert_eq!(database, Some("testdb".to_string()));
    }

    #[test]
    fn test_logging_initialization() {
        let args = Args::parse_from(["test-bin"]);
        let result = args.init_logging();
        assert!(result.is_ok());
    }

    #[test]
    fn test_handler_registration() {
        // Test that we can create the handlers without errors
        let _version_handler = GettingVersionHandlerAdapter;
        let _checking_handler = CheckingImportJobsHandler;
        let _details_handler = ShowingImportJobDetailsHandler::new(30);

        // This test ensures the handlers can be instantiated
        assert!(true);
    }

    #[test]
    #[serial]
    fn test_import_job_config_integration() {
        // Test that local ImportJobConfig works with job monitoring logic
        let import_config = ImportJobConfig {
            monitor_duration: 120,
            update_interval: 10,
            show_details: true,
        };

        assert_eq!(import_config.monitor_duration, 120);
        assert_eq!(import_config.update_interval, 10);
        assert!(import_config.show_details);

        // Test validation
        assert!(import_config.validate().is_ok());
    }

    #[test]
    #[serial]
    fn test_job_monitor_config_file_parsing() {
        let json = r#"{
            "monitor_duration": 300,
            "update_interval": 15,
            "show_details": false
        }"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(json.as_bytes()).unwrap();

        let config = ImportJobConfig::from_file(file.path()).unwrap();
        assert_eq!(config.monitor_duration, 300);
        assert_eq!(config.update_interval, 15);
        assert!(!config.show_details);
    }

    #[test]
    fn test_monitor_duration_validation() {
        // Test that monitor duration from CLI args works correctly
        let args = Args::parse_from([
            "test-bin", "-t", "90", // monitor_duration
        ]);

        assert_eq!(args.monitor_duration, 90);
    }

    #[test]
    #[serial]
    fn test_get_import_config_with_cli_override() {
        let json = r#"{
            "monitor_duration": 200,
            "update_interval": 10,
            "show_details": true
        }"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(json.as_bytes()).unwrap();

        let args = Args::parse_from([
            "test-bin",
            "--import-config",
            file.path().to_str().unwrap(),
            "-t",
            "150", // Override monitor_duration
        ]);

        let config = args.get_import_config().unwrap();
        assert_eq!(config.monitor_duration, 150); // CLI override takes precedence
        assert_eq!(config.update_interval, 10); // From config file
        assert!(config.show_details); // From config file
    }

    #[test]
    fn test_get_import_config_defaults() {
        let args = Args::parse_from(["test-bin"]);
        let config = args.get_import_config().unwrap();

        assert_eq!(config.monitor_duration, 300); // From CLI default
        assert_eq!(config.update_interval, 5); // From ImportJobConfig default
        assert!(config.show_details); // From ImportJobConfig default
    }

    #[test]
    #[serial]
    fn test_import_config_validation() {
        // Test valid config
        let valid_config = ImportJobConfig {
            monitor_duration: 100,
            update_interval: 10,
            show_details: true,
        };
        assert!(valid_config.validate().is_ok());

        // Test invalid config - update_interval > monitor_duration
        let invalid_config = ImportJobConfig {
            monitor_duration: 10,
            update_interval: 20,
            show_details: true,
        };
        assert!(invalid_config.validate().is_err());

        // Test invalid config - zero monitor_duration
        let invalid_config2 = ImportJobConfig {
            monitor_duration: 0,
            update_interval: 5,
            show_details: true,
        };
        assert!(invalid_config2.validate().is_err());
    }
}

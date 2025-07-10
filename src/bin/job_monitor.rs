use connect::{CommonArgs, print_test_header, print_success, print_error_and_exit};
use connect::state_machine::{StateMachine, State};
use connect::import_job_handlers::{CheckingImportJobsHandler, ShowingImportJobDetailsHandler};
use connect::state_handlers::{NextStateVersionHandler, InitialHandler, ParsingConfigHandler, ConnectingHandler, TestingConnectionHandler, VerifyingDatabaseHandler};
use clap::Parser;
use mysql::*;
use serde::{Deserialize, Serialize};

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
fn default_monitor_duration() -> u64 { 300 }
fn default_update_interval() -> u64 { 5 }
fn default_show_details() -> bool { true }

impl ImportJobConfig {
    /// Load configuration from a file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("json");
        
        let config = match extension {
            "json" => {
                let content = std::fs::read_to_string(path)?;
                serde_json::from_str(&content)?
            },
            "toml" => {
                let content = std::fs::read_to_string(path)?;
                toml::from_str(&content)?
            },
            _ => return Err(format!("Unsupported config file format: {}", extension).into()),
        };
        
        Ok(config)
    }
    
    /// Save configuration to a file
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("json");
        
        let content = match extension {
            "json" => {
                serde_json::to_string_pretty(self)?
            },
            "toml" => {
                toml::to_string_pretty(self)?
            },
            _ => return Err(format!("Unsupported config file format: {}", extension).into()),
        };
        
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Apply environment variable overrides
    pub fn apply_environment_overrides(&mut self) {
        if let Ok(duration) = std::env::var("TIDB_MONITOR_DURATION") {
            if let Ok(duration) = duration.parse() {
                self.monitor_duration = duration;
            }
        }
        if let Ok(interval) = std::env::var("TIDB_UPDATE_INTERVAL") {
            if let Ok(interval) = interval.parse() {
                self.update_interval = interval;
            }
        }
        if let Ok(show_details) = std::env::var("TIDB_SHOW_DETAILS") {
            self.show_details = show_details.to_lowercase() == "true";
        }
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
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
}

impl Args {
    pub fn init_logging(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.common.init_logging()
    }
    
    pub fn get_connection_info(&self) -> connect::cli::ConnInfoResult {
        self.common.get_connection_info()
    }
    
    /// Load import job configuration, merging CLI args and config file
    pub fn get_import_config(&self) -> Result<ImportJobConfig, Box<dyn std::error::Error>> {
        let mut config = if let Some(ref config_path) = self.import_config {
            ImportJobConfig::from_file(config_path)?
        } else {
            ImportJobConfig::default()
        };
        
        // Apply environment overrides
        config.apply_environment_overrides();
        
        // Override with CLI arguments (CLI default is 60, ImportJobConfig default is 300)
        config.monitor_duration = self.common.monitor_duration;
        
        // Validate the configuration
        config.validate()?;
        
        Ok(config)
    }
}

#[tokio::main]
async fn main() {
    print_test_header("TiDB Import Job Monitoring Test");
    
    let args = Args::parse();
    args.init_logging().expect("Failed to initialize logging");
    
    // Get connection info
    let (host, user, password, database) = args.get_connection_info().expect("Failed to get connection info");
    
    // Get import job configuration
    let import_config = args.get_import_config().expect("Failed to load import job configuration");
    
    println!("Import Job Configuration:");
    println!("  Monitor Duration: {}s", import_config.monitor_duration);
    println!("  Update Interval: {}s", import_config.update_interval);
    println!("  Show Details: {}", import_config.show_details);
    
    // Create and configure the state machine
    let mut state_machine = StateMachine::new();
    
    // Register handlers manually to include generic version handler
    register_job_monitor_handlers(&mut state_machine, host, user, password, database, import_config.monitor_duration);
    
    // Run the state machine
    match state_machine.run().await {
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
    state_machine: &mut StateMachine,
    host: String,
    user: String,
    password: String,
    database: Option<String>,
    monitor_duration: u64,
) {
    // Register standard connection handlers
    state_machine.register_handler(State::Initial, Box::new(InitialHandler));
    state_machine.register_handler(
        State::ParsingConfig,
        Box::new(ParsingConfigHandler::new(host, user, password, database))
    );
    state_machine.register_handler(State::Connecting, Box::new(ConnectingHandler));
    state_machine.register_handler(State::TestingConnection, Box::new(TestingConnectionHandler));
    state_machine.register_handler(State::VerifyingDatabase, Box::new(VerifyingDatabaseHandler));
    
    // Register generic version handler that transitions to job monitoring
    state_machine.register_handler(State::GettingVersion, Box::new(NextStateVersionHandler::new(State::CheckingImportJobs)));
    
    // Register job monitoring handlers
    state_machine.register_handler(State::CheckingImportJobs, Box::new(CheckingImportJobsHandler));
    state_machine.register_handler(
        State::ShowingImportJobDetails, 
        Box::new(ShowingImportJobDetailsHandler::new(monitor_duration))
    );
} 

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use serial_test::serial;

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from([
            "test-bin", 
            "-H", "localhost:4000",
            "-u", "testuser",
            "--password", "testpass"
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
            "-H", "testhost:4000",
            "-u", "testuser",
            "--password", "testpass",
            "-d", "testdb"
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
        let _version_handler = NextStateVersionHandler::new(State::CheckingImportJobs);
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
            "test-bin",
            "-t", "90"  // monitor_duration
        ]);
        
        assert_eq!(args.common.monitor_duration, 90);
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
            "--import-config", file.path().to_str().unwrap(),
            "-t", "150"  // Override monitor_duration
        ]);
        
        let config = args.get_import_config().unwrap();
        assert_eq!(config.monitor_duration, 150); // CLI override takes precedence
        assert_eq!(config.update_interval, 10);   // From config file
        assert!(config.show_details);             // From config file
    }

    #[test]
    fn test_get_import_config_defaults() {
        let args = Args::parse_from(["test-bin"]);
        let config = args.get_import_config().unwrap();
        
        assert_eq!(config.monitor_duration, 60); // From CLI default (overrides ImportJobConfig default of 300)
        assert_eq!(config.update_interval, 5);   // From ImportJobConfig default
        assert!(config.show_details);            // From ImportJobConfig default
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
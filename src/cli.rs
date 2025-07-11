//! # Command-Line Interface
//!
//! Command-line argument parsing and common CLI utilities.
//! Provides standardized argument handling for database connections and configuration.

use crate::config::AppConfig;
use crate::errors::Result;
use clap::Parser;
use rpassword::prompt_password;
use std::env;

pub type ConnInfoResult =
    std::result::Result<(String, String, String, Option<String>), Box<dyn std::error::Error>>;

#[derive(Parser, Debug, Clone)]
#[command(name = "tidb-tests")]
#[command(about = "TiDB connection and testing tests")]
pub struct CommonArgs {
    /// Configuration file path (JSON or TOML)
    #[arg(short = 'c', long)]
    pub config: Option<String>,

    /// Hostname and port in format hostname:port
    #[arg(short = 'H', long, default_value = "localhost:4000")]
    pub host: String,

    /// Username for database authentication
    #[arg(short = 'u', long, default_value = "root")]
    pub user: String,

    /// Database name (optional)
    #[arg(short = 'd', long)]
    pub database: Option<String>,

    /// Skip password prompt (for automated testing)
    #[arg(long)]
    pub no_password_prompt: bool,

    /// Password from command line (alternative to prompt)
    #[arg(long)]
    pub password: Option<String>,

    /// Print all output from test runs (stdout/stderr)
    #[arg(long)]
    pub show_output: bool,

    /// Show all SQL queries being sent to the server with connection IDs
    #[arg(long)]
    pub show_sql: bool,

    // Logging options
    /// Log level (debug, info, warn, error)
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// Enable file logging
    #[arg(long)]
    pub log_file: bool,

    /// Log file path
    #[arg(long)]
    pub log_file_path: Option<String>,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}

impl CommonArgs {
    /// Load configuration from file if specified, otherwise use defaults
    pub fn load_config(&self) -> Result<AppConfig> {
        if let Some(ref config_path) = self.config {
            AppConfig::from_file_with_env(config_path)
        } else {
            // Try to load from default config files
            let default_paths = [
                "tidb_config.json",
                "tidb_config.toml",
                "config/tidb.json",
                "config/tidb.toml",
            ];
            for path in &default_paths {
                if std::path::Path::new(path).exists() {
                    return AppConfig::from_file_with_env(path);
                }
            }
            // Use environment-based config
            AppConfig::from_env()
        }
    }

    /// Merge CLI arguments with configuration file settings
    pub fn merge_with_config(&self, config: &AppConfig) -> AppConfig {
        let mut merged_config = config.clone();

        // Override with CLI arguments if provided
        if self.host != "localhost:4000" {
            merged_config.database.host = self.host.clone();
        }
        if self.user != "root" {
            merged_config.database.username = self.user.clone();
        }
        if let Some(ref database) = self.database {
            merged_config.database.database = Some(database.clone());
        }
        if self.log_level != "info" {
            merged_config.logging.level = self.log_level.clone();
        }
        if self.verbose {
            merged_config.test.verbose = true;
        }

        merged_config
    }

    pub fn get_password(&self) -> std::result::Result<String, Box<dyn std::error::Error>> {
        if let Some(ref password) = self.password {
            return Ok(password.clone());
        }
        if let Ok(password) = env::var("TIDB_PASSWORD") {
            return Ok(password);
        }
        if !self.no_password_prompt {
            return Ok(prompt_password("Password: ")?);
        }
        Err("No password provided and password prompt is disabled".into())
    }

    pub fn get_host(&self) -> String {
        if self.host != "localhost:4000" {
            self.host.clone()
        } else {
            env::var("TIDB_HOST").unwrap_or_else(|_| self.host.clone())
        }
    }

    pub fn get_user(&self) -> String {
        if self.user != "root" {
            self.user.clone()
        } else {
            env::var("TIDB_USER").unwrap_or_else(|_| self.user.clone())
        }
    }

    pub fn get_database(&self) -> Option<String> {
        self.database
            .clone()
            .or_else(|| env::var("TIDB_DATABASE").ok())
    }

    pub fn get_connection_info(&self) -> ConnInfoResult {
        let password = self.get_password()?;
        Ok((
            self.get_host(),
            self.get_user(),
            password,
            self.get_database(),
        ))
    }

    pub fn get_connection_info_from_config(
        &self,
    ) -> Result<(String, String, String, Option<String>)> {
        let config = self.load_config()?;
        let merged_config = self.merge_with_config(&config);

        let password = if let Some(pwd) = merged_config.get_password() {
            pwd
        } else if let Ok(pwd) = self.get_password() {
            pwd
        } else {
            return Err(crate::errors::ConnectError::Configuration(
                "No password available".to_string(),
            ));
        };

        Ok((
            merged_config.database.host,
            merged_config.database.username,
            password,
            merged_config.database.database,
        ))
    }

    pub fn validate(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if !self.host.contains(':') {
            return Err("Host must be in format 'hostname:port'".into());
        }
        if let Some(port_str) = self.host.split(':').nth(1)
            && port_str.parse::<u16>().is_err()
        {
            return Err("Invalid port number".into());
        }
        if self.user.is_empty() {
            return Err("Username cannot be empty".into());
        }
        Ok(())
    }

    pub fn print_connection_info(&self) {
        println!("Connection Info:");
        println!("  Host: {}", self.host);
        println!("  User: {}", self.user);
        println!(
            "  Database: {}",
            self.database.as_deref().unwrap_or("(not specified)")
        );

        // Also print config file info if specified
        if let Some(ref config_path) = self.config {
            println!("  Config File: {config_path}");
        }
    }

    pub fn init_logging(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        use crate::logging::LogConfig;
        use std::path::PathBuf;
        use tracing::Level;

        let level = match self.log_level.to_lowercase().as_str() {
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        };
        let level = if self.verbose { Level::DEBUG } else { level };
        let mut config = LogConfig::new().with_level(level).with_console(true);
        if self.log_file {
            config = config.with_file(true);
            if let Some(ref file_path) = self.log_file_path {
                config = config.with_file_path(PathBuf::from(file_path));
            }
        }
        crate::logging::init_logging(config)
    }

    pub fn init_logging_from_config(&self) -> Result<()> {
        let app_config = self.load_config()?;
        let merged_config = self.merge_with_config(&app_config);

        use crate::logging::LogConfig;
        use std::path::PathBuf;
        use tracing::Level;

        let level = match merged_config.logging.level.to_lowercase().as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        };

        let mut log_config = LogConfig::new()
            .with_level(level)
            .with_console(merged_config.logging.console);

        if let Some(ref file_path) = merged_config.logging.file {
            log_config = log_config
                .with_file(true)
                .with_file_path(PathBuf::from(file_path));
        }

        crate::logging::init_logging(log_config)
            .map_err(|e| crate::errors::ConnectError::Logging(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_common_args_defaults() {
        let args = CommonArgs::parse_from(["test-bin"]);
        assert_eq!(args.host, "localhost:4000");
        assert_eq!(args.user, "root");
    }

    #[test]
    #[serial]
    fn test_get_host_user_database() {
        let prev_host = std::env::var("TIDB_HOST").ok();
        let prev_user = std::env::var("TIDB_USER").ok();
        let prev_database = std::env::var("TIDB_DATABASE").ok();
        unsafe {
            std::env::remove_var("TIDB_HOST");
        }
        unsafe {
            std::env::remove_var("TIDB_USER");
        }
        unsafe {
            std::env::remove_var("TIDB_DATABASE");
        }
        let args = CommonArgs::parse_from(["test-bin", "-H", "h:1", "-u", "u", "-d", "db"]);
        assert_eq!(args.get_host(), "h:1");
        assert_eq!(args.get_user(), "u");
        assert_eq!(args.get_database(), Some("db".to_string()));
        match prev_host {
            Some(val) => unsafe {
                std::env::set_var("TIDB_HOST", val);
            },
            None => unsafe {
                std::env::remove_var("TIDB_HOST");
            },
        }
        match prev_user {
            Some(val) => unsafe {
                std::env::set_var("TIDB_USER", val);
            },
            None => unsafe {
                std::env::remove_var("TIDB_USER");
            },
        }
        match prev_database {
            Some(val) => unsafe {
                std::env::set_var("TIDB_DATABASE", val);
            },
            None => unsafe {
                std::env::remove_var("TIDB_DATABASE");
            },
        }
    }

    #[test]
    #[serial]
    fn test_env_override() {
        let prev = std::env::var("TIDB_HOST").ok();
        let unique = "envhost_cli_test:123";
        unsafe {
            std::env::set_var("TIDB_HOST", unique);
        }
        let args = CommonArgs::parse_from(["test-bin"]); // use default host
        assert_eq!(args.get_host(), unique);
        match prev {
            Some(val) => unsafe {
                std::env::set_var("TIDB_HOST", val);
            },
            None => unsafe {
                std::env::remove_var("TIDB_HOST");
            },
        }
    }
}

pub fn parse_args() -> std::result::Result<CommonArgs, Box<dyn std::error::Error>> {
    let args = CommonArgs::parse();
    args.validate()?;
    Ok(args)
}

pub fn get_connection_info() -> ConnInfoResult {
    let args = parse_args()?;
    args.get_connection_info()
}

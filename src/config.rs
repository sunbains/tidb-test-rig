//! # Configuration Management
//!
//! Configuration management with support for files (JSON/TOML), environment variables,
//! and programmatic setup. Provides validation, defaults, and builder patterns.

use crate::errors::{ConnectError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Main configuration structure for the `TiDB` connection and testing framework
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// Database connection settings
    #[serde(default)]
    pub database: DatabaseConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Test-specific settings
    #[serde(default)]
    pub test: TestConfig,
    // Import job monitoring settings moved to job_monitor.rs
}

/// Database connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database host (e.g., "localhost:4000")
    #[serde(default = "default_host")]
    pub host: String,

    /// Database username
    #[serde(default = "default_username")]
    pub username: String,

    /// Database password (can be overridden by environment variable)
    #[serde(default)]
    pub password: Option<String>,

    /// Database name
    #[serde(default)]
    pub database: Option<String>,

    /// Connection pool size
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log format (json, text)
    #[serde(default = "default_log_format")]
    pub format: String,

    /// Log file path (optional)
    #[serde(default)]
    pub file: Option<String>,

    /// Enable console output
    #[serde(default = "default_console_output")]
    pub console: bool,
}

/// Test-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// Number of test rows for isolation testing
    #[serde(default = "default_test_rows")]
    pub rows: u32,

    /// Test timeout in seconds
    #[serde(default = "default_test_timeout")]
    pub timeout_secs: u64,

    /// Enable verbose output
    #[serde(default)]
    pub verbose: bool,
}

// ImportJobConfig moved to job_monitor.rs

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            username: default_username(),
            password: None,
            database: None,
            pool_size: default_pool_size(),
            timeout_secs: default_timeout(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
            file: None,
            console: default_console_output(),
        }
    }
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            rows: default_test_rows(),
            timeout_secs: default_test_timeout(),
            verbose: false,
        }
    }
}

// ImportJobConfig default implementation moved to job_monitor.rs

// Default value functions
fn default_host() -> String {
    "localhost:4000".to_string()
}
fn default_username() -> String {
    "root".to_string()
}
fn default_pool_size() -> u32 {
    5
}
fn default_timeout() -> u64 {
    30
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_log_format() -> String {
    "text".to_string()
}
fn default_console_output() -> bool {
    true
}
fn default_test_rows() -> u32 {
    10
}
fn default_test_timeout() -> u64 {
    60
}
// Import job default functions moved to job_monitor.rs

impl AppConfig {
    /// Load configuration from a file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("json");

        let config = match extension {
            "json" => {
                let content = std::fs::read_to_string(path).map_err(|e| {
                    ConnectError::Configuration(format!("Failed to read config file: {e}"))
                })?;
                serde_json::from_str(&content).map_err(|e| {
                    ConnectError::Configuration(format!("Failed to parse JSON config: {e}"))
                })?
            }
            "toml" => {
                let content = std::fs::read_to_string(path).map_err(|e| {
                    ConnectError::Configuration(format!("Failed to read config file: {e}"))
                })?;
                toml::from_str(&content).map_err(|e| {
                    ConnectError::Configuration(format!("Failed to parse TOML config: {e}"))
                })?
            }
            _ => {
                return Err(ConnectError::Configuration(format!(
                    "Unsupported config file format: {extension}"
                )));
            }
        };

        Ok(config)
    }

    /// Load configuration from file with environment variable overrides
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn from_file_with_env<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut config = Self::from_file(path)?;
        config.apply_environment_overrides();
        Ok(config)
    }

    /// Load configuration from environment variables
    ///
    /// # Errors
    ///
    /// Returns an error if required environment variables are missing or invalid.
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();
        config.apply_environment_overrides();
        Ok(config)
    }

    /// Apply environment variable overrides to the configuration
    pub fn apply_environment_overrides(&mut self) {
        // Database overrides
        if let Ok(host) = std::env::var("TIDB_HOST") {
            self.database.host = host;
        }
        if let Ok(username) = std::env::var("TIDB_USERNAME") {
            self.database.username = username;
        }
        if let Ok(password) = std::env::var("TIDB_PASSWORD") {
            self.database.password = Some(password);
        }
        if let Ok(database) = std::env::var("TIDB_DATABASE") {
            self.database.database = Some(database);
        }

        // Logging overrides
        if let Ok(level) = std::env::var("TIDB_LOG_LEVEL") {
            self.logging.level = level;
        }
        if let Ok(format) = std::env::var("TIDB_LOG_FORMAT") {
            self.logging.format = format;
        }

        // Test overrides
        if let Ok(rows) = std::env::var("TIDB_TEST_ROWS")
            && let Ok(rows) = rows.parse()
        {
            self.test.rows = rows;
        }
        if let Ok(verbose) = std::env::var("TIDB_VERBOSE") {
            self.test.verbose = verbose.to_lowercase() == "true";
        }

        // Import job overrides
        // Removed import job overrides
    }

    /// Save configuration to a file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("json");

        let content = match extension {
            "json" => serde_json::to_string_pretty(self).map_err(|e| {
                ConnectError::Configuration(format!("Failed to serialize config: {e}"))
            })?,
            "toml" => toml::to_string_pretty(self).map_err(|e| {
                ConnectError::Configuration(format!("Failed to serialize config: {e}"))
            })?,
            _ => {
                return Err(ConnectError::Configuration(format!(
                    "Unsupported config file format: {extension}"
                )));
            }
        };

        std::fs::write(path, content).map_err(|e| {
            ConnectError::Configuration(format!("Failed to write config file: {e}"))
        })?;

        Ok(())
    }

    /// Get the database password, checking environment variables if not set in config
    #[must_use]
    pub fn get_password(&self) -> Option<String> {
        self.database
            .password
            .clone()
            .or_else(|| std::env::var("TIDB_PASSWORD").ok())
    }

    /// Validate configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid.
    pub fn validate(&self) -> Result<()> {
        if self.database.host.is_empty() {
            return Err(ConnectError::Configuration(
                "Database host cannot be empty".to_string(),
            ));
        }
        if self.database.username.is_empty() {
            return Err(ConnectError::Configuration(
                "Database username cannot be empty".to_string(),
            ));
        }
        if self.database.pool_size == 0 {
            return Err(ConnectError::Configuration(
                "Database pool size must be greater than 0".to_string(),
            ));
        }
        if self.database.timeout_secs == 0 {
            return Err(ConnectError::Configuration(
                "Database timeout must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Configuration builder for programmatic configuration
pub struct ConfigBuilder {
    config: AppConfig,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: AppConfig::default(),
        }
    }

    #[must_use]
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.config.database.host = host.into();
        self
    }

    #[must_use]
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.config.database.username = username.into();
        self
    }

    #[must_use]
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.config.database.password = Some(password.into());
        self
    }

    #[must_use]
    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.config.database.database = Some(database.into());
        self
    }

    #[must_use]
    pub fn log_level(mut self, level: impl Into<String>) -> Self {
        self.config.logging.level = level.into();
        self
    }

    #[must_use]
    pub fn test_rows(mut self, rows: u32) -> Self {
        self.config.test.rows = rows;
        self
    }

    // Removed monitor_duration

    #[must_use]
    pub fn build(self) -> AppConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    #[serial]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.database.host, "localhost:4000");
        assert_eq!(config.database.username, "root");
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    #[serial]
    fn test_from_file_json() {
        let json = r#"{
            "database": {"host": "h", "username": "u"},
            "logging": {"level": "debug", "format": "text", "console": true},
            "test": {"rows": 1, "timeout_secs": 2, "verbose": false},
            "import_jobs": {"monitor_duration": 1, "update_interval": 1, "show_details": true}
        }"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(json.as_bytes()).unwrap();
        let config = AppConfig::from_file(file.path()).unwrap();
        assert_eq!(config.database.host, "h");
        assert_eq!(config.database.username, "u");
        assert_eq!(config.logging.level, "debug");
    }

    #[test]
    #[serial]
    fn test_from_file_toml() {
        let toml = r#"
            [database]
            host = "h"
            username = "u"
            [logging]
            level = "debug"
            format = "text"
            console = true
            [test]
            rows = 1
            timeout_secs = 2
            verbose = false
            [import_jobs]
            monitor_duration = 1
            update_interval = 1
            show_details = true
        "#;
        let file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        std::fs::write(file.path(), toml).unwrap();
        let config = AppConfig::from_file(file.path()).unwrap();
        assert_eq!(config.database.host, "h");
        assert_eq!(config.database.username, "u");
        assert_eq!(config.logging.level, "debug");
    }

    #[test]
    #[serial]
    fn test_env_override() {
        let prev = std::env::var("TIDB_HOST").ok();
        let unique = "envhost_config_test";
        unsafe {
            std::env::set_var("TIDB_HOST", unique);
        }
        let mut config = AppConfig::default();
        config.apply_environment_overrides();
        assert_eq!(config.database.host, unique);
        match prev {
            Some(val) => unsafe {
                std::env::set_var("TIDB_HOST", val);
            },
            None => unsafe {
                std::env::remove_var("TIDB_HOST");
            },
        }
    }

    #[test]
    #[serial]
    fn test_save_and_load_json() {
        let config = AppConfig::default();
        let file = NamedTempFile::new().unwrap();
        config.save_to_file(file.path()).unwrap();
        let loaded = AppConfig::from_file(file.path()).unwrap();
        assert_eq!(loaded.database.host, "localhost:4000");
    }
}

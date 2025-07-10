use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::errors::{ConnectError, Result};

/// Main configuration structure for the TiDB connection and testing framework
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    
    /// Import job monitoring settings
    #[serde(default)]
    pub import_jobs: ImportJobConfig,
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

/// Import job monitoring configuration
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

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            logging: LoggingConfig::default(),
            test: TestConfig::default(),
            import_jobs: ImportJobConfig::default(),
        }
    }
}

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

impl Default for ImportJobConfig {
    fn default() -> Self {
        Self {
            monitor_duration: default_monitor_duration(),
            update_interval: default_update_interval(),
            show_details: default_show_details(),
        }
    }
}

// Default value functions
fn default_host() -> String { "localhost:4000".to_string() }
fn default_username() -> String { "root".to_string() }
fn default_pool_size() -> u32 { 5 }
fn default_timeout() -> u64 { 30 }
fn default_log_level() -> String { "info".to_string() }
fn default_log_format() -> String { "text".to_string() }
fn default_console_output() -> bool { true }
fn default_test_rows() -> u32 { 10 }
fn default_test_timeout() -> u64 { 60 }
fn default_monitor_duration() -> u64 { 300 }
fn default_update_interval() -> u64 { 5 }
fn default_show_details() -> bool { true }

impl AppConfig {
    /// Load configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("json");
        
        let config = match extension {
            "json" => {
                let content = std::fs::read_to_string(path)
                    .map_err(|e| ConnectError::Configuration(format!("Failed to read config file: {}", e)))?;
                serde_json::from_str(&content)
                    .map_err(|e| ConnectError::Configuration(format!("Failed to parse JSON config: {}", e)))?
            },
            "toml" => {
                let content = std::fs::read_to_string(path)
                    .map_err(|e| ConnectError::Configuration(format!("Failed to read config file: {}", e)))?;
                toml::from_str(&content)
                    .map_err(|e| ConnectError::Configuration(format!("Failed to parse TOML config: {}", e)))?
            },
            _ => return Err(ConnectError::Configuration(format!("Unsupported config file format: {}", extension))),
        };
        
        Ok(config)
    }
    
    /// Load configuration with environment variable overrides
    pub fn from_file_with_env<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut config = Self::from_file(path)?;
        config.apply_environment_overrides();
        Ok(config)
    }
    
    /// Load configuration from environment variables only
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
        if let Ok(rows) = std::env::var("TIDB_TEST_ROWS") {
            if let Ok(rows) = rows.parse() {
                self.test.rows = rows;
            }
        }
        if let Ok(verbose) = std::env::var("TIDB_VERBOSE") {
            self.test.verbose = verbose.to_lowercase() == "true";
        }
        
        // Import job overrides
        if let Ok(duration) = std::env::var("TIDB_MONITOR_DURATION") {
            if let Ok(duration) = duration.parse() {
                self.import_jobs.monitor_duration = duration;
            }
        }
    }
    
    /// Save configuration to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("json");
        
        let content = match extension {
            "json" => {
                serde_json::to_string_pretty(self)
                    .map_err(|e| ConnectError::Configuration(format!("Failed to serialize config: {}", e)))?
            },
            "toml" => {
                toml::to_string_pretty(self)
                    .map_err(|e| ConnectError::Configuration(format!("Failed to serialize config: {}", e)))?
            },
            _ => return Err(ConnectError::Configuration(format!("Unsupported config file format: {}", extension))),
        };
        
        std::fs::write(path, content)
            .map_err(|e| ConnectError::Configuration(format!("Failed to write config file: {}", e)))?;
        
        Ok(())
    }
    
    /// Get the database password, checking environment variables if not set in config
    pub fn get_password(&self) -> Option<String> {
        self.database.password.clone()
            .or_else(|| std::env::var("TIDB_PASSWORD").ok())
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.database.host.is_empty() {
            return Err(ConnectError::Configuration("Database host cannot be empty".to_string()));
        }
        if self.database.username.is_empty() {
            return Err(ConnectError::Configuration("Database username cannot be empty".to_string()));
        }
        if self.database.pool_size == 0 {
            return Err(ConnectError::Configuration("Database pool size must be greater than 0".to_string()));
        }
        if self.database.timeout_secs == 0 {
            return Err(ConnectError::Configuration("Database timeout must be greater than 0".to_string()));
        }
        Ok(())
    }
}

/// Configuration builder for programmatic configuration
pub struct ConfigBuilder {
    config: AppConfig,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: AppConfig::default(),
        }
    }
    
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.config.database.host = host.into();
        self
    }
    
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.config.database.username = username.into();
        self
    }
    
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.config.database.password = Some(password.into());
        self
    }
    
    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.config.database.database = Some(database.into());
        self
    }
    
    pub fn log_level(mut self, level: impl Into<String>) -> Self {
        self.config.logging.level = level.into();
        self
    }
    
    pub fn test_rows(mut self, rows: u32) -> Self {
        self.config.test.rows = rows;
        self
    }
    
    pub fn monitor_duration(mut self, duration: u64) -> Self {
        self.config.import_jobs.monitor_duration = duration;
        self
    }
    
    pub fn build(self) -> AppConfig {
        self.config
    }
} 
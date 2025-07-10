use clap::Parser;
use rpassword::prompt_password;
use std::env;

#[derive(Parser)]
#[command(name = "tidb-examples")]
#[command(about = "TiDB connection and testing examples")]
pub struct CommonArgs {
    /// Hostname and port in format hostname:port
    #[arg(short = 'H', long, default_value = "localhost:4000")]
    pub host: String,
    
    /// Username for database authentication
    #[arg(short = 'u', long, default_value = "root")]
    pub user: String,
    
    /// Database name (optional)
    #[arg(short = 'd', long)]
    pub database: Option<String>,

    /// Duration to monitor import jobs in seconds (default: 60)
    #[arg(short = 't', long, default_value = "60")]
    pub monitor_duration: u64,
    
    /// Skip password prompt (for automated testing)
    #[arg(long)]
    pub no_password_prompt: bool,
    
    /// Password from command line (alternative to prompt)
    #[arg(long)]
    pub password: Option<String>,

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

    // Compile-time conditional options
    #[cfg(feature = "import_jobs")]
    /// Enable import job monitoring
    #[arg(long)]
    pub enable_import_jobs: bool,

    #[cfg(feature = "isolation_test")]
    /// Number of test rows to create for isolation testing
    #[arg(long, default_value = "10")]
    pub test_rows: u32,

    #[cfg(feature = "multi_connection")]
    /// Number of connections to create for multi-connection tests
    #[arg(long, default_value = "2")]
    pub connection_count: u32,
}

impl CommonArgs {
    /// Get the password, either from args, environment, or prompt
    pub fn get_password(&self) -> Result<String, Box<dyn std::error::Error>> {
        // First try command line argument
        if let Some(ref password) = self.password {
            return Ok(password.clone());
        }
        
        // Then try environment variables
        if let Ok(password) = env::var("TIDB_PASSWORD") {
            return Ok(password);
        }
        
        // Finally prompt user (unless disabled)
        if !self.no_password_prompt {
            return Ok(prompt_password("Password: ")?);
        }
        
        Err("No password provided and password prompt is disabled".into())
    }
    
    /// Get host from args or environment
    pub fn get_host(&self) -> String {
        env::var("TIDB_HOST").unwrap_or_else(|_| self.host.clone())
    }
    
    /// Get user from args or environment
    pub fn get_user(&self) -> String {
        env::var("TIDB_USER").unwrap_or_else(|_| self.user.clone())
    }
    
    /// Get database from args or environment
    pub fn get_database(&self) -> Option<String> {
        env::var("TIDB_DATABASE").ok().or(self.database.clone())
    }
    
    /// Get connection info as a tuple for easy use
    pub fn get_connection_info(&self) -> Result<(String, String, String, Option<String>), Box<dyn std::error::Error>> {
        let password = self.get_password()?;
        Ok((
            self.get_host(),
            self.get_user(),
            password,
            self.get_database()
        ))
    }
    
    /// Validate connection parameters
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Validate host format
        if !self.host.contains(':') {
            return Err("Host must be in format 'hostname:port'".into());
        }
        
        // Validate port number
        if let Some(port_str) = self.host.split(':').nth(1) {
            if let Err(_) = port_str.parse::<u16>() {
                return Err("Invalid port number".into());
            }
        }
        
        // Validate username
        if self.user.is_empty() {
            return Err("Username cannot be empty".into());
        }
        
        Ok(())
    }
    
    /// Print connection info (without password)
    pub fn print_connection_info(&self) {
        println!("Connection Info:");
        println!("  Host: {}", self.host);
        println!("  User: {}", self.user);
        println!("  Database: {}", self.database.as_deref().unwrap_or("(not specified)"));
        println!("  Monitor Duration: {}s", self.monitor_duration);
    }

    /// Initialize logging based on CLI arguments
    pub fn init_logging(&self) -> Result<(), Box<dyn std::error::Error>> {
        use crate::logging::LogConfig;
        use tracing::Level;
        use std::path::PathBuf;

        // Parse log level
        let level = match self.log_level.to_lowercase().as_str() {
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        };

        // Override with verbose flag
        let level = if self.verbose { Level::DEBUG } else { level };

        let mut config = LogConfig::new()
            .with_level(level)
            .with_console(true);

        // Enable file logging if requested
        if self.log_file {
            config = config.with_file(true);
            
            // Set custom file path if provided
            if let Some(ref file_path) = self.log_file_path {
                config = config.with_file_path(PathBuf::from(file_path));
            }
        }

        crate::logging::init_logging(config)
    }
}

/// Helper function to parse args and handle common errors
pub fn parse_args() -> Result<CommonArgs, Box<dyn std::error::Error>> {
    let args = CommonArgs::parse();
    args.validate()?;
    Ok(args)
}

/// Helper function to get connection info with error handling
pub fn get_connection_info() -> Result<(String, String, String, Option<String>), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    args.get_connection_info()
} 
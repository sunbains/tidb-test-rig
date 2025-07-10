// Macro to generate CLI arguments based on example type
#[macro_export]
macro_rules! generate_cli_args {
    // Simple connection example
    (simple_connection) => {
        #[derive(Parser)]
        #[command(name = "tidb-simple-connection")]
        #[command(about = "TiDB simple connection test")]
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
            
            /// Skip password prompt (for automated testing)
            #[arg(long)]
            pub no_password_prompt: bool,
            
            /// Password from command line (alternative to prompt)
            #[arg(long)]
            pub password: Option<String>,
        }
    };

    // Isolation test example
    (isolation_test) => {
        #[derive(Parser)]
        #[command(name = "tidb-isolation-test")]
        #[command(about = "TiDB repeatable read isolation test")]
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
            
            /// Number of test rows to create for isolation testing
            #[arg(long, default_value = "10")]
            pub test_rows: u32,
            
            /// Skip password prompt (for automated testing)
            #[arg(long)]
            pub no_password_prompt: bool,
            
            /// Password from command line (alternative to prompt)
            #[arg(long)]
            pub password: Option<String>,
        }
    };

    // Multi-connection example
    (multi_connection) => {
        #[derive(Parser)]
        #[command(name = "tidb-multi-connection")]
        #[command(about = "TiDB multi-connection test")]
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
            
            /// Number of connections to create for multi-connection tests
            #[arg(long, default_value = "2")]
            pub connection_count: u32,
            
            /// Skip password prompt (for automated testing)
            #[arg(long)]
            pub no_password_prompt: bool,
            
            /// Password from command line (alternative to prompt)
            #[arg(long)]
            pub password: Option<String>,
        }
    };

    // Default case
    () => {
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
        }
    };
}

// Macro to generate CLI implementation based on example type
#[macro_export]
macro_rules! generate_cli_impl {
    (simple_connection) => {
        impl CommonArgs {
            pub fn get_password(&self) -> Result<String, Box<dyn std::error::Error>> {
                if let Some(ref password) = self.password {
                    return Ok(password.clone());
                }
                
                if let Ok(password) = std::env::var("TIDB_PASSWORD") {
                    return Ok(password);
                }
                
                if !self.no_password_prompt {
                    return Ok(rpassword::prompt_password("Password: ")?);
                }
                
                Err("No password provided and password prompt is disabled".into())
            }
            
            pub fn get_connection_info(&self) -> Result<(String, String, String, Option<String>), Box<dyn std::error::Error>> {
                let password = self.get_password()?;
                Ok((
                    std::env::var("TIDB_HOST").unwrap_or_else(|_| self.host.clone()),
                    std::env::var("TIDB_USER").unwrap_or_else(|_| self.user.clone()),
                    password,
                    std::env::var("TIDB_DATABASE").ok().or(self.database.clone())
                ))
            }
            
            pub fn print_connection_info(&self) {
                println!("Connection Info:");
                println!("  Host: {}", self.host);
                println!("  User: {}", self.user);
                println!("  Database: {}", self.database.as_deref().unwrap_or("(not specified)"));
            }
        }
    };

    (isolation_test) => {
        impl CommonArgs {
            pub fn get_password(&self) -> Result<String, Box<dyn std::error::Error>> {
                if let Some(ref password) = self.password {
                    return Ok(password.clone());
                }
                
                if let Ok(password) = std::env::var("TIDB_PASSWORD") {
                    return Ok(password);
                }
                
                if !self.no_password_prompt {
                    return Ok(rpassword::prompt_password("Password: ")?);
                }
                
                Err("No password provided and password prompt is disabled".into())
            }
            
            pub fn get_connection_info(&self) -> Result<(String, String, String, Option<String>), Box<dyn std::error::Error>> {
                let password = self.get_password()?;
                Ok((
                    std::env::var("TIDB_HOST").unwrap_or_else(|_| self.host.clone()),
                    std::env::var("TIDB_USER").unwrap_or_else(|_| self.user.clone()),
                    password,
                    std::env::var("TIDB_DATABASE").ok().or(self.database.clone())
                ))
            }
            
            pub fn print_connection_info(&self) {
                println!("Connection Info:");
                println!("  Host: {}", self.host);
                println!("  User: {}", self.user);
                println!("  Database: {}", self.database.as_deref().unwrap_or("(not specified)"));
                println!("  Test Rows: {}", self.test_rows);
            }
        }
    };

    (multi_connection) => {
        impl CommonArgs {
            pub fn get_password(&self) -> Result<String, Box<dyn std::error::Error>> {
                if let Some(ref password) = self.password {
                    return Ok(password.clone());
                }
                
                if let Ok(password) = std::env::var("TIDB_PASSWORD") {
                    return Ok(password);
                }
                
                if !self.no_password_prompt {
                    return Ok(rpassword::prompt_password("Password: ")?);
                }
                
                Err("No password provided and password prompt is disabled".into())
            }
            
            pub fn get_connection_info(&self) -> Result<(String, String, String, Option<String>), Box<dyn std::error::Error>> {
                let password = self.get_password()?;
                Ok((
                    std::env::var("TIDB_HOST").unwrap_or_else(|_| self.host.clone()),
                    std::env::var("TIDB_USER").unwrap_or_else(|_| self.user.clone()),
                    password,
                    std::env::var("TIDB_DATABASE").ok().or(self.database.clone())
                ))
            }
            
            pub fn print_connection_info(&self) {
                println!("Connection Info:");
                println!("  Host: {}", self.host);
                println!("  User: {}", self.user);
                println!("  Database: {}", self.database.as_deref().unwrap_or("(not specified)"));
                println!("  Monitor Duration: {}s", self.monitor_duration);
                println!("  Connection Count: {}", self.connection_count);
            }
        }
    };

    () => {
        impl CommonArgs {
            pub fn get_password(&self) -> Result<String, Box<dyn std::error::Error>> {
                if let Some(ref password) = self.password {
                    return Ok(password.clone());
                }
                
                if let Ok(password) = std::env::var("TIDB_PASSWORD") {
                    return Ok(password);
                }
                
                if !self.no_password_prompt {
                    return Ok(rpassword::prompt_password("Password: ")?);
                }
                
                Err("No password provided and password prompt is disabled".into())
            }
            
            pub fn get_connection_info(&self) -> Result<(String, String, String, Option<String>), Box<dyn std::error::Error>> {
                let password = self.get_password()?;
                Ok((
                    std::env::var("TIDB_HOST").unwrap_or_else(|_| self.host.clone()),
                    std::env::var("TIDB_USER").unwrap_or_else(|_| self.user.clone()),
                    password,
                    std::env::var("TIDB_DATABASE").ok().or(self.database.clone())
                ))
            }
            
            pub fn print_connection_info(&self) {
                println!("Connection Info:");
                println!("  Host: {}", self.host);
                println!("  User: {}", self.user);
                println!("  Database: {}", self.database.as_deref().unwrap_or("(not specified)"));
                println!("  Monitor Duration: {}s", self.monitor_duration);
            }
        }
    };
} 
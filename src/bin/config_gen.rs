//!
//! # TiDB Configuration Generator Binary
//!
//! This binary generates configuration files for the TiDB testing framework, providing a convenient
//! way to create properly formatted configuration files with sensible defaults and customization options.
//!
//! ## Overview
//!
//! The configuration generator creates JSON or TOML configuration files that can be used by all
//! the testing binaries in this framework. It uses a builder pattern for easy configuration creation
//! and provides sensible defaults while allowing full customization of all settings.
//!
//! This tool is useful for:
//! - **Quick Setup**: Generate working configuration files with minimal effort
//! - **Environment-Specific Configs**: Create different configs for development, staging, and production
//! - **Template Creation**: Generate base configurations that can be further customized
//! - **Automation**: Integrate configuration generation into CI/CD pipelines
//!
//! ## Features
//!
//! - **Multiple Output Formats**: Supports both JSON and TOML configuration formats
//! - **Builder Pattern**: Clean, fluent API for building configurations
//! - **Sensible Defaults**: Provides reasonable defaults for all settings
//! - **Full Customization**: Override any setting via command-line arguments
//! - **Validation**: Ensures generated configurations are valid and complete
//!
//! ## CLI Options
//!
//! - `--output, -o`: Output file path (default: tidb_config.json)
//! - `--format, -f`: Output format: json or toml (default: json)
//! - `--host`: Database host (default: localhost:4000)
//! - `--username`: Database username (default: root)
//! - `--database`: Database name (optional)
//! - `--log-level`: Log level (default: info)
//!
//! ## Usage
//!
//! ```bash
//! # Generate default JSON configuration
//! cargo run --bin config_gen
//!
//! # Generate TOML configuration with custom settings
//! cargo run --bin config_gen -- --format toml --host my-tidb:4000 --username myuser --database mydb
//!
//! # Generate configuration with custom output path
//! cargo run --bin config_gen -- --output my_config.json --host prod-tidb:4000 --log-level debug
//! ```
//!
//! ## Output
//!
//! The generator creates configuration files with this structure:
//! - **Database Settings**: Host, username, database name, connection pool settings
//! - **Logging Configuration**: Log level, format, file/console output settings
//! - **Test Configuration**: Test-specific settings like number of rows, timeouts
//!
//! ## Integration
//!
//! Generated configuration files can be used with all testing binaries:
//! - `cargo run --bin basic -- -c tidb_config.json`
//! - `cargo run --bin job_monitor --features import_jobs -- -c tidb_config.json`
//! - `cargo run --bin isolation --features isolation_test -- -c tidb_config.json`
//!
//! ## Examples
//!
//! ### Development Configuration
//! ```bash
//! cargo run --bin config_gen -- --host localhost:4000 --username root --log-level debug
//! ```
//!
//! ### Production Configuration
//! ```bash
//! cargo run --bin config_gen -- --format toml --host prod-tidb:4000 --username appuser --database production --log-level warn
//! ```

use test_rig::{ConfigBuilder, apply_extensions_to_command, apply_extensions_to_config, print_extensions_help};
use clap::Command;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build the CLI command with extensions
    let app = Command::new("config-gen")
        .about("Generate TiDB configuration files")
        .arg(
            clap::Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file path")
                .default_value("tidb_config.json")
        )
        .arg(
            clap::Arg::new("format")
                .short('f')
                .long("format")
                .help("Output format (json, toml)")
                .default_value("json")
        )
        .arg(
            clap::Arg::new("host")
                .long("host")
                .help("Database host")
                .default_value("localhost:4000")
        )
        .arg(
            clap::Arg::new("username")
                .long("username")
                .help("Database username")
                .default_value("root")
        )
        .arg(
            clap::Arg::new("database")
                .long("database")
                .help("Database name")
        )
        .arg(
            clap::Arg::new("log-level")
                .long("log-level")
                .help("Log level")
                .default_value("info")
        );
    
    // Apply extensions to the command
    let app = apply_extensions_to_command(app);
    
    // Parse arguments
    let args = app.get_matches();
    
    // Extract core arguments
    let output = args.get_one::<String>("output").unwrap().to_string();
    let format = args.get_one::<String>("format").unwrap().to_string();
    let host = args.get_one::<String>("host").unwrap().to_string();
    let username = args.get_one::<String>("username").unwrap().to_string();
    let database = args.get_one::<String>("database").map(|s| s.to_string());
    let log_level = args.get_one::<String>("log-level").unwrap().to_string();
    
    // Create configuration using builder pattern
    let mut builder = ConfigBuilder::new()
        .host(host)
        .username(username)
        .log_level(log_level);
    
    if let Some(database) = database {
        builder = builder.database(database);
    }
    
    let mut config = builder.build();
    
    // Apply extensions to the configuration
    apply_extensions_to_config(&args, &mut config)?;
    
    // Determine output format
    let mut output_path = PathBuf::from(output);
    if format == "toml" {
        output_path.set_extension("toml");
    } else if format == "json" {
        output_path.set_extension("json");
    }
    
    // Save configuration
    config.save_to_file(&output_path)?;
    
    println!("Configuration saved to: {}", output_path.display());
    println!("You can now use this configuration file with:");
    println!("  cargo run --bin basic -- -c {}", output_path.display());
    println!("  cargo run --bin job_monitor --features import_jobs -- -c {}", output_path.display());
    println!("  cargo run --bin isolation --features isolation_test -- -c {}", output_path.display());
    
    // Print extension help if any extensions are registered
    print_extensions_help();
    
    Ok(())
} 

#[cfg(test)]
mod tests {
    use super::*;
    use test_rig::config::{AppConfig, ConfigBuilder};
    use tempfile::NamedTempFile;
    use std::fs;
    use serial_test::serial;

        #[test]
    fn test_args_parsing() {
        let app = Command::new("test-bin")
            .arg(clap::Arg::new("output").short('o').long("output"))
            .arg(clap::Arg::new("format").short('f').long("format"))
            .arg(clap::Arg::new("host").long("host"))
            .arg(clap::Arg::new("username").long("username"))
            .arg(clap::Arg::new("database").long("database"))
            .arg(clap::Arg::new("log-level").long("log-level"));
        
        let app = apply_extensions_to_command(app);
        let args = app.get_matches_from([
            "test-bin", 
            "--output", "test_config.json",
            "--format", "json",
            "--host", "testhost:4000",
            "--username", "testuser",
            "--database", "testdb",
            "--log-level", "debug"
        ]);
        
        assert_eq!(args.get_one::<String>("output").unwrap(), "test_config.json");
        assert_eq!(args.get_one::<String>("format").unwrap(), "json");
        assert_eq!(args.get_one::<String>("host").unwrap(), "testhost:4000");
        assert_eq!(args.get_one::<String>("username").unwrap(), "testuser");
        assert_eq!(args.get_one::<String>("database").unwrap(), "testdb");
        assert_eq!(args.get_one::<String>("log-level").unwrap(), "debug");
    }

    #[test]
    fn test_args_defaults() {
        let app = Command::new("test-bin")
            .arg(clap::Arg::new("output").short('o').long("output").default_value("tidb_config.json"))
            .arg(clap::Arg::new("format").short('f').long("format").default_value("json"))
            .arg(clap::Arg::new("host").long("host").default_value("localhost:4000"))
            .arg(clap::Arg::new("username").long("username").default_value("root"))
            .arg(clap::Arg::new("database").long("database"))
            .arg(clap::Arg::new("log-level").long("log-level").default_value("info"));
        
        let app = apply_extensions_to_command(app);
        let args = app.get_matches_from(["test-bin"]);
        
        assert_eq!(args.get_one::<String>("output").unwrap(), "tidb_config.json");
        assert_eq!(args.get_one::<String>("format").unwrap(), "json");
        assert_eq!(args.get_one::<String>("host").unwrap(), "localhost:4000");
        assert_eq!(args.get_one::<String>("username").unwrap(), "root");
        assert_eq!(args.get_one::<String>("database"), None);
        assert_eq!(args.get_one::<String>("log-level").unwrap(), "info");
    }

    #[test]
    #[serial]
    fn test_config_builder_integration() {
        let config = ConfigBuilder::new()
            .host("testhost:4000")
            .username("testuser")
            .database("testdb")
            .log_level("debug")
            .build();
        
        assert_eq!(config.database.host, "testhost:4000");
        assert_eq!(config.database.username, "testuser");
        assert_eq!(config.database.database, Some("testdb".to_string()));
        assert_eq!(config.logging.level, "debug");
    }

    #[test]
    #[serial]
    fn test_config_file_generation() {
        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path().with_extension("json");
        
        let app = Command::new("test-bin")
            .arg(clap::Arg::new("output").short('o').long("output"))
            .arg(clap::Arg::new("format").short('f').long("format"))
            .arg(clap::Arg::new("host").long("host"))
            .arg(clap::Arg::new("username").long("username"))
            .arg(clap::Arg::new("database").long("database"))
            .arg(clap::Arg::new("log-level").long("log-level").default_value("info"));
        
        let app = apply_extensions_to_command(app);
        let args = app.get_matches_from([
            "test-bin",
            "--output", output_path.to_str().unwrap(),
            "--host", "genhost:4000",
            "--username", "genuser"
        ]);
        
        // Simulate the main function logic
        let builder = ConfigBuilder::new()
            .host(args.get_one::<String>("host").unwrap())
            .username(args.get_one::<String>("username").unwrap())
            .log_level(args.get_one::<String>("log-level").unwrap());
        
        let config = builder.build();
        
        // Save and verify
        config.save_to_file(&output_path).unwrap();
        assert!(output_path.exists());
        
        // Load and verify the saved config
        let loaded_config = AppConfig::from_file(&output_path).unwrap();
        assert_eq!(loaded_config.database.host, "genhost:4000");
        assert_eq!(loaded_config.database.username, "genuser");
    }

    #[test]
    #[serial]
    fn test_toml_format_generation() {
        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path().with_extension("toml");
        
        let app = Command::new("test-bin")
            .arg(clap::Arg::new("output").short('o').long("output"))
            .arg(clap::Arg::new("format").short('f').long("format"))
            .arg(clap::Arg::new("host").long("host"))
            .arg(clap::Arg::new("username").long("username"))
            .arg(clap::Arg::new("database").long("database"))
            .arg(clap::Arg::new("log-level").long("log-level").default_value("info"));
        
        let app = apply_extensions_to_command(app);
        let args = app.get_matches_from([
            "test-bin",
            "--output", output_path.to_str().unwrap(),
            "--format", "toml",
            "--host", "tomlhost:4000"
        ]);
        
        let config = ConfigBuilder::new()
            .host(args.get_one::<String>("host").unwrap())
            .username(args.get_one::<String>("username").unwrap())
            .log_level(args.get_one::<String>("log-level").unwrap())
            .build();
        
        config.save_to_file(&output_path).unwrap();
        assert!(output_path.exists());
        
        // Verify TOML content
        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("tomlhost:4000"));
        assert!(content.contains("[database]"));
        assert!(content.contains("[logging]"));
        assert!(content.contains("[test]"));
    }

    #[test]
    fn test_config_validation() {
        let config = ConfigBuilder::new()
            .host("localhost:4000")
            .username("root")
            .build();
        
        assert!(config.validate().is_ok());
    }
} 
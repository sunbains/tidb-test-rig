use connect::ConfigBuilder;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "config-gen")]
#[command(about = "Generate TiDB configuration files")]
struct Args {
    /// Output file path
    #[arg(short, long, default_value = "tidb_config.json")]
    output: PathBuf,
    
    /// Output format (json, toml)
    #[arg(short, long, default_value = "json")]
    format: String,
    
    /// Database host
    #[arg(long, default_value = "localhost:4000")]
    host: String,
    
    /// Database username
    #[arg(long, default_value = "root")]
    username: String,
    
    /// Database name
    #[arg(long)]
    database: Option<String>,
    
    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
    
    /// Test rows
    #[arg(long, default_value = "10")]
    test_rows: u32,
    
    /// Monitor duration
    #[arg(long, default_value = "300")]
    monitor_duration: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Create configuration using builder pattern
    let mut builder = ConfigBuilder::new()
        .host(args.host)
        .username(args.username)
        .log_level(args.log_level)
        .test_rows(args.test_rows)
        .monitor_duration(args.monitor_duration);
    
    if let Some(database) = args.database {
        builder = builder.database(database);
    }
    
    let config = builder.build();
    
    // Determine output format
    let mut output_path = args.output;
    if args.format == "toml" {
        output_path.set_extension("toml");
    } else if args.format == "json" {
        output_path.set_extension("json");
    }
    
    // Save configuration
    config.save_to_file(&output_path)?;
    
    println!("Configuration saved to: {}", output_path.display());
    println!("You can now use this configuration file with:");
    println!("  cargo run --bin basic -- -c {}", output_path.display());
    println!("  cargo run --bin job_monitor --features import_jobs -- -c {}", output_path.display());
    println!("  cargo run --bin isolation --features isolation_test -- -c {}", output_path.display());
    
    Ok(())
} 
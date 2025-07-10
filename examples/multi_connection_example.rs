use connect::{MultiConnectionStateMachine, ConnectionCoordinator, ConnectionInfo, GlobalConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Multi-Connection TiDB Testing Example");
    println!("=====================================");

    // Create global configuration
    let config = GlobalConfig {
        test_duration: 120,        // 2 minutes
        coordination_timeout: 30,   // 30 seconds
        max_connections: 3,         // Max 3 connections
    };

    // Create coordinator
    let coordinator = ConnectionCoordinator::new(config);
    let mut multi_sm = MultiConnectionStateMachine::new(coordinator);

    // Define multiple connections
    let connections = vec![
        ("primary".to_string(), ConnectionInfo {
            host: "tidb-primary.example.com".to_string(),
            port: 4000,
            username: "user1".to_string(),
            password: "password1".to_string(),
            database: Some("test_db".to_string()),
            connection: None,
        }),
        ("secondary".to_string(), ConnectionInfo {
            host: "tidb-secondary.example.com".to_string(),
            port: 4000,
            username: "user2".to_string(),
            password: "password2".to_string(),
            database: Some("test_db".to_string()),
            connection: None,
        }),
        ("backup".to_string(), ConnectionInfo {
            host: "tidb-backup.example.com".to_string(),
            port: 4000,
            username: "user3".to_string(),
            password: "password3".to_string(),
            database: Some("backup_db".to_string()),
            connection: None,
        }),
    ];

    // Add connections to the multi-state machine
    for (connection_id, connection_info) in connections {
        println!("Adding connection: {}", connection_id);
        multi_sm.add_connection(connection_id, connection_info);
    }

    // Run all connections concurrently
    println!("\nStarting concurrent connection testing...");
    if let Err(e) = multi_sm.run_all().await {
        eprintln!("Failed to run multi-connection test: {}", e);
        return Err(Box::new(std::io::Error::other(e.to_string())) as Box<dyn std::error::Error>);
    }

    // Check results
    println!("\n=== Final Results ===");
    
    let shared_state = multi_sm.get_shared_state();
    if let Ok(state) = shared_state.lock() {
        println!("Connection Status:");
        for (conn_id, status) in &state.connection_status {
            println!("  {}: {:?} - {}", conn_id, status.status, status.host);
            if let Some(error) = &status.error_message {
                println!("    Error: {}", error);
            }
        }

        println!("\nActive Import Jobs:");
        for job in &state.import_jobs {
            if job.end_time.is_none() {
                println!("  Job {} on {}: {} - {}", 
                    job.job_id, job.connection_id, job.phase, job.status);
            }
        }

        println!("\nCoordination Events:");
        for event in &state.coordination_events {
            println!("  {:?}", event);
        }
    }

    println!("\n✓ Multi-connection testing completed!");
    Ok(())
} 
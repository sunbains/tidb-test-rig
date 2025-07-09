use mysql::prelude::*;
use mysql::{Pool, PooledConn, OptsBuilder};

/// Parse host and port from a string in format "hostname:port"
pub fn parse_host_port(host_port: &str) -> Result<(String, u16), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = host_port.split(':').collect();
    if parts.len() != 2 {
        return Err("Host must be in format hostname:port".into());
    }
    
    let host = parts[0].to_string();
    let port = parts[1].parse::<u16>()
        .map_err(|_| "Invalid port number")?;
    
    Ok((host, port))
}

/// Parse connection string (alias for parse_host_port for compatibility)
pub fn parse_connection_string(connection_string: &str) -> Result<(String, u16), Box<dyn std::error::Error>> {
    parse_host_port(connection_string)
}

/// Parse username and password from a string in format "username:password"
pub fn parse_user_pass(user_pass: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = user_pass.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err("User must be in format username:password".into());
    }
    
    Ok((parts[0].to_string(), parts[1].to_string()))
}

/// Create a connection pool to TiDB using the provided parameters
pub fn create_connection_pool(host: &str, port: u16, user: &str, password: &str, database: Option<&str>) 
    -> Result<Pool, Box<dyn std::error::Error>> {
    
    let mut builder = OptsBuilder::new()
        .ip_or_hostname(Some(host))
        .tcp_port(port)
        .user(Some(user))
        .pass(Some(password));
    
    if let Some(db) = database {
        builder = builder.db_name(Some(db));
    }
    
    let pool = Pool::new(builder)?;
    Ok(pool)
}

/// Create a connection to TiDB using the provided parameters
pub fn create_connection(host: &str, port: u16, user: &str, password: &str, database: Option<&str>) 
    -> Result<PooledConn, Box<dyn std::error::Error>> {
    
    let pool = create_connection_pool(host, port, user, password, database)?;
    let conn = pool.get_conn()?;
    
    Ok(conn)
}

/// Verify that a database exists
pub fn verify_database_exists(conn: &mut PooledConn, database: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let query = "SELECT SCHEMA_NAME FROM INFORMATION_SCHEMA.SCHEMATA WHERE SCHEMA_NAME = ?";
    let result: Option<String> = conn.exec_first(query, (database,))?;
    Ok(result.is_some())
}

/// Test the connection by executing a simple query
pub fn test_connection(conn: &mut PooledConn) -> Result<(), Box<dyn std::error::Error>> {
    let _result: Option<i32> = conn.query_first("SELECT 1")?;
    Ok(())
}

/// Get the server version
pub fn get_server_version(conn: &mut PooledConn) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let version: Option<String> = conn.query_first("SELECT VERSION()")?;
    Ok(version)
} 
//! # Connection Management
//!
//! Low-level database connection utilities and parsing functions.
//! Provides connection pool creation, connection testing, and host/port parsing.

use crate::errors::{ConnectionError, Result};
use mysql::prelude::*;
use mysql::{OptsBuilder, Pool, PooledConn};

/// Parse host and port from a string in format "host:port"
///
/// # Errors
///
/// Returns an error if the string format is invalid or port cannot be parsed.
pub fn parse_host_port(host_port: &str) -> Result<(String, u16)> {
    let parts: Vec<&str> = host_port.split(':').collect();
    if parts.len() != 2 {
        return Err(ConnectionError::ConnectFailed {
            host: host_port.to_string(),
            port: 0,
            message: "Host must be in format hostname:port".to_string(),
        }
        .into());
    }

    let host = parts[0].to_string();
    let port = parts[1]
        .parse::<u16>()
        .map_err(|_| ConnectionError::ConnectFailed {
            host: host.clone(),
            port: 0,
            message: "Invalid port number".to_string(),
        })?;

    Ok((host, port))
}

/// Parse connection string in format "hostname:port"
///
/// # Errors
///
/// Returns an error if the connection string format is invalid.
pub fn parse_connection_string(connection_string: &str) -> Result<(String, u16)> {
    parse_host_port(connection_string)
}

/// Parse username and password from string in format "user:pass"
///
/// # Errors
///
/// Returns an error if the format is invalid.
pub fn parse_user_pass(user_pass: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = user_pass.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(ConnectionError::AuthFailed {
            user: user_pass.to_string(),
            message: "User must be in format username:password".to_string(),
        }
        .into());
    }

    Ok((parts[0].to_string(), parts[1].to_string()))
}

/// Create a connection pool
///
/// # Errors
///
/// Returns an error if the connection pool cannot be created.
pub fn create_connection_pool(
    host: &str,
    port: u16,
    user: &str,
    password: &str,
    database: Option<&str>,
) -> Result<Pool> {
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

/// Create a single connection
///
/// # Errors
///
/// Returns an error if the connection cannot be established.
pub fn create_connection(
    host: &str,
    port: u16,
    user: &str,
    password: &str,
    database: Option<&str>,
) -> Result<PooledConn> {
    let pool = create_connection_pool(host, port, user, password, database)?;
    let conn = pool.get_conn()?;

    Ok(conn)
}

/// Verify if a database exists
///
/// # Errors
///
/// Returns an error if the query fails.
pub fn verify_database_exists(conn: &mut PooledConn, database: &str) -> Result<bool> {
    let query = "SELECT SCHEMA_NAME FROM INFORMATION_SCHEMA.SCHEMATA WHERE SCHEMA_NAME = ?";
    let result: Option<String> = conn.exec_first(query, (database,))?;
    Ok(result.is_some())
}

/// Test database connection
///
/// # Errors
///
/// Returns an error if the connection test fails.
pub fn test_connection(conn: &mut PooledConn) -> Result<()> {
    let _result: Option<i32> = conn.query_first("SELECT 1")?;
    Ok(())
}

/// Get server version information
///
/// # Errors
///
/// Returns an error if the version query fails.
pub fn get_server_version(conn: &mut PooledConn) -> Result<Option<String>> {
    let version: Option<String> = conn.query_first("SELECT VERSION()")?;
    Ok(version)
}

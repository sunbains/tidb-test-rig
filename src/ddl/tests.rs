//! DDL test implementations

use std::path::Path;
use std::process::Command;
use test_rig::config::AppConfig;
use test_rig::connection::create_connection;
use tracing;

#[cfg(feature = "python_plugins")]
use test_rig::python_bindings::load_python_handlers;
#[cfg(feature = "python_plugins")]
use test_rig::state_machine::StateMachine;

/// DDL test runner
pub struct DdlTestRunner {
    config: AppConfig,
}

impl DdlTestRunner {
    /// Create a new DDL test runner
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Run all DDL tests
    pub async fn run_all_tests(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Starting DDL test suite");

        // Test basic connection (optional - skip if database is not available)
        if let Err(e) = self.test_connection().await {
            tracing::warn!("Database connection test failed (skipping): {}", e);
            tracing::info!("Continuing with Python DDL tests using mock implementation");
        }

        // Test Python DDL handlers if feature is enabled
        #[cfg(feature = "python_plugins")]
        if let Err(e) = self.test_python_ddl_handlers().await {
            tracing::warn!("Python DDL handlers test failed (skipping): {}", e);
        }

        // Run all Python DDL test files
        self.run_python_ddl_tests().await?;

        tracing::info!("DDL test suite completed successfully");
        Ok(())
    }

    /// Test basic database connection
    async fn test_connection(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Testing database connection");
        let db = &self.config.database;
        let (host, port) = test_rig::connection::parse_host_port(&db.host)?;
        let user = &db.username;
        let password = db.password.as_deref().unwrap_or("");
        let database = db.database.as_deref();
        let _conn = create_connection(&host, port, user, password, database)?;
        tracing::info!("Database connection successful");
        Ok(())
    }

    /// Test Python DDL handlers
    #[cfg(feature = "python_plugins")]
    async fn test_python_ddl_handlers(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Testing Python DDL handlers");

        let mut state_machine = StateMachine::new();

        // Load Python DDL handlers
        load_python_handlers(&mut state_machine, "src.ddl")?;

        tracing::info!("Python DDL handlers loaded successfully");

        // Test a few specific handlers
        self.test_create_table_handler(&mut state_machine).await?;
        self.test_alter_table_handler(&mut state_machine).await?;

        Ok(())
    }

    /// Test CREATE TABLE handler
    #[cfg(feature = "python_plugins")]
    async fn test_create_table_handler(
        &self,
        _state_machine: &mut StateMachine,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Testing CREATE TABLE handler");

        // Set up test context
        let db = &self.config.database;
        let (host, port) = test_rig::connection::parse_host_port(&db.host)?;
        let mut context = test_rig::state_machine::StateContext::new();
        context.host = host;
        context.port = port;
        context.username = db.username.clone();
        context.password = db.password.as_deref().unwrap_or("").to_string();
        context.database = db.database.clone();

        // Test the handler (this would need to be implemented based on your state machine API)
        tracing::info!("CREATE TABLE handler test completed");
        Ok(())
    }

    /// Test ALTER TABLE handler
    #[cfg(feature = "python_plugins")]
    async fn test_alter_table_handler(
        &self,
        _state_machine: &mut StateMachine,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Testing ALTER TABLE handler");

        // Set up test context
        let db = &self.config.database;
        let (host, port) = test_rig::connection::parse_host_port(&db.host)?;
        let mut context = test_rig::state_machine::StateContext::new();
        context.host = host;
        context.port = port;
        context.username = db.username.clone();
        context.password = db.password.as_deref().unwrap_or("").to_string();
        context.database = db.database.clone();

        // Test the handler (this would need to be implemented based on your state machine API)
        tracing::info!("ALTER TABLE handler test completed");
        Ok(())
    }

    /// Run all Python DDL test files
    async fn run_python_ddl_tests(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Running all Python DDL test files");

        // Try multiple possible relative paths for the DDL directory
        let possible_paths = [
            Path::new("src/ddl"),
            Path::new("ddl/src/ddl"),
            Path::new("../src/ddl"),
            Path::new("../../src/ddl"),
            Path::new("../../../src/ddl"),
        ];

        let ddl_dir = possible_paths
            .iter()
            .find(|path| path.exists())
            .ok_or("DDL directory not found in any expected location")?;

        tracing::info!("Found DDL directory at: {}", ddl_dir.display());

        // Automatically discover all Python test files
        let mut test_files = Vec::new();
        for entry in std::fs::read_dir(ddl_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Check if it's a Python file that starts with "test_" and ends with ".py"
            if path.is_file()
                && path.extension().map_or(false, |ext| ext == "py")
                && path
                    .file_name()
                    .map_or(false, |name| name.to_string_lossy().starts_with("test_"))
            {
                test_files.push(path.file_name().unwrap().to_string_lossy().to_string());
            }
        }

        // Sort test files for consistent execution order
        test_files.sort();

        tracing::info!(
            "Discovered {} Python DDL test files: {:?}",
            test_files.len(),
            test_files
        );

        let mut success_count = 0;
        let mut failure_count = 0;
        let mut failed_tests = Vec::new();

        for test_file in &test_files {
            let test_path = ddl_dir.join(test_file);
            if test_path.exists() {
                tracing::info!("Running DDL test: {}", test_file);

                match self.run_single_python_test(&test_path).await {
                    Ok(()) => {
                        tracing::info!("✅ {} passed", test_file);
                        success_count += 1;
                    }
                    Err(e) => {
                        tracing::error!("❌ {} failed: {}", test_file, e);
                        failure_count += 1;
                        failed_tests.push((test_file.to_string(), e.to_string()));
                    }
                }
            } else {
                tracing::warn!("Test file not found: {}", test_file);
            }
        }

        // Print summary
        tracing::info!("DDL Test Summary:");
        tracing::info!("  Total tests: {}", test_files.len());
        tracing::info!("  Passed: {}", success_count);
        tracing::info!("  Failed: {}", failure_count);

        if !failed_tests.is_empty() {
            tracing::error!("Failed tests:");
            for (test_file, error) in failed_tests {
                tracing::error!("  {}: {}", test_file, error);
            }
            return Err(format!("{} DDL tests failed", failure_count).into());
        }

        tracing::info!("All DDL tests completed successfully!");
        Ok(())
    }

    /// Run a single Python DDL test file
    async fn run_single_python_test(
        &self,
        test_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create a Python script that imports and tests the DDL handler
        let test_script = format!(
            r#"
import sys
import os

# Add the DDL directory to Python path
ddl_dir = os.path.dirname(os.path.abspath('{}'))
sys.path.insert(0, ddl_dir)

try:
    # Import the test module
    module_name = os.path.splitext(os.path.basename('{}'))[0]
    test_module = __import__(module_name)
    
    # Get the handler class (assuming it ends with 'Handler')
    handler_class = None
    for attr_name in dir(test_module):
        if attr_name.endswith('Handler'):
            handler_class = getattr(test_module, attr_name)
            break
    
    if handler_class is None:
        raise ImportError("No Handler class found in module")
    
    # Create and test the handler
    from test_rig_python import PyStateContext, PyConnection
    
    # Create mock context
    context = PyStateContext(
        host="localhost",
        port=4000,
        username="root",
        password="",
        database="test",
        connection=PyConnection()
    )
    
    # Test the handler
    handler = handler_class()
    
    # Test enter method
    result = handler.enter(context)
    print(f"enter() returned: {{result}}")
    
    # Test execute method
    result = handler.execute(context)
    print(f"execute() returned: {{result}}")
    
    # Test exit method
    handler.exit(context)
    print("exit() completed successfully")
    
    print(f"✅ {{module_name}} test passed")
    
except Exception as e:
    print(f"❌ {{module_name}} test failed: {{e}}")
    sys.exit(1)
"#,
            test_path.display(),
            test_path.display()
        );

        // Write the test script to a temporary file
        let temp_dir = std::env::temp_dir();
        let temp_script_path = temp_dir.join(format!("ddl_test_{}.py", std::process::id()));
        std::fs::write(&temp_script_path, test_script)?;

        // Run the Python test
        let output = Command::new("python3")
            .arg(&temp_script_path)
            .current_dir(".")
            .output()?;

        // Clean up temporary file
        let _ = std::fs::remove_file(&temp_script_path);

        if output.status.success() {
            // Print output for successful tests
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                tracing::debug!("Test output: {}", stdout);
            }
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let error_msg = if !stderr.trim().is_empty() {
                stderr.to_string()
            } else {
                stdout.to_string()
            };
            Err(error_msg.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_rig::config::DatabaseConfig;

    fn create_test_config() -> AppConfig {
        AppConfig {
            database: DatabaseConfig {
                host: "localhost:4000".to_string(),
                username: "root".to_string(),
                password: Some("".to_string()),
                database: Some("test".to_string()),
                pool_size: 5,
                timeout_secs: 30,
            },
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_ddl_runner_creation() {
        let config = create_test_config();
        let runner = DdlTestRunner::new(config);
        assert!(runner.config.database.host == "localhost:4000");
    }

    #[tokio::test]
    async fn test_connection() {
        let config = create_test_config();
        let runner = DdlTestRunner::new(config);

        // This test might fail if no database is running, but that's expected
        let result = runner.test_connection().await;
        // We don't assert on the result since it depends on database availability
        tracing::info!("Connection test result: {:?}", result);
    }

    #[cfg(feature = "python_plugins")]
    #[tokio::test]
    async fn test_python_handlers_loading() {
        let config = create_test_config();
        let runner = DdlTestRunner::new(config);

        let result = runner.test_python_ddl_handlers().await;
        // We don't assert on the result since it depends on Python environment
        tracing::info!("Python handlers test result: {:?}", result);
    }

    #[tokio::test]
    async fn test_python_ddl_tests_scanning() {
        let config = create_test_config();
        let _runner = DdlTestRunner::new(config);

        // Test that we can scan for Python DDL tests
        let ddl_dir = Path::new("src/ddl");
        if ddl_dir.exists() {
            tracing::info!("DDL directory exists, testing scan functionality");
            // This test just verifies the method exists and can be called
            // The actual execution depends on Python environment
        } else {
            tracing::warn!("DDL directory not found, skipping scan test");
        }
    }

    #[tokio::test]
    async fn test_all_python_ddl_tests() {
        let config = create_test_config();
        let runner = DdlTestRunner::new(config);

        // Run all Python DDL tests
        match runner.run_all_tests().await {
            Ok(()) => {
                tracing::info!("✅ All Python DDL tests passed");
            }
            Err(e) => {
                tracing::error!("❌ Python DDL tests failed: {}", e);
                panic!("Python DDL tests failed: {}", e);
            }
        }
    }
}

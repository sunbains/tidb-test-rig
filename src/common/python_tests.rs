//! Common Python test infrastructure for test workspaces

use crate::config::DatabaseConfig;
use crate::state_machine::StateMachine;
use std::path::Path;

/// Common trait for Python test runners
pub trait PythonTestRunner: Send + Sync {
    fn name(&self) -> &str;
    fn test_dir(&self) -> &str;

    fn test_connection(
        &self,
    ) -> impl std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send {
        async {
            tracing::info!("Testing database connection");

            let config = DatabaseConfig {
                host: "testhost:4000".to_string(),
                username: "testuser".to_string(),
                password: Some("testpass".to_string()),
                database: Some("testdb".to_string()),
                pool_size: 5,
                timeout_secs: 30,
            };

            let (host, port) = crate::connection::parse_host_port(&config.host)?;
            let password = config.password.as_deref().unwrap_or("");
            let database = config.database.as_deref();

            let _connection = crate::connection::create_connection(
                &host,
                port,
                &config.username,
                password,
                database,
            )?;
            tracing::info!("Database connection successful");
            Ok(())
        }
    }

    fn test_python_handlers(
        &self,
    ) -> impl std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send {
        async {
            tracing::info!("Testing Python handlers");
            // Default implementation - can be overridden by specific test runners
            Ok(())
        }
    }

    fn test_create_table_handler(
        &self,
        _state_machine: &mut StateMachine,
    ) -> impl std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send {
        async {
            tracing::info!("Testing CREATE TABLE handler");
            // Default implementation - can be overridden by specific test runners
            Ok(())
        }
    }

    fn test_alter_table_handler(
        &self,
        _state_machine: &mut StateMachine,
    ) -> impl std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send {
        async {
            tracing::info!("Testing ALTER TABLE handler");
            // Default implementation - can be overridden by specific test runners
            Ok(())
        }
    }

    fn run_all_python_tests(
        &self,
    ) -> impl std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send {
        async {
            tracing::info!("Running all Python test files");

            let test_dir = self.test_dir();
            let test_files = self.discover_test_files(test_dir).await?;

            tracing::info!("Found {} test files", test_files.len());

            for test_file in test_files {
                tracing::info!("Running test: {}", test_file.display());
                self.run_single_python_test(&test_file).await?;
            }

            tracing::info!("All Python tests completed successfully");
            Ok(())
        }
    }

    fn discover_test_files(
        &self,
        test_dir: &str,
    ) -> impl std::future::Future<
        Output = Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error>>,
    > + Send {
        let test_dir = test_dir.to_string();
        async move {
            use std::fs;
            use std::path::PathBuf;

            let mut test_files = Vec::new();
            let test_path = PathBuf::from(&test_dir);

            if let Ok(entries) = fs::read_dir(&test_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(file_name) = path.file_name()
                        && let Some(name_str) = file_name.to_str()
                        && name_str.starts_with("test_")
                        && name_str.ends_with(".py")
                    {
                        test_files.push(path);
                    }
                }
            }

            test_files.sort();
            Ok(test_files)
        }
    }

    fn run_single_python_test(
        &self,
        test_path: &Path,
    ) -> impl std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send {
        async {
            // Create a Python script that imports and tests the handler
            let module_name = test_path.file_stem().unwrap().to_str().unwrap();
            let test_content = format!(
                r#"
import sys
import os
sys.path.insert(0, '{}')

try:
    import {}
    print(f"✅ Successfully imported {module_name}")
except Exception as e:
    print(f"❌ Failed to import {module_name}: {{e}}")
    sys.exit(1)

# Try to instantiate the handler
try:
    handler = {}.PyStateHandler()
    print(f"✅ Successfully instantiated handler for {module_name}")
except Exception as e:
    print(f"❌ Failed to instantiate handler for {module_name}: {{e}}")
    sys.exit(1)

print(f"✅ Test passed for {module_name}")
"#,
                test_path.parent().unwrap().to_str().unwrap(),
                module_name,
                module_name,
            );

            let temp_dir = std::env::temp_dir();
            let temp_script = temp_dir.join(format!(
                "test_{}.py",
                test_path.file_stem().unwrap().to_str().unwrap()
            ));

            std::fs::write(&temp_script, test_content)?;

            let output = std::process::Command::new("python3")
                .arg(&temp_script)
                .current_dir(test_path.parent().unwrap())
                .output()?;

            if output.status.success() {
                tracing::info!("✅ Test passed: {}", test_path.display());
                println!("✅ Test passed: {}", test_path.display());
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::error!("❌ Test failed: {} - {}", test_path.display(), stderr);
                println!("❌ Test failed: {} - {}", test_path.display(), stderr);
                return Err(format!("Test failed: {}", test_path.display()).into());
            }

            // Clean up temp file
            let _ = std::fs::remove_file(temp_script);

            Ok(())
        }
    }

    fn run_test_suite(
        &self,
    ) -> impl std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send {
        async {
            tracing::info!("Starting {} test suite", self.name());

            // Test database connection
            self.test_connection().await?;

            // Test Python handlers
            self.test_python_handlers().await?;

            // Run all Python tests
            self.run_all_python_tests().await?;

            tracing::info!("{} test suite completed successfully", self.name());
            println!("✅ {} test suite completed successfully", self.name());

            Ok(())
        }
    }
}

/// Configuration for a Python test suite
#[derive(Clone, Debug)]
pub struct PythonSuiteConfig {
    pub name: &'static str,
    pub test_dir: &'static str,
    pub module_prefix: &'static str,
}

/// List all Python test suites here
pub static PYTHON_SUITES: &[PythonSuiteConfig] = &[
    PythonSuiteConfig {
        name: "DDL",
        test_dir: "src/ddl",
        module_prefix: "src.ddl",
    },
    PythonSuiteConfig {
        name: "Scale",
        test_dir: "src/scale",
        module_prefix: "src.scale",
    },
    PythonSuiteConfig {
        name: "Txn",
        test_dir: "src/txn",
        module_prefix: "src.txn",
    },
    // Add more suites here as needed
];

impl PythonSuiteConfig {
    pub async fn run_suite(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Running Python test suite: {}", self.name);
        // Test database connection (optional, can be customized)
        // ...
        // Run all Python tests in the suite
        let test_files = PythonSuiteConfig::discover_test_files(self.test_dir).await?;
        tracing::info!("Found {} test files in {}", test_files.len(), self.test_dir);
        for test_file in test_files {
            tracing::info!("Running test: {}", test_file.display());
            PythonSuiteConfig::run_single_python_test(&test_file, self.module_prefix).await?;
        }
        tracing::info!("All Python tests completed successfully for {}", self.name);
        Ok(())
    }

    pub async fn discover_test_files(
        test_dir: &str,
    ) -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error>> {
        use std::fs;
        use std::path::PathBuf;
        let mut test_files = Vec::new();
        let test_path = PathBuf::from(test_dir);
        if let Ok(entries) = fs::read_dir(&test_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name()
                    && let Some(name_str) = file_name.to_str()
                    && name_str.starts_with("test_")
                    && name_str.ends_with(".py")
                    && name_str != "test_rig_python.py"
                {
                    test_files.push(path);
                }
            }
        }
        test_files.sort();
        Ok(test_files)
    }

    pub async fn run_single_python_test(
        test_path: &std::path::Path,
        _module_prefix: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let module_name = test_path.file_stem().unwrap().to_str().unwrap();
        let parent_dir = test_path.parent().unwrap();

        // Create a simple test script that executes the test file directly
        let test_content = format!(
            r#"
import sys
import os

# Add the project root to Python path so imports work correctly
project_root = os.path.abspath('.')
sys.path.insert(0, project_root)

try:
    # Execute the test file directly
    with open('{}', 'r') as f:
        test_code = f.read()
    
    # Create a namespace for the test
    test_namespace = {{}}
    
    # Execute the test file in the namespace
    exec(test_code, test_namespace)
    
    print(f"✅ Successfully executed {module_name}")
    
    # Look for handler classes in the namespace
    handler_classes = []
    for attr_name, attr_value in test_namespace.items():
        if attr_name.endswith('Handler') and not attr_name.startswith('_'):
            try:
                if hasattr(attr_value, '__bases__'):
                    # Check if it inherits from PyStateHandler
                    base_names = [base.__name__ for base in attr_value.__bases__]
                    if 'PyStateHandler' in base_names:
                        handler_classes.append(attr_name)
            except:
                pass

    if not handler_classes:
        print(f"❌ No handler classes found in {module_name}")
        sys.exit(1)

    print(f"✅ Found handler classes: {{', '.join(handler_classes)}}")

    # Try to instantiate the first handler class
    try:
        handler_class_name = handler_classes[0]
        handler_class = test_namespace[handler_class_name]
        handler = handler_class()
        print(f"✅ Successfully instantiated {{handler_class_name}} for {module_name}")
    except Exception as e:
        print(f"❌ Failed to instantiate handler for {module_name}: {{e}}")
        sys.exit(1)

    print(f"✅ Test passed for {module_name}")
    
except Exception as e:
    print(f"❌ Failed to execute {module_name}: {{e}}")
    import traceback
    traceback.print_exc()
    sys.exit(1)
"#,
            test_path.file_name().unwrap().to_str().unwrap(),
        );
        let temp_dir = std::env::temp_dir();
        let temp_script = temp_dir.join(format!(
            "test_{}.py",
            test_path.file_stem().unwrap().to_str().unwrap()
        ));
        std::fs::write(&temp_script, test_content)?;
        let output = std::process::Command::new("python3")
            .arg(&temp_script)
            .current_dir(parent_dir)
            .env("PYTHONPATH", std::env::current_dir().unwrap())
            .output()?;
        if output.status.success() {
            tracing::info!("✅ Test passed: {}", test_path.display());
            println!("✅ Test passed: {}", test_path.display());
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!(
                "❌ Test failed: {}\nstdout:\n{}\nstderr:\n{}",
                test_path.display(),
                stdout,
                stderr
            );
            println!(
                "❌ Test failed: {}\nstdout:\n{}\nstderr:\n{}",
                test_path.display(),
                stdout,
                stderr
            );
            return Err(format!("Test failed: {}", test_path.display()).into());
        }
        let _ = std::fs::remove_file(temp_script);
        Ok(())
    }
}

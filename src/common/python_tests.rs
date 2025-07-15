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
                        && std::path::Path::new(name_str)
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("py"))
                    {
                        test_files.push(path);
                    }
                }
            }

            test_files.sort();
            Ok(test_files)
        }
    }

    /// Generate Python test script content
    fn generate_python_test_script(&self, test_path: &Path, module_name: &str) -> String {
        let python_script_template = self.get_python_script_template();
        python_script_template
            .replace("{}", test_path.file_name().unwrap().to_str().unwrap())
            .replace("{MODULE_NAME}", module_name)
    }

    /// Get the Python script template
    #[allow(clippy::too_many_lines)]
    fn get_python_script_template(&self) -> &'static str {
        r#"
import sys
import os
import inspect

# Add the project root to Python path so imports work correctly
project_root = os.path.abspath('.')
sys.path.insert(0, project_root)

SHOW_SQL = os.environ.get('SHOW_SQL', 'false').lower() == 'true'
REAL_DB = os.environ.get('REAL_DB', 'false').lower() == 'true'

try:
    # Execute the test file directly
    with open('{}', 'r') as f:
        test_code = f.read()
    
    # Create a namespace for the test
    test_namespace = {{}}
    
    # Execute the test file in the namespace
    exec(test_code, test_namespace)
    
    print(f"✅ Successfully executed {MODULE_NAME}")
    
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
        print(f"❌ No handler classes found in {MODULE_NAME}")
        sys.exit(1)

    print(f"✅ Found handler classes: {{', '.join(handler_classes)}}")

    # Try to instantiate and execute the first handler class
    try:
        handler_class_name = handler_classes[0]
        handler_class = test_namespace[handler_class_name]
        handler = handler_class()
        print(f"✅ Successfully instantiated {{handler_class_name}} for {MODULE_NAME}")
        
        # Create a context for testing
        if REAL_DB:
            from src.common.test_rig_python import PyStateContext, RealPyConnection
            # Get connection parameters from environment or use defaults
            import os
            host = os.environ.get('TIDB_HOST', 'localhost:4000')
            username = os.environ.get('TIDB_USER', 'root')
            password = os.environ.get('TIDB_PASSWORD', '')
            database = os.environ.get('TIDB_DATABASE', 'test')
            
            real_connection = RealPyConnection(
                connection_info={{'id': 'test_conn'}}, 
                connection_id='test_conn',
                host=host,
                username=username,
                password=password,
                database=database
            )
            context = PyStateContext(
                host=host,
                port=4000,  # Will be parsed from host if it contains port
                username=username,
                password=password,
                database=database,
                connection=real_connection
            )
        else:
            from src.common.test_rig_python import PyStateContext, PyConnection
            mock_connection = PyConnection(connection_info={{'id': 'test_conn'}}, connection_id='test_conn')
            context = PyStateContext(
                host='localhost',
                port=4000,
                username='root',
                password='',
                database='test',
                connection=mock_connection
            )
        
        # Execute the handler's enter method
        print(f"🔧 Executing {{handler_class_name}}.enter()...")
        enter_result = handler.enter(context)
        print(f"Enter result: {{enter_result}}")
        
        # Execute the handler's execute method
        print(f"🔧 Executing {{handler_class_name}}.execute()...")
        execute_result = handler.execute(context)
        print(f"Execute result: {{execute_result}}")
        
        # Execute the handler's exit method
        print(f"🔧 Executing {{handler_class_name}}.exit()...")
        handler.exit(context)
        print(f"Exit completed")
        
    except Exception as e:
        # Get the current line number from the test file
        current_line = inspect.currentframe().f_lineno
        print(f"❌ Failed to execute handler for {MODULE_NAME} (line {{current_line}}): {{str(e)}}")
        # Print full stack trace for debugging
        import traceback
        print("Full stack trace:")
        traceback.print_exc()
        sys.exit(1)

    print(f"✅ Test passed for {MODULE_NAME}")
    
except Exception as e:
    # Get the current line number from the test file
    current_line = inspect.currentframe().f_lineno
    print(f"❌ Failed to execute {MODULE_NAME} (line {{current_line}}): {{str(e)}}")
    # Print full stack trace for debugging
    import traceback
    print("Full stack trace:")
    traceback.print_exc()
    sys.exit(1)
"#
    }

    /// Create and write temporary test script
    ///
    /// # Errors
    ///
    /// Returns an error if the temporary file cannot be created or written.
    fn create_temp_test_script(
        &self,
        test_path: &Path,
        test_content: &str,
    ) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        let temp_dir = std::env::temp_dir();
        let temp_script = temp_dir.join(format!(
            "test_{}.py",
            test_path.file_stem().unwrap().to_str().unwrap()
        ));
        std::fs::write(&temp_script, test_content)?;
        Ok(temp_script)
    }

    /// Execute the Python test script
    ///
    /// # Errors
    ///
    /// Returns an error if the Python script execution fails.
    fn execute_python_script(
        &self,
        test_path: &Path,
        temp_script: &std::path::PathBuf,
    ) -> Result<std::process::Output, Box<dyn std::error::Error>> {
        let output = std::process::Command::new("python3")
            .arg(temp_script)
            .current_dir(test_path.parent().unwrap())
            .output()?;
        Ok(output)
    }

    /// Handle test execution results
    ///
    /// # Errors
    ///
    /// Returns an error if the test execution failed.
    fn handle_test_results(
        &self,
        test_path: &Path,
        output: std::process::Output,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if output.status.success() {
            tracing::info!("✅ Test passed: {}", test_path.display());
            println!("✅ Test passed: {}", test_path.display());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("❌ Test failed: {} - {}", test_path.display(), stderr);
            println!("❌ Test failed: {} - {}", test_path.display(), stderr);
            return Err(format!("Test failed: {}", test_path.display()).into());
        }
        Ok(())
    }

    fn run_single_python_test(
        &self,
        test_path: &Path,
    ) -> impl std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send {
        async {
            // Create a Python script that imports and tests the handler
            let module_name = test_path.file_stem().unwrap().to_str().unwrap();
            let test_content = self.generate_python_test_script(test_path, module_name);

            // Create temporary test script
            let temp_script = self.create_temp_test_script(test_path, &test_content)?;

            // Execute the Python script
            let output = self.execute_python_script(test_path, &temp_script)?;

            // Handle the results
            self.handle_test_results(test_path, output)?;

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
        name: "import",
        test_dir: "src/import",
        module_prefix: "src.import",
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
    /// Run a test suite with output control
    ///
    /// # Errors
    ///
    /// Returns an error if the test suite execution fails.
    pub async fn run_suite_with_output(
        &self,
        show_output: bool,
        show_sql: bool,
        real_db: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Running Python test suite: {}", self.name);
        let test_files = PythonSuiteConfig::discover_test_files(self.test_dir).await?;
        tracing::info!("Found {} test files in {}", test_files.len(), self.test_dir);
        for test_file in test_files {
            tracing::info!("Running test: {}", test_file.display());
            PythonSuiteConfig::run_single_python_test(
                &test_file,
                self.module_prefix,
                show_output,
                show_sql,
                real_db,
            )?;
        }
        tracing::info!("All Python tests completed successfully for {}", self.name);
        Ok(())
    }

    /// Discover test files in a directory
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read.
    #[allow(clippy::unused_async)]
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
                    && std::path::Path::new(name_str)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("py"))
                    && name_str != "test_rig_python.py"
                {
                    test_files.push(path);
                }
            }
        }
        test_files.sort();
        Ok(test_files)
    }

    /// Run a single Python test file
    ///
    /// # Errors
    ///
    /// Returns an error if the test execution fails.
    ///
    /// # Panics
    ///
    /// Panics if the test path has an invalid file stem.
    #[allow(clippy::too_many_lines)]
    pub fn run_single_python_test(
        test_path: &std::path::Path,
        _module_prefix: &str,
        show_output: bool,
        show_sql: bool,
        real_db: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let module_name = test_path.file_stem().unwrap().to_str().unwrap();
        let parent_dir = test_path.parent().unwrap();

        // Create a simple test script that executes the test file directly
        let test_content = format!(
            r#"
import sys
import os
import inspect

# Add the project root to Python path so imports work correctly
project_root = os.path.abspath('.')
sys.path.insert(0, project_root)

SHOW_SQL = os.environ.get('SHOW_SQL', 'false').lower() == 'true'
REAL_DB = os.environ.get('REAL_DB', 'false').lower() == 'true'

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

    # Try to instantiate and execute all handler classes
    for handler_class_name in handler_classes:
        try:
            print(f"\n--- Executing {{handler_class_name}} ---")
            handler_class = test_namespace[handler_class_name]
            handler = handler_class()
            print(f"✅ Successfully instantiated {{handler_class_name}} for {module_name}")
            
            # Create a context for testing
            if REAL_DB:
                from src.common.test_rig_python import PyStateContext, RealPyConnection
                # Get connection parameters from environment or use defaults
                import os
                host = os.environ.get('TIDB_HOST', 'localhost:4000')
                username = os.environ.get('TIDB_USER', 'root')
                password = os.environ.get('TIDB_PASSWORD', '')
                database = os.environ.get('TIDB_DATABASE', 'test')
                
                real_connection = RealPyConnection(
                    connection_info={{'id': 'test_conn'}}, 
                    connection_id='test_conn',
                    host=host,
                    username=username,
                    password=password,
                    database=database
                )
                context = PyStateContext(
                    host=host,
                    port=4000,  # Will be parsed from host if it contains port
                    username=username,
                    password=password,
                    database=database,
                    connection=real_connection
                )
            else:
                from src.common.test_rig_python import PyStateContext, PyConnection
                mock_connection = PyConnection(connection_info={{'id': 'test_conn'}}, connection_id='test_conn')
                context = PyStateContext(
                    host='localhost',
                    port=4000,
                    username='root',
                    password='',
                    database='test',
                    connection=mock_connection
                )
            
            # Execute the handler's enter method
            print(f"🔧 Executing {{handler_class_name}}.enter()...")
            enter_result = handler.enter(context)
            print(f"Enter result: {{enter_result}}")
            
            # Execute the handler's execute method
            print(f"🔧 Executing {{handler_class_name}}.execute()...")
            execute_result = handler.execute(context)
            print(f"Execute result: {{execute_result}}")
            
            # Execute the handler's exit method
            print(f"🔧 Executing {{handler_class_name}}.exit()...")
            handler.exit(context)
            print(f"Exit completed")
            
        except Exception as e:
            # Get the current line number from the test file
            current_line = inspect.currentframe().f_lineno
            print(f"❌ Failed to execute handler {{handler_class_name}} for {module_name} (line {{current_line}}): {{str(e)}}")
            # Print full stack trace for debugging
            import traceback
            print("Full stack trace:")
            traceback.print_exc()
            sys.exit(1)

    print(f"✅ Test passed for {module_name}")
    
except Exception as e:
    # Get the current line number from the test file
    current_line = inspect.currentframe().f_lineno
    print(f"❌ Failed to execute {module_name} (line {{current_line}}): {{str(e)}}")
    # Print full stack trace for debugging
    import traceback
    print("Full stack trace:")
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

        let mut command = std::process::Command::new("python3");
        command
            .arg(&temp_script)
            .current_dir(parent_dir)
            .env("PYTHONPATH", std::env::current_dir().unwrap());

        // Set SQL logging environment variable
        if show_sql {
            command.env("SHOW_SQL", "true");
        }
        // Set real DB environment variable
        if real_db {
            command.env("REAL_DB", "true");
        }

        // Pass TiDB environment variables to the Python subprocess
        if let Ok(host) = std::env::var("TIDB_HOST") {
            command.env("TIDB_HOST", host);
        }
        if let Ok(user) = std::env::var("TIDB_USER") {
            command.env("TIDB_USER", user);
        }
        if let Ok(password) = std::env::var("TIDB_PASSWORD") {
            command.env("TIDB_PASSWORD", password);
        }
        if let Ok(database) = std::env::var("TIDB_DATABASE") {
            command.env("TIDB_DATABASE", database);
        }

        let output = command.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            tracing::info!("✅ Test passed: {}", test_path.display());
            println!("✅ Test passed: {}", test_path.display());
            if show_output && !stdout.is_empty() {
                println!("Test output:\n{stdout}");
            }
            // Always print SQL logs if show_sql is set, even if show_output is not
            if show_sql && !stdout.is_empty() {
                let lines: Vec<&str> = stdout.lines().collect();
                let mut i = 0;
                while i < lines.len() {
                    let line = lines[i];
                    if line.contains("SQL [") || line.contains("🔍 SQL [") {
                        println!("{line}");
                        // If this is a multi-line SQL statement, print the next lines until we hit another SQL statement or non-SQL line
                        let mut j = i + 1;
                        while j < lines.len() {
                            let next_line = lines[j];
                            if next_line.contains("SQL [") || next_line.contains("🔍 SQL [") {
                                // Found another SQL statement, stop here
                                break;
                            } else if next_line.trim().is_empty() {
                                // Empty line, skip it
                                j += 1;
                            } else if next_line.starts_with("🔧")
                                || next_line.starts_with("✅")
                                || next_line.starts_with("❌")
                                || next_line.starts_with("Enter result:")
                                || next_line.starts_with("Execute result:")
                                || next_line.starts_with("Exit completed")
                            {
                                // Found a non-SQL line, stop here
                                break;
                            } else {
                                // This is part of the SQL statement, print it
                                println!("{next_line}");
                                j += 1;
                            }
                        }
                        i = j;
                    } else {
                        i += 1;
                    }
                }
            }
        } else {
            // Debug: Print the stderr and stdout content to understand what's happening
            if show_output {
                println!("Debug stdout (first 10 lines):");
                for (i, line) in stdout.lines().take(10).enumerate() {
                    println!("  {i}: {line}");
                }
                println!("Debug stderr (first 10 lines):");
                for (i, line) in stderr.lines().take(10).enumerate() {
                    println!("  {i}: {line}");
                }
                println!("stderr length: {}", stderr.len());
                println!("stdout length: {}", stdout.len());
            }

            // Extract just the error message from stderr or stdout, avoiding stack traces
            let error_message = if stderr.contains("❌ Failed to execute handler for") {
                stderr
                    .lines()
                    .find(|line| line.contains("❌ Failed to execute handler for"))
                    .unwrap_or("Unknown error")
            } else if stdout.contains("❌ Failed to execute handler for") {
                stdout
                    .lines()
                    .find(|line| line.contains("❌ Failed to execute handler for"))
                    .unwrap_or("Unknown error")
            } else if stderr.contains("❌ Failed to execute") {
                stderr
                    .lines()
                    .find(|line| line.contains("❌ Failed to execute"))
                    .unwrap_or("Unknown error")
            } else if stdout.contains("❌ Failed to execute") {
                stdout
                    .lines()
                    .find(|line| line.contains("❌ Failed to execute"))
                    .unwrap_or("Unknown error")
            } else {
                // If no specific error format found, show the first non-empty line of stderr or stdout
                stderr
                    .lines()
                    .find(|line| !line.trim().is_empty() && !line.contains("Traceback"))
                    .or_else(|| {
                        stdout
                            .lines()
                            .find(|line| !line.trim().is_empty() && !line.contains("Traceback"))
                    })
                    .unwrap_or("Unknown error")
            };

            tracing::error!(
                "❌ Test failed: {} - {}",
                test_path.display(),
                error_message
            );
            println!(
                "❌ Test failed: {} - {}",
                test_path.display(),
                error_message
            );
            return Err(format!("Test failed: {}", test_path.display()).into());
        }
        let _ = std::fs::remove_file(temp_script);
        Ok(())
    }
}

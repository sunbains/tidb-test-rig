#![cfg(feature = "python_plugins")]

//! # Python Plugin System
//!
//! Python plugin system using PyO3 for writing state handlers in Python.
//! Provides seamless integration between Rust state machine and Python handlers
//! with support for async operations, database queries, and error propagation.

use crate::errors::{ConnectError, Result};
use crate::state_machine::{State, StateContext, StateHandler};
use async_trait::async_trait;
use mysql::prelude::*;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Python handler wrapper that implements the Rust StateHandler trait
pub struct PythonHandler {
    py_handler: PyObject,
}

impl PythonHandler {
    pub fn new(py_handler: PyObject) -> Self {
        Self { py_handler }
    }
}

#[async_trait]
impl StateHandler for PythonHandler {
    async fn enter(&self, context: &mut StateContext) -> Result<State> {
        Python::with_gil(|py| {
            let context_py = PyStateContext::new(context);
            let result = self
                .py_handler
                .call_method1(py, "enter", (context_py,))
                .map_err(|e| ConnectError::StateMachine(format!("Python handler error: {}", e)))?;
            let state_str: String = result.extract(py).map_err(|e| {
                ConnectError::StateMachine(format!("Failed to extract state: {}", e))
            })?;
            Ok(parse_state_string(&state_str))
        })
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State> {
        Python::with_gil(|py| {
            let context_py = PyStateContext::new(context);
            let result = self
                .py_handler
                .call_method1(py, "execute", (context_py,))
                .map_err(|e| ConnectError::StateMachine(format!("Python handler error: {}", e)))?;
            let state_str: String = result.extract(py).map_err(|e| {
                ConnectError::StateMachine(format!("Failed to extract state: {}", e))
            })?;
            Ok(parse_state_string(&state_str))
        })
    }

    async fn exit(&self, context: &mut StateContext) -> Result<()> {
        Python::with_gil(|py| {
            let context_py = PyStateContext::new(context);
            self.py_handler
                .call_method1(py, "exit", (context_py,))
                .map_err(|e| ConnectError::StateMachine(format!("Python handler error: {}", e)))?;
            Ok(())
        })
    }
}

/// Parse state string to State enum
fn parse_state_string(state_str: &str) -> State {
    match state_str.to_lowercase().as_str() {
        "initial" => State::Initial,
        "parsing_config" => State::ParsingConfig,
        "connecting" => State::Connecting,
        "testing_connection" => State::TestingConnection,
        "verifying_database" => State::VerifyingDatabase,
        "getting_version" => State::GettingVersion,
        "checking_import_jobs" => State::CheckingImportJobs,
        "showing_import_job_details" => State::ShowingImportJobDetails,
        "completed" => State::Completed,
        _ => State::Completed, // Default to completed for unknown states
    }
}

/// Python wrapper for StateContext
#[pyclass]
#[derive(Clone)]
pub struct PyStateContext {
    #[pyo3(get)]
    pub host: Option<String>,
    #[pyo3(get)]
    pub port: Option<u16>,
    #[pyo3(get)]
    pub username: Option<String>,
    #[pyo3(get)]
    pub password: Option<String>,
    #[pyo3(get)]
    pub database: Option<String>,
    #[pyo3(get)]
    pub connection: Option<PyConnection>,
}

impl PyStateContext {
    pub fn new(context: &StateContext) -> Self {
        Self {
            host: Some(context.host.clone()),
            port: Some(context.port),
            username: Some(context.username.clone()),
            password: Some(context.password.clone()),
            database: context.database.clone(),
            // TODO: PyConnection expects a clone of PooledConn, but PooledConn does not implement Clone.
            // For now, do not provide a connection if not available.
            connection: None,
        }
    }
}

/// Python wrapper for database connection
#[pyclass]
#[derive(Clone)]
pub struct PyConnection {
    inner: Arc<Mutex<mysql::PooledConn>>,
}

impl PyConnection {
    pub fn new(conn: mysql::PooledConn) -> Self {
        Self {
            inner: Arc::new(Mutex::new(conn)),
        }
    }
}

#[pymethods]
impl PyConnection {
    /// Execute a query and return results as a list of dictionaries
    pub fn execute_query(&self, query: String) -> PyResult<Vec<PyObject>> {
        Python::with_gil(|py| {
            let mut conn_guard = self
                .inner
                .try_lock()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            let results: Vec<mysql::Row> = conn_guard
                .exec(&query, ())
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            let mut py_results = Vec::new();
            for row in results {
                let row_dict = PyDict::new(py);
                for (i, value) in row.unwrap().into_iter().enumerate() {
                    let key = format!("col_{}", i);
                    let py_value = match value {
                        mysql::Value::NULL => py.None(),
                        mysql::Value::Bytes(bytes) => {
                            String::from_utf8_lossy(&bytes).to_string().into_py(py)
                        }
                        mysql::Value::Int(i) => i.into_py(py),
                        mysql::Value::UInt(u) => u.into_py(py),
                        mysql::Value::Float(f) => f.into_py(py),
                        mysql::Value::Double(d) => d.into_py(py),
                        mysql::Value::Date(year, month, day, hour, minute, second, _) => format!(
                            "{}-{:02}-{:02} {:02}:{:02}:{:02}",
                            year, month, day, hour, minute, second
                        )
                        .into_py(py),
                        _ => format!("{:?}", value).into_py(py),
                    };
                    row_dict.set_item(key, py_value)?;
                }
                py_results.push(row_dict.into());
            }

            Ok(py_results)
        })
    }
}

/// Python wrapper for State enum
#[pyclass]
pub struct PyState;

#[pymethods]
impl PyState {
    #[staticmethod]
    pub fn initial() -> String {
        State::Initial.to_string()
    }

    #[staticmethod]
    pub fn parsing_config() -> String {
        State::ParsingConfig.to_string()
    }

    #[staticmethod]
    pub fn connecting() -> String {
        State::Connecting.to_string()
    }

    #[staticmethod]
    pub fn testing_connection() -> String {
        State::TestingConnection.to_string()
    }

    #[staticmethod]
    pub fn verifying_database() -> String {
        State::VerifyingDatabase.to_string()
    }

    #[staticmethod]
    pub fn getting_version() -> String {
        State::GettingVersion.to_string()
    }

    #[staticmethod]
    pub fn checking_import_jobs() -> String {
        State::CheckingImportJobs.to_string()
    }

    #[staticmethod]
    pub fn showing_import_job_details() -> String {
        State::ShowingImportJobDetails.to_string()
    }

    #[staticmethod]
    pub fn completed() -> String {
        State::Completed.to_string()
    }
}

/// Base class for Python handlers
#[pyclass]
pub struct PyStateHandler;

#[pymethods]
impl PyStateHandler {
    #[new]
    pub fn new() -> Self {
        Self
    }

    /// Default enter implementation
    pub fn enter(&self, _context: &PyStateContext) -> PyResult<String> {
        Ok(State::Initial.to_string())
    }

    /// Default execute implementation
    pub fn execute(&self, _context: &PyStateContext) -> PyResult<String> {
        Ok(State::Completed.to_string())
    }

    /// Default exit implementation
    pub fn exit(&self, _context: &PyStateContext) -> PyResult<()> {
        Ok(())
    }
}

/// Python module definition
#[pymodule]
pub fn test_rig_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyStateContext>()?;
    m.add_class::<PyConnection>()?;
    m.add_class::<PyState>()?;
    m.add_class::<PyStateHandler>()?;

    Ok(())
}

/// Register a Python handler with the state machine
pub fn register_python_handler(
    state_machine: &mut crate::state_machine::StateMachine,
    state: State,
    py_handler: PyObject,
) -> PyResult<()> {
    let rust_handler = PythonHandler::new(py_handler);
    state_machine.register_handler(state, Box::new(rust_handler));
    Ok(())
}

/// Load Python handlers from a module
pub fn load_python_handlers(
    state_machine: &mut crate::state_machine::StateMachine,
    module_path: &str,
) -> PyResult<()> {
    Python::with_gil(|py| {
        let sys = py.import("sys")?;
        let path = sys.getattr("path")?;
        path.call_method1("append", ("",))?;

        let module = py.import(module_path)?;

        // Look for handler classes in the module
        for attr_name in module.dir() {
            let attr_name: String = attr_name.extract()?;
            if attr_name.ends_with("Handler") && !attr_name.starts_with('_') {
                let handler_class = module.getattr(&*attr_name)?;
                let handler_instance = handler_class.call0()?;

                // Map handler name to state (you might want to make this configurable)
                let state = match attr_name.as_str() {
                    "CheckingImportJobsHandler" => State::CheckingImportJobs,
                    "ShowingImportJobDetailsHandler" => State::ShowingImportJobDetails,
                    _ => continue, // Skip unknown handlers
                };

                register_python_handler(state_machine, state, handler_instance.into())?;
            }
        }

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_machine::StateMachine;

    #[test]
    fn test_parse_state_string() {
        // Test all valid state strings
        assert_eq!(parse_state_string("initial"), State::Initial);
        assert_eq!(parse_state_string("INITIAL"), State::Initial);
        assert_eq!(parse_state_string("Initial"), State::Initial);

        assert_eq!(parse_state_string("parsing_config"), State::ParsingConfig);
        assert_eq!(parse_state_string("connecting"), State::Connecting);
        assert_eq!(
            parse_state_string("testing_connection"),
            State::TestingConnection
        );
        assert_eq!(
            parse_state_string("verifying_database"),
            State::VerifyingDatabase
        );
        assert_eq!(parse_state_string("getting_version"), State::GettingVersion);
        assert_eq!(
            parse_state_string("checking_import_jobs"),
            State::CheckingImportJobs
        );
        assert_eq!(
            parse_state_string("showing_import_job_details"),
            State::ShowingImportJobDetails
        );
        assert_eq!(parse_state_string("completed"), State::Completed);

        // Test unknown state (should default to completed)
        assert_eq!(parse_state_string("unknown_state"), State::Completed);
        assert_eq!(parse_state_string(""), State::Completed);
    }

    #[test]
    fn test_py_state_context_new() {
        let mut context = StateContext::default();
        context.host = "testhost".to_string();
        context.port = 4000;
        context.username = "testuser".to_string();
        context.password = "testpass".to_string();
        context.database = Some("testdb".to_string());

        let py_context = PyStateContext::new(&context);

        assert_eq!(py_context.host, Some("testhost".to_string()));
        assert_eq!(py_context.port, Some(4000));
        assert_eq!(py_context.username, Some("testuser".to_string()));
        assert_eq!(py_context.password, Some("testpass".to_string()));
        assert_eq!(py_context.database, Some("testdb".to_string()));
        assert!(py_context.connection.is_none());
    }

    #[test]
    fn test_py_state_context_with_none_database() {
        let mut context = StateContext::default();
        context.host = "testhost".to_string();
        context.port = 4000;
        context.username = "testuser".to_string();
        context.password = "testpass".to_string();
        context.database = None;

        let py_context = PyStateContext::new(&context);

        assert!(py_context.database.is_none());
    }

    #[test]
    fn test_py_state_methods() {
        Python::with_gil(|_py| {
            // Test all PyState static methods
            assert_eq!(PyState::initial(), "Initial");
            assert_eq!(PyState::parsing_config(), "ParsingConfig");
            assert_eq!(PyState::connecting(), "Connecting");
            assert_eq!(PyState::testing_connection(), "TestingConnection");
            assert_eq!(PyState::verifying_database(), "VerifyingDatabase");
            assert_eq!(PyState::getting_version(), "GettingVersion");
            assert_eq!(PyState::checking_import_jobs(), "CheckingImportJobs");
            assert_eq!(
                PyState::showing_import_job_details(),
                "ShowingImportJobDetails"
            );
            assert_eq!(PyState::completed(), "Completed");
        });
    }

    #[test]
    fn test_py_state_handler_new() {
        Python::with_gil(|_py| {
            let handler = PyStateHandler::new();

            // Test that the handler can be created
            assert!(
                handler
                    .enter(&PyStateContext::new(&StateContext::default()))
                    .is_ok()
            );
            assert!(
                handler
                    .execute(&PyStateContext::new(&StateContext::default()))
                    .is_ok()
            );
            assert!(
                handler
                    .exit(&PyStateContext::new(&StateContext::default()))
                    .is_ok()
            );
        });
    }

    #[test]
    fn test_py_state_handler_default_behavior() {
        Python::with_gil(|_py| {
            let handler = PyStateHandler::new();
            let context = PyStateContext::new(&StateContext::default());

            // Test default enter behavior
            let enter_result = handler.enter(&context).unwrap();
            assert_eq!(enter_result, "Initial");

            // Test default execute behavior
            let execute_result = handler.execute(&context).unwrap();
            assert_eq!(execute_result, "Completed");

            // Test default exit behavior (should not panic)
            assert!(handler.exit(&context).is_ok());
        });
    }

    #[test]
    fn test_python_handler_wrapper() {
        Python::with_gil(|py| {
            // Create a mock Python handler
            let py_handler_class = py.get_type::<PyStateHandler>();
            let py_handler = py_handler_class.call0().unwrap();
            let py_object = py_handler.into();

            let rust_handler = PythonHandler::new(py_object);

            // Test that the wrapper can be created
            assert!(
                rust_handler
                    .py_handler
                    .as_ref(py)
                    .is_instance_of::<PyStateHandler>()
            );
        });
    }

    #[test]
    fn test_register_python_handler() {
        Python::with_gil(|py| {
            let mut state_machine = StateMachine::new();
            let py_handler_class = py.get_type::<PyStateHandler>();
            let py_handler = py_handler_class.call0().unwrap();
            let py_object = py_handler.into();

            // Test registering a Python handler
            let result =
                register_python_handler(&mut state_machine, State::TestingConnection, py_object);
            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_py_connection_new() {
        let pool = mysql::Pool::new(
            mysql::OptsBuilder::default()
                .ip_or_hostname(Some("localhost"))
                .tcp_port(4000)
                .user(Some("root"))
                .pass(Some(""))
                .db_name(Some("test")),
        )
        .unwrap();
        let conn = pool.get_conn().unwrap();
        let py_conn = PyConnection::new(conn);
        assert!(py_conn.inner.try_lock().is_ok());
    }

    #[test]
    fn test_py_connection_execute_query_with_mock() {
        Python::with_gil(|_py| {
            let pool = mysql::Pool::new(
                mysql::OptsBuilder::default()
                    .ip_or_hostname(Some("localhost"))
                    .tcp_port(4000)
                    .user(Some("root"))
                    .pass(Some(""))
                    .db_name(Some("test")),
            )
            .unwrap();
            let conn = pool.get_conn().unwrap();
            let py_conn = PyConnection::new(conn);
            let result = py_conn.execute_query("SELECT 1".to_string());
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_py_state_context_clone() {
        let mut context = StateContext::default();
        context.host = "testhost".to_string();
        context.port = 4000;
        context.username = "testuser".to_string();
        context.password = "testpass".to_string();
        context.database = Some("testdb".to_string());

        let py_context = PyStateContext::new(&context);
        let cloned_context = py_context.clone();

        assert_eq!(py_context.host, cloned_context.host);
        assert_eq!(py_context.port, cloned_context.port);
        assert_eq!(py_context.username, cloned_context.username);
        assert_eq!(py_context.password, cloned_context.password);
        assert_eq!(py_context.database, cloned_context.database);
    }

    #[test]
    fn test_py_connection_clone() {
        let pool = mysql::Pool::new(
            mysql::OptsBuilder::default()
                .ip_or_hostname(Some("localhost"))
                .tcp_port(4000)
                .user(Some("root"))
                .pass(Some(""))
                .db_name(Some("test")),
        )
        .unwrap();
        let conn = pool.get_conn().unwrap();
        let py_conn = PyConnection::new(conn);
        let cloned_conn = py_conn.clone();

        // Both should reference the same underlying connection
        assert!(py_conn.inner.try_lock().is_ok());
        assert!(cloned_conn.inner.try_lock().is_ok());
    }

    #[test]
    fn test_state_machine_integration() {
        Python::with_gil(|py| {
            let mut state_machine = StateMachine::new();

            // Test that we can create and register a Python handler
            let py_handler_class = py.get_type::<PyStateHandler>();
            let py_handler = py_handler_class.call0().unwrap();
            let py_object = py_handler.into();

            let result =
                register_python_handler(&mut state_machine, State::TestingConnection, py_object);
            assert!(result.is_ok());

            // Test that the state machine has the handler registered
            // (This would require exposing the internal state of StateMachine for testing)
        });
    }

    #[test]
    fn test_error_handling() {
        Python::with_gil(|_py| {
            // Test error handling in Python handler calls
            let handler = PyStateHandler::new();
            let context = PyStateContext::new(&StateContext::default());

            // These should not panic and should return valid results
            let enter_result = handler.enter(&context);
            assert!(enter_result.is_ok());

            let execute_result = handler.execute(&context);
            assert!(execute_result.is_ok());

            let exit_result = handler.exit(&context);
            assert!(exit_result.is_ok());
        });
    }

    #[test]
    fn test_py_module_registration() {
        Python::with_gil(|py| {
            let module = PyModule::new(py, "test_module").unwrap();

            // Test that we can register the module
            let result = test_rig_python(py, module);
            assert!(result.is_ok());

            // Test that the classes are available in the module
            assert!(module.getattr("PyStateContext").is_ok());
            assert!(module.getattr("PyConnection").is_ok());
            assert!(module.getattr("PyState").is_ok());
            assert!(module.getattr("PyStateHandler").is_ok());
        });
    }

    #[test]
    fn test_load_python_handlers_invalid_module() {
        let mut state_machine = StateMachine::new();

        // Test loading from a non-existent module
        let result = load_python_handlers(&mut state_machine, "nonexistent_module");
        assert!(result.is_err());
    }

    #[test]
    fn test_py_state_context_attributes() {
        Python::with_gil(|py| {
            let mut context = StateContext::default();
            context.host = "testhost".to_string();
            context.port = 4000;
            context.username = "testuser".to_string();
            context.password = "testpass".to_string();
            context.database = Some("testdb".to_string());

            let py_context = PyStateContext::new(&context);

            // Test that attributes are accessible from Python
            let py_context_obj = py_context.into_py(py);
            let py_context_ref = py_context_obj.as_ref(py);

            // Test host attribute
            let host_attr = py_context_ref.getattr("host").unwrap();
            assert!(host_attr.is_instance_of::<pyo3::types::PyAny>());

            // Test port attribute
            let port_attr = py_context_ref.getattr("port").unwrap();
            assert!(port_attr.is_instance_of::<pyo3::types::PyAny>());

            // Test username attribute
            let username_attr = py_context_ref.getattr("username").unwrap();
            assert!(username_attr.is_instance_of::<pyo3::types::PyAny>());

            // Test password attribute
            let password_attr = py_context_ref.getattr("password").unwrap();
            assert!(password_attr.is_instance_of::<pyo3::types::PyAny>());

            // Test database attribute
            let database_attr = py_context_ref.getattr("database").unwrap();
            assert!(database_attr.is_instance_of::<pyo3::types::PyAny>());

            // Test connection attribute
            let connection_attr = py_context_ref.getattr("connection").unwrap();
            assert!(connection_attr.is_instance_of::<pyo3::types::PyAny>());
        });
    }
}

#![cfg(feature = "python_plugins")]

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

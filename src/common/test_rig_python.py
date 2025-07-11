"""
Python stub module for test_rig_python to support all Python test suites (DDL, Scale, etc).
This provides the necessary classes and methods for testing without requiring the full Rust extension.
"""

from typing import Optional, List, Any, Dict
from dataclasses import dataclass

@dataclass
class PyStateContext:
    """Python wrapper for StateContext"""
    host: Optional[str] = None
    port: Optional[int] = None
    username: Optional[str] = None
    password: Optional[str] = None
    database: Optional[str] = None
    connection: Optional['PyConnection'] = None

class PyConnection:
    """Python wrapper for database connection"""
    def __init__(self, connection_info: Dict[str, Any] = None):
        self.connection_info = connection_info or {}
    def execute_query(self, query: str) -> List[Dict[str, Any]]:
        print(f"Mock: Executing query: {query}")
        if "SHOW TABLES" in query:
            return [{"col_0": "ddl_test"}]
        elif "SHOW DATABASES" in query:
            return [{"col_0": "test_db"}]
        elif "SHOW INDEX" in query:
            if "WHERE Key_name='idx_name'" in query:
                return [{"col_0": "idx_name"}] if "CREATE INDEX" in query else []
        elif "SHOW COLUMNS" in query:
            return [{"col_0": "age"}] if "age" in query else []
        elif "SELECT COUNT" in query:
            return [{"col_0": 1}] if "INSERT" in query else [{"col_0": 0}]
        elif "SELECT DEFAULT_CHARACTER_SET_NAME" in query:
            return [{"col_0": "latin1"}]
        elif "SHOW PROCEDURE STATUS" in query:
            return [{"col_0": "p_test"}]
        elif "SHOW FULL TABLES" in query:
            return [{"col_0": "v_test"}]
        elif "SELECT TABLE_NAME" in query:
            return [{"col_0": "ddl_test"}]
        else:
            return []

class PyStateHandler:
    """Base class for Python state handlers"""
    def __init__(self):
        pass
    
    def enter(self, context: PyStateContext) -> str:
        """Called when entering the state"""
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Called during state execution"""
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Called when exiting the state"""
        pass

class PyState:
    """Python wrapper for State enum"""
    @staticmethod
    def initial() -> str:
        return "Initial"
    @staticmethod
    def parsing_config() -> str:
        return "ParsingConfig"
    @staticmethod
    def connecting() -> str:
        return "Connecting"
    @staticmethod
    def testing_connection() -> str:
        return "TestingConnection"
    @staticmethod
    def verifying_database() -> str:
        return "VerifyingDatabase"
    @staticmethod
    def getting_version() -> str:
        return "GettingVersion"
    @staticmethod
    def checking_import_jobs() -> str:
        return "CheckingImportJobs"
    @staticmethod
    def showing_import_job_details() -> str:
        return "ShowingImportJobDetails"
    @staticmethod
    def completed() -> str:
        return "Completed"
    @staticmethod
    def error(message: str) -> str:
        return f"Error: {message}" 
"""
Python stub module for test_rig_python to support all Python test suites (DDL, Scale, etc).
This provides the necessary classes and methods for testing without requiring the full Rust extension.
"""

from typing import Optional, List, Any, Dict
from dataclasses import dataclass
import threading
import time
import os

# Check if SQL logging is enabled
SHOW_SQL = os.environ.get('SHOW_SQL', 'false').lower() == 'true'
REAL_DB = os.environ.get('REAL_DB', 'false').lower() == 'true'

try:
    import mysql.connector
except ImportError:
    mysql = None

@dataclass
class PyStateContext:
    """Python wrapper for StateContext"""
    host: Optional[str] = None
    port: Optional[int] = None
    username: Optional[str] = None
    password: Optional[str] = None
    database: Optional[str] = None
    connection: Optional['PyConnection'] = None
    # Support for multiple concurrent connections
    connections: Optional[List['PyConnection']] = None

class PyConnection:
    """Python wrapper for database connection"""
    def __init__(self, connection_info: Dict[str, Any] = None, connection_id: str = "default"):
        self.connection_info = connection_info or {}
        self.connection_id = connection_id
        self._lock = threading.Lock()
    
    def execute_query(self, query: str) -> List[Dict[str, Any]]:
        with self._lock:
            if SHOW_SQL:
                print(f"🔍 SQL [{self.connection_id}]: {query}")
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
    
    def start_transaction(self) -> None:
        """Start a new transaction"""
        with self._lock:
            if SHOW_SQL:
                print(f"🔍 SQL [{self.connection_id}]: START TRANSACTION")
    
    def commit(self) -> None:
        """Commit the current transaction"""
        with self._lock:
            if SHOW_SQL:
                print(f"🔍 SQL [{self.connection_id}]: COMMIT")
    
    def rollback(self) -> None:
        """Rollback the current transaction"""
        with self._lock:
            if SHOW_SQL:
                print(f"🔍 SQL [{self.connection_id}]: ROLLBACK")

class RealPyConnection:
    """Real database connection using mysql-connector-python"""
    def __init__(self, connection_info: dict = None, connection_id: str = "default", host: str = None, port: int = None, username: str = None, password: str = None, database: str = None):
        self.connection_info = connection_info or {}
        self.connection_id = connection_id
        self._lock = threading.Lock()
        self._conn = None
        self._server_type = None
        
        # Parse host and port if host contains port
        if host and ':' in host:
            host_parts = host.split(':')
            self._host = host_parts[0]
            self._port = int(host_parts[1]) if len(host_parts) > 1 else (port or 4000)
        else:
            self._host = host or os.environ.get('TIDB_HOST', 'localhost')
            self._port = port or int(os.environ.get('TIDB_PORT', '4000'))
        
        self._username = username or os.environ.get('TIDB_USER', 'root')
        self._password = password or os.environ.get('TIDB_PASSWORD', '')
        # Use database from environment or parameter
        self._database = database or os.environ.get('TIDB_DATABASE', 'test')
        
        self._connect()

    def _connect(self):
        if mysql is None:
            raise ImportError("mysql-connector-python is not installed. Please install it with 'pip install mysql-connector-python'.")
        
        print(f"[RealPyConnection] Connecting to {self._host}:{self._port} as {self._username}")
        
        # Connect with database
        connect_params = {
            'host': self._host,
            'port': self._port,
            'user': self._username,
            'password': self._password,
            'autocommit': True
        }
        
        # Add database if specified
        if self._database:
            connect_params['database'] = self._database
            
        try:
            self._conn = mysql.connector.connect(**connect_params)
        except Exception as e:
            print(f"[RealPyConnection] Connection failed: {str(e)}")
            raise
        # Detect server type
        cursor = self._conn.cursor()
        cursor.execute("SELECT VERSION()")
        version = cursor.fetchone()[0]
        if 'tidb' in version.lower():
            self._server_type = 'TiDB'
        elif 'mysql' in version.lower():
            self._server_type = 'MySQL'
        else:
            self._server_type = 'Unknown'
        print(f"[RealPyConnection] Connected to server type: {self._server_type} (version: {version})")
        cursor.close()

    def execute_query(self, query: str):
        with self._lock:
            if SHOW_SQL:
                print(f"🔍 SQL [{self.connection_id}]: {query}")
            cursor = self._conn.cursor(dictionary=True)
            try:
                cursor.execute(query)
                if cursor.with_rows:
                    result = cursor.fetchall()
                else:
                    result = []
                return result
            finally:
                cursor.close()

    def start_transaction(self):
        with self._lock:
            if SHOW_SQL:
                print(f"🔍 SQL [{self.connection_id}]: START TRANSACTION")
            self._conn.start_transaction()

    def commit(self):
        with self._lock:
            if SHOW_SQL:
                print(f"🔍 SQL [{self.connection_id}]: COMMIT")
            self._conn.commit()

    def rollback(self):
        with self._lock:
            if SHOW_SQL:
                print(f"🔍 SQL [{self.connection_id}]: ROLLBACK")
            self._conn.rollback()

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

class MultiConnectionTestHandler(PyStateHandler):
    """Base class for handlers that need multiple concurrent connections"""
    
    def __init__(self, connection_count: int = 3):
        super().__init__()
        self.connection_count = connection_count
        self.connections = []
    
    def enter(self, context: PyStateContext) -> str:
        """Setup multiple connections"""
        # Create multiple connections for concurrent testing
        self.connections = []
        for i in range(self.connection_count):
            if REAL_DB:
                # Use real connections when REAL_DB is enabled
                conn = RealPyConnection(
                    connection_info={"id": f"conn_{i}"},
                    connection_id=f"conn_{i}",
                    host=context.host,
                    username=context.username,
                    password=context.password,
                    database=context.database
                )
            else:
                # Use mock connections
                conn = PyConnection(
                    connection_info={"id": f"conn_{i}"},
                    connection_id=f"conn_{i}"
                )
            self.connections.append(conn)
        
        # Store connections in context for access by other methods
        context.connections = self.connections
        print(f"✓ Created {self.connection_count} concurrent connections")
        return PyState.connecting()
    
    def execute_concurrent_operations(self, operations: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """Execute operations concurrently across multiple connections"""
        import concurrent.futures
        
        def execute_operation(op_data):
            conn_id = op_data.get('connection_id', 0)
            operation = op_data.get('operation')
            query = op_data.get('query')
            
            if conn_id < len(self.connections):
                conn = self.connections[conn_id]
                if operation == 'query':
                    return {
                        'connection_id': conn_id,
                        'result': conn.execute_query(query),
                        'status': 'success'
                    }
                elif operation == 'start_transaction':
                    conn.start_transaction()
                    return {
                        'connection_id': conn_id,
                        'result': None,
                        'status': 'transaction_started'
                    }
                elif operation == 'commit':
                    conn.commit()
                    return {
                        'connection_id': conn_id,
                        'result': None,
                        'status': 'committed'
                    }
                elif operation == 'rollback':
                    conn.rollback()
                    return {
                        'connection_id': conn_id,
                        'result': None,
                        'status': 'rolled_back'
                    }
            
            return {
                'connection_id': conn_id,
                'result': None,
                'status': 'error',
                'error': 'Invalid connection ID'
            }
        
        # Execute operations concurrently
        with concurrent.futures.ThreadPoolExecutor(max_workers=self.connection_count) as executor:
            futures = [executor.submit(execute_operation, op) for op in operations]
            results = [future.result() for future in futures]
        
        return results

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
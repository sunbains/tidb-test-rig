#!/usr/bin/env python3
"""
Example Python handlers for the test_rig framework.

This demonstrates how to write state handlers in Python that can be used
with the Rust-based state machine.
"""

import time
import sys
from typing import Optional, Dict, Any, List

# Try to import the real bindings, fall back to mocks for standalone testing
try:
    # pylint: disable=import-error
    # type: ignore
    from test_rig_python import PyStateHandler, PyStateContext, PyState
    print("[INFO] Using real test_rig_python bindings")
except ImportError:
    print("[INFO] Using mock classes for standalone testing")
    
    # Mock classes for standalone testing
    class PyStateHandler:
        """Mock base class for Python state handlers."""
        def __init__(self):
            pass
        
        def enter(self, context):
            """Called when entering this state."""
            return "initial"
        
        def execute(self, context):
            """Called during state execution."""
            return "completed"
        
        def exit(self, context):
            """Called when exiting this state."""
            pass
    
    class PyStateContext:
        """Mock context for testing."""
        def __init__(self, host=None, database=None, connection=None):
            self.host = host
            self.database = database
            self.connection = connection
    
    class PyState:
        """Mock state enum for testing."""
        @staticmethod
        def initial(): return "initial"
        @staticmethod
        def completed(): return "completed"
        @staticmethod
        def connecting(): return "connecting"
        @staticmethod
        def testing_connection(): return "testing_connection"
        @staticmethod
        def testing_isolation(): return "testing_isolation"
        @staticmethod
        def checking_import_jobs(): return "checking_import_jobs"
        @staticmethod
        def showing_import_job_details(): return "showing_import_job_details"


class ExamplePythonHandler(PyStateHandler):
    """Example Python handler that demonstrates basic functionality."""
    
    def __init__(self) -> None:
        super().__init__()
        self.counter = 0
    
    def enter(self, context: PyStateContext) -> str:
        """Called when entering this state."""
        print(f"Python handler entering state...")
        print(f"  Host: {context.host}")
        print(f"  Database: {context.database}")
        return PyState.initial()
    
    def execute(self, context: PyStateContext) -> str:
        """Called during state execution."""
        self.counter += 1
        print(f"Python handler executing (attempt {self.counter})...")
        
        # Example: Check if we have a database connection
        if context.connection:
            try:
                # Execute a simple query
                results = context.connection.execute_query("SELECT 1 as test")
                print(f"  Query executed successfully, got {len(results)} results")
                
                # If we have results, move to next state
                if results:
                    return PyState.completed()
                else:
                    return PyState.testing_connection()
                    
            except Exception as e:
                print(f"  Query failed: {e}")
                return PyState.testing_connection()
        else:
            print("  No database connection available")
            return PyState.connecting()
    
    def exit(self, context: PyStateContext) -> None:
        """Called when exiting this state."""
        print(f"Python handler exiting state (total executions: {self.counter})")


class ImportJobPythonHandler(PyStateHandler):
    """Python handler for checking import jobs."""
    
    def __init__(self, monitor_duration: int = 30) -> None:
        super().__init__()
        self.monitor_duration = monitor_duration
        self.start_time: Optional[float] = None
    
    def enter(self, context: PyStateContext) -> str:
        """Enter the import job checking state."""
        print("Python handler: Checking for active import jobs...")
        self.start_time = time.time()
        return PyState.checking_import_jobs()
    
    def execute(self, context: PyStateContext) -> str:
        """Execute import job checking logic."""
        if not context.connection:
            print("  No database connection available")
            return PyState.completed()
        
        try:
            # Query for active import jobs
            results = context.connection.execute_query("SHOW IMPORT JOBS")
            
            # Find active jobs (where End_Time is NULL)
            active_jobs: List[str] = []
            for row in results:
                # Look for End_Time column (assuming it's in the results)
                end_time = None
                for key, value in row.items():
                    if 'end_time' in key.lower() or 'End_Time' in key:
                        end_time = value
                        break
                
                if end_time is None or end_time == 'NULL':
                    # This is an active job
                    job_id = None
                    for key, value in row.items():
                        if 'job_id' in key.lower() or 'Job_ID' in key:
                            job_id = value
                            break
                    
                    if job_id:
                        active_jobs.append(job_id)
            
            if active_jobs:
                print(f"  Found {len(active_jobs)} active import job(s)")
                return PyState.showing_import_job_details()
            else:
                print("  No active import jobs found")
                return PyState.completed()
                
        except Exception as e:
            print(f"  Error checking import jobs: {e}")
            return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Exit the import job checking state."""
        if self.start_time:
            duration = time.time() - self.start_time
            print(f"Python handler: Import job check completed in {duration:.2f}s")


class MonitoringPythonHandler(PyStateHandler):
    """Python handler for monitoring import job details."""
    
    def __init__(self, monitor_duration: int = 30) -> None:
        super().__init__()
        self.monitor_duration = monitor_duration
        self.start_time: Optional[float] = None
    
    def enter(self, context: PyStateContext) -> str:
        """Enter the monitoring state."""
        print(f"Python handler: Starting import job monitoring for {self.monitor_duration}s...")
        self.start_time = time.time()
        return PyState.showing_import_job_details()
    
    def execute(self, context: PyStateContext) -> str:
        """Execute monitoring logic."""
        if not context.connection:
            print("  No database connection available")
            return PyState.completed()
        
        # Check if monitoring time has elapsed
        if self.start_time and (time.time() - self.start_time) >= self.monitor_duration:
            print(f"  Monitoring completed after {self.monitor_duration}s")
            return PyState.completed()
        
        try:
            # Get active jobs and their details
            results = context.connection.execute_query("SHOW IMPORT JOBS")
            
            active_jobs: List[Dict[str, Any]] = []
            for row in results:
                # Check if job is active
                end_time = None
                for key, value in row.items():
                    if 'end_time' in key.lower() or 'End_Time' in key:
                        end_time = value
                        break
                
                if end_time is None or end_time == 'NULL':
                    # Extract job details
                    job_info = {}
                    for key, value in row.items():
                        job_info[key] = value
                    active_jobs.append(job_info)
            
            if active_jobs:
                print(f"  Monitoring {len(active_jobs)} active job(s)...")
                for job in active_jobs:
                    job_id = job.get('Job_ID', 'Unknown')
                    phase = job.get('Phase', 'Unknown')
                    print(f"    Job {job_id}: {phase}")
            
            # Sleep for a short interval before next check
            time.sleep(5)
            return PyState.showing_import_job_details()
            
        except Exception as e:
            print(f"  Error during monitoring: {e}")
            return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Exit the monitoring state."""
        if self.start_time:
            duration = time.time() - self.start_time
            print(f"Python handler: Monitoring completed in {duration:.2f}s")


class IsolationTestPythonHandler(PyStateHandler):
    """Python handler for testing repeatable read isolation."""

    def __init__(self, test_rows: int = 10) -> None:
        super().__init__()
        self.test_rows = test_rows
        self.table_name = f"isolation_test_{int(time.time())}"

    def enter(self, context: PyStateContext) -> str:
        """Enter the isolation test state."""
        print("[IsolationTest] Entering isolation test state...")
        return PyState.testing_isolation()

    def execute(self, context: PyStateContext) -> str:
        """Execute the isolation test logic."""
        if not context.connection:
            print("  No database connection available")
            return PyState.completed()
        
        try:
            conn1 = context.connection
            conn2 = context.connection  # For demo, use the same connection
            
            # Step 1: Both connections read the same data
            results1 = conn1.execute_query(
                f"SELECT id, name, value FROM {self.table_name} ORDER BY id"
            )
            results2 = conn2.execute_query(
                f"SELECT id, name, value FROM {self.table_name} ORDER BY id"
            )
            print(f"✓ Connection 1 read {len(results1)} rows")
            print(f"✓ Connection 2 read {len(results2)} rows")
            
            # Step 2: Start transactions
            conn1.execute_query("START TRANSACTION")
            conn2.execute_query("START TRANSACTION")
            print("✓ Started transactions on both connections")
            
            # Step 3: Connection 1 updates a row
            conn1.execute_query(
                f"UPDATE {self.table_name} SET value = 999 WHERE id = 5"
            )
            print("✓ Connection 1 updated row with id=5 (value=999)")
            
            # Step 4: Connection 2 reads the same row (should see old value)
            value = conn2.execute_query(
                f"SELECT value FROM {self.table_name} WHERE id = 5"
            )[0]["value"]
            
            if value == 50:
                print(
                    "✓ Connection 2 correctly sees old value (50) - "
                    "Repeatable Read working!"
                )
            else:
                print(
                    f"✗ Connection 2 sees new value ({value}) - "
                    "Repeatable Read may not be working"
                )
            
            # Step 5: Connection 1 commits
            conn1.execute_query("COMMIT")
            print("✓ Connection 1 committed transaction")
            
            # Step 6: Connection 2 reads again (should still see old value)
            value = conn2.execute_query(
                f"SELECT value FROM {self.table_name} WHERE id = 5"
            )[0]["value"]
            
            if value == 50:
                print(
                    "✓ Connection 2 still sees old value (50) - "
                    "Isolation maintained!"
                )
            else:
                print(
                    f"✗ Connection 2 sees new value ({value}) - "
                    "Isolation may be broken"
                )
            
            # Step 7: Connection 2 commits and reads again
            conn2.execute_query("COMMIT")
            print("✓ Connection 2 committed transaction")
            
            value = conn2.execute_query(
                f"SELECT value FROM {self.table_name} WHERE id = 5"
            )[0]["value"]
            
            if value == 999:
                print(
                    "✓ Connection 2 now sees updated value (999) - "
                    "Transaction isolation working correctly!"
                )
            else:
                print(f"✗ Connection 2 sees unexpected value ({value})")
            
            print("[IsolationTest] Isolation test completed.")
            
        except Exception as e:
            print(f"[IsolationTest] Error during isolation test: {e}")
        
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        """Exit the isolation test state."""
        print("[IsolationTest] Exiting isolation test state.")


# Register the handler for the 'testing_isolation' state
handler_registry = {
    'testing_isolation': IsolationTestPythonHandler,
}

# Example of how to use these handlers
if __name__ == "__main__":
    print("Python handlers loaded successfully!")
    print("Available handlers:")
    print("  - ExamplePythonHandler")
    print("  - ImportJobPythonHandler")
    print("  - MonitoringPythonHandler")
    print("  - IsolationTestPythonHandler (testing_isolation)") 
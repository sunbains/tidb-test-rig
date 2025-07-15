"""
TiDB IMPORT multi-connection test suite.

This test file contains multi-connection tests for TiDB IMPORT functionality.
One connection performs the import operations while another connection monitors
the import job status in real-time.
"""

import os
import tempfile
import subprocess
import time
import threading
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState


class MultiConnectionImportHandler(PyStateHandler):
    """Test IMPORT INTO with multi-connection monitoring."""
    
    def __init__(self):
        self.import_connection = None
        self.monitor_connection = None
        self.import_job_id = None
        self.monitoring_active = False
        self.monitor_thread = None
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            # Create the import table
            context.connection.execute_query("DROP TABLE IF EXISTS multi_import_test")
            context.connection.execute_query("""
                CREATE TABLE multi_import_test (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    age INT,
                    email VARCHAR(200),
                    city VARCHAR(50),
                    department VARCHAR(50),
                    salary INT
                )
            """)
            
            # Store this as the import connection
            self.import_connection = context.connection
            
            # Create a second connection for monitoring
            try:
                from src.common.test_rig_python import RealPyConnection
                import os
                
                host = os.environ.get('TIDB_HOST', 'localhost:4000')
                username = os.environ.get('TIDB_USER', 'root')
                password = os.environ.get('TIDB_PASSWORD', '')
                database = os.environ.get('TIDB_DATABASE', 'test')
                
                self.monitor_connection = RealPyConnection(
                    connection_info={'id': 'monitor_conn'}, 
                    connection_id='monitor_conn',
                    host=host,
                    username=username,
                    password=password,
                    database=database
                )
                
                print("✅ Created monitor connection")
            except Exception as e:
                print(f"⚠️  Could not create monitor connection: {e}")
                self.monitor_connection = None
        
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Generate test data using create_import.py
            temp_file = "temp_multi_import.csv"
            try:
                # Generate 50k rows of complex data
                script_path = os.path.join(os.path.dirname(__file__), "create_import.py")
                cmd = ["python", script_path, "--rows", "50000", "--output", temp_file]
                result = subprocess.run(cmd, capture_output=True, text=True, cwd=os.getcwd())
                
                if result.returncode != 0:
                    return PyState.error(f"Failed to generate test data: {result.stderr}")
                
                print(f"Generated multi-connection test data: {temp_file}")
                
                # Start monitoring thread if we have a monitor connection
                if self.monitor_connection:
                    self.start_monitoring()
                
                # Execute IMPORT INTO with monitoring
                start_time = time.time()
                context.connection.execute_query(f"""
                    IMPORT INTO multi_import_test (id, name, age, email, city, department, salary)
                    FROM '{temp_file}'
                    FORMAT CSV
                    FIELDS TERMINATED BY ','
                    OPTIONALLY ENCLOSED BY '"'
                    LINES TERMINATED BY '\n'
                    IGNORE 1 LINES
                """)
                import_time = time.time() - start_time
                
                # Stop monitoring
                if self.monitor_connection:
                    self.stop_monitoring()
                
                # Verify the data was imported
                result = context.connection.execute_query("SELECT COUNT(*) as count FROM multi_import_test")
                if result and result[0].get('count', 0) == 50000:
                    print(f"✅ Multi-connection import successful: 50,000 rows in {import_time:.2f} seconds")
                    return PyState.completed()
                else:
                    return PyState.error(f"Data count verification failed. Expected 50000, got {result[0].get('count', 0) if result else 0}")
            finally:
                # Clean up temp file
                if os.path.exists(temp_file):
                    os.unlink(temp_file)
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        # Stop monitoring if still active
        if self.monitoring_active:
            self.stop_monitoring()
        
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS multi_import_test")
    
    def start_monitoring(self):
        """Start monitoring import jobs in a separate thread."""
        if not self.monitor_connection or self.monitoring_active:
            return
        
        self.monitoring_active = True
        self.monitor_thread = threading.Thread(target=self._monitor_import_jobs)
        self.monitor_thread.daemon = True
        self.monitor_thread.start()
        print("🔍 Started import job monitoring thread")
    
    def stop_monitoring(self):
        """Stop monitoring import jobs."""
        self.monitoring_active = False
        if self.monitor_thread and self.monitor_thread.is_alive():
            self.monitor_thread.join(timeout=5)
        print("🔍 Stopped import job monitoring")
    
    def _monitor_import_jobs(self):
        """Monitor import jobs and print status updates."""
        try:
            while self.monitoring_active:
                try:
                    # Get active import jobs
                    results = self.monitor_connection.execute_query("SHOW IMPORT JOBS")
                    
                    active_jobs = []
                    for row in results:
                        # Check if job is active (no end time)
                        end_time = None
                        for key, value in row.items():
                            if 'end_time' in key.lower() or 'End_Time' in key:
                                end_time = value
                                break
                        
                        if end_time is None or end_time == 'NULL':
                            active_jobs.append(row)
                    
                    if active_jobs:
                        print(f"📊 Monitoring {len(active_jobs)} active import job(s)...")
                        for job in active_jobs:
                            job_id = job.get('Job_ID', 'Unknown')
                            phase = job.get('Phase', 'Unknown')
                            status = job.get('Status', 'Unknown')
                            imported_rows = job.get('Imported_Rows', 0)
                            source_size = job.get('Source_File_Size', 'Unknown')
                            
                            print(f"   Job {job_id}: {phase} | {status} | {imported_rows} rows | {source_size}")
                    else:
                        print("📊 No active import jobs found")
                    
                    # Sleep before next check
                    time.sleep(3)
                    
                except Exception as e:
                    print(f"⚠️  Error during monitoring: {e}")
                    time.sleep(5)
                    
        except Exception as e:
            print(f"❌ Monitoring thread error: {e}")


class ConcurrentImportHandler(PyStateHandler):
    """Test concurrent imports with monitoring."""
    
    def __init__(self):
        self.import_connections = []
        self.monitor_connection = None
        self.monitoring_active = False
        self.monitor_thread = None
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            # Create multiple import tables
            for i in range(3):
                table_name = f"concurrent_import_test_{i}"
                context.connection.execute_query(f"DROP TABLE IF EXISTS {table_name}")
                context.connection.execute_query(f"""
                    CREATE TABLE {table_name} (
                        id INT PRIMARY KEY,
                        name VARCHAR(100),
                        age INT,
                        email VARCHAR(200)
                    )
                """)
            
            # Store this as the first import connection
            self.import_connections.append(context.connection)
            
            # Create a monitor connection
            try:
                from src.common.test_rig_python import RealPyConnection
                import os
                
                host = os.environ.get('TIDB_HOST', 'localhost:4000')
                username = os.environ.get('TIDB_USER', 'root')
                password = os.environ.get('TIDB_PASSWORD', '')
                database = os.environ.get('TIDB_DATABASE', 'test')
                
                self.monitor_connection = RealPyConnection(
                    connection_info={'id': 'monitor_conn'}, 
                    connection_id='monitor_conn',
                    host=host,
                    username=username,
                    password=password,
                    database=database
                )
                
                print("✅ Created monitor connection for concurrent imports")
            except Exception as e:
                print(f"⚠️  Could not create monitor connection: {e}")
                self.monitor_connection = None
        
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Generate test data files
            temp_files = []
            try:
                for i in range(3):
                    temp_file = f"temp_concurrent_{i}.csv"
                    # Generate 10k rows for each table
                    script_path = os.path.join(os.path.dirname(__file__), "create_import.py")
                    cmd = ["python", script_path, "--rows", "10000", "--output", temp_file]
                    result = subprocess.run(cmd, capture_output=True, text=True, cwd=os.getcwd())
                    
                    if result.returncode != 0:
                        return PyState.error(f"Failed to generate test data {i}: {result.stderr}")
                    
                    temp_files.append(temp_file)
                
                print(f"Generated {len(temp_files)} concurrent test data files")
                
                # Start monitoring
                if self.monitor_connection:
                    self.start_monitoring()
                
                # Execute concurrent imports
                start_time = time.time()
                import_threads = []
                
                for i, temp_file in enumerate(temp_files):
                    table_name = f"concurrent_import_test_{i}"
                    
                    def run_import(table_name, file_path, conn):
                        try:
                            conn.execute_query(f"""
                                IMPORT INTO {table_name} (id, name, age, email)
                                FROM '{file_path}'
                                FORMAT CSV
                                FIELDS TERMINATED BY ','
                                OPTIONALLY ENCLOSED BY '"'
                                LINES TERMINATED BY '\n'
                                IGNORE 1 LINES
                            """)
                            print(f"✅ Import completed for {table_name}")
                        except Exception as e:
                            print(f"❌ Import failed for {table_name}: {e}")
                    
                    thread = threading.Thread(
                        target=run_import,
                        args=(table_name, temp_file, context.connection)
                    )
                    thread.start()
                    import_threads.append(thread)
                
                # Wait for all imports to complete
                for thread in import_threads:
                    thread.join()
                
                import_time = time.time() - start_time
                
                # Stop monitoring
                if self.monitor_connection:
                    self.stop_monitoring()
                
                # Verify all imports
                total_rows = 0
                for i in range(3):
                    table_name = f"concurrent_import_test_{i}"
                    result = context.connection.execute_query(f"SELECT COUNT(*) as count FROM {table_name}")
                    if result:
                        row_count = result[0].get('count', 0)
                        total_rows += row_count
                        print(f"📊 {table_name}: {row_count} rows")
                
                if total_rows == 30000:  # 3 tables * 10000 rows
                    print(f"✅ Concurrent import successful: {total_rows} total rows in {import_time:.2f} seconds")
                    return PyState.completed()
                else:
                    return PyState.error(f"Data count verification failed. Expected 30000, got {total_rows}")
                    
            finally:
                # Clean up temp files
                for temp_file in temp_files:
                    if os.path.exists(temp_file):
                        os.unlink(temp_file)
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        # Stop monitoring if still active
        if self.monitoring_active:
            self.stop_monitoring()
        
        if context.connection:
            # Clean up all tables
            for i in range(3):
                table_name = f"concurrent_import_test_{i}"
                context.connection.execute_query(f"DROP TABLE IF EXISTS {table_name}")
    
    def start_monitoring(self):
        """Start monitoring import jobs in a separate thread."""
        if not self.monitor_connection or self.monitoring_active:
            return
        
        self.monitoring_active = True
        self.monitor_thread = threading.Thread(target=self._monitor_import_jobs)
        self.monitor_thread.daemon = True
        self.monitor_thread.start()
        print("🔍 Started concurrent import monitoring thread")
    
    def stop_monitoring(self):
        """Stop monitoring import jobs."""
        self.monitoring_active = False
        if self.monitor_thread and self.monitor_thread.is_alive():
            self.monitor_thread.join(timeout=5)
        print("🔍 Stopped concurrent import monitoring")
    
    def _monitor_import_jobs(self):
        """Monitor import jobs and print status updates."""
        try:
            while self.monitoring_active:
                try:
                    # Get active import jobs
                    results = self.monitor_connection.execute_query("SHOW IMPORT JOBS")
                    
                    active_jobs = []
                    for row in results:
                        # Check if job is active (no end time)
                        end_time = None
                        for key, value in row.items():
                            if 'end_time' in key.lower() or 'End_Time' in key:
                                end_time = value
                                break
                        
                        if end_time is None or end_time == 'NULL':
                            active_jobs.append(row)
                    
                    if active_jobs:
                        print(f"📊 Concurrent monitoring: {len(active_jobs)} active import job(s)")
                        for job in active_jobs:
                            job_id = job.get('Job_ID', 'Unknown')
                            phase = job.get('Phase', 'Unknown')
                            status = job.get('Status', 'Unknown')
                            imported_rows = job.get('Imported_Rows', 0)
                            target_table = job.get('Target_Table', 'Unknown')
                            
                            print(f"   Job {job_id} ({target_table}): {phase} | {status} | {imported_rows} rows")
                    else:
                        print("📊 No active import jobs found")
                    
                    # Sleep before next check
                    time.sleep(2)
                    
                except Exception as e:
                    print(f"⚠️  Error during concurrent monitoring: {e}")
                    time.sleep(5)
                    
        except Exception as e:
            print(f"❌ Concurrent monitoring thread error: {e}")


class LargeImportWithMonitoringHandler(PyStateHandler):
    """Test large import with detailed monitoring."""
    
    def __init__(self):
        self.monitor_connection = None
        self.monitoring_active = False
        self.monitor_thread = None
        self.import_start_time = None
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            # Create a large import table
            context.connection.execute_query("DROP TABLE IF EXISTS large_monitored_import")
            context.connection.execute_query("""
                CREATE TABLE large_monitored_import (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    email VARCHAR(200),
                    phone VARCHAR(20),
                    city VARCHAR(50),
                    department VARCHAR(50),
                    job_title VARCHAR(100),
                    salary INT,
                    performance_score DECIMAL(5,2),
                    hire_date DATE,
                    is_active BOOLEAN,
                    notes TEXT,
                    years_experience INT,
                    projects_completed INT
                )
            """)
            
            # Create a monitor connection
            try:
                from src.common.test_rig_python import RealPyConnection
                import os
                
                host = os.environ.get('TIDB_HOST', 'localhost:4000')
                username = os.environ.get('TIDB_USER', 'root')
                password = os.environ.get('TIDB_PASSWORD', '')
                database = os.environ.get('TIDB_DATABASE', 'test')
                
                self.monitor_connection = RealPyConnection(
                    connection_info={'id': 'monitor_conn'}, 
                    connection_id='monitor_conn',
                    host=host,
                    username=username,
                    password=password,
                    database=database
                )
                
                print("✅ Created monitor connection for large import")
            except Exception as e:
                print(f"⚠️  Could not create monitor connection: {e}")
                self.monitor_connection = None
        
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Generate large test data
            temp_file = "temp_large_monitored.csv"
            try:
                # Generate 100k rows of complex data
                script_path = os.path.join(os.path.dirname(__file__), "create_import.py")
                cmd = ["python", script_path, "--rows", "100000", "--output", temp_file]
                result = subprocess.run(cmd, capture_output=True, text=True, cwd=os.getcwd())
                
                if result.returncode != 0:
                    return PyState.error(f"Failed to generate test data: {result.stderr}")
                
                print(f"Generated large monitored test data: {temp_file}")
                
                # Start monitoring
                if self.monitor_connection:
                    self.start_monitoring()
                
                # Execute large import with monitoring
                self.import_start_time = time.time()
                context.connection.execute_query(f"""
                    IMPORT INTO large_monitored_import
                    FROM '{temp_file}'
                    FORMAT CSV
                    FIELDS TERMINATED BY ','
                    OPTIONALLY ENCLOSED BY '"'
                    LINES TERMINATED BY '\n'
                    IGNORE 1 LINES
                """)
                import_time = time.time() - self.import_start_time
                
                # Stop monitoring
                if self.monitor_connection:
                    self.stop_monitoring()
                
                # Verify the data was imported
                result = context.connection.execute_query("SELECT COUNT(*) as count FROM large_monitored_import")
                if result and result[0].get('count', 0) == 100000:
                    print(f"✅ Large monitored import successful: 100,000 rows in {import_time:.2f} seconds")
                    return PyState.completed()
                else:
                    return PyState.error(f"Data count verification failed. Expected 100000, got {result[0].get('count', 0) if result else 0}")
            finally:
                # Clean up temp file
                if os.path.exists(temp_file):
                    os.unlink(temp_file)
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        # Stop monitoring if still active
        if self.monitoring_active:
            self.stop_monitoring()
        
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS large_monitored_import")
    
    def start_monitoring(self):
        """Start monitoring import jobs in a separate thread."""
        if not self.monitor_connection or self.monitoring_active:
            return
        
        self.monitoring_active = True
        self.monitor_thread = threading.Thread(target=self._monitor_import_jobs)
        self.monitor_thread.daemon = True
        self.monitor_thread.start()
        print("🔍 Started large import monitoring thread")
    
    def stop_monitoring(self):
        """Stop monitoring import jobs."""
        self.monitoring_active = False
        if self.monitor_thread and self.monitor_thread.is_alive():
            self.monitor_thread.join(timeout=5)
        print("🔍 Stopped large import monitoring")
    
    def _monitor_import_jobs(self):
        """Monitor import jobs with detailed progress tracking."""
        try:
            while self.monitoring_active:
                try:
                    # Get active import jobs
                    results = self.monitor_connection.execute_query("SHOW IMPORT JOBS")
                    
                    active_jobs = []
                    for row in results:
                        # Check if job is active (no end time)
                        end_time = None
                        for key, value in row.items():
                            if 'end_time' in key.lower() or 'End_Time' in key:
                                end_time = value
                                break
                        
                        if end_time is None or end_time == 'NULL':
                            active_jobs.append(row)
                    
                    if active_jobs:
                        current_time = time.time()
                        elapsed = current_time - self.import_start_time if self.import_start_time else 0
                        
                        print(f"📊 Large import monitoring ({elapsed:.1f}s elapsed): {len(active_jobs)} active job(s)")
                        for job in active_jobs:
                            job_id = job.get('Job_ID', 'Unknown')
                            phase = job.get('Phase', 'Unknown')
                            status = job.get('Status', 'Unknown')
                            imported_rows = job.get('Imported_Rows', 0)
                            source_size = job.get('Source_File_Size', 'Unknown')
                            target_table = job.get('Target_Table', 'Unknown')
                            
                            # Calculate progress if we have source size info
                            progress_info = ""
                            if source_size and source_size != 'Unknown':
                                try:
                                    # Try to extract numeric value from source size
                                    if 'MiB' in source_size:
                                        size_mb = float(source_size.replace('MiB', ''))
                                        if imported_rows > 0:
                                            estimated_total = int(size_mb * 1000)  # Rough estimate
                                            progress = min(100, (imported_rows / estimated_total) * 100)
                                            progress_info = f" | ~{progress:.1f}%"
                                except:
                                    pass
                            
                            print(f"   Job {job_id} ({target_table}): {phase} | {status} | {imported_rows:,} rows{progress_info} | {source_size}")
                    else:
                        print("📊 No active import jobs found")
                    
                    # Sleep before next check
                    time.sleep(5)
                    
                except Exception as e:
                    print(f"⚠️  Error during large import monitoring: {e}")
                    time.sleep(5)
                    
        except Exception as e:
            print(f"❌ Large import monitoring thread error: {e}") 
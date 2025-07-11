"""
Transaction performance tests for TiDB.

This test suite covers performance-related transaction scenarios including:
- Bulk operations
- Large transactions
- Performance monitoring
- Transaction size limits
"""

from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState
import time


class BulkInsertTestHandler(PyStateHandler):
    """Test bulk insert operations within transactions."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"bulk_insert_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )
            """)
            
            print(f"✓ Created bulk insert test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test bulk insert operations"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction")
                
                # Perform bulk insert (100 rows)
                for i in range(1, 101):
                    conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES ({i}, 'bulk_row_{i}', {i * 10})")
                
                results.append("✓ Inserted 100 rows in bulk")
                
                # Verify all rows were inserted
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 100:
                    results.append("✓ Verified 100 rows inserted")
                else:
                    results.append("✗ Row count verification failed")
                    return PyState.error("Row count verification failed")
                
                # Verify some specific rows
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 50")
                if data and data[0].get('value') == 500:
                    results.append("✓ Verified specific row (id=50, value=500)")
                else:
                    results.append("✗ Specific row verification failed")
                    return PyState.error("Specific row verification failed")
                
                # Commit transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed bulk insert transaction")
                
                # Verify final state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 100:
                    results.append("✓ Verified final state (100 rows)")
                else:
                    results.append("✗ Final state verification failed")
                    return PyState.error("Final state verification failed")
                
                print("\n=== BULK INSERT TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during bulk insert test: {e}")
                return PyState.error(f"Bulk insert test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up bulk insert test table")


class BulkUpdateTestHandler(PyStateHandler):
    """Test bulk update operations within transactions."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table with data"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"bulk_update_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT,
                    status VARCHAR(20) DEFAULT 'pending'
                )
            """)
            
            # Insert test data
            for i in range(1, 51):
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES ({i}, 'row_{i}', {i * 10})")
            
            print(f"✓ Created bulk update test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test bulk update operations"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction")
                
                # Bulk update all rows
                conn.execute_query(f"UPDATE {self.table_name} SET value = value + 100, status = 'processed' WHERE id <= 50")
                results.append("✓ Updated all 50 rows in bulk")
                
                # Verify updates
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name} WHERE status = 'processed'")
                if data and data[0].get('col_0', 0) == 50:
                    results.append("✓ Verified all rows marked as processed")
                else:
                    results.append("✗ Status update verification failed")
                    return PyState.error("Status update verification failed")
                
                # Verify value updates
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 25")
                if data and data[0].get('value') == 350:  # 25 * 10 + 100
                    results.append("✓ Verified value update (id=25, value=350)")
                else:
                    results.append("✗ Value update verification failed")
                    return PyState.error("Value update verification failed")
                
                # Commit transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed bulk update transaction")
                
                # Verify final state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name} WHERE status = 'processed' AND value > 100")
                if data and data[0].get('col_0', 0) == 50:
                    results.append("✓ Verified final state (50 processed rows)")
                else:
                    results.append("✗ Final state verification failed")
                    return PyState.error("Final state verification failed")
                
                print("\n=== BULK UPDATE TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during bulk update test: {e}")
                return PyState.error(f"Bulk update test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up bulk update test table")


class LargeTransactionTestHandler(PyStateHandler):
    """Test large transaction scenarios."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"large_txn_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT,
                    data TEXT,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )
            """)
            
            print(f"✓ Created large transaction test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test large transaction scenarios"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started large transaction")
                
                # Insert large amount of data
                for i in range(1, 201):
                    large_data = f"Large data content for row {i} with some additional text to make it substantial"
                    conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value, data) VALUES ({i}, 'large_row_{i}', {i * 5}, '{large_data}')")
                
                results.append("✓ Inserted 200 rows with large data")
                
                # Update some rows
                for i in range(1, 51):
                    conn.execute_query(f"UPDATE {self.table_name} SET value = value * 2 WHERE id = {i}")
                
                results.append("✓ Updated first 50 rows")
                
                # Verify transaction state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 200:
                    results.append("✓ Verified 200 rows in transaction")
                else:
                    results.append("✗ Row count verification failed")
                    return PyState.error("Row count verification failed")
                
                # Verify updates
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 25")
                if data and data[0].get('value') == 250:  # 25 * 5 * 2
                    results.append("✓ Verified update (id=25, value=250)")
                else:
                    results.append("✗ Update verification failed")
                    return PyState.error("Update verification failed")
                
                # Commit large transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed large transaction")
                
                # Verify final state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 200:
                    results.append("✓ Verified final state (200 rows)")
                else:
                    results.append("✗ Final state verification failed")
                    return PyState.error("Final state verification failed")
                
                print("\n=== LARGE TRANSACTION TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during large transaction test: {e}")
                return PyState.error(f"Large transaction test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up large transaction test table")


class TransactionPerformanceMonitoringTestHandler(PyStateHandler):
    """Test transaction performance monitoring."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"perf_monitor_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT,
                    execution_time_ms INT DEFAULT 0
                )
            """)
            
            print(f"✓ Created performance monitoring test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test transaction performance monitoring"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction")
                
                # Record start time
                start_time = time.time()
                
                # Perform operations
                for i in range(1, 21):
                    conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES ({i}, 'perf_row_{i}', {i * 10})")
                
                # Record end time
                end_time = time.time()
                execution_time_ms = int((end_time - start_time) * 1000)
                
                results.append(f"✓ Inserted 20 rows in {execution_time_ms}ms")
                
                # Update execution time
                conn.execute_query(f"UPDATE {self.table_name} SET execution_time_ms = {execution_time_ms} WHERE id = 1")
                results.append("✓ Recorded execution time")
                
                # Verify operations
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 20:
                    results.append("✓ Verified 20 rows inserted")
                else:
                    results.append("✗ Row count verification failed")
                    return PyState.error("Row count verification failed")
                
                # Commit transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed transaction")
                
                # Verify final state
                data = conn.execute_query(f"SELECT execution_time_ms FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('execution_time_ms') == execution_time_ms:
                    results.append("✓ Verified execution time recorded")
                else:
                    results.append("✗ Execution time verification failed")
                    return PyState.error("Execution time verification failed")
                
                print("\n=== TRANSACTION PERFORMANCE MONITORING TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during performance monitoring test: {e}")
                return PyState.error(f"Performance monitoring test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up performance monitoring test table")


class TransactionSizeLimitTestHandler(PyStateHandler):
    """Test transaction size limits and constraints."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"size_limit_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT,
                    large_field TEXT
                )
            """)
            
            print(f"✓ Created transaction size limit test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test transaction size limits"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction")
                
                # Insert data with varying sizes
                for i in range(1, 11):
                    # Create data of increasing size
                    large_data = "x" * (i * 100)  # 100, 200, 300, ..., 1000 characters
                    conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value, large_field) VALUES ({i}, 'size_row_{i}', {i * 10}, '{large_data}')")
                
                results.append("✓ Inserted 10 rows with varying data sizes")
                
                # Verify all rows were inserted
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 10:
                    results.append("✓ Verified 10 rows inserted")
                else:
                    results.append("✗ Row count verification failed")
                    return PyState.error("Row count verification failed")
                
                # Verify largest data field
                data = conn.execute_query(f"SELECT LENGTH(large_field) FROM {self.table_name} WHERE id = 10")
                if data and data[0].get('LENGTH(large_field)') == 1000:
                    results.append("✓ Verified largest data field (1000 characters)")
                else:
                    results.append("✗ Data size verification failed")
                    return PyState.error("Data size verification failed")
                
                # Commit transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed transaction")
                
                # Verify final state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 10:
                    results.append("✓ Verified final state (10 rows)")
                else:
                    results.append("✗ Final state verification failed")
                    return PyState.error("Final state verification failed")
                
                print("\n=== TRANSACTION SIZE LIMIT TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during transaction size limit test: {e}")
                return PyState.error(f"Transaction size limit test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up transaction size limit test table") 
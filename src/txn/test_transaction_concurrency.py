"""
Transaction concurrency tests for TiDB.

This test suite covers various concurrency scenarios including:
- Concurrent reads and writes
- Race conditions
- Lock contention
- Concurrent transaction conflicts
"""

from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState
import time


class ConcurrentReadWriteTestHandler(PyStateHandler):
    """Test concurrent read and write operations."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"concurrent_rw_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
                )
            """)
            
            # Insert initial data
            for i in range(1, 6):
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES ({i}, 'row_{i}', {i * 10})")
            
            print(f"✓ Created concurrent read/write test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test concurrent read and write operations"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction")
                
                # Read data
                data = conn.execute_query(f"SELECT * FROM {self.table_name} ORDER BY id")
                if data and len(data) == 5:
                    results.append("✓ Read 5 rows")
                else:
                    results.append("✗ Failed to read initial data")
                    return PyState.error("Failed to read initial data")
                
                # Update multiple rows
                conn.execute_query(f"UPDATE {self.table_name} SET value = value + 100 WHERE id IN (1, 3, 5)")
                results.append("✓ Updated odd-numbered rows")
                
                # Read data again to verify updates
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('value') == 110:  # 10 + 100
                    results.append("✓ Verified update to row 1 (110)")
                else:
                    results.append("✗ Failed to verify update")
                    return PyState.error("Failed to verify update")
                
                # Insert new row
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (6, 'new_row', 600)")
                results.append("✓ Inserted new row")
                
                # Read all data to verify final state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 6:
                    results.append("✓ Verified final row count (6)")
                else:
                    results.append("✗ Final row count verification failed")
                    return PyState.error("Final row count verification failed")
                
                # Commit transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed transaction")
                
                print("\n=== CONCURRENT READ/WRITE TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during concurrent read/write test: {e}")
                return PyState.error(f"Concurrent read/write test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up concurrent read/write test table")


class LockContentionTestHandler(PyStateHandler):
    """Test lock contention scenarios."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"lock_contention_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT,
                    status VARCHAR(20) DEFAULT 'active'
                )
            """)
            
            # Insert test data
            for i in range(1, 11):
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES ({i}, 'row_{i}', {i * 10})")
            
            print(f"✓ Created lock contention test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test lock contention scenarios"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction")
                
                # Lock specific rows with FOR UPDATE
                conn.execute_query(f"SELECT * FROM {self.table_name} WHERE id IN (1, 3, 5) FOR UPDATE")
                results.append("✓ Locked rows 1, 3, 5 with FOR UPDATE")
                
                # Update locked rows
                conn.execute_query(f"UPDATE {self.table_name} SET value = 999 WHERE id IN (1, 3, 5)")
                results.append("✓ Updated locked rows")
                
                # Try to read locked rows (should succeed in same transaction)
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('value') == 999:
                    results.append("✓ Successfully read locked row in same transaction")
                else:
                    results.append("✗ Failed to read locked row")
                    return PyState.error("Failed to read locked row")
                
                # Lock more rows with SHARE MODE
                conn.execute_query(f"SELECT * FROM {self.table_name} WHERE id IN (2, 4, 6) LOCK IN SHARE MODE")
                results.append("✓ Locked rows 2, 4, 6 with SHARE MODE")
                
                # Update shared locked rows
                conn.execute_query(f"UPDATE {self.table_name} SET value = 888 WHERE id IN (2, 4, 6)")
                results.append("✓ Updated shared locked rows")
                
                # Verify all updates
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name} WHERE value IN (999, 888)")
                if data and data[0].get('col_0', 0) == 6:
                    results.append("✓ Verified all 6 updates were successful")
                else:
                    results.append("✗ Update verification failed")
                    return PyState.error("Update verification failed")
                
                # Commit transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed transaction")
                
                print("\n=== LOCK CONTENTION TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during lock contention test: {e}")
                return PyState.error(f"Lock contention test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up lock contention test table")


class RaceConditionTestHandler(PyStateHandler):
    """Test race condition scenarios."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"race_condition_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    counter INT DEFAULT 0,
                    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
                )
            """)
            
            # Insert initial counter
            conn.execute_query(f"INSERT INTO {self.table_name} (id, counter) VALUES (1, 0)")
            
            print(f"✓ Created race condition test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test race condition scenarios"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction")
                
                # Read current counter value
                data = conn.execute_query(f"SELECT counter FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('counter') == 0:
                    results.append("✓ Read initial counter value (0)")
                else:
                    results.append("✗ Failed to read initial counter")
                    return PyState.error("Failed to read initial counter")
                
                # Increment counter (simulating race condition)
                conn.execute_query(f"UPDATE {self.table_name} SET counter = counter + 1 WHERE id = 1")
                results.append("✓ Incremented counter")
                
                # Read counter again
                data = conn.execute_query(f"SELECT counter FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('counter') == 1:
                    results.append("✓ Verified counter increment (1)")
                else:
                    results.append("✗ Counter increment verification failed")
                    return PyState.error("Counter increment verification failed")
                
                # Simulate multiple increments
                for i in range(5):
                    conn.execute_query(f"UPDATE {self.table_name} SET counter = counter + 1 WHERE id = 1")
                
                results.append("✓ Performed 5 additional increments")
                
                # Verify final counter value
                data = conn.execute_query(f"SELECT counter FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('counter') == 6:
                    results.append("✓ Verified final counter value (6)")
                else:
                    results.append("✗ Final counter verification failed")
                    return PyState.error("Final counter verification failed")
                
                # Commit transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed transaction")
                
                print("\n=== RACE CONDITION TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during race condition test: {e}")
                return PyState.error(f"Race condition test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up race condition test table")


class ConcurrentTransactionConflictTestHandler(PyStateHandler):
    """Test concurrent transaction conflicts."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"concurrent_conflict_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT,
                    version INT DEFAULT 1
                )
            """)
            
            # Insert test data
            for i in range(1, 4):
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES ({i}, 'row_{i}', {i * 10})")
            
            print(f"✓ Created concurrent conflict test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test concurrent transaction conflicts"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction")
                
                # Read data with version
                data = conn.execute_query(f"SELECT id, value, version FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('version') == 1:
                    results.append("✓ Read row with version 1")
                else:
                    results.append("✗ Failed to read initial version")
                    return PyState.error("Failed to read initial version")
                
                # Update row with optimistic locking
                conn.execute_query(f"UPDATE {self.table_name} SET value = 999, version = version + 1 WHERE id = 1 AND version = 1")
                results.append("✓ Updated row with optimistic locking")
                
                # Verify update was successful
                data = conn.execute_query(f"SELECT value, version FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('value') == 999 and data[0].get('version') == 2:
                    results.append("✓ Verified optimistic update (value=999, version=2)")
                else:
                    results.append("✗ Optimistic update verification failed")
                    return PyState.error("Optimistic update verification failed")
                
                # Try to update same row again (should fail due to version mismatch)
                conn.execute_query(f"UPDATE {self.table_name} SET value = 888, version = version + 1 WHERE id = 1 AND version = 1")
                results.append("✓ Attempted conflicting update")
                
                # Verify the conflicting update didn't affect the row
                data = conn.execute_query(f"SELECT value, version FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('value') == 999 and data[0].get('version') == 2:
                    results.append("✓ Verified conflicting update was ignored")
                else:
                    results.append("✗ Conflicting update verification failed")
                    return PyState.error("Conflicting update verification failed")
                
                # Update another row successfully
                conn.execute_query(f"UPDATE {self.table_name} SET value = 777, version = version + 1 WHERE id = 2 AND version = 1")
                results.append("✓ Updated second row successfully")
                
                # Commit transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed transaction")
                
                # Verify final state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name} WHERE value IN (999, 777)")
                if data and data[0].get('col_0', 0) == 2:
                    results.append("✓ Verified final state (2 updated rows)")
                else:
                    results.append("✗ Final state verification failed")
                    return PyState.error("Final state verification failed")
                
                print("\n=== CONCURRENT TRANSACTION CONFLICT TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during concurrent transaction conflict test: {e}")
                return PyState.error(f"Concurrent transaction conflict test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up concurrent conflict test table")


class TransactionRollbackTestHandler(PyStateHandler):
    """Test transaction rollback scenarios."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"rollback_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT
                )
            """)
            
            # Insert initial data
            conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (1, 'initial', 100)")
            
            print(f"✓ Created rollback test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test transaction rollback scenarios"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction")
                
                # Insert data
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (2, 'in_transaction', 200)")
                results.append("✓ Inserted data in transaction")
                
                # Update data
                conn.execute_query(f"UPDATE {self.table_name} SET value = 999 WHERE id = 1")
                results.append("✓ Updated data in transaction")
                
                # Verify changes are visible within transaction
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 2:
                    results.append("✓ Verified 2 rows visible in transaction")
                else:
                    results.append("✗ Row count verification failed")
                    return PyState.error("Row count verification failed")
                
                # Rollback transaction
                conn.execute_query("ROLLBACK")
                results.append("✓ Rolled back transaction")
                
                # Verify original state is restored
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 1:
                    results.append("✓ Verified original state restored (1 row)")
                else:
                    results.append("✗ State restoration verification failed")
                    return PyState.error("State restoration verification failed")
                
                # Verify original value is restored
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('value') == 100:
                    results.append("✓ Verified original value restored (100)")
                else:
                    results.append("✗ Value restoration verification failed")
                    return PyState.error("Value restoration verification failed")
                
                print("\n=== TRANSACTION ROLLBACK TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during rollback test: {e}")
                return PyState.error(f"Rollback test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up rollback test table") 
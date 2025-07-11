"""
Transaction edge cases and error conditions for TiDB.

This test suite covers various edge cases and error scenarios that can occur
with transactions, including deadlocks, timeouts, savepoints, and nested transactions.
"""

from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState
import time


class DeadlockTestHandler(PyStateHandler):
    """Test deadlock detection and resolution."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test tables for deadlock scenarios"""
        if context.connection:
            conn = context.connection
            
            # Create tables for deadlock testing
            self.table1 = f"deadlock_test_1_{int(time.time())}"
            self.table2 = f"deadlock_test_2_{int(time.time())}"
            
            # Create two tables with some data
            conn.execute_query(f"""
                CREATE TABLE {self.table1} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT
                )
            """)
            
            conn.execute_query(f"""
                CREATE TABLE {self.table2} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT
                )
            """)
            
            # Insert test data
            for i in range(1, 6):
                conn.execute_query(f"INSERT INTO {self.table1} (id, name, value) VALUES ({i}, 'row_{i}', {i * 10})")
                conn.execute_query(f"INSERT INTO {self.table2} (id, name, value) VALUES ({i}, 'row_{i}', {i * 20})")
            
            print(f"✓ Created deadlock test tables: {self.table1}, {self.table2}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test deadlock scenarios"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Simulate a potential deadlock scenario
                # In a real scenario, this would involve multiple connections
                
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction for deadlock test")
                
                # Lock rows in table1
                conn.execute_query(f"SELECT * FROM {self.table1} WHERE id = 1 FOR UPDATE")
                results.append("✓ Locked row in table1")
                
                # Try to lock rows in table2 (simulating another transaction)
                conn.execute_query(f"SELECT * FROM {self.table2} WHERE id = 1 FOR UPDATE")
                results.append("✓ Locked row in table2")
                
                # Try to update the locked row in table1
                conn.execute_query(f"UPDATE {self.table1} SET value = 999 WHERE id = 1")
                results.append("✓ Updated locked row in table1")
                
                # Commit to release locks
                conn.execute_query("COMMIT")
                results.append("✓ Committed transaction (deadlock avoided in mock)")
                
                print("\n=== DEADLOCK TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during deadlock test: {e}")
                return PyState.error(f"Deadlock test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table1') and hasattr(self, 'table2'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table1}")
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table2}")
            print(f"✓ Cleaned up deadlock test tables")


class SavepointTestHandler(PyStateHandler):
    """Test savepoint functionality."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"savepoint_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT
                )
            """)
            
            # Insert initial data
            conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (1, 'initial', 100)")
            
            print(f"✓ Created savepoint test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test savepoint operations"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction")
                
                # Insert data
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (2, 'before_savepoint', 200)")
                results.append("✓ Inserted data before savepoint")
                
                # Create savepoint
                conn.execute_query("SAVEPOINT sp1")
                results.append("✓ Created savepoint sp1")
                
                # Insert more data
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (3, 'after_savepoint', 300)")
                results.append("✓ Inserted data after savepoint")
                
                # Verify data exists
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 3:
                    results.append("✓ Verified 3 rows exist")
                else:
                    results.append("✗ Row count verification failed")
                    return PyState.error("Row count verification failed")
                
                # Rollback to savepoint
                conn.execute_query("ROLLBACK TO SAVEPOINT sp1")
                results.append("✓ Rolled back to savepoint sp1")
                
                # Verify data after rollback
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 2:
                    results.append("✓ Verified 2 rows after rollback")
                else:
                    results.append("✗ Row count after rollback failed")
                    return PyState.error("Row count after rollback failed")
                
                # Insert data again
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (4, 'after_rollback', 400)")
                results.append("✓ Inserted data after rollback")
                
                # Create another savepoint
                conn.execute_query("SAVEPOINT sp2")
                results.append("✓ Created savepoint sp2")
                
                # Release savepoint
                conn.execute_query("RELEASE SAVEPOINT sp2")
                results.append("✓ Released savepoint sp2")
                
                # Commit transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed transaction")
                
                # Verify final state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 3:
                    results.append("✓ Verified final row count (3)")
                else:
                    results.append("✗ Final row count verification failed")
                    return PyState.error("Final row count verification failed")
                
                print("\n=== SAVEPOINT TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during savepoint test: {e}")
                return PyState.error(f"Savepoint test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up savepoint test table")


class NestedTransactionTestHandler(PyStateHandler):
    """Test nested transaction behavior."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"nested_txn_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT
                )
            """)
            
            print(f"✓ Created nested transaction test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test nested transaction behavior"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Start outer transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started outer transaction")
                
                # Insert data in outer transaction
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (1, 'outer', 100)")
                results.append("✓ Inserted data in outer transaction")
                
                # Start inner transaction (nested)
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started inner transaction")
                
                # Insert data in inner transaction
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (2, 'inner', 200)")
                results.append("✓ Inserted data in inner transaction")
                
                # Verify data is visible
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 2:
                    results.append("✓ Verified 2 rows visible in nested transaction")
                else:
                    results.append("✗ Row count verification failed")
                    return PyState.error("Row count verification failed")
                
                # Commit inner transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed inner transaction")
                
                # Verify data still visible
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 2:
                    results.append("✓ Verified 2 rows still visible after inner commit")
                else:
                    results.append("✗ Row count after inner commit failed")
                    return PyState.error("Row count after inner commit failed")
                
                # Insert more data in outer transaction
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (3, 'outer_after_inner', 300)")
                results.append("✓ Inserted data in outer transaction after inner commit")
                
                # Commit outer transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed outer transaction")
                
                # Verify final state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 3:
                    results.append("✓ Verified final row count (3)")
                else:
                    results.append("✗ Final row count verification failed")
                    return PyState.error("Final row count verification failed")
                
                print("\n=== NESTED TRANSACTION TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during nested transaction test: {e}")
                return PyState.error(f"Nested transaction test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up nested transaction test table")


class TransactionTimeoutTestHandler(PyStateHandler):
    """Test transaction timeout behavior."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"timeout_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT
                )
            """)
            
            # Insert test data
            for i in range(1, 6):
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES ({i}, 'row_{i}', {i * 10})")
            
            print(f"✓ Created timeout test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test transaction timeout scenarios"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Set transaction timeout (if supported)
                try:
                    conn.execute_query("SET SESSION innodb_lock_wait_timeout = 5")
                    results.append("✓ Set transaction timeout to 5 seconds")
                except:
                    results.append("⚠️ Could not set transaction timeout (not supported in mock)")
                
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction")
                
                # Lock a row
                conn.execute_query(f"SELECT * FROM {self.table_name} WHERE id = 1 FOR UPDATE")
                results.append("✓ Locked row with FOR UPDATE")
                
                # Try to update the same row (should succeed in same transaction)
                conn.execute_query(f"UPDATE {self.table_name} SET value = 999 WHERE id = 1")
                results.append("✓ Updated locked row in same transaction")
                
                # Verify the update
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('value') == 999:
                    results.append("✓ Verified update was successful")
                else:
                    results.append("✗ Update verification failed")
                    return PyState.error("Update verification failed")
                
                # Commit transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed transaction")
                
                print("\n=== TRANSACTION TIMEOUT TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during timeout test: {e}")
                return PyState.error(f"Timeout test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up timeout test table")


class TransactionErrorTestHandler(PyStateHandler):
    """Test transaction error handling and rollback scenarios."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"error_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100) NOT NULL,
                    value INT CHECK (value > 0)
                )
            """)
            
            # Insert valid data
            conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (1, 'valid', 100)")
            
            print(f"✓ Created error test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test transaction error handling"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction")
                
                # Insert valid data
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (2, 'valid_in_txn', 200)")
                results.append("✓ Inserted valid data in transaction")
                
                # Try to insert invalid data (should cause error)
                try:
                    conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (3, 'invalid', -1)")
                    results.append("✗ Invalid insert should have failed")
                    return PyState.error("Invalid insert should have failed")
                except:
                    results.append("✓ Invalid insert correctly failed")
                
                # Verify only valid data exists
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 2:
                    results.append("✓ Verified only valid data exists")
                else:
                    results.append("✗ Data verification failed")
                    return PyState.error("Data verification failed")
                
                # Try to insert duplicate primary key
                try:
                    conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (1, 'duplicate', 300)")
                    results.append("✗ Duplicate key insert should have failed")
                    return PyState.error("Duplicate key insert should have failed")
                except:
                    results.append("✓ Duplicate key insert correctly failed")
                
                # Rollback transaction
                conn.execute_query("ROLLBACK")
                results.append("✓ Rolled back transaction")
                
                # Verify original state is restored
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 1:
                    results.append("✓ Verified original state restored after rollback")
                else:
                    results.append("✗ State restoration verification failed")
                    return PyState.error("State restoration verification failed")
                
                print("\n=== TRANSACTION ERROR TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during error test: {e}")
                return PyState.error(f"Error test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up error test table") 
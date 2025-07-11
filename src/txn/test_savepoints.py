"""
Comprehensive transaction savepoint tests for TiDB.

This test suite covers:
- Savepoint creation and naming
- Rollback to savepoints
- Savepoint release
- Nested savepoints
- Savepoint edge cases and error conditions
- Savepoint with different isolation levels
- Savepoint performance and limits
"""

from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState
import time


class BasicSavepointTestHandler(PyStateHandler):
    """Test basic savepoint operations."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"basic_savepoint_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT,
                    status VARCHAR(20) DEFAULT 'pending'
                )
            """)
            
            # Insert initial data
            conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (1, 'initial', 100)")
            
            print(f"✓ Created basic savepoint test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test basic savepoint operations"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction")
                
                # Insert data before savepoint
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (2, 'before_savepoint', 200)")
                results.append("✓ Inserted data before savepoint")
                
                # Create savepoint
                conn.execute_query("SAVEPOINT sp1")
                results.append("✓ Created savepoint sp1")
                
                # Insert data after savepoint
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
                
                # Commit transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed transaction")
                
                # Verify final state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 3:
                    results.append("✓ Verified final state (3 rows)")
                else:
                    results.append("✗ Final state verification failed")
                    return PyState.error("Final state verification failed")
                
                print("\n=== BASIC SAVEPOINT TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during basic savepoint test: {e}")
                return PyState.error(f"Basic savepoint test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up basic savepoint test table")


class NestedSavepointTestHandler(PyStateHandler):
    """Test nested savepoint operations."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"nested_savepoint_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT,
                    level INT DEFAULT 0
                )
            """)
            
            print(f"✓ Created nested savepoint test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test nested savepoint operations"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction")
                
                # Level 1: Insert data
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value, level) VALUES (1, 'level1', 100, 1)")
                results.append("✓ Inserted level 1 data")
                
                # Create first savepoint
                conn.execute_query("SAVEPOINT level1")
                results.append("✓ Created savepoint level1")
                
                # Level 2: Insert more data
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value, level) VALUES (2, 'level2', 200, 2)")
                results.append("✓ Inserted level 2 data")
                
                # Create second savepoint
                conn.execute_query("SAVEPOINT level2")
                results.append("✓ Created savepoint level2")
                
                # Level 3: Insert more data
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value, level) VALUES (3, 'level3', 300, 3)")
                results.append("✓ Inserted level 3 data")
                
                # Create third savepoint
                conn.execute_query("SAVEPOINT level3")
                results.append("✓ Created savepoint level3")
                
                # Verify all data exists
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 3:
                    results.append("✓ Verified 3 rows exist")
                else:
                    results.append("✗ Row count verification failed")
                    return PyState.error("Row count verification failed")
                
                # Rollback to level2
                conn.execute_query("ROLLBACK TO SAVEPOINT level2")
                results.append("✓ Rolled back to savepoint level2")
                
                # Verify level2 state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 2:
                    results.append("✓ Verified level2 state (2 rows)")
                else:
                    results.append("✗ Level2 state verification failed")
                    return PyState.error("Level2 state verification failed")
                
                # Insert data at level2
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value, level) VALUES (4, 'level2_new', 400, 2)")
                results.append("✓ Inserted new level2 data")
                
                # Rollback to level1
                conn.execute_query("ROLLBACK TO SAVEPOINT level1")
                results.append("✓ Rolled back to savepoint level1")
                
                # Verify level1 state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 1:
                    results.append("✓ Verified level1 state (1 row)")
                else:
                    results.append("✗ Level1 state verification failed")
                    return PyState.error("Level1 state verification failed")
                
                # Insert data at level1
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value, level) VALUES (5, 'level1_new', 500, 1)")
                results.append("✓ Inserted new level1 data")
                
                # Commit transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed transaction")
                
                # Verify final state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 2:
                    results.append("✓ Verified final state (2 rows)")
                else:
                    results.append("✗ Final state verification failed")
                    return PyState.error("Final state verification failed")
                
                print("\n=== NESTED SAVEPOINT TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during nested savepoint test: {e}")
                return PyState.error(f"Nested savepoint test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up nested savepoint test table")


class SavepointReleaseTestHandler(PyStateHandler):
    """Test savepoint release operations."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"savepoint_release_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT
                )
            """)
            
            print(f"✓ Created savepoint release test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test savepoint release operations"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction")
                
                # Insert initial data
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (1, 'initial', 100)")
                results.append("✓ Inserted initial data")
                
                # Create first savepoint
                conn.execute_query("SAVEPOINT sp1")
                results.append("✓ Created savepoint sp1")
                
                # Insert data after sp1
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (2, 'after_sp1', 200)")
                results.append("✓ Inserted data after sp1")
                
                # Create second savepoint
                conn.execute_query("SAVEPOINT sp2")
                results.append("✓ Created savepoint sp2")
                
                # Insert data after sp2
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (3, 'after_sp2', 300)")
                results.append("✓ Inserted data after sp2")
                
                # Verify all data exists
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 3:
                    results.append("✓ Verified 3 rows exist")
                else:
                    results.append("✗ Row count verification failed")
                    return PyState.error("Row count verification failed")
                
                # Release savepoint sp2
                conn.execute_query("RELEASE SAVEPOINT sp2")
                results.append("✓ Released savepoint sp2")
                
                # Try to rollback to released savepoint (should fail)
                try:
                    conn.execute_query("ROLLBACK TO SAVEPOINT sp2")
                    results.append("✗ Rollback to released savepoint should have failed")
                    return PyState.error("Rollback to released savepoint should have failed")
                except:
                    results.append("✓ Rollback to released savepoint correctly failed")
                
                # Rollback to sp1 (should still work)
                conn.execute_query("ROLLBACK TO SAVEPOINT sp1")
                results.append("✓ Rolled back to savepoint sp1")
                
                # Verify sp1 state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 2:
                    results.append("✓ Verified sp1 state (2 rows)")
                else:
                    results.append("✗ Sp1 state verification failed")
                    return PyState.error("Sp1 state verification failed")
                
                # Release savepoint sp1
                conn.execute_query("RELEASE SAVEPOINT sp1")
                results.append("✓ Released savepoint sp1")
                
                # Try to rollback to released savepoint (should fail)
                try:
                    conn.execute_query("ROLLBACK TO SAVEPOINT sp1")
                    results.append("✗ Rollback to released savepoint should have failed")
                    return PyState.error("Rollback to released savepoint should have failed")
                except:
                    results.append("✓ Rollback to released savepoint correctly failed")
                
                # Insert final data
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (4, 'final', 400)")
                results.append("✓ Inserted final data")
                
                # Commit transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed transaction")
                
                # Verify final state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 3:
                    results.append("✓ Verified final state (3 rows)")
                else:
                    results.append("✗ Final state verification failed")
                    return PyState.error("Final state verification failed")
                
                print("\n=== SAVEPOINT RELEASE TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during savepoint release test: {e}")
                return PyState.error(f"Savepoint release test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up savepoint release test table")


class SavepointErrorTestHandler(PyStateHandler):
    """Test savepoint error conditions and edge cases."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"savepoint_error_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100) NOT NULL,
                    value INT CHECK (value > 0)
                )
            """)
            
            # Insert valid data
            conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (1, 'valid', 100)")
            
            print(f"✓ Created savepoint error test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test savepoint error conditions"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction")
                
                # Insert valid data
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (2, 'valid_in_txn', 200)")
                results.append("✓ Inserted valid data")
                
                # Create savepoint
                conn.execute_query("SAVEPOINT error_test")
                results.append("✓ Created savepoint error_test")
                
                # Try to insert invalid data (should cause error)
                try:
                    conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (3, 'invalid', -1)")
                    results.append("✗ Invalid insert should have failed")
                    return PyState.error("Invalid insert should have failed")
                except:
                    results.append("✓ Invalid insert correctly failed")
                
                # Try to insert duplicate primary key
                try:
                    conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (1, 'duplicate', 300)")
                    results.append("✗ Duplicate key insert should have failed")
                    return PyState.error("Duplicate key insert should have failed")
                except:
                    results.append("✓ Duplicate key insert correctly failed")
                
                # Verify savepoint state is maintained
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 2:
                    results.append("✓ Verified savepoint state maintained")
                else:
                    results.append("✗ Savepoint state verification failed")
                    return PyState.error("Savepoint state verification failed")
                
                # Rollback to savepoint
                conn.execute_query("ROLLBACK TO SAVEPOINT error_test")
                results.append("✓ Rolled back to savepoint")
                
                # Verify rollback state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 2:
                    results.append("✓ Verified rollback state (2 rows)")
                else:
                    results.append("✗ Rollback state verification failed")
                    return PyState.error("Rollback state verification failed")
                
                # Insert valid data after rollback
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (3, 'after_rollback', 300)")
                results.append("✓ Inserted valid data after rollback")
                
                # Commit transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed transaction")
                
                # Verify final state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 3:
                    results.append("✓ Verified final state (3 rows)")
                else:
                    results.append("✗ Final state verification failed")
                    return PyState.error("Final state verification failed")
                
                print("\n=== SAVEPOINT ERROR TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during savepoint error test: {e}")
                return PyState.error(f"Savepoint error test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up savepoint error test table")


class SavepointIsolationTestHandler(PyStateHandler):
    """Test savepoints with different isolation levels."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"savepoint_isolation_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT
                )
            """)
            
            print(f"✓ Created savepoint isolation test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test savepoints with different isolation levels"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Test with READ COMMITTED
                conn.execute_query("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
                results.append("✓ Set isolation level to READ COMMITTED")
                
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction with READ COMMITTED")
                
                # Insert data
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (1, 'read_committed', 100)")
                results.append("✓ Inserted data with READ COMMITTED")
                
                # Create savepoint
                conn.execute_query("SAVEPOINT rc_sp")
                results.append("✓ Created savepoint with READ COMMITTED")
                
                conn.execute_query("COMMIT")
                results.append("✓ Committed READ COMMITTED transaction")
                
                # Test with REPEATABLE READ
                conn.execute_query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
                results.append("✓ Set isolation level to REPEATABLE READ")
                
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction with REPEATABLE READ")
                
                # Insert data
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (2, 'repeatable_read', 200)")
                results.append("✓ Inserted data with REPEATABLE READ")
                
                # Create savepoint
                conn.execute_query("SAVEPOINT rr_sp")
                results.append("✓ Created savepoint with REPEATABLE READ")
                
                # Insert more data
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (3, 'after_savepoint', 300)")
                results.append("✓ Inserted data after savepoint")
                
                # Rollback to savepoint
                conn.execute_query("ROLLBACK TO SAVEPOINT rr_sp")
                results.append("✓ Rolled back to savepoint with REPEATABLE READ")
                
                # Verify rollback state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 2:
                    results.append("✓ Verified rollback state with REPEATABLE READ")
                else:
                    results.append("✗ Rollback state verification failed")
                    return PyState.error("Rollback state verification failed")
                
                conn.execute_query("COMMIT")
                results.append("✓ Committed REPEATABLE READ transaction")
                
                # Test with SERIALIZABLE
                conn.execute_query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
                results.append("✓ Set isolation level to SERIALIZABLE")
                
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction with SERIALIZABLE")
                
                # Read all data (establishes consistent view)
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 2:
                    results.append("✓ Read data with SERIALIZABLE")
                else:
                    results.append("✗ Failed to read data with SERIALIZABLE")
                    return PyState.error("Failed to read data with SERIALIZABLE")
                
                # Create savepoint
                conn.execute_query("SAVEPOINT ser_sp")
                results.append("✓ Created savepoint with SERIALIZABLE")
                
                conn.execute_query("COMMIT")
                results.append("✓ Committed SERIALIZABLE transaction")
                
                # Verify final state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 2:
                    results.append("✓ Verified final state across isolation levels")
                else:
                    results.append("✗ Final state verification failed")
                    return PyState.error("Final state verification failed")
                
                print("\n=== SAVEPOINT ISOLATION TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during savepoint isolation test: {e}")
                return PyState.error(f"Savepoint isolation test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up savepoint isolation test table")


class SavepointPerformanceTestHandler(PyStateHandler):
    """Test savepoint performance and limits."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"savepoint_perf_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT,
                    savepoint_id INT DEFAULT 0
                )
            """)
            
            print(f"✓ Created savepoint performance test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test savepoint performance and limits"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction")
                
                # Insert initial data
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (1, 'initial', 100)")
                results.append("✓ Inserted initial data")
                
                # Create multiple savepoints with data operations
                for i in range(1, 11):
                    # Create savepoint
                    conn.execute_query(f"SAVEPOINT sp_{i}")
                    results.append(f"✓ Created savepoint sp_{i}")
                    
                    # Insert data
                    conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value, savepoint_id) VALUES ({i+1}, 'row_{i}', {i*10}, {i})")
                    results.append(f"✓ Inserted data for savepoint {i}")
                
                # Verify all data exists
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 11:
                    results.append("✓ Verified 11 rows exist")
                else:
                    results.append("✗ Row count verification failed")
                    return PyState.error("Row count verification failed")
                
                # Rollback to middle savepoint
                conn.execute_query("ROLLBACK TO SAVEPOINT sp_5")
                results.append("✓ Rolled back to savepoint sp_5")
                
                # Verify rollback state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 6:
                    results.append("✓ Verified rollback state (6 rows)")
                else:
                    results.append("✗ Rollback state verification failed")
                    return PyState.error("Rollback state verification failed")
                
                # Insert data after rollback
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value, savepoint_id) VALUES (12, 'after_rollback', 1200, 0)")
                results.append("✓ Inserted data after rollback")
                
                # Rollback to first savepoint
                conn.execute_query("ROLLBACK TO SAVEPOINT sp_1")
                results.append("✓ Rolled back to savepoint sp_1")
                
                # Verify first savepoint state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 2:
                    results.append("✓ Verified first savepoint state (2 rows)")
                else:
                    results.append("✗ First savepoint state verification failed")
                    return PyState.error("First savepoint state verification failed")
                
                # Insert final data
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value, savepoint_id) VALUES (13, 'final', 1300, 0)")
                results.append("✓ Inserted final data")
                
                # Commit transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed transaction")
                
                # Verify final state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 3:
                    results.append("✓ Verified final state (3 rows)")
                else:
                    results.append("✗ Final state verification failed")
                    return PyState.error("Final state verification failed")
                
                print("\n=== SAVEPOINT PERFORMANCE TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during savepoint performance test: {e}")
                return PyState.error(f"Savepoint performance test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up savepoint performance test table") 
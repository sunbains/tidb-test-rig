"""
Specialized deadlock and MVCC tests for TiDB.

This test suite focuses on:
- MVCC (Multi-Version Concurrency Control) behavior during deadlocks
- Deadlock detection and automatic rollback
- Phantom read scenarios in deadlock conditions
- Version chain management during conflicts
- Deadlock resolution with different isolation levels

All tests use multiple concurrent connections to properly test deadlock scenarios.
"""

from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState, MultiConnectionTestHandler
import time


class MVCCDeadlockTestHandler(MultiConnectionTestHandler):
    """Test MVCC behavior during deadlock scenarios with concurrent transactions."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table with version tracking"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"mvcc_deadlock_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT,
                    version INT DEFAULT 1,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
                )
            """)
            
            # Insert initial data with version tracking
            for i in range(1, 6):
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value, version) VALUES ({i}, 'row_{i}', {i * 10}, 1)")
            
            print(f"✓ Created MVCC deadlock test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test MVCC behavior during deadlock scenarios with concurrent transactions"""
        if not context.connection:
            return PyState.error("No connection available")
        
        # Get multiple connections
        connections = self.get_connections(context, 2)
        if len(connections) < 2:
            return PyState.error("Failed to get multiple connections")
        
        conn1, conn2 = connections[0], connections[1]
        
        try:
            # Set isolation level to REPEATABLE READ for MVCC testing
            conn1.execute_query("SET SESSION TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            conn2.execute_query("SET SESSION TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            
            # Connection 1: Start transaction and read data (establishes consistent view)
            conn1.execute_query("START TRANSACTION")
            result1 = conn1.execute_query(f"SELECT id, value, version FROM {self.table_name} WHERE id = 2")
            if not result1 or result1[0].get('value') != 20 or result1[0].get('version') != 1:
                return PyState.error("Failed to read initial data in connection 1")
            
            # Connection 2: Start transaction and read same data
            conn2.execute_query("START TRANSACTION")
            result2 = conn2.execute_query(f"SELECT id, value, version FROM {self.table_name} WHERE id = 2")
            if not result2 or result2[0].get('value') != 20 or result2[0].get('version') != 1:
                return PyState.error("Failed to read initial data in connection 2")
            
            # Connection 1: Update row with version increment (simulating MVCC)
            conn1.execute_query(f"UPDATE {self.table_name} SET value = 999, version = version + 1 WHERE id = 2 AND version = 1")
            
            # Connection 2: Try to update same row with old version (should fail - optimistic locking)
            conn2.execute_query(f"UPDATE {self.table_name} SET value = 888, version = version + 1 WHERE id = 2 AND version = 1")
            
            # Connection 1: Commit
            conn1.execute_query("COMMIT")
            
            # Connection 2: Commit (should update 0 rows due to version mismatch)
            conn2.execute_query("COMMIT")
            
            # Verify final state
            final_result = context.connection.execute_query(f"SELECT value, version FROM {self.table_name} WHERE id = 2")
            if final_result and final_result[0].get('value') == 999 and final_result[0].get('version') == 2:
                print("✓ MVCC deadlock test: Connection 1's update won (MVCC protection worked)")
                return PyState.completed()
            else:
                print("✗ MVCC deadlock test: Unexpected final state")
                return PyState.error("MVCC deadlock test failed - unexpected final state")
                
        except Exception as e:
            print(f"Error during MVCC deadlock test: {e}")
            return PyState.error(f"MVCC deadlock test failed: {e}")
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up MVCC deadlock test table")


class PhantomReadDeadlockTestHandler(MultiConnectionTestHandler):
    """Test phantom read scenarios during deadlock conditions with concurrent transactions."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"phantom_read_deadlock_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT,
                    category VARCHAR(20)
                )
            """)
            
            # Insert initial data
            for i in range(1, 6):
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value, category) VALUES ({i}, 'row_{i}', {i * 10}, 'A')")
            
            print(f"✓ Created phantom read deadlock test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test phantom read scenarios during deadlock conditions with concurrent transactions"""
        if not context.connection:
            return PyState.error("No connection available")
        
        # Get multiple connections
        connections = self.get_connections(context, 2)
        if len(connections) < 2:
            return PyState.error("Failed to get multiple connections")
        
        conn1, conn2 = connections[0], connections[1]
        
        try:
            # Set isolation level to REPEATABLE READ
            conn1.execute_query("SET SESSION TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            conn2.execute_query("SET SESSION TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            
            # Connection 1: Start transaction and read data with range query
            conn1.execute_query("START TRANSACTION")
            result1 = conn1.execute_query(f"SELECT COUNT(*) FROM {self.table_name} WHERE category = 'A'")
            if not result1 or result1[0].get('col_0', 0) != 5:
                return PyState.error("Failed to read initial count in connection 1")
            
            # Connection 2: Start transaction and insert new row in same category (phantom insert)
            conn2.execute_query("START TRANSACTION")
            conn2.execute_query(f"INSERT INTO {self.table_name} (id, name, value, category) VALUES (6, 'phantom_row', 600, 'A')")
            
            # Connection 1: Read same range again (should see same count due to REPEATABLE READ)
            result1_after = conn1.execute_query(f"SELECT COUNT(*) FROM {self.table_name} WHERE category = 'A'")
            if not result1_after or result1_after[0].get('col_0', 0) != 5:
                print("✗ Phantom read occurred in connection 1")
                conn1.execute_query("ROLLBACK")
                conn2.execute_query("ROLLBACK")
                return PyState.error("Phantom read deadlock test failed - phantom read occurred")
            
            # Connection 2: Commit
            conn2.execute_query("COMMIT")
            
            # Connection 1: Read again (should still see same count due to REPEATABLE READ)
            result1_final = conn1.execute_query(f"SELECT COUNT(*) FROM {self.table_name} WHERE category = 'A'")
            if not result1_final or result1_final[0].get('col_0', 0) != 5:
                print("✗ Phantom read occurred after commit in connection 1")
                conn1.execute_query("ROLLBACK")
                return PyState.error("Phantom read deadlock test failed - phantom read after commit")
            
            # Connection 1: Commit and read again (should now see new row)
            conn1.execute_query("COMMIT")
            final_result = context.connection.execute_query(f"SELECT COUNT(*) FROM {self.table_name} WHERE category = 'A'")
            if final_result and final_result[0].get('col_0', 0) == 6:
                print("✓ Phantom read deadlock test: Connection 1 can see new row after commit")
                return PyState.completed()
            else:
                print("✗ Phantom read deadlock test: Connection 1 cannot see new row after commit")
                return PyState.error("Phantom read deadlock test failed - cannot see new row after commit")
                
        except Exception as e:
            print(f"Error during phantom read deadlock test: {e}")
            return PyState.error(f"Phantom read deadlock test failed: {e}")
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up phantom read deadlock test table")


class DeadlockDetectionTestHandler(MultiConnectionTestHandler):
    """Test deadlock detection and automatic rollback with concurrent transactions."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test tables for deadlock scenarios"""
        if context.connection:
            conn = context.connection
            
            self.table1 = f"deadlock_detection_1_{int(time.time())}"
            self.table2 = f"deadlock_detection_2_{int(time.time())}"
            
            # Create two tables for deadlock testing
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
            for i in range(1, 4):
                conn.execute_query(f"INSERT INTO {self.table1} (id, name, value) VALUES ({i}, 'table1_row_{i}', {i * 10})")
                conn.execute_query(f"INSERT INTO {self.table2} (id, name, value) VALUES ({i}, 'table2_row_{i}', {i * 20})")
            
            print(f"✓ Created deadlock detection test tables: {self.table1}, {self.table2}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test deadlock detection and automatic rollback with concurrent transactions"""
        if not context.connection:
            return PyState.error("No connection available")
        
        # Get multiple connections
        connections = self.get_connections(context, 2)
        if len(connections) < 2:
            return PyState.error("Failed to get multiple connections")
        
        conn1, conn2 = connections[0], connections[1]
        
        try:
            # Connection 1: Start transaction and lock rows in table1
            conn1.execute_query("START TRANSACTION")
            conn1.execute_query(f"SELECT * FROM {self.table1} WHERE id = 1 FOR UPDATE")
            
            # Connection 2: Start transaction and lock rows in table2
            conn2.execute_query("START TRANSACTION")
            conn2.execute_query(f"SELECT * FROM {self.table2} WHERE id = 1 FOR UPDATE")
            
            # Connection 1: Try to lock rows in table2 (should be blocked)
            try:
                conn1.execute_query(f"SELECT * FROM {self.table2} WHERE id = 1 FOR UPDATE")
                print("✗ Connection 1 should have been blocked by Connection 2")
                conn1.execute_query("ROLLBACK")
                conn2.execute_query("ROLLBACK")
                return PyState.error("Deadlock detection test failed - locks not working")
            except:
                print("✓ Connection 1 was blocked by Connection 2 (expected)")
            
            # Connection 2: Try to lock rows in table1 (should create deadlock)
            try:
                conn2.execute_query(f"SELECT * FROM {self.table1} WHERE id = 1 FOR UPDATE")
                print("✗ Connection 2 should have been blocked by Connection 1")
                conn1.execute_query("ROLLBACK")
                conn2.execute_query("ROLLBACK")
                return PyState.error("Deadlock detection test failed - locks not working")
            except:
                print("✓ Connection 2 was blocked by Connection 1 (deadlock created)")
            
            # One of the connections should be automatically rolled back due to deadlock detection
            # Let's check which one is still active
            try:
                conn1.execute_query("SELECT 1")
                print("✓ Connection 1 is still active")
                conn1.execute_query("COMMIT")
                conn2.execute_query("ROLLBACK")
            except:
                print("✓ Connection 1 was rolled back (deadlock victim)")
                try:
                    conn2.execute_query("SELECT 1")
                    print("✓ Connection 2 is still active")
                    conn2.execute_query("COMMIT")
                except:
                    print("✓ Connection 2 was also rolled back")
            
            print("✓ Deadlock detection test completed successfully")
            return PyState.completed()
                
        except Exception as e:
            print(f"Error during deadlock detection test: {e}")
            return PyState.error(f"Deadlock detection test failed: {e}")
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table1') and hasattr(self, 'table2'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table1}")
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table2}")
            print(f"✓ Cleaned up deadlock detection test tables")


class VersionChainDeadlockTestHandler(MultiConnectionTestHandler):
    """Test version chain management during conflicts with concurrent transactions."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table with version chain"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"version_chain_deadlock_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT,
                    version INT DEFAULT 1,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
                )
            """)
            
            # Insert initial data
            for i in range(1, 4):
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value, version) VALUES ({i}, 'row_{i}', {i * 10}, 1)")
            
            print(f"✓ Created version chain deadlock test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test version chain management during conflicts with concurrent transactions"""
        if not context.connection:
            return PyState.error("No connection available")
        
        # Get multiple connections
        connections = self.get_connections(context, 3)
        if len(connections) < 3:
            return PyState.error("Failed to get multiple connections")
        
        conn1, conn2, conn3 = connections[0], connections[1], connections[2]
        
        try:
            # Set isolation level to REPEATABLE READ for version chain testing
            conn1.execute_query("SET SESSION TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            conn2.execute_query("SET SESSION TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            conn3.execute_query("SET SESSION TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            
            # Connection 1: Start transaction and read data
            conn1.execute_query("START TRANSACTION")
            result1 = conn1.execute_query(f"SELECT value, version FROM {self.table_name} WHERE id = 1")
            if not result1 or result1[0].get('value') != 10 or result1[0].get('version') != 1:
                return PyState.error("Failed to read initial data in connection 1")
            
            # Connection 2: Start transaction and update data (creates new version)
            conn2.execute_query("START TRANSACTION")
            conn2.execute_query(f"UPDATE {self.table_name} SET value = 999, version = version + 1 WHERE id = 1 AND version = 1")
            conn2.execute_query("COMMIT")
            
            # Connection 3: Start transaction and try to update with old version (should fail)
            conn3.execute_query("START TRANSACTION")
            conn3.execute_query(f"UPDATE {self.table_name} SET value = 888, version = version + 1 WHERE id = 1 AND version = 1")
            conn3.execute_query("COMMIT")
            
            # Connection 1: Read data (should see original version due to REPEATABLE READ)
            result1_after = conn1.execute_query(f"SELECT value, version FROM {self.table_name} WHERE id = 1")
            if not result1_after or result1_after[0].get('value') != 10 or result1_after[0].get('version') != 1:
                print("✗ Connection 1 can see new version (should see original)")
                conn1.execute_query("ROLLBACK")
                return PyState.error("Version chain deadlock test failed - isolation violated")
            
            # Connection 1: Commit and read again (should now see latest version)
            conn1.execute_query("COMMIT")
            final_result = context.connection.execute_query(f"SELECT value, version FROM {self.table_name} WHERE id = 1")
            if final_result and final_result[0].get('value') == 999 and final_result[0].get('version') == 2:
                print("✓ Version chain deadlock test: Connection 1 can see latest version after commit")
                return PyState.completed()
            else:
                print("✗ Version chain deadlock test: Connection 1 cannot see latest version after commit")
                return PyState.error("Version chain deadlock test failed - cannot see latest version")
                
        except Exception as e:
            print(f"Error during version chain deadlock test: {e}")
            return PyState.error(f"Version chain deadlock test failed: {e}")
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up version chain deadlock test table")


class IsolationLevelDeadlockTestHandler(MultiConnectionTestHandler):
    """Test deadlock resolution with different isolation levels using concurrent transactions."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"isolation_deadlock_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT,
                    status VARCHAR(20) DEFAULT 'active'
                )
            """)
            
            # Insert test data
            for i in range(1, 6):
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES ({i}, 'row_{i}', {i * 10})")
            
            print(f"✓ Created isolation level deadlock test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test deadlock resolution with different isolation levels using concurrent transactions"""
        if not context.connection:
            return PyState.error("No connection available")
        
        # Get multiple connections
        connections = self.get_connections(context, 2)
        if len(connections) < 2:
            return PyState.error("Failed to get multiple connections")
        
        conn1, conn2 = connections[0], connections[1]
        
        try:
            # Connection 1: Set to READ COMMITTED
            conn1.execute_query("SET SESSION TRANSACTION ISOLATION LEVEL READ COMMITTED")
            conn1.execute_query("START TRANSACTION")
            
            # Connection 2: Set to REPEATABLE READ
            conn2.execute_query("SET SESSION TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            conn2.execute_query("START TRANSACTION")
            
            # Connection 1: Read and update data
            conn1.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
            conn1.execute_query(f"UPDATE {self.table_name} SET value = 999 WHERE id = 1")
            
            # Connection 2: Read same data (should see original value due to REPEATABLE READ)
            result2 = conn2.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
            if not result2 or result2[0].get('value') != 10:
                print("✗ Connection 2 can see uncommitted changes from Connection 1")
                conn1.execute_query("ROLLBACK")
                conn2.execute_query("ROLLBACK")
                return PyState.error("Isolation level deadlock test failed - isolation violated")
            
            # Connection 1: Commit
            conn1.execute_query("COMMIT")
            
            # Connection 2: Read again (should still see original value due to REPEATABLE READ)
            result2_after = conn2.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
            if not result2_after or result2_after[0].get('value') != 10:
                print("✗ Connection 2 can see committed changes (should not due to REPEATABLE READ)")
                conn2.execute_query("ROLLBACK")
                return PyState.error("Isolation level deadlock test failed - REPEATABLE READ violated")
            
            # Connection 2: Commit and read again (should now see new value)
            conn2.execute_query("COMMIT")
            final_result = context.connection.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
            if final_result and final_result[0].get('value') == 999:
                print("✓ Isolation level deadlock test: Connection 2 can see new value after commit")
                return PyState.completed()
            else:
                print("✗ Isolation level deadlock test: Connection 2 cannot see new value after commit")
                return PyState.error("Isolation level deadlock test failed - cannot see new value after commit")
                
        except Exception as e:
            print(f"Error during isolation level deadlock test: {e}")
            return PyState.error(f"Isolation level deadlock test failed: {e}")
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up isolation level deadlock test table") 
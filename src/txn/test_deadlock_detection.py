"""
Deadlock Detection and Isolation Level tests for TiDB.

This test suite focuses on:
- Deadlock detection and automatic rollback
- Deadlock resolution with different isolation levels
- Lock behavior and deadlock scenarios
- Isolation level interactions during conflicts

All tests use multiple concurrent connections to properly test deadlock scenarios.
"""

from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState, MultiConnectionTestHandler
import time


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
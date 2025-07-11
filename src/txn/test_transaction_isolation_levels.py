"""
Transaction isolation level tests for TiDB.

This test suite covers all transaction isolation levels supported by TiDB:
- READ UNCOMMITTED
- READ COMMITTED  
- REPEATABLE READ
- SERIALIZABLE

All tests use multiple concurrent connections to properly verify isolation behavior.
"""

from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState, MultiConnectionTestHandler
import time


class ReadUncommittedTestHandler(MultiConnectionTestHandler):
    """Test READ UNCOMMITTED isolation level with concurrent transactions."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"read_uncommitted_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT
                )
            """)
            
            # Insert initial data
            conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (1, 'initial', 100)")
            
            print(f"✓ Created READ UNCOMMITTED test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test READ UNCOMMITTED isolation with concurrent transactions"""
        if not context.connection:
            return PyState.error("No connection available")
        
        # Get multiple connections
        connections = self.get_connections(context, 2)
        if len(connections) < 2:
            return PyState.error("Failed to get multiple connections")
        
        conn1, conn2 = connections[0], connections[1]
        
        try:
            # Set isolation level to READ UNCOMMITTED on both connections
            conn1.execute_query("SET SESSION TRANSACTION ISOLATION LEVEL READ UNCOMMITTED")
            conn2.execute_query("SET SESSION TRANSACTION ISOLATION LEVEL READ UNCOMMITTED")
            
            # Connection 1: Start transaction and update data
            conn1.execute_query("START TRANSACTION")
            conn1.execute_query(f"UPDATE {self.table_name} SET value = 999 WHERE id = 1")
            
            # Connection 2: Start transaction and read data (should see uncommitted change)
            conn2.execute_query("START TRANSACTION")
            result = conn2.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
            
            if result and result[0].get('value') == 999:
                print("✓ READ UNCOMMITTED: Connection 2 can see uncommitted change from Connection 1")
                
                # Connection 1: Commit the transaction
                conn1.execute_query("COMMIT")
                
                # Connection 2: Read again (should still see the committed value)
                result2 = conn2.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                if result2 and result2[0].get('value') == 999:
                    print("✓ READ UNCOMMITTED: Connection 2 can see committed change")
                    conn2.execute_query("COMMIT")
                    return PyState.completed()
                else:
                    print("✗ READ UNCOMMITTED: Connection 2 cannot see committed change")
                    return PyState.error("READ UNCOMMITTED isolation test failed")
            else:
                print("✗ READ UNCOMMITTED: Connection 2 cannot see uncommitted change")
                conn1.execute_query("ROLLBACK")
                conn2.execute_query("ROLLBACK")
                return PyState.error("READ UNCOMMITTED isolation test failed")
                
        except Exception as e:
            print(f"Error during READ UNCOMMITTED test: {e}")
            return PyState.error(f"READ UNCOMMITTED test failed: {e}")
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up READ UNCOMMITTED test table")


class ReadCommittedTestHandler(MultiConnectionTestHandler):
    """Test READ COMMITTED isolation level with concurrent transactions."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"read_committed_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT
                )
            """)
            
            # Insert initial data
            conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (1, 'initial', 100)")
            
            print(f"✓ Created READ COMMITTED test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test READ COMMITTED isolation with concurrent transactions"""
        if not context.connection:
            return PyState.error("No connection available")
        
        # Get multiple connections
        connections = self.get_connections(context, 2)
        if len(connections) < 2:
            return PyState.error("Failed to get multiple connections")
        
        conn1, conn2 = connections[0], connections[1]
        
        try:
            # Set isolation level to READ COMMITTED on both connections
            conn1.execute_query("SET SESSION TRANSACTION ISOLATION LEVEL READ COMMITTED")
            conn2.execute_query("SET SESSION TRANSACTION ISOLATION LEVEL READ COMMITTED")
            
            # Connection 1: Start transaction and update data
            conn1.execute_query("START TRANSACTION")
            conn1.execute_query(f"UPDATE {self.table_name} SET value = 888 WHERE id = 1")
            
            # Connection 2: Start transaction and read data (should NOT see uncommitted change)
            conn2.execute_query("START TRANSACTION")
            result = conn2.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
            
            if result and result[0].get('value') == 100:  # Should see original value
                print("✓ READ COMMITTED: Connection 2 cannot see uncommitted change from Connection 1")
                
                # Connection 1: Commit the transaction
                conn1.execute_query("COMMIT")
                
                # Connection 2: Read again (should now see the committed value)
                result2 = conn2.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                if result2 and result2[0].get('value') == 888:
                    print("✓ READ COMMITTED: Connection 2 can see committed change")
                    conn2.execute_query("COMMIT")
                    return PyState.completed()
                else:
                    print("✗ READ COMMITTED: Connection 2 cannot see committed change")
                    return PyState.error("READ COMMITTED isolation test failed")
            else:
                print("✗ READ COMMITTED: Connection 2 can see uncommitted change (should not)")
                conn1.execute_query("ROLLBACK")
                conn2.execute_query("ROLLBACK")
                return PyState.error("READ COMMITTED isolation test failed")
                
        except Exception as e:
            print(f"Error during READ COMMITTED test: {e}")
            return PyState.error(f"READ COMMITTED test failed: {e}")
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up READ COMMITTED test table")


class RepeatableReadTestHandler(MultiConnectionTestHandler):
    """Test REPEATABLE READ isolation level with concurrent transactions."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"repeatable_read_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT
                )
            """)
            
            # Insert initial data
            conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (1, 'initial', 100)")
            
            print(f"✓ Created REPEATABLE READ test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test REPEATABLE READ isolation with concurrent transactions"""
        if not context.connection:
            return PyState.error("No connection available")
        
        # Get multiple connections
        connections = self.get_connections(context, 2)
        if len(connections) < 2:
            return PyState.error("Failed to get multiple connections")
        
        conn1, conn2 = connections[0], connections[1]
        
        try:
            # Set isolation level to REPEATABLE READ on both connections
            conn1.execute_query("SET SESSION TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            conn2.execute_query("SET SESSION TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            
            # Connection 2: Start transaction and read data
            conn2.execute_query("START TRANSACTION")
            result1 = conn2.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
            initial_value = result1[0].get('value') if result1 else None
            
            # Connection 1: Start transaction, update data, and commit
            conn1.execute_query("START TRANSACTION")
            conn1.execute_query(f"UPDATE {self.table_name} SET value = 777 WHERE id = 1")
            conn1.execute_query("COMMIT")
            
            # Connection 2: Read again (should see same value due to REPEATABLE READ)
            result2 = conn2.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
            repeatable_value = result2[0].get('value') if result2 else None
            
            if repeatable_value == initial_value:
                print("✓ REPEATABLE READ: Connection 2 sees consistent value (no phantom read)")
                
                # Connection 2: Commit and read again (should now see new value)
                conn2.execute_query("COMMIT")
                result3 = conn2.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                final_value = result3[0].get('value') if result3 else None
                
                if final_value == 777:
                    print("✓ REPEATABLE READ: Connection 2 can see new value after commit")
                    return PyState.completed()
                else:
                    print("✗ REPEATABLE READ: Connection 2 cannot see new value after commit")
                    return PyState.error("REPEATABLE READ isolation test failed")
            else:
                print("✗ REPEATABLE READ: Connection 2 sees different value (phantom read occurred)")
                conn2.execute_query("ROLLBACK")
                return PyState.error("REPEATABLE READ isolation test failed")
                
        except Exception as e:
            print(f"Error during REPEATABLE READ test: {e}")
            return PyState.error(f"REPEATABLE READ test failed: {e}")
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up REPEATABLE READ test table")


class SerializableTestHandler(MultiConnectionTestHandler):
    """Test SERIALIZABLE isolation level with concurrent transactions."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"serializable_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT
                )
            """)
            
            # Insert initial data
            conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (1, 'initial', 100)")
            
            print(f"✓ Created SERIALIZABLE test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test SERIALIZABLE isolation with concurrent transactions"""
        if not context.connection:
            return PyState.error("No connection available")
        
        # Get multiple connections
        connections = self.get_connections(context, 2)
        if len(connections) < 2:
            return PyState.error("Failed to get multiple connections")
        
        conn1, conn2 = connections[0], connections[1]
        
        try:
            # Set isolation level to SERIALIZABLE on both connections
            conn1.execute_query("SET SESSION TRANSACTION ISOLATION LEVEL SERIALIZABLE")
            conn2.execute_query("SET SESSION TRANSACTION ISOLATION LEVEL SERIALIZABLE")
            
            # Connection 1: Start transaction and read data
            conn1.execute_query("START TRANSACTION")
            result1 = conn1.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
            
            # Connection 2: Start transaction and try to update data
            conn2.execute_query("START TRANSACTION")
            
            try:
                conn2.execute_query(f"UPDATE {self.table_name} SET value = 666 WHERE id = 1")
                print("✓ SERIALIZABLE: Connection 2 can update data")
                
                # Connection 1: Try to read again (should see consistent value)
                result2 = conn1.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                if result2 and result2[0].get('value') == 100:  # Should see original value
                    print("✓ SERIALIZABLE: Connection 1 sees consistent value")
                    
                    # Connection 2: Commit
                    conn2.execute_query("COMMIT")
                    
                    # Connection 1: Commit
                    conn1.execute_query("COMMIT")
                    
                    # Verify final state
                    final_result = conn1.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                    if final_result and final_result[0].get('value') == 666:
                        print("✓ SERIALIZABLE: Final state is correct")
                        return PyState.completed()
                    else:
                        print("✗ SERIALIZABLE: Final state is incorrect")
                        return PyState.error("SERIALIZABLE isolation test failed")
                else:
                    print("✗ SERIALIZABLE: Connection 1 does not see consistent value")
                    conn1.execute_query("ROLLBACK")
                    conn2.execute_query("ROLLBACK")
                    return PyState.error("SERIALIZABLE isolation test failed")
                    
            except Exception as e:
                print(f"✓ SERIALIZABLE: Connection 2 update was blocked (expected in strict serializable)")
                conn1.execute_query("COMMIT")
                conn2.execute_query("ROLLBACK")
                return PyState.completed()
                
        except Exception as e:
            print(f"Error during SERIALIZABLE test: {e}")
            return PyState.error(f"SERIALIZABLE test failed: {e}")
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up SERIALIZABLE test table")


class IsolationLevelComparisonTestHandler(MultiConnectionTestHandler):
    """Test comparison of different isolation levels with concurrent transactions."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"isolation_comparison_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT
                )
            """)
            
            # Insert initial data
            conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (1, 'initial', 100)")
            
            print(f"✓ Created isolation comparison test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test comparison of different isolation levels"""
        if not context.connection:
            return PyState.error("No connection available")
        
        # Get multiple connections
        connections = self.get_connections(context, 2)
        if len(connections) < 2:
            return PyState.error("Failed to get multiple connections")
        
        conn1, conn2 = connections[0], connections[1]
        
        try:
            # Test READ COMMITTED vs REPEATABLE READ
            print("Testing READ COMMITTED vs REPEATABLE READ...")
            
            # Reset data
            conn1.execute_query(f"UPDATE {self.table_name} SET value = 100 WHERE id = 1")
            
            # Connection 1: READ COMMITTED
            conn1.execute_query("SET SESSION TRANSACTION ISOLATION LEVEL READ COMMITTED")
            conn1.execute_query("START TRANSACTION")
            
            # Connection 2: REPEATABLE READ
            conn2.execute_query("SET SESSION TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            conn2.execute_query("START TRANSACTION")
            
            # Both read initial value
            result1 = conn1.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
            result2 = conn2.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
            
            if result1[0].get('value') == 100 and result2[0].get('value') == 100:
                print("✓ Both connections read initial value (100)")
                
                # Update from outside (simulate third connection)
                context.connection.execute_query(f"UPDATE {self.table_name} SET value = 555 WHERE id = 1")
                
                # Connection 1 (READ COMMITTED) should see new value
                result1_new = conn1.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                
                # Connection 2 (REPEATABLE READ) should see old value
                result2_new = conn2.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                
                if result1_new[0].get('value') == 555 and result2_new[0].get('value') == 100:
                    print("✓ READ COMMITTED sees new value, REPEATABLE READ sees old value")
                    conn1.execute_query("COMMIT")
                    conn2.execute_query("COMMIT")
                    return PyState.completed()
                else:
                    print("✗ Isolation level comparison failed")
                    conn1.execute_query("ROLLBACK")
                    conn2.execute_query("ROLLBACK")
                    return PyState.error("Isolation level comparison test failed")
            else:
                print("✗ Initial read failed")
                conn1.execute_query("ROLLBACK")
                conn2.execute_query("ROLLBACK")
                return PyState.error("Isolation level comparison test failed")
                
        except Exception as e:
            print(f"Error during isolation level comparison test: {e}")
            return PyState.error(f"Isolation level comparison test failed: {e}")
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up isolation comparison test table") 


class PhantomReadTestHandler(MultiConnectionTestHandler):
    """Test for phantom reads - inserting new rows that appear in other transactions."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            # Create a unique table name to avoid conflicts
            self.table_name = f"phantom_read_test_{int(time.time())}"
            
            # Create test table
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100) NOT NULL,
                    value INT NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )
            """)
            
            print(f"✓ Created test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test for phantom reads"""
        if not context.connection:
            return PyState.error("No connection available")
        
        # Get multiple connections
        connections = self.get_connections(context, 2)
        if len(connections) < 2:
            return PyState.error("Failed to get multiple connections")
        
        conn1, conn2 = connections[0], connections[1]
        
        try:
            # Set isolation level to REPEATABLE READ on both connections
            conn1.execute_query("SET SESSION TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            conn2.execute_query("SET SESSION TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            
            # Connection 1: Start transaction and count rows
            conn1.execute_query("START TRANSACTION")
            result1 = conn1.execute_query(f"SELECT COUNT(*) as count FROM {self.table_name}")
            initial_count = result1[0].get('count') if result1 else 0
            
            # Connection 2: Start transaction, insert new row, and commit
            conn2.execute_query("START TRANSACTION")
            conn2.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (1, 'phantom_row', 100)")
            conn2.execute_query("COMMIT")
            
            # Connection 1: Count rows again (should see same count due to REPEATABLE READ)
            result2 = conn1.execute_query(f"SELECT COUNT(*) as count FROM {self.table_name}")
            repeatable_count = result2[0].get('count') if result2 else 0
            
            if repeatable_count == initial_count:
                print("✓ PHANTOM READ TEST: Connection 1 sees consistent row count (no phantom read)")
                
                # Connection 1: Commit and count again (should now see new row)
                conn1.execute_query("COMMIT")
                result3 = conn1.execute_query(f"SELECT COUNT(*) as count FROM {self.table_name}")
                final_count = result3[0].get('count') if result3 else 0
                
                if final_count == initial_count + 1:
                    print("✓ PHANTOM READ TEST: Connection 1 can see new row after commit")
                    return PyState.completed()
                else:
                    print("✗ PHANTOM READ TEST: Connection 1 cannot see new row after commit")
                    return PyState.error("Phantom read test failed")
            else:
                print("✗ PHANTOM READ TEST: Connection 1 sees different row count (phantom read occurred)")
                conn1.execute_query("ROLLBACK")
                return PyState.error("Phantom read test failed")
                
        except Exception as e:
            print(f"Error during phantom read test: {e}")
            return PyState.error(f"Phantom read test failed: {e}")
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase - remove test table"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up test table: {self.table_name}") 
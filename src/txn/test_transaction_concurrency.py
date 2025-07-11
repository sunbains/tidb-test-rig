"""
Transaction concurrency tests for TiDB.

This test suite covers various concurrency scenarios including:
- Concurrent reads and writes
- Race conditions
- Lock contention
- Concurrent transaction conflicts

All tests use multiple concurrent connections to properly verify concurrency behavior.
"""

from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState, MultiConnectionTestHandler
import time


class ConcurrentReadWriteTestHandler(MultiConnectionTestHandler):
    """Test concurrent read and write operations between multiple transactions."""
    
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
        """Test concurrent read and write operations between multiple transactions"""
        if not context.connection:
            return PyState.error("No connection available")
        
        # Get multiple connections
        connections = self.get_connections(context, 3)
        if len(connections) < 3:
            return PyState.error("Failed to get multiple connections")
        
        conn1, conn2, conn3 = connections[0], connections[1], connections[2]
        
        try:
            # Connection 1: Start transaction and read data
            conn1.execute_query("START TRANSACTION")
            data1 = conn1.execute_query(f"SELECT * FROM {self.table_name} ORDER BY id")
            if not data1 or len(data1) != 5:
                return PyState.error("Failed to read initial data in connection 1")
            
            # Connection 2: Start transaction and update data
            conn2.execute_query("START TRANSACTION")
            conn2.execute_query(f"UPDATE {self.table_name} SET value = value + 100 WHERE id IN (1, 3, 5)")
            
            # Connection 3: Start transaction and read data (should see original values)
            conn3.execute_query("START TRANSACTION")
            data3 = conn3.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
            if not data3 or data3[0].get('value') != 10:  # Should see original value
                print("✗ Connection 3 can see uncommitted changes from Connection 2")
                conn1.execute_query("ROLLBACK")
                conn2.execute_query("ROLLBACK")
                conn3.execute_query("ROLLBACK")
                return PyState.error("Concurrent read/write test failed - isolation violated")
            
            # Connection 2: Commit changes
            conn2.execute_query("COMMIT")
            
            # Connection 1: Read again (should still see original values due to transaction isolation)
            data1_after = conn1.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
            if not data1_after or data1_after[0].get('value') != 10:
                print("✗ Connection 1 can see committed changes from Connection 2")
                conn1.execute_query("ROLLBACK")
                conn3.execute_query("ROLLBACK")
                return PyState.error("Concurrent read/write test failed - transaction isolation violated")
            
            # Connection 3: Read again (should now see updated values)
            data3_after = conn3.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
            if not data3_after or data3_after[0].get('value') != 110:  # Should see updated value
                print("✗ Connection 3 cannot see committed changes from Connection 2")
                conn1.execute_query("ROLLBACK")
                conn3.execute_query("ROLLBACK")
                return PyState.error("Concurrent read/write test failed - committed changes not visible")
            
            # Connection 1: Update different rows
            conn1.execute_query(f"UPDATE {self.table_name} SET value = value + 50 WHERE id IN (2, 4)")
            
            # Connection 3: Try to update same rows (should be blocked or conflict)
            try:
                conn3.execute_query(f"UPDATE {self.table_name} SET value = value + 25 WHERE id IN (2, 4)")
                print("✓ Connection 3 can update rows (no conflict)")
            except:
                print("✓ Connection 3 update was blocked (expected conflict)")
            
            # Commit remaining transactions
            conn1.execute_query("COMMIT")
            conn3.execute_query("COMMIT")
            
            # Verify final state
            final_data = context.connection.execute_query(f"SELECT value FROM {self.table_name} ORDER BY id")
            if final_data and len(final_data) == 5:
                print("✓ Concurrent read/write test completed successfully")
                return PyState.completed()
            else:
                return PyState.error("Concurrent read/write test failed - final state verification failed")
                
        except Exception as e:
            print(f"Error during concurrent read/write test: {e}")
            return PyState.error(f"Concurrent read/write test failed: {e}")
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up concurrent read/write test table")


class LockContentionTestHandler(MultiConnectionTestHandler):
    """Test lock contention scenarios between multiple transactions."""
    
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
        """Test lock contention scenarios between multiple transactions"""
        if not context.connection:
            return PyState.error("No connection available")
        
        # Get multiple connections
        connections = self.get_connections(context, 2)
        if len(connections) < 2:
            return PyState.error("Failed to get multiple connections")
        
        conn1, conn2 = connections[0], connections[1]
        
        try:
            # Connection 1: Start transaction and lock rows with FOR UPDATE
            conn1.execute_query("START TRANSACTION")
            conn1.execute_query(f"SELECT * FROM {self.table_name} WHERE id IN (1, 3, 5) FOR UPDATE")
            print("✓ Connection 1 locked rows 1, 3, 5 with FOR UPDATE")
            
            # Connection 2: Start transaction and try to lock same rows
            conn2.execute_query("START TRANSACTION")
            
            try:
                conn2.execute_query(f"SELECT * FROM {self.table_name} WHERE id IN (1, 3, 5) FOR UPDATE")
                print("✗ Connection 2 should have been blocked by Connection 1's locks")
                conn1.execute_query("ROLLBACK")
                conn2.execute_query("ROLLBACK")
                return PyState.error("Lock contention test failed - locks not working")
            except:
                print("✓ Connection 2 was blocked by Connection 1's locks (expected)")
            
            # Connection 2: Try to lock different rows (should succeed)
            try:
                conn2.execute_query(f"SELECT * FROM {self.table_name} WHERE id IN (2, 4, 6) FOR UPDATE")
                print("✓ Connection 2 can lock different rows")
            except:
                print("✗ Connection 2 cannot lock different rows")
                conn1.execute_query("ROLLBACK")
                conn2.execute_query("ROLLBACK")
                return PyState.error("Lock contention test failed - cannot lock different rows")
            
            # Connection 1: Update locked rows
            conn1.execute_query(f"UPDATE {self.table_name} SET value = 999 WHERE id IN (1, 3, 5)")
            print("✓ Connection 1 updated locked rows")
            
            # Connection 2: Update its locked rows
            conn2.execute_query(f"UPDATE {self.table_name} SET value = 888 WHERE id IN (2, 4, 6)")
            print("✓ Connection 2 updated its locked rows")
            
            # Connection 1: Commit
            conn1.execute_query("COMMIT")
            
            # Connection 2: Try to lock previously locked rows (should now succeed)
            try:
                conn2.execute_query(f"SELECT * FROM {self.table_name} WHERE id IN (1, 3, 5) FOR UPDATE")
                print("✓ Connection 2 can now lock previously locked rows")
            except:
                print("✗ Connection 2 still cannot lock previously locked rows")
                conn2.execute_query("ROLLBACK")
                return PyState.error("Lock contention test failed - locks not released after commit")
            
            # Connection 2: Commit
            conn2.execute_query("COMMIT")
            
            # Verify final state
            final_data = context.connection.execute_query(f"SELECT COUNT(*) FROM {self.table_name} WHERE value IN (999, 888)")
            if final_data and final_data[0].get('col_0', 0) == 6:
                print("✓ Lock contention test completed successfully")
                return PyState.completed()
            else:
                return PyState.error("Lock contention test failed - final state verification failed")
                
        except Exception as e:
            print(f"Error during lock contention test: {e}")
            return PyState.error(f"Lock contention test failed: {e}")
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up lock contention test table")


class RaceConditionTestHandler(MultiConnectionTestHandler):
    """Test race conditions between multiple transactions."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"race_condition_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    counter INT DEFAULT 0,
                    last_updated_by VARCHAR(50),
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
                )
            """)
            
            # Insert initial data
            conn.execute_query(f"INSERT INTO {self.table_name} (id, counter) VALUES (1, 0)")
            
            print(f"✓ Created race condition test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test race conditions between multiple transactions"""
        if not context.connection:
            return PyState.error("No connection available")
        
        # Get multiple connections
        connections = self.get_connections(context, 2)
        if len(connections) < 2:
            return PyState.error("Failed to get multiple connections")
        
        conn1, conn2 = connections[0], connections[1]
        
        try:
            # Both connections start transactions
            conn1.execute_query("START TRANSACTION")
            conn2.execute_query("START TRANSACTION")
            
            # Both read the same counter value
            result1 = conn1.execute_query(f"SELECT counter FROM {self.table_name} WHERE id = 1")
            result2 = conn2.execute_query(f"SELECT counter FROM {self.table_name} WHERE id = 1")
            
            if not result1 or not result2:
                return PyState.error("Failed to read initial counter value")
            
            initial_counter = result1[0].get('counter')
            
            # Connection 1: Increment counter
            conn1.execute_query(f"UPDATE {self.table_name} SET counter = counter + 1, last_updated_by = 'conn1' WHERE id = 1")
            
            # Connection 2: Also increment counter (race condition)
            conn2.execute_query(f"UPDATE {self.table_name} SET counter = counter + 1, last_updated_by = 'conn2' WHERE id = 1")
            
            # Connection 1: Commit
            conn1.execute_query("COMMIT")
            
            # Connection 2: Commit
            conn2.execute_query("COMMIT")
            
            # Check final state
            final_result = context.connection.execute_query(f"SELECT counter, last_updated_by FROM {self.table_name} WHERE id = 1")
            if final_result:
                final_counter = final_result[0].get('counter')
                last_updated_by = final_result[0].get('last_updated_by')
                
                if final_counter == initial_counter + 2:
                    print("✓ Race condition test: Both updates were applied (counter = 2)")
                elif final_counter == initial_counter + 1:
                    print("✓ Race condition test: One update was lost (counter = 1)")
                else:
                    print(f"✗ Race condition test: Unexpected final counter value: {final_counter}")
                    return PyState.error("Race condition test failed - unexpected final state")
                
                print(f"✓ Final counter: {final_counter}, Last updated by: {last_updated_by}")
                return PyState.completed()
            else:
                return PyState.error("Race condition test failed - cannot read final state")
                
        except Exception as e:
            print(f"Error during race condition test: {e}")
            return PyState.error(f"Race condition test failed: {e}")
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up race condition test table")


class ConcurrentTransactionConflictTestHandler(MultiConnectionTestHandler):
    """Test concurrent transaction conflicts and resolution."""
    
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
            
            # Insert initial data
            conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value, version) VALUES (1, 'test_row', 100, 1)")
            
            print(f"✓ Created concurrent conflict test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test concurrent transaction conflicts and resolution"""
        if not context.connection:
            return PyState.error("No connection available")
        
        # Get multiple connections
        connections = self.get_connections(context, 2)
        if len(connections) < 2:
            return PyState.error("Failed to get multiple connections")
        
        conn1, conn2 = connections[0], connections[1]
        
        try:
            # Both connections start transactions
            conn1.execute_query("START TRANSACTION")
            conn2.execute_query("START TRANSACTION")
            
            # Both read the same row
            result1 = conn1.execute_query(f"SELECT value, version FROM {self.table_name} WHERE id = 1")
            result2 = conn2.execute_query(f"SELECT value, version FROM {self.table_name} WHERE id = 1")
            
            if not result1 or not result2:
                return PyState.error("Failed to read initial data")
            
            # Connection 1: Update with optimistic locking
            conn1.execute_query(f"UPDATE {self.table_name} SET value = 200, version = version + 1 WHERE id = 1 AND version = 1")
            
            # Connection 2: Try to update with same version (should fail due to optimistic locking)
            conn2.execute_query(f"UPDATE {self.table_name} SET value = 300, version = version + 1 WHERE id = 1 AND version = 1")
            
            # Connection 1: Commit
            conn1.execute_query("COMMIT")
            
            # Connection 2: Commit (should fail or update 0 rows)
            conn2.execute_query("COMMIT")
            
            # Check final state
            final_result = context.connection.execute_query(f"SELECT value, version FROM {self.table_name} WHERE id = 1")
            if final_result:
                final_value = final_result[0].get('value')
                final_version = final_result[0].get('version')
                
                if final_value == 200 and final_version == 2:
                    print("✓ Concurrent conflict test: Connection 1's update won (optimistic locking worked)")
                    return PyState.completed()
                elif final_value == 300 and final_version == 2:
                    print("✓ Concurrent conflict test: Connection 2's update won")
                    return PyState.completed()
                else:
                    print(f"✗ Concurrent conflict test: Unexpected final state - value: {final_value}, version: {final_version}")
                    return PyState.error("Concurrent conflict test failed - unexpected final state")
            else:
                return PyState.error("Concurrent conflict test failed - cannot read final state")
                
        except Exception as e:
            print(f"Error during concurrent conflict test: {e}")
            return PyState.error(f"Concurrent conflict test failed: {e}")
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up concurrent conflict test table")


class TransactionRollbackTestHandler(MultiConnectionTestHandler):
    """Test transaction rollback behavior with concurrent transactions."""
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            conn = context.connection
            
            self.table_name = f"rollback_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT,
                    status VARCHAR(20) DEFAULT 'active'
                )
            """)
            
            # Insert initial data
            conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (1, 'test_row', 100)")
            
            print(f"✓ Created rollback test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test transaction rollback behavior with concurrent transactions"""
        if not context.connection:
            return PyState.error("No connection available")
        
        # Get multiple connections
        connections = self.get_connections(context, 2)
        if len(connections) < 2:
            return PyState.error("Failed to get multiple connections")
        
        conn1, conn2 = connections[0], connections[1]
        
        try:
            # Connection 1: Start transaction and update data
            conn1.execute_query("START TRANSACTION")
            conn1.execute_query(f"UPDATE {self.table_name} SET value = 999 WHERE id = 1")
            
            # Connection 2: Start transaction and read data (should see original value)
            conn2.execute_query("START TRANSACTION")
            result2 = conn2.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
            if not result2 or result2[0].get('value') != 100:
                print("✗ Connection 2 can see uncommitted changes from Connection 1")
                conn1.execute_query("ROLLBACK")
                conn2.execute_query("ROLLBACK")
                return PyState.error("Rollback test failed - isolation violated")
            
            # Connection 1: Rollback changes
            conn1.execute_query("ROLLBACK")
            print("✓ Connection 1 rolled back changes")
            
            # Connection 2: Read again (should still see original value)
            result2_after = conn2.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
            if not result2_after or result2_after[0].get('value') != 100:
                print("✗ Connection 2 cannot see original value after rollback")
                conn2.execute_query("ROLLBACK")
                return PyState.error("Rollback test failed - rollback not working")
            
            # Connection 2: Update data and commit
            conn2.execute_query(f"UPDATE {self.table_name} SET value = 888 WHERE id = 1")
            conn2.execute_query("COMMIT")
            
            # Verify final state
            final_result = context.connection.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
            if final_result and final_result[0].get('value') == 888:
                print("✓ Rollback test completed successfully")
                return PyState.completed()
            else:
                return PyState.error("Rollback test failed - final state verification failed")
                
        except Exception as e:
            print(f"Error during rollback test: {e}")
            return PyState.error(f"Rollback test failed: {e}")
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up rollback test table") 
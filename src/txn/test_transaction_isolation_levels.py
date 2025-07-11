"""
Transaction isolation level tests for TiDB.

This test suite covers all transaction isolation levels supported by TiDB:
- READ UNCOMMITTED
- READ COMMITTED  
- REPEATABLE READ
- SERIALIZABLE
"""

from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState
import time


class ReadUncommittedTestHandler(PyStateHandler):
    """Test READ UNCOMMITTED isolation level."""
    
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
        """Test READ UNCOMMITTED isolation"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Set isolation level to READ UNCOMMITTED
                conn.execute_query("SET TRANSACTION ISOLATION LEVEL READ UNCOMMITTED")
                results.append("✓ Set isolation level to READ UNCOMMITTED")
                
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction")
                
                # Read initial data
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('value') == 100:
                    results.append("✓ Read initial value (100)")
                else:
                    results.append("✗ Failed to read initial value")
                    return PyState.error("Failed to read initial value")
                
                # Update data (simulating uncommitted change)
                conn.execute_query(f"UPDATE {self.table_name} SET value = 999 WHERE id = 1")
                results.append("✓ Updated value to 999")
                
                # Read the updated data (should see uncommitted change)
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('value') == 999:
                    results.append("✓ Read uncommitted value (999)")
                else:
                    results.append("✗ Failed to read uncommitted value")
                    return PyState.error("Failed to read uncommitted value")
                
                # Commit transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed transaction")
                
                # Verify final state
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('value') == 999:
                    results.append("✓ Verified final committed value (999)")
                else:
                    results.append("✗ Final value verification failed")
                    return PyState.error("Final value verification failed")
                
                print("\n=== READ UNCOMMITTED TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during READ UNCOMMITTED test: {e}")
                return PyState.error(f"READ UNCOMMITTED test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up READ UNCOMMITTED test table")


class ReadCommittedTestHandler(PyStateHandler):
    """Test READ COMMITTED isolation level."""
    
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
        """Test READ COMMITTED isolation"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Set isolation level to READ COMMITTED
                conn.execute_query("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
                results.append("✓ Set isolation level to READ COMMITTED")
                
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction")
                
                # Read initial data
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('value') == 100:
                    results.append("✓ Read initial value (100)")
                else:
                    results.append("✗ Failed to read initial value")
                    return PyState.error("Failed to read initial value")
                
                # Update data
                conn.execute_query(f"UPDATE {self.table_name} SET value = 888 WHERE id = 1")
                results.append("✓ Updated value to 888")
                
                # Read the updated data (should see committed change)
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('value') == 888:
                    results.append("✓ Read updated value (888)")
                else:
                    results.append("✗ Failed to read updated value")
                    return PyState.error("Failed to read updated value")
                
                # Commit transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed transaction")
                
                # Verify final state
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('value') == 888:
                    results.append("✓ Verified final committed value (888)")
                else:
                    results.append("✗ Final value verification failed")
                    return PyState.error("Final value verification failed")
                
                print("\n=== READ COMMITTED TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during READ COMMITTED test: {e}")
                return PyState.error(f"READ COMMITTED test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up READ COMMITTED test table")


class RepeatableReadTestHandler(PyStateHandler):
    """Test REPEATABLE READ isolation level."""
    
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
            for i in range(1, 6):
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES ({i}, 'row_{i}', {i * 10})")
            
            print(f"✓ Created REPEATABLE READ test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test REPEATABLE READ isolation"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Set isolation level to REPEATABLE READ
                conn.execute_query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
                results.append("✓ Set isolation level to REPEATABLE READ")
                
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction")
                
                # Read initial data
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 2")
                if data and data[0].get('value') == 20:
                    results.append("✓ Read initial value (20) for id=2")
                else:
                    results.append("✗ Failed to read initial value")
                    return PyState.error("Failed to read initial value")
                
                # Update the same row
                conn.execute_query(f"UPDATE {self.table_name} SET value = 777 WHERE id = 2")
                results.append("✓ Updated value to 777 for id=2")
                
                # Read the same row again (should see consistent value)
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 2")
                if data and data[0].get('value') == 777:
                    results.append("✓ Read consistent value (777) for id=2")
                else:
                    results.append("✗ Failed to read consistent value")
                    return PyState.error("Failed to read consistent value")
                
                # Read other rows to test consistency
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 5:
                    results.append("✓ Verified consistent row count (5)")
                else:
                    results.append("✗ Row count verification failed")
                    return PyState.error("Row count verification failed")
                
                # Commit transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed transaction")
                
                # Verify final state
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 2")
                if data and data[0].get('value') == 777:
                    results.append("✓ Verified final committed value (777)")
                else:
                    results.append("✗ Final value verification failed")
                    return PyState.error("Final value verification failed")
                
                print("\n=== REPEATABLE READ TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during REPEATABLE READ test: {e}")
                return PyState.error(f"REPEATABLE READ test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up REPEATABLE READ test table")


class SerializableTestHandler(PyStateHandler):
    """Test SERIALIZABLE isolation level."""
    
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
            for i in range(1, 4):
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES ({i}, 'row_{i}', {i * 10})")
            
            print(f"✓ Created SERIALIZABLE test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test SERIALIZABLE isolation"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Set isolation level to SERIALIZABLE
                conn.execute_query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
                results.append("✓ Set isolation level to SERIALIZABLE")
                
                # Start transaction
                conn.execute_query("START TRANSACTION")
                results.append("✓ Started transaction")
                
                # Read all data (establishes a consistent view)
                data = conn.execute_query(f"SELECT * FROM {self.table_name} ORDER BY id")
                if data and len(data) == 3:
                    results.append("✓ Read all data (3 rows)")
                else:
                    results.append("✗ Failed to read initial data")
                    return PyState.error("Failed to read initial data")
                
                # Update a row
                conn.execute_query(f"UPDATE {self.table_name} SET value = 999 WHERE id = 1")
                results.append("✓ Updated value to 999 for id=1")
                
                # Insert a new row
                conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (4, 'new_row', 400)")
                results.append("✓ Inserted new row (id=4)")
                
                # Read data again (should see consistent view)
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 4:
                    results.append("✓ Verified consistent view (4 rows)")
                else:
                    results.append("✗ Consistent view verification failed")
                    return PyState.error("Consistent view verification failed")
                
                # Verify specific values
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('value') == 999:
                    results.append("✓ Verified updated value (999) for id=1")
                else:
                    results.append("✗ Updated value verification failed")
                    return PyState.error("Updated value verification failed")
                
                # Commit transaction
                conn.execute_query("COMMIT")
                results.append("✓ Committed transaction")
                
                # Verify final state
                data = conn.execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
                if data and data[0].get('col_0', 0) == 4:
                    results.append("✓ Verified final state (4 rows)")
                else:
                    results.append("✗ Final state verification failed")
                    return PyState.error("Final state verification failed")
                
                print("\n=== SERIALIZABLE TEST RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during SERIALIZABLE test: {e}")
                return PyState.error(f"SERIALIZABLE test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up SERIALIZABLE test table")


class IsolationLevelComparisonTestHandler(PyStateHandler):
    """Compare behavior across different isolation levels."""
    
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
            conn.execute_query(f"INSERT INTO {self.table_name} (id, name, value) VALUES (1, 'test', 100)")
            
            print(f"✓ Created isolation comparison test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test and compare different isolation levels"""
        if context.connection:
            conn = context.connection
            results = []
            
            try:
                # Test READ UNCOMMITTED
                conn.execute_query("SET TRANSACTION ISOLATION LEVEL READ UNCOMMITTED")
                conn.execute_query("START TRANSACTION")
                conn.execute_query(f"UPDATE {self.table_name} SET value = 111 WHERE id = 1")
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('value') == 111:
                    results.append("✓ READ UNCOMMITTED: Can see own uncommitted changes")
                conn.execute_query("COMMIT")
                
                # Test READ COMMITTED
                conn.execute_query("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
                conn.execute_query("START TRANSACTION")
                conn.execute_query(f"UPDATE {self.table_name} SET value = 222 WHERE id = 1")
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('value') == 222:
                    results.append("✓ READ COMMITTED: Can see own uncommitted changes")
                conn.execute_query("COMMIT")
                
                # Test REPEATABLE READ
                conn.execute_query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
                conn.execute_query("START TRANSACTION")
                conn.execute_query(f"UPDATE {self.table_name} SET value = 333 WHERE id = 1")
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('value') == 333:
                    results.append("✓ REPEATABLE READ: Can see own uncommitted changes")
                conn.execute_query("COMMIT")
                
                # Test SERIALIZABLE
                conn.execute_query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
                conn.execute_query("START TRANSACTION")
                conn.execute_query(f"UPDATE {self.table_name} SET value = 444 WHERE id = 1")
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('value') == 444:
                    results.append("✓ SERIALIZABLE: Can see own uncommitted changes")
                conn.execute_query("COMMIT")
                
                # Verify final state
                data = conn.execute_query(f"SELECT value FROM {self.table_name} WHERE id = 1")
                if data and data[0].get('value') == 444:
                    results.append("✓ Verified final state after all isolation level tests")
                else:
                    results.append("✗ Final state verification failed")
                    return PyState.error("Final state verification failed")
                
                print("\n=== ISOLATION LEVEL COMPARISON RESULTS ===")
                for result in results:
                    print(result)
                
                return PyState.completed()
                
            except Exception as e:
                print(f"Error during isolation level comparison test: {e}")
                return PyState.error(f"Isolation level comparison test failed: {e}")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connection and hasattr(self, 'table_name'):
            context.connection.execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up isolation comparison test table") 
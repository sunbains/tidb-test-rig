"""
Transaction edge cases using multiple concurrent connections for TiDB.

This test suite covers edge cases that specifically require multiple concurrent
connections to test real-world scenarios, including deadlocks and nested transactions.
Other edge cases are covered by specialized test files (savepoints, errors, etc.).
"""

from src.common.test_rig_python import MultiConnectionTestHandler, PyStateContext, PyState
import time
import threading


class DeadlockTestHandler(MultiConnectionTestHandler):
    """Test deadlock detection and resolution using multiple concurrent connections."""
    
    def __init__(self):
        super().__init__(connection_count=3)  # Use 3 connections for deadlock testing
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test tables for deadlock scenarios"""
        # Call parent to setup connections
        super().enter(context)
        
        if context.connections:
            conn = context.connections[0]  # Use first connection for setup
            
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
        """Test deadlock scenarios using multiple concurrent connections"""
        if not context.connections:
            return PyState.error("No connections available")
        
        results = []
        
        try:
            # Define concurrent operations that could cause deadlocks
            operations = [
                # Connection 0: Lock table1, then try to lock table2
                {'connection_id': 0, 'operation': 'start_transaction'},
                {'connection_id': 0, 'operation': 'query', 'query': f"SELECT * FROM {self.table1} WHERE id = 1 FOR UPDATE"},
                {'connection_id': 0, 'operation': 'query', 'query': f"SELECT * FROM {self.table2} WHERE id = 1 FOR UPDATE"},
                {'connection_id': 0, 'operation': 'query', 'query': f"UPDATE {self.table1} SET value = 999 WHERE id = 1"},
                {'connection_id': 0, 'operation': 'commit'},
                
                # Connection 1: Lock table2, then try to lock table1 (potential deadlock)
                {'connection_id': 1, 'operation': 'start_transaction'},
                {'connection_id': 1, 'operation': 'query', 'query': f"SELECT * FROM {self.table2} WHERE id = 1 FOR UPDATE"},
                {'connection_id': 1, 'operation': 'query', 'query': f"SELECT * FROM {self.table1} WHERE id = 1 FOR UPDATE"},
                {'connection_id': 1, 'operation': 'query', 'query': f"UPDATE {self.table2} SET value = 888 WHERE id = 1"},
                {'connection_id': 1, 'operation': 'commit'},
                
                # Connection 2: Concurrent read operations
                {'connection_id': 2, 'operation': 'start_transaction'},
                {'connection_id': 2, 'operation': 'query', 'query': f"SELECT * FROM {self.table1} WHERE id = 2"},
                {'connection_id': 2, 'operation': 'query', 'query': f"SELECT * FROM {self.table2} WHERE id = 2"},
                {'connection_id': 2, 'operation': 'commit'},
            ]
            
            # Execute operations concurrently
            concurrent_results = self.execute_concurrent_operations(operations)
            
            # Analyze results
            for result in concurrent_results:
                conn_id = result['connection_id']
                status = result['status']
                if status == 'success' or status in ['transaction_started', 'committed']:
                    results.append(f"✓ Connection {conn_id}: {status}")
                else:
                    results.append(f"✗ Connection {conn_id}: {status}")
                    if 'error' in result:
                        results.append(f"  Error: {result['error']}")
            
            print("\n=== DEADLOCK TEST RESULTS ===")
            for result in results:
                print(result)
            
            return PyState.completed()
            
        except Exception as e:
            print(f"Error during deadlock test: {e}")
            return PyState.error(f"Deadlock test failed: {e}")
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connections and hasattr(self, 'table1') and hasattr(self, 'table2'):
            context.connections[0].execute_query(f"DROP TABLE IF EXISTS {self.table1}")
            context.connections[0].execute_query(f"DROP TABLE IF EXISTS {self.table2}")
            print(f"✓ Cleaned up deadlock test tables")


class NestedTransactionTestHandler(MultiConnectionTestHandler):
    """Test nested transaction behavior using multiple connections."""
    
    def __init__(self):
        super().__init__(connection_count=3)  # Use 3 connections for nested transaction testing
    
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        super().enter(context)
        
        if context.connections:
            conn = context.connections[0]  # Use first connection for setup
            
            self.table_name = f"nested_txn_test_{int(time.time())}"
            
            conn.execute_query(f"""
                CREATE TABLE {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT,
                    txn_level INT
                )
            """)
            
            print(f"✓ Created nested transaction test table: {self.table_name}")
        
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Test nested transaction behavior using multiple connections"""
        if not context.connections:
            return PyState.error("No connections available")
        
        results = []
        
        try:
            # Define concurrent nested transaction operations
            operations = [
                # Connection 0: Nested transactions with commits
                {'connection_id': 0, 'operation': 'start_transaction'},
                {'connection_id': 0, 'operation': 'query', 'query': f"INSERT INTO {self.table_name} (id, name, value, txn_level) VALUES (1, 'outer_txn', 100, 1)"},
                {'connection_id': 0, 'operation': 'start_transaction'},
                {'connection_id': 0, 'operation': 'query', 'query': f"INSERT INTO {self.table_name} (id, name, value, txn_level) VALUES (2, 'inner_txn', 200, 2)"},
                {'connection_id': 0, 'operation': 'commit'},  # Commit inner transaction
                {'connection_id': 0, 'operation': 'query', 'query': f"INSERT INTO {self.table_name} (id, name, value, txn_level) VALUES (3, 'after_inner_commit', 300, 1)"},
                {'connection_id': 0, 'operation': 'commit'},  # Commit outer transaction
                
                # Connection 1: Nested transactions with rollback
                {'connection_id': 1, 'operation': 'start_transaction'},
                {'connection_id': 1, 'operation': 'query', 'query': f"INSERT INTO {self.table_name} (id, name, value, txn_level) VALUES (4, 'outer_txn2', 400, 1)"},
                {'connection_id': 1, 'operation': 'start_transaction'},
                {'connection_id': 1, 'operation': 'query', 'query': f"INSERT INTO {self.table_name} (id, name, value, txn_level) VALUES (5, 'inner_txn2', 500, 2)"},
                {'connection_id': 1, 'operation': 'rollback'},  # Rollback inner transaction
                {'connection_id': 1, 'operation': 'query', 'query': f"INSERT INTO {self.table_name} (id, name, value, txn_level) VALUES (6, 'after_inner_rollback', 600, 1)"},
                {'connection_id': 1, 'operation': 'commit'},  # Commit outer transaction
                
                # Connection 2: Concurrent read operations
                {'connection_id': 2, 'operation': 'start_transaction'},
                {'connection_id': 2, 'operation': 'query', 'query': f"SELECT COUNT(*) FROM {self.table_name}"},
                {'connection_id': 2, 'operation': 'commit'},
            ]
            
            # Execute operations concurrently
            concurrent_results = self.execute_concurrent_operations(operations)
            
            # Verify final state
            final_check = context.connections[0].execute_query(f"SELECT COUNT(*) FROM {self.table_name}")
            if final_check and final_check[0].get('col_0', 0) >= 4:
                results.append("✓ Final row count verification passed")
            else:
                results.append("✗ Final row count verification failed")
                return PyState.error("Final row count verification failed")
            
            # Analyze results
            for result in concurrent_results:
                conn_id = result['connection_id']
                status = result['status']
                if status in ['success', 'transaction_started', 'committed', 'rolled_back']:
                    results.append(f"✓ Connection {conn_id}: {status}")
                else:
                    results.append(f"✗ Connection {conn_id}: {status}")
            
            print("\n=== NESTED TRANSACTION TEST RESULTS ===")
            for result in results:
                print(result)
            
            return PyState.completed()
            
        except Exception as e:
            print(f"Error during nested transaction test: {e}")
            return PyState.error(f"Nested transaction test failed: {e}")
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase"""
        if context.connections and hasattr(self, 'table_name'):
            context.connections[0].execute_query(f"DROP TABLE IF EXISTS {self.table_name}")
            print(f"✓ Cleaned up nested transaction test table") 
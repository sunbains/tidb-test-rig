"""
Basic transaction tests for TiDB.
"""

from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState

class BasicTransactionHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        """Setup phase - create test table"""
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS txn_test")
            context.connection.execute_query("""
                CREATE TABLE txn_test (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    balance DECIMAL(10,2)
                )
            """)
        return PyState.connecting()
    
    def execute(self, context: PyStateContext) -> str:
        """Main test logic - test basic transaction operations"""
        if context.connection:
            conn = context.connection
            
            # Start transaction
            conn.execute_query("START TRANSACTION")
            
            # Insert data
            conn.execute_query("INSERT INTO txn_test (id, name, balance) VALUES (1, 'Alice', 100.00)")
            conn.execute_query("INSERT INTO txn_test (id, name, balance) VALUES (2, 'Bob', 200.00)")
            
            # Verify data is visible within transaction
            result = conn.execute_query("SELECT COUNT(*) FROM txn_test")
            if result and result[0].get('col_0', 0) == 2:
                # Commit transaction
                conn.execute_query("COMMIT")
                return PyState.completed()
            else:
                # Rollback on failure
                conn.execute_query("ROLLBACK")
                return PyState.error("Transaction verification failed")
        
        return PyState.completed()
    
    def exit(self, context: PyStateContext) -> None:
        """Cleanup phase - remove test table"""
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS txn_test") 
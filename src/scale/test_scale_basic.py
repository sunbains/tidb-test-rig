"""
Basic scale test for demonstrating the infrastructure.
"""

from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState


class ScaleBasicHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            # Create test table for scale testing
            context.connection.execute_query("DROP TABLE IF EXISTS scale_test")
            context.connection.execute_query("""
                CREATE TABLE scale_test (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    value INT,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )
            """)
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            conn = context.connection
            
            # Basic scale operations
            # 1. Insert multiple rows
            for i in range(100):
                conn.execute_query(f"INSERT INTO scale_test (id, name, value) VALUES ({i}, 'row_{i}', {i * 10})")
            
            # 2. Update multiple rows
            conn.execute_query("UPDATE scale_test SET value = value * 2 WHERE id < 50")
            
            # 3. Delete some rows
            conn.execute_query("DELETE FROM scale_test WHERE id >= 90")
            
            # 4. Query with aggregation
            result = conn.execute_query("SELECT COUNT(*), AVG(value), MAX(value) FROM scale_test")
            if result:
                return PyState.completed()
                
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS scale_test") 
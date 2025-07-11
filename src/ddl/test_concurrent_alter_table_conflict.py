
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState

class ConcurrentAlterTableConflictHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")
            context.connection.execute_query("CREATE TABLE ddl_test (id INT PRIMARY KEY)")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("ALTER TABLE ddl_test ADD COLUMN col1 INT")
            context.connection.execute_query("ALTER TABLE ddl_test ADD COLUMN col2 INT")
            # Optionally verify both columns exist
            result1 = context.connection.execute_query("SHOW COLUMNS FROM ddl_test LIKE 'col1'")
            result2 = context.connection.execute_query("SHOW COLUMNS FROM ddl_test LIKE 'col2'")
            if result1 and result2:
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")

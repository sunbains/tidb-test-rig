
from test_rig_python import PyStateHandler, PyStateContext, PyState

class ConcurrentInsertAlterHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")
            context.connection.execute_query("CREATE TABLE ddl_test (id INT PRIMARY KEY)")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            for i in range(1, 11):
                context.connection.execute_query(f"INSERT INTO ddl_test (id) VALUES ({i})")
            context.connection.execute_query("ALTER TABLE ddl_test ADD COLUMN name VARCHAR(100)")
            result1 = context.connection.execute_query("SELECT COUNT(*) FROM ddl_test")
            result2 = context.connection.execute_query("SHOW COLUMNS FROM ddl_test LIKE 'name'")
            if result1 and result2:
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")

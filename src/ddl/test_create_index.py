
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState

class CreateIndexHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")
            context.connection.execute_query("CREATE TABLE ddl_test (id INT PRIMARY KEY, name VARCHAR(100))")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("CREATE INDEX idx_name ON ddl_test(name)")
            result = context.connection.execute_query("SHOW INDEX FROM ddl_test WHERE Key_name='idx_name'")
            if result and len(result) == 1:
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")


from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState

class DropIndexHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")
            context.connection.execute_query("CREATE TABLE ddl_test (id INT PRIMARY KEY, name VARCHAR(100))")
            context.connection.execute_query("CREATE INDEX idx_name ON ddl_test(name)")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP INDEX idx_name ON ddl_test")
            result = context.connection.execute_query("SHOW INDEX FROM ddl_test WHERE Key_name='idx_name'")
            if result == []:
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")

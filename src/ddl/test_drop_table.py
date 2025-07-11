
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState

class DropTableHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("CREATE TABLE IF NOT EXISTS ddl_test (id INT PRIMARY KEY)")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE ddl_test")
            result = context.connection.execute_query("SHOW TABLES LIKE 'ddl_test'")
            if result == []:
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        pass

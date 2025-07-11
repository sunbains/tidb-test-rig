
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState

class CreateDatabaseHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP DATABASE IF EXISTS test_db")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("CREATE DATABASE test_db")
            result = context.connection.execute_query("SHOW DATABASES LIKE 'test_db'")
            if result and any('test_db' in str(row) for row in result):
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP DATABASE IF EXISTS test_db")

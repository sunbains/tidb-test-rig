
from test_rig_python import PyStateHandler, PyStateContext, PyState

class AlterDatabaseHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP DATABASE IF EXISTS test_db")
            context.connection.execute_query("CREATE DATABASE test_db DEFAULT CHARACTER SET utf8mb4")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("ALTER DATABASE test_db CHARACTER SET latin1")
            result = context.connection.execute_query("SELECT DEFAULT_CHARACTER_SET_NAME FROM information_schema.SCHEMATA WHERE SCHEMA_NAME='test_db'")
            if result and any('latin1' in str(row) for row in result):
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP DATABASE IF EXISTS test_db")

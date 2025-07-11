
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState

class RenameTableHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS old_name")
            context.connection.execute_query("CREATE TABLE old_name (id INT PRIMARY KEY)")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("RENAME TABLE old_name TO new_name")
            result = context.connection.execute_query("SHOW TABLES LIKE 'new_name'")
            if result and any('new_name' in str(row) for row in result):
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS new_name")

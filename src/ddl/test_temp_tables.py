
from test_rig_python import PyStateHandler, PyStateContext, PyState

class TempTablesHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        # No setup needed for temp tables
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("CREATE TEMPORARY TABLE tmp_test (id INT)")
            result = context.connection.execute_query("SHOW TABLES LIKE 'tmp_test'")
            if result and any('tmp_test' in str(row) for row in result):
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        # Temp tables are automatically dropped when session ends
        pass

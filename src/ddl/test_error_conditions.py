
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState

class ErrorConditionsHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")
            context.connection.execute_query("CREATE TABLE ddl_test (id INT PRIMARY KEY)")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            try:
                context.connection.execute_query("ALTER TABLE ddl_test MODIFY COLUMN non_existent INT")
            except Exception:
                # Expecting an error
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")

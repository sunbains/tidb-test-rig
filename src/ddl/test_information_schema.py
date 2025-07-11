
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState

class InformationSchemaHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")
            context.connection.execute_query("CREATE TABLE ddl_test (id INT PRIMARY KEY)")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            result = context.connection.execute_query("SELECT TABLE_NAME FROM information_schema.TABLES WHERE TABLE_NAME='ddl_test'")
            if result and any('ddl_test' in str(row) for row in result):
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")

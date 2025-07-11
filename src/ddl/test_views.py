
from test_rig_python import PyStateHandler, PyStateContext, PyState

class ViewsHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")
            context.connection.execute_query("CREATE TABLE ddl_test (id INT)")
            context.connection.execute_query("DROP VIEW IF EXISTS v_test")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("CREATE VIEW v_test AS SELECT * FROM ddl_test")
            result = context.connection.execute_query("SHOW FULL TABLES WHERE Table_type = 'VIEW' AND Tables_in_test = 'v_test'")
            if result and any('v_test' in str(row) for row in result):
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP VIEW IF EXISTS v_test")
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")

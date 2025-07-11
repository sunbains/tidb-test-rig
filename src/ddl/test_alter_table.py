
from test_rig_python import PyStateHandler, PyStateContext, PyState

class AlterTableHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")
            context.connection.execute_query("CREATE TABLE ddl_test (id INT PRIMARY KEY, name VARCHAR(100))")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            conn = context.connection
            conn.execute_query("ALTER TABLE ddl_test ADD COLUMN age INT")
            conn.execute_query("ALTER TABLE ddl_test MODIFY COLUMN name TEXT")
            conn.execute_query("ALTER TABLE ddl_test DROP COLUMN age")
            # Optionally check results here
            return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")

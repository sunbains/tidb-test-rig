
from test_rig_python import PyStateHandler, PyStateContext, PyState

class CreateTableHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        # Drop table if exists
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")
        return PyState.connecting()  # or next logical state

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("CREATE TABLE ddl_test (id INT PRIMARY KEY AUTO_INCREMENT, name VARCHAR(100) NOT NULL)")
            result = context.connection.execute_query("SHOW TABLES LIKE 'ddl_test'")
            if result and any('ddl_test' in str(row) for row in result):
                return PyState.completed()
        return PyState.completed()  # or PyState.error("Table creation failed") if error state is defined

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")

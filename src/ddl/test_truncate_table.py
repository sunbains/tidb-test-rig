
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState

class TruncateTableHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")
            context.connection.execute_query("CREATE TABLE ddl_test (id INT PRIMARY KEY AUTO_INCREMENT)")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("INSERT INTO ddl_test VALUES (NULL)")
            result1 = context.connection.execute_query("SELECT COUNT(*) FROM ddl_test")
            if result1 and any('1' in str(row) for row in result1):
                context.connection.execute_query("TRUNCATE TABLE ddl_test")
                result2 = context.connection.execute_query("SELECT COUNT(*) FROM ddl_test")
                if result2 and any('0' in str(row) for row in result2):
                    return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")

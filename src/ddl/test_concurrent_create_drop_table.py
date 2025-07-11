
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState

class ConcurrentCreateDropTableHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        # No-op for setup
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("CREATE TABLE IF NOT EXISTS ddl_test (id INT PRIMARY KEY)")
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")
            # Table may or may not exist depending on execution order
            return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        pass

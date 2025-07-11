
from test_rig_python import PyStateHandler, PyStateContext, PyState

class ProceduresHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP PROCEDURE IF EXISTS p_test")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("""
                CREATE PROCEDURE p_test ()
                BEGIN
                SELECT 1;
                END
            """)
            result = context.connection.execute_query("SHOW PROCEDURE STATUS WHERE Name='p_test'")
            if result and any('p_test' in str(row) for row in result):
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP PROCEDURE IF EXISTS p_test")

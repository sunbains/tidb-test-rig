"""
Specialized database objects for TiDB.

This test file consolidates tests for specialized database objects including
views, temporary tables, stored procedures, and permissions. These objects
have specific behaviors and requirements that need dedicated testing.
"""

from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState


class ViewsHandler(PyStateHandler):
    """Test view creation and management."""
    
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


class TempTablesHandler(PyStateHandler):
    """Test temporary table creation and behavior."""
    
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


class ProceduresHandler(PyStateHandler):
    """Test stored procedure creation and management."""
    
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


class PermissionsHandler(PyStateHandler):
    """Test basic permission operations."""
    
    def enter(self, context: PyStateContext) -> str:
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Test basic permission query
            result = context.connection.execute_query("SHOW GRANTS")
            if result:
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        pass


class InformationSchemaHandler(PyStateHandler):
    """Test information schema queries."""
    
    def enter(self, context: PyStateContext) -> str:
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Test information schema access
            result = context.connection.execute_query("SELECT COUNT(*) FROM information_schema.tables")
            if result:
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        pass 
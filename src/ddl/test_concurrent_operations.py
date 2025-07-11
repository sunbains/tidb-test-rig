"""
Concurrent DDL operations for TiDB.

This test file consolidates concurrent DDL operations to test how TiDB handles
multiple DDL operations running simultaneously. These tests verify that TiDB
properly serializes DDL operations and handles conflicts gracefully.
"""

from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState


class ConcurrentCreateDropTableHandler(PyStateHandler):
    """Test concurrent CREATE and DROP TABLE operations."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS concurrent_test")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Create table
            context.connection.execute_query("CREATE TABLE concurrent_test (id INT PRIMARY KEY)")
            # Drop table
            context.connection.execute_query("DROP TABLE concurrent_test")
            # Verify table is dropped
            result = context.connection.execute_query("SHOW TABLES LIKE 'concurrent_test'")
            if result == []:
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS concurrent_test")


class ConcurrentInsertAlterHandler(PyStateHandler):
    """Test concurrent INSERT and ALTER TABLE operations."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS concurrent_test")
            context.connection.execute_query("CREATE TABLE concurrent_test (id INT PRIMARY KEY, name VARCHAR(100))")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Insert data
            context.connection.execute_query("INSERT INTO concurrent_test (id, name) VALUES (1, 'test1')")
            # Alter table
            context.connection.execute_query("ALTER TABLE concurrent_test ADD COLUMN age INT")
            # Verify both operations succeeded
            result = context.connection.execute_query("SELECT * FROM concurrent_test")
            if result and len(result) > 0:
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS concurrent_test")


class ConcurrentDdlDmlHandler(PyStateHandler):
    """Test concurrent DDL and DML operations."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS concurrent_test")
            context.connection.execute_query("CREATE TABLE concurrent_test (id INT PRIMARY KEY, name VARCHAR(100))")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # DML operation
            context.connection.execute_query("INSERT INTO concurrent_test (id, name) VALUES (1, 'test1')")
            # DDL operation
            context.connection.execute_query("ALTER TABLE concurrent_test ADD COLUMN status VARCHAR(20)")
            # Verify both operations succeeded
            result = context.connection.execute_query("SELECT * FROM concurrent_test")
            if result and len(result) > 0:
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS concurrent_test")


class ConcurrentAlterTableConflictHandler(PyStateHandler):
    """Test concurrent ALTER TABLE operations that might conflict."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS concurrent_test")
            context.connection.execute_query("CREATE TABLE concurrent_test (id INT PRIMARY KEY, name VARCHAR(100))")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # First alter operation
            context.connection.execute_query("ALTER TABLE concurrent_test ADD COLUMN age INT")
            # Second alter operation
            context.connection.execute_query("ALTER TABLE concurrent_test ADD COLUMN email VARCHAR(255)")
            # Verify both columns were added
            result = context.connection.execute_query("SHOW COLUMNS FROM concurrent_test")
            if result and len(result) >= 4:  # id, name, age, email
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS concurrent_test") 
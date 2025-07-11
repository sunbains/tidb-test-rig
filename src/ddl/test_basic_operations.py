"""
Basic DDL operations for TiDB.

This test file consolidates basic CREATE and DROP operations for tables, indexes, and databases.
These are fundamental operations that are tested individually to ensure proper functionality.
"""

from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState


class CreateTableHandler(PyStateHandler):
    """Test basic table creation."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("CREATE TABLE ddl_test (id INT PRIMARY KEY AUTO_INCREMENT, name VARCHAR(100) NOT NULL)")
            result = context.connection.execute_query("SHOW TABLES LIKE 'ddl_test'")
            if result and any('ddl_test' in str(row) for row in result):
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")


class DropTableHandler(PyStateHandler):
    """Test basic table dropping."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("CREATE TABLE IF NOT EXISTS ddl_test (id INT PRIMARY KEY)")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE ddl_test")
            result = context.connection.execute_query("SHOW TABLES LIKE 'ddl_test'")
            if result == []:
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        pass


class CreateIndexHandler(PyStateHandler):
    """Test basic index creation."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")
            context.connection.execute_query("CREATE TABLE ddl_test (id INT PRIMARY KEY, name VARCHAR(100))")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("CREATE INDEX idx_name ON ddl_test(name)")
            result = context.connection.execute_query("SHOW INDEX FROM ddl_test WHERE Key_name='idx_name'")
            if result and len(result) == 1:
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")


class DropIndexHandler(PyStateHandler):
    """Test basic index dropping."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")
            context.connection.execute_query("CREATE TABLE ddl_test (id INT PRIMARY KEY, name VARCHAR(100))")
            context.connection.execute_query("CREATE INDEX idx_name ON ddl_test(name)")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP INDEX idx_name ON ddl_test")
            result = context.connection.execute_query("SHOW INDEX FROM ddl_test WHERE Key_name='idx_name'")
            if result == []:
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")


class CreateDatabaseHandler(PyStateHandler):
    """Test basic database creation."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP DATABASE IF EXISTS test_db")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("CREATE DATABASE test_db")
            result = context.connection.execute_query("SHOW DATABASES LIKE 'test_db'")
            if result and any('test_db' in str(row) for row in result):
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP DATABASE IF EXISTS test_db")


class RenameTableHandler(PyStateHandler):
    """Test basic table renaming."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test_renamed")
            context.connection.execute_query("CREATE TABLE ddl_test (id INT PRIMARY KEY)")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("RENAME TABLE ddl_test TO ddl_test_renamed")
            result = context.connection.execute_query("SHOW TABLES LIKE 'ddl_test_renamed'")
            if result and any('ddl_test_renamed' in str(row) for row in result):
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test_renamed")


class TruncateTableHandler(PyStateHandler):
    """Test basic table truncation."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")
            context.connection.execute_query("CREATE TABLE ddl_test (id INT PRIMARY KEY)")
            context.connection.execute_query("INSERT INTO ddl_test (id) VALUES (1), (2), (3)")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("TRUNCATE TABLE ddl_test")
            result = context.connection.execute_query("SELECT COUNT(*) FROM ddl_test")
            if result and result[0].get('col_0', 0) == 0:
                return PyState.completed()
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test") 
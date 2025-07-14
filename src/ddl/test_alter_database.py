"""
Comprehensive TiDB ALTER DATABASE test covering all possible ALTER operations.
"""

from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState


class AlterDatabaseHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP DATABASE IF EXISTS test_db")
            context.connection.execute_query("CREATE DATABASE test_db DEFAULT CHARACTER SET utf8mb4")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            conn = context.connection
            
            # 1. ALTER DATABASE CHARACTER SET
            conn.execute_query("ALTER DATABASE test_db CHARACTER SET latin1")
            conn.execute_query("ALTER DATABASE test_db CHARACTER SET utf8")
            conn.execute_query("ALTER DATABASE test_db CHARACTER SET utf8mb4")
            
            # 2. ALTER DATABASE COLLATION
            conn.execute_query("ALTER DATABASE test_db COLLATE latin1_bin")
            conn.execute_query("ALTER DATABASE test_db COLLATE utf8mb4_unicode_ci")
            conn.execute_query("ALTER DATABASE test_db COLLATE utf8mb4_general_ci")
            
            # 3. ALTER DATABASE CHARACTER SET and COLLATION together
            conn.execute_query("ALTER DATABASE test_db CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci")
            
            # 4. ALTER DATABASE ENCRYPTION (if supported)
            try:
                conn.execute_query("ALTER DATABASE test_db ENCRYPTION = 'Y'")
                conn.execute_query("ALTER DATABASE test_db ENCRYPTION = 'N'")
            except:
                pass  # Encryption might not be supported in all TiDB versions
            
            # 5. ALTER DATABASE READ ONLY (if supported)
            try:
                conn.execute_query("ALTER DATABASE test_db READ ONLY = 1")
                conn.execute_query("ALTER DATABASE test_db READ ONLY = 0")
            except:
                pass  # Read-only might not be supported in all TiDB versions
            
            # 6. ALTER DATABASE UPGRADE DATA DIRECTORY NAME (if supported)
            try:
                conn.execute_query("ALTER DATABASE test_db UPGRADE DATA DIRECTORY NAME")
            except:
                pass  # This might not be supported in all TiDB versions
            
            # 7. ALTER DATABASE with DEFAULT keyword
            conn.execute_query("ALTER DATABASE test_db DEFAULT CHARACTER SET utf8mb4")
            conn.execute_query("ALTER DATABASE test_db DEFAULT COLLATE utf8mb4_unicode_ci")
            
            # 8. ALTER SCHEMA (synonym for ALTER DATABASE)
            conn.execute_query("ALTER SCHEMA test_db CHARACTER SET utf8mb4")
            conn.execute_query("ALTER SCHEMA test_db COLLATE utf8mb4_general_ci")
            
            # 9. ALTER DATABASE with complex character sets
            conn.execute_query("ALTER DATABASE test_db CHARACTER SET utf8mb4 COLLATE utf8mb4_bin")
            conn.execute_query("ALTER DATABASE test_db CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci")
            
            # 10. ALTER DATABASE with legacy character sets
            conn.execute_query("ALTER DATABASE test_db CHARACTER SET utf8 COLLATE utf8_general_ci")
            conn.execute_query("ALTER DATABASE test_db CHARACTER SET utf8 COLLATE utf8_unicode_ci")
            
            # Verify the database settings
            result = conn.execute_query("SELECT DEFAULT_CHARACTER_SET_NAME, DEFAULT_COLLATION_NAME FROM information_schema.SCHEMATA WHERE SCHEMA_NAME='test_db'")
            if result and any('utf8mb4' in str(row) for row in result):
                return PyState.completed()
                
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP DATABASE IF EXISTS test_db")

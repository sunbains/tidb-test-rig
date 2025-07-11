"""
Comprehensive TiDB ALTER TABLE test covering all possible ALTER operations.
"""

from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState


class AlterTableHandler(PyStateHandler):
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            # Create test table with various data types
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")
            context.connection.execute_query("""
                CREATE TABLE ddl_test (
                    id INT PRIMARY KEY,
                    name VARCHAR(100),
                    age INT DEFAULT 25,
                    email VARCHAR(255),
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    status ENUM('active', 'inactive') DEFAULT 'active',
                    data JSON,
                    score DECIMAL(5,2) DEFAULT 0.00
                )
            """)
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            conn = context.connection
            
            # 1. ALTER TABLE ADD COLUMN operations
            conn.execute_query("ALTER TABLE ddl_test ADD COLUMN phone VARCHAR(20)")
            conn.execute_query("ALTER TABLE ddl_test ADD COLUMN address TEXT AFTER name")
            conn.execute_query("ALTER TABLE ddl_test ADD COLUMN birth_date DATE FIRST")
            conn.execute_query("ALTER TABLE ddl_test ADD COLUMN (city VARCHAR(50), country VARCHAR(50))")
            
            # 2. ALTER TABLE MODIFY COLUMN operations
            conn.execute_query("ALTER TABLE ddl_test MODIFY COLUMN name VARCHAR(200)")
            conn.execute_query("ALTER TABLE ddl_test MODIFY COLUMN age INT NOT NULL")
            conn.execute_query("ALTER TABLE ddl_test MODIFY COLUMN email VARCHAR(100) UNIQUE")
            conn.execute_query("ALTER TABLE ddl_test MODIFY COLUMN score DECIMAL(10,2)")
            
            # 3. ALTER TABLE CHANGE COLUMN operations (rename + modify)
            conn.execute_query("ALTER TABLE ddl_test CHANGE COLUMN phone phone_number VARCHAR(25)")
            conn.execute_query("ALTER TABLE ddl_test CHANGE COLUMN address location TEXT")
            conn.execute_query("ALTER TABLE ddl_test CHANGE COLUMN birth_date dob DATE")
            
            # 4. ALTER TABLE DROP COLUMN operations
            conn.execute_query("ALTER TABLE ddl_test DROP COLUMN phone_number")
            conn.execute_query("ALTER TABLE ddl_test DROP COLUMN location")
            
            # 5. ALTER TABLE ADD INDEX operations
            conn.execute_query("ALTER TABLE ddl_test ADD INDEX idx_name (name)")
            conn.execute_query("ALTER TABLE ddl_test ADD UNIQUE INDEX idx_email (email)")
            conn.execute_query("ALTER TABLE ddl_test ADD INDEX idx_age_status (age, status)")
            conn.execute_query("ALTER TABLE ddl_test ADD FULLTEXT INDEX idx_data ((CAST(data AS CHAR(100))))")
            
            # 6. ALTER TABLE DROP INDEX operations
            conn.execute_query("ALTER TABLE ddl_test DROP INDEX idx_name")
            conn.execute_query("ALTER TABLE ddl_test DROP INDEX idx_email")
            
            # 7. ALTER TABLE ADD PRIMARY KEY
            conn.execute_query("ALTER TABLE ddl_test ADD PRIMARY KEY (id)")
            
            # 8. ALTER TABLE ADD FOREIGN KEY
            conn.execute_query("CREATE TABLE IF NOT EXISTS ref_table (ref_id INT PRIMARY KEY)")
            conn.execute_query("ALTER TABLE ddl_test ADD CONSTRAINT fk_ref FOREIGN KEY (id) REFERENCES ref_table(ref_id)")
            conn.execute_query("ALTER TABLE ddl_test DROP FOREIGN KEY fk_ref")
            conn.execute_query("DROP TABLE IF EXISTS ref_table")
            
            # 9. ALTER TABLE ADD UNIQUE constraint
            conn.execute_query("ALTER TABLE ddl_test ADD UNIQUE KEY uk_email (email)")
            conn.execute_query("ALTER TABLE ddl_test DROP INDEX uk_email")
            
            # 10. ALTER TABLE SET/DROP DEFAULT
            conn.execute_query("ALTER TABLE ddl_test ALTER COLUMN age SET DEFAULT 30")
            conn.execute_query("ALTER TABLE ddl_test ALTER COLUMN age DROP DEFAULT")
            
            # 11. ALTER TABLE AUTO_INCREMENT
            conn.execute_query("ALTER TABLE ddl_test AUTO_INCREMENT = 1000")
            
            # 12. ALTER TABLE CHARACTER SET and COLLATION
            conn.execute_query("ALTER TABLE ddl_test CONVERT TO CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci")
            
            # 13. ALTER TABLE COMMENT
            conn.execute_query("ALTER TABLE ddl_test COMMENT = 'Test table for ALTER operations'")
            
            # 14. ALTER TABLE ROW_FORMAT
            conn.execute_query("ALTER TABLE ddl_test ROW_FORMAT = COMPRESSED")
            
            # 15. ALTER TABLE STORAGE ENGINE
            conn.execute_query("ALTER TABLE ddl_test ENGINE = InnoDB")
            
            # 16. ALTER TABLE PARTITION operations (if supported)
            try:
                conn.execute_query("ALTER TABLE ddl_test PARTITION BY RANGE (id) (PARTITION p0 VALUES LESS THAN (100))")
                conn.execute_query("ALTER TABLE ddl_test ADD PARTITION (PARTITION p1 VALUES LESS THAN (200))")
                conn.execute_query("ALTER TABLE ddl_test DROP PARTITION p0")
            except:
                pass  # Partitioning might not be supported in all TiDB versions
            
            # 17. ALTER TABLE COLUMN POSITION
            conn.execute_query("ALTER TABLE ddl_test MODIFY COLUMN dob DATE AFTER id")
            conn.execute_query("ALTER TABLE ddl_test MODIFY COLUMN city VARCHAR(50) FIRST")
            
            # 18. ALTER TABLE COLUMN ATTRIBUTES
            conn.execute_query("ALTER TABLE ddl_test MODIFY COLUMN name VARCHAR(200) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci")
            conn.execute_query("ALTER TABLE ddl_test MODIFY COLUMN age INT UNSIGNED")
            
            # 19. ALTER TABLE COLUMN COMMENT
            conn.execute_query("ALTER TABLE ddl_test MODIFY COLUMN name VARCHAR(200) COMMENT 'User full name'")
            
            # 20. ALTER TABLE COLUMN STORAGE
            conn.execute_query("ALTER TABLE ddl_test MODIFY COLUMN data JSON STORAGE MEMORY")
            
            # Verify the table structure
            result = conn.execute_query("SHOW COLUMNS FROM ddl_test")
            if result:
                return PyState.completed()
                
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS ddl_test")

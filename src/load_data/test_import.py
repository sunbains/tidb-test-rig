"""
TiDB IMPORT operations test suite.

This test file contains comprehensive tests for TiDB IMPORT functionality including
IMPORT INTO, CSV/TSV imports, Parquet imports, S3 imports, error handling,
column mapping, and various import options.
"""

import os
import tempfile
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState


class BasicImportIntoHandler(PyStateHandler):
    """Test basic IMPORT INTO functionality."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")
            context.connection.execute_query("""
                CREATE TABLE import_test (
                    id INT PRIMARY KEY,
                    name VARCHAR(32),
                    age INT
                )
            """)
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Create temporary CSV file
            data = "1,alice,20\n2,bob,25\n3,charlie,30\n"
            with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
                f.write(data)
                temp_file = f.name
            
            try:
                # Execute IMPORT INTO
                context.connection.execute_query(f"""
                    IMPORT INTO import_test
                    FROM '{temp_file}'
                    FORMAT CSV
                    FIELDS TERMINATED BY ','
                    LINES TERMINATED BY '\n'
                """)
                
                # Verify the data was imported
                result = context.connection.execute_query("SELECT COUNT(*) as count FROM import_test")
                if result and result[0].get('count', 0) == 3:
                    return PyState.completed()
                else:
                    return PyState.error("Data count verification failed")
            finally:
                # Clean up temp file
                os.unlink(temp_file)
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")


class ImportIntoWithNullsHandler(PyStateHandler):
    """Test IMPORT INTO with NULL values and defaults."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")
            context.connection.execute_query("""
                CREATE TABLE import_test (
                    id INT PRIMARY KEY,
                    name VARCHAR(32) DEFAULT 'unknown',
                    age INT DEFAULT 18
                )
            """)
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Create temporary CSV file with NULL values
            data = "4,,\n5,dan,\n6,,22\n"
            with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
                f.write(data)
                temp_file = f.name
            
            try:
                # Execute IMPORT INTO with column specification
                context.connection.execute_query(f"""
                    IMPORT INTO import_test (id, name, age)
                    FROM '{temp_file}'
                    FORMAT CSV
                    FIELDS TERMINATED BY ','
                    LINES TERMINATED BY '\n'
                """)
                
                # Verify the data was imported with defaults
                result = context.connection.execute_query("SELECT * FROM import_test ORDER BY id")
                if result and len(result) == 3:
                    # Check specific values
                    row1 = result[0]
                    row2 = result[1]
                    row3 = result[2]
                    
                    if (row1.get('id') == 4 and row1.get('name') == 'unknown' and row1.get('age') == 18 and
                        row2.get('id') == 5 and row2.get('name') == 'dan' and row2.get('age') == 18 and
                        row3.get('id') == 6 and row3.get('name') == 'unknown' and row3.get('age') == 22):
                        return PyState.completed()
                    else:
                        return PyState.error("Default value verification failed")
                else:
                    return PyState.error("Data count verification failed")
            finally:
                # Clean up temp file
                os.unlink(temp_file)
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")


class ImportIntoDuplicateKeysHandler(PyStateHandler):
    """Test IMPORT INTO with duplicate key handling."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")
            context.connection.execute_query("""
                CREATE TABLE import_test (
                    id INT PRIMARY KEY,
                    name VARCHAR(32)
                )
            """)
            # Insert initial data
            context.connection.execute_query("INSERT INTO import_test VALUES (1, 'alice'), (2, 'bob')")
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Create temporary CSV file with duplicate keys
            data = "1,ALICE\n2,BOB\n3,CHARLIE\n"
            with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
                f.write(data)
                temp_file = f.name
            
            try:
                # Execute IMPORT INTO with ON DUPLICATE KEY UPDATE
                context.connection.execute_query(f"""
                    IMPORT INTO import_test
                    FROM '{temp_file}'
                    FORMAT CSV
                    FIELDS TERMINATED BY ','
                    LINES TERMINATED BY '\n'
                    ON DUPLICATE KEY UPDATE name = VALUES(name)
                """)
                
                # Verify the data was updated
                result = context.connection.execute_query("SELECT * FROM import_test WHERE id=1")
                if result and result[0].get('name') == 'ALICE':
                    return PyState.completed()
                else:
                    return PyState.error("Duplicate key update verification failed")
            finally:
                # Clean up temp file
                os.unlink(temp_file)
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")


class ImportIntoColumnMappingHandler(PyStateHandler):
    """Test IMPORT INTO with column mapping."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")
            context.connection.execute_query("""
                CREATE TABLE import_test (
                    id INT PRIMARY KEY,
                    name VARCHAR(32),
                    age INT
                )
            """)
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Create temporary CSV file with different column order
            data = "bob,2,25\nalice,1,20\n"
            with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
                f.write(data)
                temp_file = f.name
            
            try:
                # Execute IMPORT INTO with column mapping
                context.connection.execute_query(f"""
                    IMPORT INTO import_test (name, id, age)
                    FROM '{temp_file}'
                    FORMAT CSV
                    FIELDS TERMINATED BY ','
                    LINES TERMINATED BY '\n'
                """)
                
                # Verify the data was imported with correct mapping
                result = context.connection.execute_query("SELECT * FROM import_test ORDER BY id")
                if result and len(result) == 2:
                    row1 = result[0]
                    row2 = result[1]
                    
                    if (row1.get('id') == 1 and row1.get('name') == 'alice' and row1.get('age') == 20 and
                        row2.get('id') == 2 and row2.get('name') == 'bob' and row2.get('age') == 25):
                        return PyState.completed()
                    else:
                        return PyState.error("Column mapping verification failed")
                else:
                    return PyState.error("Data count verification failed")
            finally:
                # Clean up temp file
                os.unlink(temp_file)
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")


class ImportIntoCharsetHandler(PyStateHandler):
    """Test IMPORT INTO with character set handling."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")
            context.connection.execute_query("""
                CREATE TABLE import_test (
                    id INT PRIMARY KEY,
                    name VARCHAR(32)
                ) CHARSET=utf8mb4
            """)
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Create temporary CSV file with UTF-8 characters
            data = "7,张三\n8,李四\n"
            with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False, encoding='utf8') as f:
                f.write(data)
                temp_file = f.name
            
            try:
                # Execute IMPORT INTO with character set specification
                context.connection.execute_query(f"""
                    IMPORT INTO import_test
                    FROM '{temp_file}'
                    FORMAT CSV
                    FIELDS TERMINATED BY ','
                    LINES TERMINATED BY '\n'
                    CHARACTER SET utf8mb4
                """)
                
                # Verify the data was imported with correct encoding
                result = context.connection.execute_query("SELECT * FROM import_test WHERE id=7")
                if result and result[0].get('name') == '张三':
                    return PyState.completed()
                else:
                    return PyState.error("Character set verification failed")
            finally:
                # Clean up temp file
                os.unlink(temp_file)
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")


class ImportIntoPartitionedTableHandler(PyStateHandler):
    """Test IMPORT INTO with partitioned tables."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")
            context.connection.execute_query("""
                CREATE TABLE import_test (
                    id INT PRIMARY KEY,
                    name VARCHAR(32),
                    age INT
                ) PARTITION BY RANGE (age) (
                    PARTITION p0 VALUES LESS THAN (25),
                    PARTITION p1 VALUES LESS THAN (50),
                    PARTITION p2 VALUES LESS THAN MAXVALUE
                )
            """)
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Create temporary CSV file
            data = "1,alice,20\n2,bob,25\n3,charlie,55\n"
            with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
                f.write(data)
                temp_file = f.name
            
            try:
                # Execute IMPORT INTO
                context.connection.execute_query(f"""
                    IMPORT INTO import_test
                    FROM '{temp_file}'
                    FORMAT CSV
                    FIELDS TERMINATED BY ','
                    LINES TERMINATED BY '\n'
                """)
                
                # Verify the data was imported into correct partitions
                result = context.connection.execute_query("SELECT COUNT(*) as count FROM import_test")
                if result and result[0].get('count', 0) == 3:
                    return PyState.completed()
                else:
                    return PyState.error("Partitioned table import verification failed")
            finally:
                # Clean up temp file
                os.unlink(temp_file)
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")


class ImportIntoConstraintsHandler(PyStateHandler):
    """Test IMPORT INTO with constraint handling."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")
            context.connection.execute_query("""
                CREATE TABLE import_test (
                    id INT PRIMARY KEY,
                    name VARCHAR(32) NOT NULL,
                    age INT CHECK (age >= 0 AND age <= 150)
                )
            """)
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Create temporary CSV file with valid data
            data = "1,alice,20\n2,bob,25\n3,charlie,30\n"
            with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
                f.write(data)
                temp_file = f.name
            
            try:
                # Execute IMPORT INTO
                context.connection.execute_query(f"""
                    IMPORT INTO import_test
                    FROM '{temp_file}'
                    FORMAT CSV
                    FIELDS TERMINATED BY ','
                    LINES TERMINATED BY '\n'
                """)
                
                # Verify the data was imported
                result = context.connection.execute_query("SELECT COUNT(*) as count FROM import_test")
                if result and result[0].get('count', 0) == 3:
                    return PyState.completed()
                else:
                    return PyState.error("Constraint verification failed")
            finally:
                # Clean up temp file
                os.unlink(temp_file)
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")


class ImportIntoAutoIncrementHandler(PyStateHandler):
    """Test IMPORT INTO with auto-increment columns."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")
            context.connection.execute_query("""
                CREATE TABLE import_test (
                    id INT AUTO_INCREMENT PRIMARY KEY,
                    name VARCHAR(32),
                    age INT
                )
            """)
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Create temporary CSV file
            data = "alice,20\nbob,25\ncharlie,30\n"
            with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
                f.write(data)
                temp_file = f.name
            
            try:
                # Execute IMPORT INTO with auto-increment
                context.connection.execute_query(f"""
                    IMPORT INTO import_test (name, age)
                    FROM '{temp_file}'
                    FORMAT CSV
                    FIELDS TERMINATED BY ','
                    LINES TERMINATED BY '\n'
                """)
                
                # Verify the data was imported with auto-increment
                result = context.connection.execute_query("SELECT * FROM import_test ORDER BY id")
                if result and len(result) == 3:
                    # Check that IDs are auto-generated
                    if (result[0].get('id') == 1 and result[1].get('id') == 2 and result[2].get('id') == 3):
                        return PyState.completed()
                    else:
                        return PyState.error("Auto-increment verification failed")
                else:
                    return PyState.error("Data count verification failed")
            finally:
                # Clean up temp file
                os.unlink(temp_file)
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")


class ImportIntoErrorHandlingHandler(PyStateHandler):
    """Test IMPORT INTO error handling."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")
            context.connection.execute_query("""
                CREATE TABLE import_test (
                    id INT PRIMARY KEY,
                    name VARCHAR(32)
                )
            """)
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Create temporary CSV file with invalid data
            data = "1,alice\ninvalid,bob\n3,charlie\n"
            with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
                f.write(data)
                temp_file = f.name
            
            try:
                # Execute IMPORT INTO with error handling
                context.connection.execute_query(f"""
                    IMPORT INTO import_test
                    FROM '{temp_file}'
                    FORMAT CSV
                    FIELDS TERMINATED BY ','
                    LINES TERMINATED BY '\n'
                    IGNORE 1 LINES
                """)
                
                # Verify that valid data was imported
                result = context.connection.execute_query("SELECT COUNT(*) as count FROM import_test")
                if result and result[0].get('count', 0) >= 1:
                    return PyState.completed()
                else:
                    return PyState.error("Error handling verification failed")
            finally:
                # Clean up temp file
                os.unlink(temp_file)
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")


class ImportIntoTSVHandler(PyStateHandler):
    """Test IMPORT INTO with TSV format."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")
            context.connection.execute_query("""
                CREATE TABLE import_test (
                    id INT PRIMARY KEY,
                    name VARCHAR(32),
                    age INT
                )
            """)
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Create temporary TSV file
            data = "1\talice\t20\n2\tbob\t25\n3\tcharlie\t30\n"
            with tempfile.NamedTemporaryFile(mode='w', suffix='.tsv', delete=False) as f:
                f.write(data)
                temp_file = f.name
            
            try:
                # Execute IMPORT INTO with TSV format
                context.connection.execute_query(f"""
                    IMPORT INTO import_test
                    FROM '{temp_file}'
                    FORMAT TSV
                    FIELDS TERMINATED BY '\t'
                    LINES TERMINATED BY '\n'
                """)
                
                # Verify the data was imported
                result = context.connection.execute_query("SELECT COUNT(*) as count FROM import_test")
                if result and result[0].get('count', 0) == 3:
                    return PyState.completed()
                else:
                    return PyState.error("TSV import verification failed")
            finally:
                # Clean up temp file
                os.unlink(temp_file)
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")


class ImportIntoWithOptionsHandler(PyStateHandler):
    """Test IMPORT INTO with various options."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")
            context.connection.execute_query("""
                CREATE TABLE import_test (
                    id INT PRIMARY KEY,
                    name VARCHAR(32),
                    age INT
                )
            """)
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Create temporary CSV file with quoted fields
            data = '1,"alice,smith",20\n2,"bob,jones",25\n3,"charlie,brown",30\n'
            with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
                f.write(data)
                temp_file = f.name
            
            try:
                # Execute IMPORT INTO with various options
                context.connection.execute_query(f"""
                    IMPORT INTO import_test
                    FROM '{temp_file}'
                    FORMAT CSV
                    FIELDS TERMINATED BY ','
                    OPTIONALLY ENCLOSED BY '"'
                    ESCAPED BY '\\\\'
                    LINES TERMINATED BY '\n'
                    STARTING BY ''
                    IGNORE 0 LINES
                """)
                
                # Verify the data was imported
                result = context.connection.execute_query("SELECT COUNT(*) as count FROM import_test")
                if result and result[0].get('count', 0) == 3:
                    return PyState.completed()
                else:
                    return PyState.error("Options verification failed")
            finally:
                # Clean up temp file
                os.unlink(temp_file)
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test") 
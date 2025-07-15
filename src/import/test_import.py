"""
TiDB Import operations test suite.

This test file contains comprehensive tests for TiDB import functionality including
LOAD DATA, CSV/TSV import, error handling, duplicate keys, column mapping, charset,
partitioned tables, constraints, and auto-increment scenarios.
"""

import os
import tempfile
from src.common.test_rig_python import PyStateHandler, PyStateContext, PyState


class BasicLoadDataHandler(PyStateHandler):
    """Test basic LOAD DATA functionality."""
    
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
                # Execute LOAD DATA
                context.connection.execute_query(f"""
                    LOAD DATA LOCAL INFILE '{temp_file}'
                    INTO TABLE import_test
                    FIELDS TERMINATED BY ','
                    LINES TERMINATED BY '\n'
                """)
                
                # Verify the data was loaded
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


class LoadDataWithNullsHandler(PyStateHandler):
    """Test LOAD DATA with NULL values and defaults."""
    
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
                # Execute LOAD DATA with column specification
                context.connection.execute_query(f"""
                    LOAD DATA LOCAL INFILE '{temp_file}'
                    INTO TABLE import_test
                    FIELDS TERMINATED BY ','
                    LINES TERMINATED BY '\n'
                    (id, name, age)
                """)
                
                # Verify the data was loaded with defaults
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


class LoadDataDuplicateKeysHandler(PyStateHandler):
    """Test LOAD DATA with duplicate key handling (REPLACE)."""
    
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
                # Execute LOAD DATA with REPLACE
                context.connection.execute_query(f"""
                    LOAD DATA LOCAL INFILE '{temp_file}'
                    REPLACE
                    INTO TABLE import_test
                    FIELDS TERMINATED BY ','
                    LINES TERMINATED BY '\n'
                """)
                
                # Verify the data was replaced
                result = context.connection.execute_query("SELECT * FROM import_test WHERE id=1")
                if result and result[0].get('name') == 'ALICE':
                    return PyState.completed()
                else:
                    return PyState.error("REPLACE verification failed")
            finally:
                # Clean up temp file
                os.unlink(temp_file)
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")


class LoadDataColumnMappingHandler(PyStateHandler):
    """Test LOAD DATA with column mapping."""
    
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
                # Execute LOAD DATA with column mapping
                context.connection.execute_query(f"""
                    LOAD DATA LOCAL INFILE '{temp_file}'
                    INTO TABLE import_test
                    FIELDS TERMINATED BY ','
                    LINES TERMINATED BY '\n'
                    (name, id, age)
                """)
                
                # Verify the data was loaded with correct mapping
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


class LoadDataCharsetHandler(PyStateHandler):
    """Test LOAD DATA with character set handling."""
    
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
                # Execute LOAD DATA with character set specification
                context.connection.execute_query(f"""
                    LOAD DATA LOCAL INFILE '{temp_file}'
                    INTO TABLE import_test
                    CHARACTER SET utf8mb4
                    FIELDS TERMINATED BY ','
                    LINES TERMINATED BY '\n'
                """)
                
                # Verify the data was loaded with correct encoding
                result = context.connection.execute_query("SELECT name FROM import_test WHERE id=7")
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


class LoadDataPartitionedTableHandler(PyStateHandler):
    """Test LOAD DATA into partitioned tables."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_partitioned")
            context.connection.execute_query("""
                CREATE TABLE import_partitioned (
                    id INT PRIMARY KEY,
                    region VARCHAR(16)
                ) PARTITION BY HASH(id) PARTITIONS 2
            """)
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Create temporary CSV file
            data = "1,us\n2,eu\n3,ap\n"
            with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
                f.write(data)
                temp_file = f.name
            
            try:
                # Execute LOAD DATA into partitioned table
                context.connection.execute_query(f"""
                    LOAD DATA LOCAL INFILE '{temp_file}'
                    INTO TABLE import_partitioned
                    FIELDS TERMINATED BY ','
                    LINES TERMINATED BY '\n'
                """)
                
                # Verify the data was loaded
                result = context.connection.execute_query("SELECT COUNT(*) as count FROM import_partitioned")
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
            context.connection.execute_query("DROP TABLE IF EXISTS import_partitioned")


class LoadDataConstraintsHandler(PyStateHandler):
    """Test LOAD DATA with foreign key constraints."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_fk_child")
            context.connection.execute_query("DROP TABLE IF EXISTS import_fk_parent")
            context.connection.execute_query("""
                CREATE TABLE import_fk_parent (
                    id INT PRIMARY KEY
                )
            """)
            context.connection.execute_query("INSERT INTO import_fk_parent VALUES (1), (2)")
            context.connection.execute_query("""
                CREATE TABLE import_fk_child (
                    id INT PRIMARY KEY,
                    parent_id INT,
                    FOREIGN KEY (parent_id) REFERENCES import_fk_parent(id)
                )
            """)
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Create temporary CSV file with valid foreign keys
            data = "10,1\n11,2\n"
            with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
                f.write(data)
                temp_file = f.name
            
            try:
                # Execute LOAD DATA with foreign key constraints
                context.connection.execute_query(f"""
                    LOAD DATA LOCAL INFILE '{temp_file}'
                    INTO TABLE import_fk_child
                    FIELDS TERMINATED BY ','
                    LINES TERMINATED BY '\n'
                """)
                
                # Verify the data was loaded
                result = context.connection.execute_query("SELECT COUNT(*) as count FROM import_fk_child")
                if result and result[0].get('count', 0) == 2:
                    return PyState.completed()
                else:
                    return PyState.error("Foreign key constraint import verification failed")
            finally:
                # Clean up temp file
                os.unlink(temp_file)
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_fk_child")
            context.connection.execute_query("DROP TABLE IF EXISTS import_fk_parent")


class LoadDataAutoIncrementHandler(PyStateHandler):
    """Test LOAD DATA with auto-increment columns."""
    
    def enter(self, context: PyStateContext) -> str:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")
            context.connection.execute_query("""
                CREATE TABLE import_test (
                    id INT PRIMARY KEY AUTO_INCREMENT,
                    name VARCHAR(32)
                )
            """)
        return PyState.connecting()

    def execute(self, context: PyStateContext) -> str:
        if context.connection:
            # Create temporary CSV file with empty auto-increment values
            data = ",alice\n,bob\n"
            with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
                f.write(data)
                temp_file = f.name
            
            try:
                # Execute LOAD DATA with auto-increment
                context.connection.execute_query(f"""
                    LOAD DATA LOCAL INFILE '{temp_file}'
                    INTO TABLE import_test
                    FIELDS TERMINATED BY ','
                    LINES TERMINATED BY '\n'
                    (id, name)
                """)
                
                # Verify the data was loaded with auto-generated IDs
                result = context.connection.execute_query("SELECT COUNT(*) as count FROM import_test")
                if result and result[0].get('count', 0) == 2:
                    return PyState.completed()
                else:
                    return PyState.error("Auto-increment import verification failed")
            finally:
                # Clean up temp file
                os.unlink(temp_file)
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")


class LoadDataErrorHandlingHandler(PyStateHandler):
    """Test LOAD DATA error handling for duplicate primary keys."""
    
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
            # Create temporary CSV file with duplicate primary keys
            data = "1,alice\n2,bob\n1,duplicate\n"
            with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
                f.write(data)
                temp_file = f.name
            
            try:
                # Execute LOAD DATA - this should fail due to duplicate primary key
                try:
                    context.connection.execute_query(f"""
                        LOAD DATA LOCAL INFILE '{temp_file}'
                        INTO TABLE import_test
                        FIELDS TERMINATED BY ','
                        LINES TERMINATED BY '\n'
                    """)
                    # If we get here, the test should fail because we expected an error
                    return PyState.error("Expected duplicate key error but import succeeded")
                except Exception as e:
                    # This is expected - duplicate primary key should cause an error
                    if "Duplicate entry" in str(e) or "PRIMARY" in str(e):
                        return PyState.completed()
                    else:
                        return PyState.error(f"Unexpected error: {e}")
            finally:
                # Clean up temp file
                os.unlink(temp_file)
        return PyState.completed()

    def exit(self, context: PyStateContext) -> None:
        if context.connection:
            context.connection.execute_query("DROP TABLE IF EXISTS import_test")

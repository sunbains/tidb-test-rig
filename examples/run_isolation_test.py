#!/usr/bin/env python3
"""
Standalone Python script to run the isolation test.

This script can be run directly without the Rust framework.
"""

try:
    import mysql.connector
except ImportError as e:
    print(f"[WARNING] Could not import mysql.connector: {e}")
    mysql = None
import time
import sys
from typing import Optional


class IsolationTest:
    """Standalone isolation test implementation."""

    def __init__(self, host: str, user: str, password: str, database: str, test_rows: int = 10):
        # Parse host and port
        if ":" in host:
            host_parts = host.split(":", 1)
            self.host = host_parts[0]
            self.port = int(host_parts[1])
        else:
            self.host = host
            self.port = 4000  # Default TiDB port
        
        self.user = user
        self.password = password
        self.database = database
        self.test_rows = test_rows
        self.table_name = f"isolation_test_{int(time.time())}"
        self.results = []

    def connect(self):
        """Create a database connection."""
        return mysql.connector.connect(
            host=self.host,
            port=self.port,
            user=self.user,
            password=self.password,
            database=self.database,
            autocommit=False
        )

    def setup(self):
        """Create the test table."""
        print(f"Creating test table: {self.table_name}")
        conn = self.connect()
        cursor = conn.cursor()
        
        try:
            cursor.execute(f"""
                CREATE TABLE IF NOT EXISTS {self.table_name} (
                    id INT PRIMARY KEY,
                    name VARCHAR(100) NOT NULL,
                    value INT NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )
            """)
            conn.commit()
            self.results.append(f"✓ Created test table: {self.table_name}")
        except Exception as e:
            print(f"Error creating table: {e}")
            raise
        finally:
            cursor.close()
            conn.close()

    def populate(self):
        """Insert test data."""
        print(f"Inserting {self.test_rows} rows...")
        conn = self.connect()
        cursor = conn.cursor()
        
        try:
            for i in range(1, self.test_rows + 1):
                cursor.execute(
                    f"INSERT INTO {self.table_name} (id, name, value) VALUES (%s, %s, %s)",
                    (i, f"row_{i}", i * 10)
                )
            conn.commit()
            self.results.append(f"✓ Inserted {self.test_rows} rows")
        except Exception as e:
            print(f"Error populating data: {e}")
            raise
        finally:
            cursor.close()
            conn.close()

    def test_isolation(self):
        """Run the isolation test."""
        print("Testing repeatable read isolation...")
        conn1 = self.connect()
        conn2 = self.connect()
        cur1 = conn1.cursor(dictionary=True)
        cur2 = conn2.cursor(dictionary=True)

        try:
            # Step 1: Both connections read the same data
            cur1.execute(f"SELECT id, name, value FROM {self.table_name} ORDER BY id")
            data1 = cur1.fetchall()
            cur2.execute(f"SELECT id, name, value FROM {self.table_name} ORDER BY id")
            data2 = cur2.fetchall()
            self.results.append(f"✓ Connection 1 read {len(data1)} rows")
            self.results.append(f"✓ Connection 2 read {len(data2)} rows")

            # Step 2: Start transactions
            cur1.execute("START TRANSACTION")
            cur2.execute("START TRANSACTION")
            self.results.append("✓ Started transactions on both connections")

            # Step 3: Connection 1 updates a row
            cur1.execute(f"UPDATE {self.table_name} SET value = 999 WHERE id = 5")
            self.results.append("✓ Connection 1 updated row with id=5 (value=999)")

            # Step 4: Connection 2 reads the same row (should see old value)
            cur2.execute(f"SELECT value FROM {self.table_name} WHERE id = 5")
            row = cur2.fetchone()
            if row:
                value = row['value']
                if value == 50:
                    self.results.append("✓ Connection 2 correctly sees old value (50) - Repeatable Read working!")
                else:
                    self.results.append(f"✗ Connection 2 sees new value ({value}) - Repeatable Read may not be working")

            # Step 5: Connection 1 commits
            conn1.commit()
            self.results.append("✓ Connection 1 committed transaction")

            # Step 6: Connection 2 reads again (should still see old value)
            cur2.execute(f"SELECT value FROM {self.table_name} WHERE id = 5")
            row = cur2.fetchone()
            if row:
                value = row['value']
                if value == 50:
                    self.results.append("✓ Connection 2 still sees old value (50) - Isolation maintained!")
                else:
                    self.results.append(f"✗ Connection 2 sees new value ({value}) - Isolation may be broken")

            # Step 7: Connection 2 commits and reads again
            conn2.commit()
            self.results.append("✓ Connection 2 committed transaction")
            cur2.execute(f"SELECT value FROM {self.table_name} WHERE id = 5")
            row = cur2.fetchone()
            if row:
                value = row['value']
                if value == 999:
                    self.results.append("✓ Connection 2 now sees updated value (999) - Transaction isolation working correctly!")
                else:
                    self.results.append(f"✗ Connection 2 sees unexpected value ({value})")

        except Exception as e:
            print(f"Error during isolation test: {e}")
            raise
        finally:
            cur1.close()
            cur2.close()
            conn1.close()
            conn2.close()

    def cleanup(self):
        """Clean up the test table."""
        print(f"Dropping test table: {self.table_name}")
        conn = self.connect()
        cursor = conn.cursor()
        
        try:
            cursor.execute(f"DROP TABLE IF EXISTS {self.table_name}")
            conn.commit()
            self.results.append(f"✓ Cleaned up test table: {self.table_name}")
        except Exception as e:
            print(f"Error cleaning up: {e}")
        finally:
            cursor.close()
            conn.close()

    def run(self):
        """Run the complete isolation test."""
        print("=== Python Isolation Test ===")
        print(f"Host: {self.host}")
        print(f"Database: {self.database}")
        print(f"Test rows: {self.test_rows}")
        print()
        
        try:
            self.setup()
            self.populate()
            self.test_isolation()
            self.cleanup()
            
            print("\n=== ISOLATION TEST SUMMARY ===")
            for result in self.results:
                print(result)
            
            success_count = sum(1 for r in self.results if "✓" in r)
            failure_count = sum(1 for r in self.results if "✗" in r)
            
            print(f"\nSuccessful steps: {success_count}")
            print(f"Failed steps: {failure_count}")
            
            if failure_count == 0:
                print("🎉 All isolation tests passed! Repeatable Read isolation is working correctly.")
            else:
                print("⚠️  Some isolation tests failed. Check the results above.")
                
        except Exception as e:
            print(f"Test failed with error: {e}")
            sys.exit(1)


def main():
    """Main function to run the isolation test."""
    import argparse
    import os
    import json
    
    def load_config_from_file(config_path):
        """Load configuration from JSON file."""
        try:
            with open(config_path, 'r') as f:
                config = json.load(f)
            return config
        except (FileNotFoundError, json.JSONDecodeError):
            return None
    
    parser = argparse.ArgumentParser(description="Run TiDB isolation test")
    parser.add_argument("--host", help="Database host:port (overrides config file and env vars)")
    parser.add_argument("--user", help="Database user (overrides config file and env vars)")
    parser.add_argument("--password", help="Database password (overrides config file and env vars)")
    parser.add_argument("--database", help="Database name (overrides config file and env vars)")
    parser.add_argument("--test-rows", type=int, help="Number of test rows (overrides config file and env vars)")
    parser.add_argument("--config", default="tidb_config.json", help="Configuration file path")
    
    args = parser.parse_args()
    
    # Load configuration from file first
    config = load_config_from_file(args.config)
    print(f"Debug: Config file '{args.config}' loaded: {config is not None}")
    if config:
        print(f"Debug: Config file host: {config.get('database', {}).get('host')}")
    
    # Priority: command line args > environment variables > config file > defaults
    host = (args.host or 
            os.environ.get("TIDB_HOST") or 
            (config.get("database", {}).get("host") if config else None) or 
            "localhost:4000")
    
    user = (args.user or 
            os.environ.get("TIDB_USER") or 
            (config.get("database", {}).get("username") if config else None) or 
            "root")
    
    password = (args.password or 
                os.environ.get("TIDB_PASSWORD") or 
                (config.get("database", {}).get("password") if config else None) or 
                "")
    
    database = (args.database or 
                os.environ.get("TIDB_DATABASE") or 
                (config.get("database", {}).get("database") if config else None) or 
                "test")
    
    test_rows = (args.test_rows or 
                 int(os.environ.get("TIDB_TEST_ROWS", "0")) or 
                 (config.get("test", {}).get("rows") if config else None) or 
                 10)
    
    print(f"Debug: Final configuration:")
    print(f"  Host: '{host}' (from: {'args' if args.host else 'env' if os.environ.get('TIDB_HOST') else 'config' if config else 'default'})")
    print(f"  User: '{user}' (from: {'args' if args.user else 'env' if os.environ.get('TIDB_USER') else 'config' if config else 'default'})")
    print(f"  Database: '{database}' (from: {'args' if args.database else 'env' if os.environ.get('TIDB_DATABASE') else 'config' if config else 'default'})")
    print(f"  Test rows: {test_rows} (from: {'args' if args.test_rows else 'env' if os.environ.get('TIDB_TEST_ROWS') else 'config' if config else 'default'})")
    print()
    
    # Get password if not provided
    if not password:
        import getpass
        password = getpass.getpass("Password: ")
    
    print(f"Configuration:")
    print(f"  Host: {host}")
    print(f"  User: {user}")
    print(f"  Database: {database}")
    print(f"  Test rows: {test_rows}")
    print()
    
    # Run the test
    test = IsolationTest(host, user, password, database, test_rows)
    test.run()


if __name__ == "__main__":
    main() 
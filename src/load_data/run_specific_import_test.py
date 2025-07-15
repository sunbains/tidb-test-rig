#!/usr/bin/env python3
"""
Custom script to run specific import test files.
This allows testing individual handlers without running the full suite.
"""

import sys
import os
import argparse
import importlib
from pathlib import Path

# Add the project root to Python path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

def run_specific_test(test_file, handler_name=None, real_db=False):
    """Run a specific test file with optional handler selection."""
    
    # Import the test module
    try:
        if test_file == "test_import_large.py":
            module = importlib.import_module("src.load_data.test_import_large")
            print(f"✅ Loaded {test_file}")
        elif test_file == "test_import.py":
            module = importlib.import_module("src.load_data.test_import")
            print(f"✅ Loaded {test_file}")
        elif test_file == "test_load_data.py":
            module = importlib.import_module("src.load_data.test_load_data")
            print(f"✅ Loaded {test_file}")
        elif test_file == "test_import_and_monitor.py":
            module = importlib.import_module("src.load_data.test_import_and_monitor")
            print(f"✅ Loaded {test_file}")
        else:
            print(f"❌ Unknown test file: {test_file}")
            return False
    except ImportError as e:
        print(f"❌ Failed to import {test_file}: {e}")
        return False
    
    # Get all handler classes from the module
    handler_classes = []
    for attr_name, attr_value in module.__dict__.items():
        if attr_name.endswith('Handler') and not attr_name.startswith('_'):
            try:
                if hasattr(attr_value, '__bases__'):
                    base_names = [base.__name__ for base in attr_value.__bases__]
                    if 'PyStateHandler' in base_names:
                        handler_classes.append(attr_name)
            except:
                pass
    
    print(f"📋 Available handlers in {test_file}:")
    for i, handler in enumerate(handler_classes, 1):
        print(f"  {i}. {handler}")
    
    # Run specific handler or all handlers
    if handler_name:
        if handler_name in handler_classes:
            handlers_to_run = [handler_name]
        else:
            print(f"❌ Handler '{handler_name}' not found in {test_file}")
            return False
    else:
        handlers_to_run = handler_classes
    
    print(f"\n🚀 Running {len(handlers_to_run)} handler(s)...")
    
    # Run each handler
    for handler_class_name in handlers_to_run:
        try:
            print(f"\n--- Testing {handler_class_name} ---")
            handler_class = getattr(module, handler_class_name)
            handler = handler_class()
            
            # Create a mock context for testing
            from src.common.test_rig_python import PyStateContext, PyConnection
            
            if real_db:
                from src.common.test_rig_python import RealPyConnection
                # Get connection parameters from environment
                host = os.environ.get('TIDB_HOST', 'localhost:4000')
                username = os.environ.get('TIDB_USER', 'root')
                password = os.environ.get('TIDB_PASSWORD', '')
                database = os.environ.get('TIDB_DATABASE', 'test')
                
                try:
                    connection = RealPyConnection(
                        connection_info={'id': 'test_conn'}, 
                        connection_id='test_conn',
                        host=host,
                        username=username,
                        password=password,
                        database=database
                    )
                    print(f"✅ Connected to TiDB at {host}")
                except Exception as e:
                    print(f"⚠️  Could not connect to TiDB: {e}")
                    print("   Using mock connection instead")
                    connection = PyConnection(connection_info={'id': 'test_conn'}, connection_id='test_conn')
            else:
                connection = PyConnection(connection_info={'id': 'test_conn'}, connection_id='test_conn')
            
            context = PyStateContext(
                host='localhost',
                port=4000,
                username='root',
                password='',
                database='test',
                connection=connection
            )
            
            # Test the handler
            print(f"🔧 Testing {handler_class_name}.enter()...")
            enter_result = handler.enter(context)
            print(f"   Enter result: {enter_result}")
            
            print(f"🔧 Testing {handler_class_name}.execute()...")
            execute_result = handler.execute(context)
            print(f"   Execute result: {execute_result}")
            
            print(f"🔧 Testing {handler_class_name}.exit()...")
            handler.exit(context)
            print(f"   Exit completed")
            
            print(f"✅ {handler_class_name} completed successfully")
            
        except Exception as e:
            print(f"❌ Failed to test {handler_class_name}: {e}")
            import traceback
            traceback.print_exc()
            return False
    
    print(f"\n✅ All handlers in {test_file} completed")
    return True

def main():
    parser = argparse.ArgumentParser(description="Run specific import test files")
    parser.add_argument("test_file", nargs="?", help="Test file to run (e.g., test_import_large.py)")
    parser.add_argument("--handler", help="Specific handler to test (optional)")
    parser.add_argument("--real-db", action="store_true", help="Use real database connection")
    parser.add_argument("--list", action="store_true", help="List available test files")
    
    args = parser.parse_args()
    
    if args.list:
        print("📋 Available test files:")
        print("  - test_import_large.py (Large dataset performance tests)")
        print("  - test_import.py (Modern IMPORT INTO tests)")
        print("  - test_load_data.py (Traditional LOAD DATA tests)")
        print("  - test_import_and_monitor.py (Multi-connection tests)")
        return
    
    if not args.test_file:
        print("❌ You must specify a test_file unless using --list. Use --help for usage.")
        return
    
    # Validate test file
    test_files = [
        "test_import_large.py",
        "test_import.py", 
        "test_load_data.py",
        "test_import_and_monitor.py"
    ]
    
    if args.test_file not in test_files:
        print(f"❌ Unknown test file: {args.test_file}")
        print("Use --list to see available test files")
        return
    
    # Run the test
    success = run_specific_test(args.test_file, args.handler, args.real_db)
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main() 
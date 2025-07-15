# TiDB Import Test Suite

This directory contains comprehensive tests for TiDB's IMPORT functionality, including both traditional LOAD DATA and modern IMPORT INTO statements.

## Directory Structure

```
src/load_data/
├── README.md                           # This file
├── run_specific_import_test.py         # Custom test runner for individual files
├── create_import.py                    # Data generator for import tests
├── test_import.py                      # Modern IMPORT INTO tests
├── test_import_large.py                # Large dataset performance tests
├── test_import_and_monitor.py          # Multi-connection monitoring tests
└── test_load_data.py                   # Traditional LOAD DATA tests
```

## 🧪 Test Files

### 1. `test_load_data.py` - Traditional LOAD DATA Tests
Comprehensive tests for TiDB's traditional LOAD DATA functionality:

**Test Handlers:**
- `BasicLoadDataHandler` - Basic LOAD DATA functionality
- `LoadDataWithNullsHandler` - NULL values and defaults
- `LoadDataDuplicateKeysHandler` - Duplicate key handling (REPLACE)
- `LoadDataColumnMappingHandler` - Column mapping and reordering
- `LoadDataCharsetHandler` - Character set handling (UTF-8)
- `LoadDataPartitionedTableHandler` - Partitioned table imports
- `LoadDataConstraintsHandler` - Constraint handling
- `LoadDataAutoIncrementHandler` - Auto-increment columns
- `LoadDataErrorHandlingHandler` - Error handling and recovery

**Features Tested:**
- CSV/TSV format support
- Field and line terminators
- Character set specifications
- Column mapping and reordering
- Default value handling
- Duplicate key resolution
- Partitioned table imports
- Constraint validation
- Auto-increment behavior
- Error handling and recovery

### 2. `test_import.py` - Modern IMPORT INTO Tests
Tests for TiDB's modern IMPORT INTO functionality:

**Test Handlers:**
- `BasicImportIntoHandler` - Basic IMPORT INTO functionality
- `ImportIntoWithNullsHandler` - NULL values and defaults
- `ImportIntoDuplicateKeysHandler` - Duplicate key handling
- `ImportIntoColumnMappingHandler` - Column mapping
- `ImportIntoCharsetHandler` - Character set handling
- `ImportIntoPartitionedTableHandler` - Partitioned tables
- `ImportIntoConstraintsHandler` - Constraint handling
- `ImportIntoAutoIncrementHandler` - Auto-increment columns
- `ImportIntoErrorHandlingHandler` - Error handling
- `ImportIntoTSVHandler` - TSV format support
- `ImportIntoWithOptionsHandler` - Advanced import options

**Features Tested:**
- Modern IMPORT INTO syntax
- Multiple data formats (CSV, TSV)
- Advanced column mapping
- Character set specifications
- Partitioned table support
- Constraint validation
- Auto-increment handling
- Error recovery mechanisms
- Quoted fields and escape characters

### 3. `test_import_large.py` - Large Dataset Performance Tests
Performance and scalability tests using generated datasets:

**Test Handlers:**
- `LargeDatasetImportHandler` - 100k rows simple data
- `ComplexDatasetImportHandler` - 10k rows complex data
- `TSVImportHandler` - 5k rows TSV format
- `PartitionedLargeImportHandler` - 50k rows partitioned table
- `DuplicateKeyLargeImportHandler` - 1k rows with duplicates
- `PerformanceImportHandler` - 200k rows performance test
- `ErrorHandlingLargeImportHandler` - Error handling with invalid data

**Features Tested:**
- Large dataset performance
- Complex data type handling
- Partitioned table scalability
- Duplicate key resolution
- Performance benchmarking
- Error resilience
- Memory efficiency

### 4. `test_import_multi_connection.py` - Multi-Connection Import Tests
Multi-connection tests with real-time import job monitoring:

**Test Handlers:**
- `MultiConnectionImportHandler` - Single import with monitoring
- `ConcurrentImportHandler` - Multiple concurrent imports with monitoring
- `LargeImportWithMonitoringHandler` - Large import with detailed monitoring

**Features Tested:**
- Multi-connection coordination
- Real-time import job monitoring
- Concurrent import operations
- Import job status tracking
- Progress monitoring and reporting
- Thread-safe monitoring
- Performance metrics across connections

**Multi-Connection Architecture:**
- **Import Connection**: Performs the actual IMPORT operations
- **Monitor Connection**: Tracks import job status in real-time
- **Threading**: Separate monitoring thread for non-blocking status updates
- **Real-time Updates**: Continuous monitoring with configurable intervals
- **Progress Tracking**: Detailed progress reporting with timing information

## 🛠️ Test Data Generator

### `create_import.py` - Standalone Data Generator
A comprehensive test data generator for creating realistic datasets:

**Usage:**
```bash
# Basic usage (100k rows CSV)
python create_import.py

# Generate 1M rows for performance testing
python create_import.py --rows 1000000

# Generate TSV format
python create_import.py --format tsv --rows 50000

# Generate simple format (id,name,age)
python create_import.py --simple --rows 1000

# Specify output file
python create_import.py --output my_test_data.csv --rows 10000

# Reproducible data generation
python create_import.py --seed 12345 --rows 1000
```

**Data Types Generated:**
- **Simple Format**: id, name, age
- **Complex Format**: 14 columns including emails, phones, dates, salaries, etc.

**Features:**
- Configurable row counts (default: 100k)
- Multiple formats (CSV, TSV)
- Realistic data generation
- Performance tracking
- Memory-efficient streaming
- Reproducible with seeds

## 🚀 Running the Tests

### Using Makefile
```bash
# Run all import tests
make run-import-tests

# Run with real database connection
make run-import-tests REAL_DB=1

# Run with SQL statement logging
make run-import-tests REAL_DB=1 SHOW_SQL=1

# Run with verbose output
make run-import-tests REAL_DB=1 SHOW_SQL=1 --output-level verbose
```

### Direct Execution
```bash
# Run specific test files
cd src/load_data
python -m pytest test_load_data.py
python -m pytest test_import.py
python -m pytest test_import_large.py
```

## 📋 **Practical Examples**

### **Basic Import Testing**

#### **1. Test Data Generation (No Database Required)**
```bash
# Generate test data for different scenarios
cd src/load_data

# Generate 1k rows for quick testing
python create_import.py --rows 1000 --simple --output quick_test.csv

# Generate 10k rows for medium testing
python create_import.py --rows 10000 --output medium_test.csv

# Generate 100k rows for performance testing
python create_import.py --rows 100000 --simple --output large_test.csv

# Generate TSV format data
python create_import.py --rows 5000 --format tsv --output test_data.tsv

# Generate with specific filename
python create_import.py --rows 50000 --output my_custom_data.csv

# Generate reproducible data (same seed)
python create_import.py --rows 1000 --seed 12345 --output reproducible.csv
```

#### **2. Traditional LOAD DATA Tests**
```bash
# Run all LOAD DATA tests
make run-import-tests REAL_DB=1 SHOW_SQL=1

# Test specific LOAD DATA scenarios:
# - BasicLoadDataHandler: Basic CSV import
# - LoadDataWithNullsHandler: NULL value handling
# - LoadDataDuplicateKeysHandler: Duplicate key resolution
# - LoadDataColumnMappingHandler: Column reordering
# - LoadDataCharsetHandler: UTF-8 character handling
# - LoadDataPartitionedTableHandler: Partitioned table imports
# - LoadDataConstraintsHandler: Constraint validation
# - LoadDataAutoIncrementHandler: Auto-increment behavior
# - LoadDataErrorHandlingHandler: Error recovery
```

#### **3. Modern IMPORT INTO Tests**
```bash
# Run all IMPORT INTO tests
make run-import-tests REAL_DB=1 SHOW_SQL=1

# Test specific IMPORT INTO scenarios:
# - BasicImportIntoHandler: Basic IMPORT INTO functionality
# - ImportIntoWithNullsHandler: NULL values and defaults
# - ImportIntoDuplicateKeysHandler: ON DUPLICATE KEY UPDATE
# - ImportIntoColumnMappingHandler: Column mapping
# - ImportIntoCharsetHandler: Character set handling
# - ImportIntoPartitionedTableHandler: Partitioned tables
# - ImportIntoConstraintsHandler: Constraint handling
# - ImportIntoAutoIncrementHandler: Auto-increment columns
# - ImportIntoErrorHandlingHandler: Error handling
# - ImportIntoTSVHandler: TSV format support
# - ImportIntoWithOptionsHandler: Advanced import options
```

### **Large Dataset Testing**

#### **4. Large Import Performance Tests**
```bash
# Run large import tests with real database
TIDB_HOST=your-tidb-host:4000 TIDB_USER=your-user TIDB_PASSWORD=your-password make run-import-tests REAL_DB=1 SHOW_SQL=1

# Test specific large import scenarios:

# LargeDatasetImportHandler (100k rows, simple data)
# - Generates: 100,000 rows of simple data (id, name, age)
# - Use case: Basic performance testing
# - Expected output: "✅ Large import successful: 100,000 rows in X.XX seconds"

# ComplexDatasetImportHandler (10k rows, complex data)
# - Generates: 10,000 rows with 14 columns (emails, phones, dates, etc.)
# - Use case: Real-world data import testing
# - Expected output: "✅ Complex import successful: 10,000 rows in X.XX seconds"

# TSVImportHandler (5k rows, TSV format)
# - Generates: 5,000 rows in TSV format
# - Use case: Alternative format testing
# - Expected output: "✅ TSV import successful: 5,000 rows in X.XX seconds"

# PartitionedLargeImportHandler (50k rows, partitioned table)
# - Generates: 50,000 rows for range-partitioned table
# - Use case: Partitioned table performance testing
# - Expected output: "✅ Partitioned import successful: 50,000 rows in X.XX seconds"

# PerformanceImportHandler (200k rows, performance test)
# - Generates: 200,000 rows for performance benchmarking
# - Use case: Scalability and performance testing
# - Expected output: "✅ Performance import successful: 200,000 rows in X.XX seconds (X rows/sec)"

# DuplicateKeyLargeImportHandler (1k rows, duplicate handling)
# - Generates: 1,000 rows with potential duplicates
# - Use case: Conflict resolution testing
# - Expected output: "✅ Duplicate key import successful: X rows in X.XX seconds"

# ErrorHandlingLargeImportHandler (1k rows, error scenarios)
# - Generates: 1,000 rows + intentionally invalid rows
# - Use case: Robustness testing
# - Expected output: "✅ Error handling import successful: X rows in X.XX seconds"
```

#### **5. Large Dataset Data Generation Examples**
```bash
# Generate data for different large import scenarios
cd src/load_data

# For LargeDatasetImportHandler (100k simple rows)
python create_import.py --rows 100000 --simple --output large_simple.csv

# For ComplexDatasetImportHandler (10k complex rows)
python create_import.py --rows 10000 --output large_complex.csv

# For TSVImportHandler (5k TSV rows)
python create_import.py --rows 5000 --format tsv --output large_tsv.tsv

# For PartitionedLargeImportHandler (50k simple rows)
python create_import.py --rows 50000 --simple --output large_partitioned.csv

# For PerformanceImportHandler (200k complex rows)
python create_import.py --rows 200000 --output large_performance.csv

# For DuplicateKeyLargeImportHandler (1k simple rows)
python create_import.py --rows 1000 --simple --output large_duplicate.csv

# For ErrorHandlingLargeImportHandler (1k simple rows)
python create_import.py --rows 1000 --simple --output large_error.csv
```

### **Multi-Connection Testing**

#### **6. Multi-Connection Import Tests**
```bash
# Run multi-connection tests
make run-import-tests REAL_DB=1 SHOW_SQL=1

# Test specific multi-connection scenarios:

# MultiConnectionImportHandler
# - Single import with real-time monitoring
# - 50k rows with complex data
# - Real-time import job status updates
# - Expected output: Real-time monitoring updates + "✅ Multi-connection import successful"

# ConcurrentImportHandler
# - Multiple concurrent imports with monitoring
# - 3 concurrent imports (10k rows each)
# - Shared monitoring across all imports
# - Expected output: Concurrent monitoring + "✅ Concurrent import successful: 30,000 total rows"

# LargeImportWithMonitoringHandler
# - Large-scale import with detailed progress tracking
# - 100k rows with 14-column complex schema
# - Progress percentage estimation
# - Expected output: Detailed progress updates + "✅ Large monitored import successful: 100,000 rows"
```

#### **7. Multi-Connection Monitoring Examples**
```bash
# Example monitoring output during multi-connection tests:
# 🔍 Started import job monitoring thread
# 📊 Monitoring 1 active import job(s)...
#    Job 1: global-sorting | running | 25000 rows | 50.2 MiB
# 📊 Monitoring 1 active import job(s)...
#    Job 1: writing | running | 50000 rows | 50.2 MiB
# ✅ Multi-connection import successful: 50,000 rows in 45.23 seconds
# 🔍 Stopped import job monitoring

# For concurrent imports:
# 🔍 Started concurrent import monitoring thread
# 📊 Concurrent monitoring: 3 active import job(s)
#    Job 1 (concurrent_import_test_0): global-sorting | running | 5000 rows
#    Job 2 (concurrent_import_test_1): writing | running | 8000 rows
#    Job 3 (concurrent_import_test_2): completed | success | 10000 rows
# ✅ Concurrent import successful: 30,000 total rows in 67.89 seconds
```

### **Environment Configuration**

#### **8. Setting Up TiDB Connection**
```bash
# Set TiDB connection environment variables
export TIDB_HOST=your-tidb-host:4000
export TIDB_USER=your-username
export TIDB_PASSWORD=your-password
export TIDB_DATABASE=test

# Enable SQL logging and real database mode
export SHOW_SQL=1
export REAL_DB=1

# Run tests with environment variables
make run-import-tests
```

#### **9. Different TiDB Connection Examples**
```bash
# Local TiDB instance
TIDB_HOST=localhost:4000 TIDB_USER=root TIDB_PASSWORD= make run-import-tests REAL_DB=1

# TiDB Cloud connection
TIDB_HOST=your-cluster.tidbcloud.com:4000 TIDB_USER=your-user TIDB_PASSWORD=your-password make run-import-tests REAL_DB=1

# Custom database
TIDB_HOST=localhost:4000 TIDB_USER=root TIDB_DATABASE=my_test_db make run-import-tests REAL_DB=1
```

### **Performance Testing Examples**

#### **10. Performance Benchmarking**
```bash
# Test different dataset sizes for performance comparison
cd src/load_data

# Small dataset (1k rows) - Quick testing
python create_import.py --rows 1000 --simple --output perf_small.csv
# Expected: ~1-5 seconds

# Medium dataset (10k rows) - Standard testing
python create_import.py --rows 10000 --output perf_medium.csv
# Expected: ~10-30 seconds

# Large dataset (100k rows) - Performance testing
python create_import.py --rows 100000 --simple --output perf_large.csv
# Expected: ~1-5 minutes

# Very large dataset (1M rows) - Stress testing
python create_import.py --rows 1000000 --simple --output perf_xlarge.csv
# Expected: ~10-30 minutes

# Run performance tests
TIDB_HOST=localhost:4000 make run-import-tests REAL_DB=1 SHOW_SQL=1
```

#### **11. Format Comparison Testing**
```bash
# Compare CSV vs TSV performance
cd src/load_data

# Generate CSV data
python create_import.py --rows 50000 --output csv_test.csv

# Generate TSV data
python create_import.py --rows 50000 --format tsv --output tsv_test.tsv

# Run format comparison tests
make run-import-tests REAL_DB=1 SHOW_SQL=1
```

### **Error Testing Examples**

#### **12. Error Handling Tests**
```bash
# Test error handling scenarios
cd src/load_data

# Generate data with errors
python create_import.py --rows 1000 --simple --output error_test.csv

# Manually add invalid rows to test error handling
echo "invalid,data,here" >> error_test.csv
echo "999999,Valid Name,25" >> error_test.csv

# Run error handling tests
make run-import-tests REAL_DB=1 SHOW_SQL=1
```

### **Debugging Examples**

#### **13. Debug Mode Testing**
```bash
# Enable verbose logging for debugging
RUST_LOG=debug make run-import-tests REAL_DB=1 SHOW_SQL=1 --output-level verbose

# Test individual handlers
python -c "
import sys; sys.path.append('src/load_data')
from test_import_large import LargeDatasetImportHandler
handler = LargeDatasetImportHandler()
print('Handler created successfully')
"

# Check generated data
cd src/load_data
python create_import.py --rows 10 --simple --output debug_test.csv
head -5 debug_test.csv
wc -l debug_test.csv
rm debug_test.csv
```

### **Production Testing Examples**

#### **14. Production-like Scenarios**
```bash
# Large production dataset simulation
cd src/load_data

# Generate production-like data (1M rows)
python create_import.py --rows 1000000 --output production_data.csv

# Test with production settings
TIDB_HOST=prod-tidb.example.com:4000 \
TIDB_USER=prod_user \
TIDB_PASSWORD=prod_password \
TIDB_DATABASE=prod_db \
make run-import-tests REAL_DB=1 SHOW_SQL=1

# Monitor import progress in real-time
# Expected: Detailed progress updates and performance metrics
```

#### **15. Load Testing Examples**
```bash
# Concurrent load testing
cd src/load_data

# Generate multiple datasets for concurrent testing
python create_import.py --rows 10000 --output load_test_1.csv
python create_import.py --rows 10000 --output load_test_2.csv
python create_import.py --rows 10000 --output load_test_3.csv

# Run concurrent import tests
make run-import-tests REAL_DB=1 SHOW_SQL=1

# Expected: Multiple concurrent imports with shared monitoring
```

## 📊 Test Coverage

### Import Functionality
- ✅ Basic import operations
- ✅ Multiple data formats (CSV, TSV)
- ✅ Character set handling
- ✅ Column mapping and reordering
- ✅ NULL value handling
- ✅ Default value processing
- ✅ Duplicate key resolution
- ✅ Constraint validation
- ✅ Auto-increment behavior
- ✅ Error handling and recovery
- ✅ Partitioned table support

### Performance Testing
- ✅ Small datasets (1k-10k rows)
- ✅ Medium datasets (50k-100k rows)
- ✅ Large datasets (200k+ rows)
- ✅ Performance metrics (rows/sec)
- ✅ Memory efficiency
- ✅ Scalability testing
- ✅ Multi-connection coordination
- ✅ Real-time monitoring
- ✅ Concurrent operations

### Data Validation
- ✅ Row count verification
- ✅ Data integrity checks
- ✅ Type validation
- ✅ Constraint enforcement
- ✅ Error recovery testing

## 🔧 Configuration

### Environment Variables
- `REAL_DB=1` - Use real TiDB database connection
- `SHOW_SQL=1` - Display SQL statements during execution
- `TIDB_HOST` - TiDB server hostname
- `TIDB_PORT` - TiDB server port
- `TIDB_USER` - Database username
- `TIDB_PASSWORD` - Database password

### Test Parameters
- Row counts: 1k to 1M+ rows
- Data formats: CSV, TSV
- Data complexity: Simple to complex schemas
- Performance targets: Configurable benchmarks

## 📈 Performance Benchmarks

The test suite includes performance benchmarks for:
- **Import Speed**: Rows per second
- **Memory Usage**: Efficient data streaming
- **Scalability**: Large dataset handling
- **Error Recovery**: Robust error handling
- **Format Support**: CSV vs TSV performance

## 🐛 Troubleshooting

### Common Issues
1. **Connection Errors**: Verify TiDB host and credentials
2. **Permission Errors**: Ensure write access for temp files
3. **Memory Issues**: Reduce row count for large datasets
4. **Timeout Errors**: Increase timeout for large imports

### Debug Mode
```bash
# Enable verbose logging
make run-import-tests REAL_DB=1 SHOW_SQL=1 --output-level verbose

# Run individual tests
python -m pytest test_import.py::BasicImportIntoHandler
```

## 🤝 Contributing

When adding new tests:
1. Follow the existing handler pattern
2. Include proper cleanup in `exit()` method
3. Add comprehensive error handling
4. Document new features in this README
5. Test with both small and large datasets

## 📝 Notes

- All tests use temporary files that are automatically cleaned up
- Large dataset tests may take significant time to complete
- Performance metrics are logged for benchmarking
- Tests are designed to be idempotent and safe to run multiple times
- The test suite supports both development and production environments


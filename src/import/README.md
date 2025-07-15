# TiDB IMPORT Test Suite

This directory contains comprehensive tests for TiDB import functionality, including both traditional LOAD DATA operations and modern IMPORT INTO statements. The test suite provides extensive coverage of import scenarios, performance testing, and data validation.

## 📁 File Structure

```
src/import/
├── README.md                    # This documentation file
├── test_load_data.py           # Traditional LOAD DATA tests
├── test_import.py              # Modern IMPORT INTO tests
├── test_import_large.py        # Large dataset performance tests
├── create_import.py            # Test data generator
├── lib.rs                      # Rust module definition
├── Cargo.toml                  # Rust dependencies
└── __init__.py                 # Python package initialization
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
cd src/import
python -m pytest test_load_data.py
python -m pytest test_import.py
python -m pytest test_import_large.py
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


# TiDB Multi-Connection Test Tool Makefile

# Environment variables for configuration
RUST_LOG ?= info
TIDB_HOST ?= localhost:4000
TIDB_USER ?= root
TIDB_DATABASE ?= test
TIDB_PASSWORD ?=
LOG_LEVEL ?= info
LOG_FILE ?= false
LOG_FILE_PATH ?= logs/tidb_connect.log
MONITOR_DURATION ?= 60

.PHONY: help build db_tests clean db_tests build-test run-simple run-advanced check format lint \
	run-cli-test run-logging-test \
	run-basic-test run-basic-debug-test run-basic-verbose-test run-job-monitor-test \
	run-python-tests run-all-python-tests run-ddl-tests run-scale-tests run-txn-tests \
	run-python-suite

# Default target
help:
	@echo "TiDB Multi-Connection DB Tests Tool"
	@echo ""
	@echo "Environment Variables:"
	@echo "  RUST_LOG          - Log level (debug, info, warn, error) [default: info]"
	@echo "  TIDB_HOST         - TiDB host and port [default: localhost:4000]"
	@echo "  TIDB_USER         - Database username [default: root]"
	@echo "  TIDB_DATABASE     - Database name [default: test]"
	@echo "  TIDB_PASSWORD     - Database password (if not set, will prompt)"
	@echo "  LOG_LEVEL         - Log level for db_tests [default: info]"
	@echo "  LOG_FILE          - Enable file logging [default: false]"
	@echo "  LOG_FILE_PATH     - Log file path [default: logs/tidb_connect.log]"
	@echo "  MONITOR_DURATION - Duration for job monitoring db_tests in seconds [default: 60]"
	@echo ""
	@echo "Available targets:"
	@echo "  build                  - Build the main application"
	@echo "  db_tests               - Build all db_tests"
	@echo "  build-test         - Build all db_tests"
	@echo "  run-basic-test     - Run basic connection db_tests with env vars"
	@echo "  run-basic-debug-test - Run basic db_tests with debug logging"
	@echo "  run-basic-verbose-test - Run basic db_tests with verbose output"
	@echo "  run-advanced           - Run advanced multi-connection db_tests"
	@echo "  run-cli-test       - Run CLI db_tests"
	@echo "  run-logging-test   - Run logging db_tests"
	@echo "  run-job-monitor-test - Run job monitoring db_tests"
	@echo ""
	@echo "Python Test Suite Targets:"
	@echo "  run-python-tests       - Run all Python test suites (DDL, Scale, Txn)"
	@echo "  run-all-python-tests   - Alias for run-python-tests"
	@echo "  run-ddl-tests          - Run DDL Python test suite only"
	@echo "  run-scale-tests        - Run Scale Python test suite only"
	@echo "  run-txn-tests          - Run Txn Python test suite only"
	@echo "  run-python-suite       - Run a specific Python test suite (use SUITE=name)"
	@echo ""
	@echo "Utility targets:"
	@echo "  check                  - Check if code compiles without building"
	@echo "  format                 - Format code with rustfmt"
	@echo "  lint                   - Run clippy linter"
	@echo "  help                   - Show this help message"
	@echo ""
	@echo "Examples:"
	@echo "  RUST_LOG=debug make run-basic-test"
	@echo "  TIDB_HOST=myhost:4000 TIDB_USER=admin make run-basic-test"
	@echo "  LOG_LEVEL=debug LOG_FILE=true make run-logging-test"
	@echo "  make run-python-tests                    # Run all Python test suites"
	@echo "  make run-ddl-tests                       # Run only DDL tests"
	@echo "  make run-python-suite SUITE=txn          # Run specific suite"
	@echo "  make run-python-suite SUITE=scale        # Run specific suite"

# Build the main application
build:
	cargo build --release

# Build all db_tests
db_tests: build-test

build-test:
	cargo test --no-run

# Run specific db_tests with environment variables
run-simple:
	RUST_LOG=$(RUST_LOG) cargo run --bin simple_multi_connection --

run-advanced:
	RUST_LOG=$(RUST_LOG) cargo run --bin multi_connection --

run-basic-test:
	RUST_LOG=$(RUST_LOG) cargo run --bin basic -- -H $(TIDB_HOST) -u $(TIDB_USER) -d $(TIDB_DATABASE) $(if $(TIDB_PASSWORD),--password $(TIDB_PASSWORD),--no-password-prompt)

run-basic-debug-test:
	RUST_LOG=debug cargo run --bin basic -- -H $(TIDB_HOST) -u $(TIDB_USER) -d $(TIDB_DATABASE) $(if $(TIDB_PASSWORD),--password $(TIDB_PASSWORD),--no-password-prompt)

run-basic-verbose-test:
	RUST_LOG=debug cargo run --bin basic -- -H $(TIDB_HOST) -u $(TIDB_USER) -d $(TIDB_DATABASE) $(if $(TIDB_PASSWORD),--password $(TIDB_PASSWORD),--no-password-prompt) --verbose

run-cli-test:
	RUST_LOG=$(RUST_LOG) cargo run --bin cli --features="isolation_test" -- $(ARGS)

run-logging-test:
	RUST_LOG=$(RUST_LOG) cargo run --bin logging -- --log-level $(LOG_LEVEL) $(if $(LOG_FILE),--log-file --log-file-path $(LOG_FILE_PATH),)

run-job-monitor-test:
	RUST_LOG=$(RUST_LOG) cargo run --bin job_monitor --features="import_jobs" -- -H $(TIDB_HOST) -u $(TIDB_USER) -d $(TIDB_DATABASE) $(if $(TIDB_PASSWORD),--password $(TIDB_PASSWORD),--no-password-prompt) --monitor-duration $(MONITOR_DURATION)

# Python Test Suite Targets
run-python-tests: run-all-python-tests

run-all-python-tests:
	@echo "Running all Python test suites..."
	RUST_LOG=$(RUST_LOG) cargo run --bin python_test_runner --features="python_plugins" -- --all $(if $(SHOW_OUTPUT),--show-output)

run-ddl-tests:
	@echo "Running DDL Python test suite..."
	RUST_LOG=$(RUST_LOG) cargo run --bin python_test_runner --features="python_plugins" -- --suite ddl $(if $(SHOW_OUTPUT),--show-output)

run-scale-tests:
	@echo "Running Scale Python test suite..."
	RUST_LOG=$(RUST_LOG) cargo run --bin python_test_runner --features="python_plugins" -- --suite scale $(if $(SHOW_OUTPUT),--show-output)

run-txn-tests:
	@echo "Running Txn Python test suite..."
	RUST_LOG=$(RUST_LOG) cargo run --bin python_test_runner --features="python_plugins" -- --suite txn $(if $(SHOW_OUTPUT),--show-output)

run-python-suite:
	@if [ -z "$(SUITE)" ]; then \
		echo "Error: SUITE variable is required. Usage: make run-python-suite SUITE=<suite_name>"; \
		echo "Available suites: ddl, scale, txn"; \
		exit 1; \
	fi
	@echo "Running Python test suite: $(SUITE)"
	RUST_LOG=$(RUST_LOG) cargo run --bin python_test_runner --features="python_plugins" -- --suite $(SUITE) $(if $(SHOW_OUTPUT),--show-output)

# Check if code compiles
check:
	cargo check
	cargo check --tests

# Format code
format:
	cargo fmt

# Run linter
lint:
	cargo clippy
	cargo clippy --tests 

lint-fix:
	cargo clippy --fix


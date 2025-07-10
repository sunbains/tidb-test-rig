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

.PHONY: help build db_tests clean db_tests build-db-tests run-simple run-advanced check format lint \
	run-simple-connection run-isolation-db-tests run-cli-db-tests run-logging-db-tests \
	run-basic-db-tests run-basic-debug-db-tests run-basic-verbose-db-tests run-job-monitor-db-tests

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
	@echo "  build-db-tests         - Build all db_tests"
	@echo "  run-basic-db-tests     - Run basic connection db_tests with env vars"
	@echo "  run-basic-debug-db-tests - Run basic db_tests with debug logging"
	@echo "  run-basic-verbose-db-tests - Run basic db_tests with verbose output"
	@echo "  run-simple             - Run simple multi-connection db_tests"
	@echo "  run-advanced           - Run advanced multi-connection db_tests"
	@echo "  run-simple-connection  - Run simple connection db_tests"
	@echo "  run-isolation-db-tests - Run isolation db_tests"
	@echo "  run-cli-db-tests       - Run CLI db_tests"
	@echo "  run-logging-db-tests   - Run logging db_tests"
	@echo "  run-job-monitor-db-tests - Run job monitoring db_tests"
	@echo "  check                  - Check if code compiles without building"
	@echo "  format                 - Format code with rustfmt"
	@echo "  lint                   - Run clippy linter"
	@echo "  help                   - Show this help message"
	@echo ""
	@echo "Examples:"
	@echo "  RUST_LOG=debug make run-basic-db-tests"
	@echo "  TIDB_HOST=myhost:4000 TIDB_USER=admin make run-basic-db-tests"
	@echo "  LOG_LEVEL=debug LOG_FILE=true make run-logging-db-tests"

# Build the main application
build:
	cargo build --release

# Build all db_tests
db_tests: build-db-tests

build-db-tests:
	cargo test --no-run

# Run specific db_tests with environment variables
run-simple:
	RUST_LOG=$(RUST_LOG) cargo run --bin simple_multi_connection --

run-advanced:
	RUST_LOG=$(RUST_LOG) cargo run --bin multi_connection --

run-basic-db-tests:
	RUST_LOG=$(RUST_LOG) cargo run --bin basic -- -H $(TIDB_HOST) -u $(TIDB_USER) -d $(TIDB_DATABASE) $(if $(TIDB_PASSWORD),--password $(TIDB_PASSWORD),--no-password-prompt)

run-basic-debug-db-tests:
	RUST_LOG=debug cargo run --bin basic -- -H $(TIDB_HOST) -u $(TIDB_USER) -d $(TIDB_DATABASE) $(if $(TIDB_PASSWORD),--password $(TIDB_PASSWORD),--no-password-prompt)

run-basic-verbose-db-tests:
	RUST_LOG=debug cargo run --bin basic -- -H $(TIDB_HOST) -u $(TIDB_USER) -d $(TIDB_DATABASE) $(if $(TIDB_PASSWORD),--password $(TIDB_PASSWORD),--no-password-prompt) --verbose

run-simple-connection:
	RUST_LOG=$(RUST_LOG) cargo run --bin simple_connection -- -H $(TIDB_HOST) -u $(TIDB_USER) -d $(TIDB_DATABASE) $(if $(TIDB_PASSWORD),--password $(TIDB_PASSWORD),--no-password-prompt)

run-isolation-db-tests:
	RUST_LOG=$(RUST_LOG) cargo run --bin isolation -- -H $(TIDB_HOST) -u $(TIDB_USER) -d $(TIDB_DATABASE) $(if $(TIDB_PASSWORD),--password $(TIDB_PASSWORD),--no-password-prompt)

run-cli-db-tests:
	RUST_LOG=$(RUST_LOG) cargo run --bin cli --features="isolation_test" -- $(ARGS)

run-logging-db-tests:
	RUST_LOG=$(RUST_LOG) cargo run --bin logging -- --log-level $(LOG_LEVEL) $(if $(LOG_FILE),--log-file --log-file-path $(LOG_FILE_PATH),)

run-job-monitor-db-tests:
	RUST_LOG=$(RUST_LOG) cargo run --bin job_monitor --features="import_jobs" -- -H $(TIDB_HOST) -u $(TIDB_USER) -d $(TIDB_DATABASE) $(if $(TIDB_PASSWORD),--password $(TIDB_PASSWORD),--no-password-prompt) --monitor-duration $(MONITOR_DURATION)

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
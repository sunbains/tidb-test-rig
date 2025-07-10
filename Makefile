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
	run-simple-connection run-isolation-test run-cli-test run-logging-test \
	run-basic-test run-basic-debug-test run-basic-verbose-test run-job-monitor-test

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
	@echo "  run-simple             - Run simple multi-connection db_tests"
	@echo "  run-advanced           - Run advanced multi-connection db_tests"
	@echo "  run-simple-connection  - Run simple connection db_tests"
	@echo "  run-isolation-test - Run isolation db_tests"
	@echo "  run-cli-test       - Run CLI db_tests"
	@echo "  run-logging-test   - Run logging db_tests"
	@echo "  run-job-monitor-test - Run job monitoring db_tests"
	@echo "  check                  - Check if code compiles without building"
	@echo "  format                 - Format code with rustfmt"
	@echo "  lint                   - Run clippy linter"
	@echo "  help                   - Show this help message"
	@echo ""
	@echo "Examples:"
	@echo "  RUST_LOG=debug make run-basic-test"
	@echo "  TIDB_HOST=myhost:4000 TIDB_USER=admin make run-basic-test"
	@echo "  LOG_LEVEL=debug LOG_FILE=true make run-logging-test"

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

run-simple-connection:
	RUST_LOG=$(RUST_LOG) cargo run --bin simple_connection -- -H $(TIDB_HOST) -u $(TIDB_USER) -d $(TIDB_DATABASE) $(if $(TIDB_PASSWORD),--password $(TIDB_PASSWORD),--no-password-prompt)

run-isolation-test:
	RUST_LOG=$(RUST_LOG) cargo run --bin isolation -- -H $(TIDB_HOST) -u $(TIDB_USER) -d $(TIDB_DATABASE) $(if $(TIDB_PASSWORD),--password $(TIDB_PASSWORD),--no-password-prompt)

run-cli-test:
	RUST_LOG=$(RUST_LOG) cargo run --bin cli --features="isolation_test" -- $(ARGS)

run-logging-test:
	RUST_LOG=$(RUST_LOG) cargo run --bin logging -- --log-level $(LOG_LEVEL) $(if $(LOG_FILE),--log-file --log-file-path $(LOG_FILE_PATH),)

run-job-monitor-test:
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

lint-fix:
	cargo clippy --fix


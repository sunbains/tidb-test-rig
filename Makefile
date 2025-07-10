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

.PHONY: help build test clean tests build-tests run-simple run-advanced check format lint \
	run-simple-connection run-isolation-test run-cli-test run-logging-test \
	run-basic run-basic-debug run-basic-verbose

# Default target
help:
	@echo "TiDB Multi-Connection Test Tool"
	@echo ""
	@echo "Environment Variables:"
	@echo "  RUST_LOG          - Log level (debug, info, warn, error) [default: info]"
	@echo "  TIDB_HOST         - TiDB host and port [default: localhost:4000]"
	@echo "  TIDB_USER         - Database username [default: root]"
	@echo "  TIDB_DATABASE     - Database name [default: test]"
	@echo "  TIDB_PASSWORD     - Database password (if not set, will prompt)"
	@echo "  LOG_LEVEL         - Log level for tests [default: info]"
	@echo "  LOG_FILE          - Enable file logging [default: false]"
	@echo "  LOG_FILE_PATH     - Log file path [default: logs/tidb_connect.log]"
	@echo ""
	@echo "Available targets:"
	@echo "  build                 - Build the main application"
	@echo "  test                  - Run tests"
	@echo "  clean                 - Clean build artifacts"
	@echo "  tests                 - Build all tests"
	@echo "  build-tests           - Build all tests"
	@echo "  run-basic             - Run basic connection test with env vars"
	@echo "  run-basic-debug       - Run basic test with debug logging"
	@echo "  run-basic-verbose     - Run basic test with verbose output"
	@echo "  run-simple            - Run simple multi-connection test"
	@echo "  run-advanced          - Run advanced multi-connection test"
	@echo "  run-simple-connection - Run simple connection test"
	@echo "  run-isolation-test    - Run isolation test"
	@echo "  run-cli-test          - Run CLI test"
	@echo "  run-logging-test      - Run logging test"
	@echo "  check                 - Check if code compiles without building"
	@echo "  format                - Format code with rustfmt"
	@echo "  lint                  - Run clippy linter"
	@echo "  help                  - Show this help message"
	@echo ""
	@echo "Examples:"
	@echo "  RUST_LOG=debug make run-basic"
	@echo "  TIDB_HOST=myhost:4000 TIDB_USER=admin make run-basic"
	@echo "  LOG_LEVEL=debug LOG_FILE=true make run-logging-test"

# Build the main application
build:
	cargo build --release

# Run tests
test:
	RUST_LOG=$(RUST_LOG) cargo test

# Clean build artifacts
clean:
	cargo clean

# Build all tests
tests: build-tests

build-tests:
	cargo test --no-run

# Run specific tests with environment variables
run-simple:
	RUST_LOG=$(RUST_LOG) cargo test --test simple_multi_connection --

run-advanced:
	RUST_LOG=$(RUST_LOG) cargo test --test multi_connection_test --

run-basic:
	RUST_LOG=$(RUST_LOG) cargo test --test basic_test -- -- -H $(TIDB_HOST) -u $(TIDB_USER) -d $(TIDB_DATABASE) $(if $(TIDB_PASSWORD),--password $(TIDB_PASSWORD),--no-password-prompt)

run-basic-debug:
	RUST_LOG=debug cargo test --test basic_test -- -- -H $(TIDB_HOST) -u $(TIDB_USER) -d $(TIDB_DATABASE) $(if $(TIDB_PASSWORD),--password $(TIDB_PASSWORD),--no-password-prompt)

run-basic-verbose:
	RUST_LOG=debug cargo test --test basic_test -- -- -H $(TIDB_HOST) -u $(TIDB_USER) -d $(TIDB_DATABASE) $(if $(TIDB_PASSWORD),--password $(TIDB_PASSWORD),--no-password-prompt) --verbose

run-simple-connection:
	RUST_LOG=$(RUST_LOG) cargo test --test simple_connection_test -- -- -H $(TIDB_HOST) -u $(TIDB_USER) -d $(TIDB_DATABASE) $(if $(TIDB_PASSWORD),--password $(TIDB_PASSWORD),--no-password-prompt)

run-isolation-test:
	RUST_LOG=$(RUST_LOG) cargo test --test isolation_test -- -- -H $(TIDB_HOST) -u $(TIDB_USER) -d $(TIDB_DATABASE) $(if $(TIDB_PASSWORD),--password $(TIDB_PASSWORD),--no-password-prompt)

run-cli-test:
	RUST_LOG=$(RUST_LOG) cargo test --test cli_test --features="isolation_test" -- -- $(ARGS)

run-logging-test:
	RUST_LOG=$(RUST_LOG) cargo test --test logging_test -- -- --log-level $(LOG_LEVEL) $(if $(LOG_FILE),--log-file --log-file-path $(LOG_FILE_PATH),)

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
# TiDB Multi-Connection Test Tool Makefile

.PHONY: help build test clean examples build-examples run-simple run-advanced check format lint \
	run-simple-connection run-isolation-test run-macro-cli run-logging-example

# Default target
help:
	@echo "TiDB Multi-Connection Test Tool"
	@echo ""
	@echo "Available targets:"
	@echo "  build                 - Build the main application"
	@echo "  test                  - Run tests"
	@echo "  clean                 - Clean build artifacts"
	@echo "  examples              - Build all examples"
	@echo "  build-examples        - Build all examples"
	@echo "  run-simple            - Build and run simple multi-connection example"
	@echo "  run-advanced          - Build and run advanced multi-connection example"
	@echo "  run-simple-connection - Build and run simple connection example"
	@echo "  run-isolation-test    - Build and run isolation test example"
	@echo "  run-macro-cli         - Build and run macro-based CLI example"
	@echo "  run-logging-example   - Build and run logging example"
	@echo "  check                 - Check if code compiles without building"
	@echo "  format                - Format code with rustfmt"
	@echo "  lint                  - Run clippy linter"
	@echo "  help                  - Show this help message"

# Build the main application
build:
	cargo build --release

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean

# Build all examples
examples: build-examples

build-examples:
	cargo build --examples

# Run specific examples
run-simple:
	cargo run --example simple_multi_connection

run-advanced:
	cargo run --example multi_connection_example

run-simple-connection:
	cargo run --example simple_connection -- -H localhost:4000 -u root -d test

run-isolation-test:
	cargo run --example isolation_test_example -- -H localhost:4000 -u root -d test

run-macro-cli:
	cargo run --example macro_cli_example -- -H localhost:4000 -u root -d test --test-rows 20

run-logging-example:
	cargo run --example logging_example -- --log-level debug --log-file --log-file-path logs/mylog.log

# Check if code compiles
check:
	cargo check
	cargo check --examples

# Format code
format:
	cargo fmt

# Run linter
lint:
	cargo clippy
	cargo clippy --examples 
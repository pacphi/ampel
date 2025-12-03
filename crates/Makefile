# Backend Makefile
# Run from crates/ directory or via root Makefile

.PHONY: help build build-release test lint format format-check audit clean run-api run-worker

# Default target
help:
	@echo "Backend targets:"
	@echo "  build         - Build debug binaries"
	@echo "  build-release - Build release binaries"
	@echo "  test          - Run all tests"
	@echo "  lint          - Run clippy linter"
	@echo "  format        - Format code with rustfmt"
	@echo "  format-check  - Check formatting"
	@echo "  audit         - Run security audit"
	@echo "  clean         - Remove build artifacts"
	@echo "  run-api       - Run API server"
	@echo "  run-worker    - Run background worker"

# Build
build:
	cd .. && cargo build

build-release:
	cd .. && cargo build --release

# Testing
test:
	cd .. && cargo test --all-features

# Quality
lint:
	cd .. && cargo clippy --all-targets --all-features -- -D warnings

format:
	cd .. && cargo fmt --all

format-check:
	cd .. && cargo fmt --all -- --check

audit:
	cd .. && cargo audit

# Run
run-api:
	cd .. && cargo run --bin ampel-api

run-worker:
	cd .. && cargo run --bin ampel-worker

# Cleanup
clean:
	cd .. && cargo clean

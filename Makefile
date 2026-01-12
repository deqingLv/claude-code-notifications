# Makefile for claude-code-notifications
# High-performance Rust CLI tool for Claude Code desktop notifications

.PHONY: all build build-release install test fmt clippy clean help

# Default target
all: build-release

# Development build
build:
	@echo "Building development binary..."
	cargo build

# Optimized release build
build-release:
	@echo "Building optimized release binary..."
	cargo build --release

# Install CLI globally
install: build-release
	@echo "Installing claude-code-notifications globally..."
	cargo install --path .

# Run test suite
test:
	@echo "Running test suite..."
	cargo test

# Format Rust code
fmt:
	@echo "Formatting Rust code..."
	cargo fmt

# Lint Rust code with clippy
clippy:
	@echo "Running clippy linter..."
	cargo clippy

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Show help information
help:
	@echo "Claude Code Notifications - Makefile Commands"
	@echo "============================================="
	@echo "make build           - Compile debug binary with symbols"
	@echo "make build-release   - Compile optimized release binary (default)"
	@echo "make install         - Build and install CLI globally"
	@echo "make test            - Run complete test suite"
	@echo "make fmt             - Format Rust code using rustfmt"
	@echo "make clippy          - Lint Rust code with clippy"
	@echo "make clean           - Clean all build artifacts"
	@echo "make help            - Show this help message"

# Quality assurance pipeline
qa: test fmt clippy
	@echo "Quality assurance checks completed successfully!"
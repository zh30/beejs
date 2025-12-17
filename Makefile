# Beejs Build System
# High-performance JavaScript/TypeScript runtime

.PHONY: all build test clean run help install dev release

# Default target
all: build test

# Build the project
build:
	@echo "Building Beejs..."
	cargo build --release

# Run tests
test:
	@echo "Running tests..."
	cargo test
	@echo "✅ All tests passed!"

# Run with specific file
run: build
	@echo "Running example..."
	./target/release/beejs $(file) --verbose

# Clean build artifacts
clean:
	@echo "Cleaning..."
	cargo clean

# Install to system
install: build
	@echo "Installing Beejs..."
	sudo cp target/release/beejs /usr/local/bin/

# Development build (faster)
dev: build

# Release build (optimized)
release: build

# Performance test
perf: build
	@echo "Running performance test..."
	./target/release/beejs examples/performance_test.js --verbose

# Hello world example
hello: build
	@echo "Running hello world example..."
	./target/release/beejs examples/hello_world.js --verbose

# Check formatting
fmt:
	@echo "Checking code formatting..."
	cargo fmt --check

# Lint code
lint:
	@echo "Linting code..."
	cargo clippy

# Show help
help:
	@echo "Beejs - High-performance JavaScript/TypeScript runtime"
	@echo ""
	@echo "Available targets:"
	@echo "  build   - Build the project"
	@echo "  test    - Run all tests"
	@echo "  run     - Run with a specific file (use: make run file=script.js)"
	@echo "  clean   - Clean build artifacts"
	@echo "  install - Install to system"
	@echo "  dev     - Development build"
	@echo "  release - Release build"
	@echo "  perf    - Run performance test"
	@echo "  hello   - Run hello world example"
	@echo "  fmt     - Check code formatting"
	@echo "  lint    - Lint code"
	@echo "  help    - Show this help"

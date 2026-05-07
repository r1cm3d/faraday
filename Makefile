.PHONY: all build release run test clean fmt clippy doc help

# Default target
all: build

# Build the project
build:
	cargo build

# Build for release
release:
	cargo build --release

# Run tests
test:
	cargo test --all-features

# Format code
fmt:
	cargo fmt

# Check code with clippy
clippy:
	cargo clippy -- -D warnings

# Build documentation
doc:
	cargo doc --no-deps --open

# Clean build artifacts
clean:
	cargo clean

# Check dependencies
check:
	cargo check

# Help target
help:
	@echo "Available targets:"
	@echo "  build       - Build the project (dev)"
	@echo "  release     - Build the project (release)"
	@echo "  test        - Run tests with all features"
	@echo "  fmt         - Format code using rustfmt"
	@echo "  clippy      - Run clippy linter"
	@echo "  doc         - Build and open documentation"
	@echo "  clean       - Clean build artifacts"
	@echo "  check       - Check dependencies"
	@echo "  help        - Show this help message"
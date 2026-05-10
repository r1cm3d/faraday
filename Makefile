# Faraday Makefile
# Following project guidelines for target naming and organization

.PHONY: help have deps build test clean lint fmt doc install

# Default target
help: ## Show this help message
	@grep -E '^[a-zA-Z_/-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# Dependency checks
have: ## Check if required tools are installed
	@command -v rustc >/dev/null 2>&1 || { echo "❌ Rust is not installed"; exit 1; }
	@command -v cargo >/dev/null 2>&1 || { echo "❌ Cargo is not installed"; exit 1; }
	@echo "✅ All required tools are available"

# Install dependencies
deps: have ## Install project dependencies
	@echo "📦 Installing dependencies..."
	@cargo fetch

# Build targets
build: ## Build all crates in release mode
	@echo "🔨 Building all crates..."
	@cargo build --release --all

build/debug: ## Build all crates in debug mode
	@echo "🔨 Building all crates (debug)..."
	@cargo build --all

build/check: ## Fast check compilation without code generation
	@echo "🔍 Checking compilation..."
	@cargo check --all

# Test targets
test: ## Run all tests
	@echo "🧪 Running tests..."
	@cargo test --all-features

test/unit: ## Run unit tests only
	@echo "🧪 Running unit tests..."
	@cargo test --lib --all-features

test/integration: ## Run integration tests only
	@echo "🧪 Running integration tests..."
	@cargo test --test '*' --all-features

# Quality targets
lint: ## Run clippy linter
	@echo "🔍 Running clippy..."
	@cargo clippy --all-features -- -D warnings

fmt: ## Format code with rustfmt
	@echo "🎨 Formatting code..."
	@cargo fmt

fmt/check: ## Check code formatting without modifying
	@echo "🎨 Checking code formatting..."
	@cargo fmt -- --check

# Documentation
doc: ## Generate documentation
	@echo "📚 Generating documentation..."
	@cargo doc --no-deps --all-features

doc/open: ## Generate and open documentation
	@echo "📚 Generating and opening documentation..."
	@cargo doc --no-deps --all-features --open

# Installation
install: build ## Install faraday CLI globally
	@echo "⚡ Installing faraday CLI..."
	@CARGO_INSTALL_ROOT=$(HOME)/.cargo cargo install --path crates/faraday-cli --force

install/tui: build ## Install faraday TUI globally
	@echo "📺 Installing faraday TUI..."
	@CARGO_INSTALL_ROOT=$(HOME)/.cargo cargo install --path crates/faraday-tui --force

# Clean targets
clean: ## Clean build artifacts
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean

clean/all: clean ## Clean everything including cache
	@echo "🧹 Cleaning everything..."
	@rm -rf target/
	@rm -f Cargo.lock

# Development workflow targets
dev/check: fmt lint build/check test ## Full development check (format, lint, compile, test)

dev/quick: build/check test/unit ## Quick development check (compile, unit tests)

# Phase 1 specific targets
phase1/build: ## Build Phase 1 components (read-only HS-CAN)
	@echo "🚗 Building Phase 1 components..."
	@cargo build --package faraday-core --package faraday-cli

phase1/test: ## Test Phase 1 functionality
	@echo "🚗 Testing Phase 1 functionality..."
	@cargo test --package faraday-core --package faraday-cli

# Utility targets
deps/update: ## Update dependencies
	@echo "📦 Updating dependencies..."
	@cargo update

deps/audit: ## Audit dependencies for security issues
	@echo "🔒 Auditing dependencies..."
	@cargo audit

size: build ## Show binary sizes
	@echo "📏 Binary sizes:"
	@ls -lh target/release/faraday 2>/dev/null || echo "❌ faraday binary not found"
	@ls -lh target/release/faraday-tui 2>/dev/null || echo "❌ faraday-tui binary not found"

# Git integration
git/status: ## Show git status
	@git status --short

git/prepare: dev/check ## Prepare for commit (run all checks)
	@echo "✅ Ready for commit"
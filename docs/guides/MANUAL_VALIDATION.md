# Manual Validation Guide

This document provides step-by-step instructions for manually validating all aspects of the Faraday project according to the requirements specified in README.md and CLAUDE.md.

## Prerequisites

Before starting validation, ensure you have:

- Rust 1.70 or higher installed
- Cargo (included with Rust)
- Git for version control
- Optional: cargo-audit for dependency security checks
- Optional: cargo-udeps for unused dependency detection

### Verify Prerequisites

```bash
# Check Rust version
rustc --version

# Check Cargo version
cargo --version

# Verify project tools are available
make have
```

## 1. Code Quality Validation

### 1.1 Code Formatting

Validate that all code follows Rust formatting standards:

```bash
# Check if code is properly formatted (fails if not formatted)
make fmt/check

# Or using cargo directly
cargo fmt -- --check
```

**Expected Result:** No output indicates proper formatting. If files need formatting, they will be listed.

### 1.2 Linting

Run Clippy to catch common mistakes and enforce best practices:

```bash
# Run clippy with project settings (treats warnings as errors)
make lint

# Or using cargo directly
cargo clippy --all-features -- -D warnings
```

**Expected Result:** No warnings or errors. The build should succeed with `warnings = 0`.

### 1.3 Compilation Check

Verify the project compiles without generating artifacts:

```bash
# Fast compilation check
make build/check

# Or using cargo directly
cargo check --all
```

**Expected Result:** Successful compilation with no errors.

## 2. Build Validation

### 2.1 Debug Build

Build all crates in debug mode:

```bash
# Debug build
make build/debug

# Or using cargo directly
cargo build --all
```

**Expected Result:** All crates compile successfully, producing debug binaries.

### 2.2 Release Build

Build all crates in optimized release mode:

```bash
# Release build
make build

# Or using cargo directly
cargo build --release --all
```

**Expected Result:** All crates compile successfully, producing optimized binaries.

### 2.3 Verify Binary Creation

Check that the expected binaries are created:

```bash
# Check binary sizes and existence
make size

# Manual verification
ls -la target/release/faraday
ls -la target/release/faraday-tui
```

**Expected Result:** Both `faraday` and `faraday-tui` binaries should exist and be reasonably sized.

### 2.4 Phase-Specific Builds

Test building specific project phases:

```bash
# Build Phase 1 components only
make phase1/build

# Verify core and CLI packages build independently
cargo build --package faraday-core
cargo build --package faraday-cli
cargo build --package faraday-asbuilt
cargo build --package faraday-tui
```

**Expected Result:** Each package should build successfully in isolation.

## 3. Testing Validation

### 3.1 Complete Test Suite

Run all tests across the project:

```bash
# Run all tests with all features enabled
make test

# Or using cargo directly
cargo test --all-features
```

**Expected Result:** All tests pass. Look for output indicating test counts and zero failures.

### 3.2 Unit Tests

Run only unit tests:

```bash
# Unit tests only
make test/unit

# Or using cargo directly
cargo test --lib --all-features
```

**Expected Result:** All unit tests pass.

### 3.3 Integration Tests

Run only integration tests:

```bash
# Integration tests only
make test/integration

# Or using cargo directly
cargo test --test '*' --all-features
```

**Expected Result:** All integration tests pass.

### 3.4 Phase-Specific Testing

Test specific project phases:

```bash
# Test Phase 1 functionality
make phase1/test

# Test individual packages
cargo test --package faraday-core
cargo test --package faraday-cli
cargo test --package faraday-asbuilt
cargo test --package faraday-tui
```

**Expected Result:** Phase 1 tests should pass completely. Other phases may have partial test coverage.

### 3.5 Single Test Execution

Validate ability to run specific tests:

```bash
# Run a specific test by name (example)
cargo test test_name

# Run tests in a specific module
cargo test module_name::
```

**Expected Result:** Specific tests run and pass as expected.

## 4. Documentation Validation

### 4.1 Generate Documentation

Build project documentation:

```bash
# Generate documentation
make doc

# Or using cargo directly
cargo doc --no-deps --all-features
```

**Expected Result:** Documentation builds without errors.

### 4.2 Documentation Coverage

Verify documentation coverage:

```bash
# Generate and open documentation for review
make doc/open

# Or using cargo directly
cargo doc --no-deps --all-features --open
```

**Expected Result:** Documentation opens in browser, all public items have doc comments.

### 4.3 Documentation Standards

Manual verification checklist:
- [ ] All public functions have `///` doc comments
- [ ] All modules have module-level documentation
- [ ] Examples compile (if present)
- [ ] Documentation is in English
- [ ] No broken links in documentation

## 5. Dependency Validation

### 5.1 Dependency Fetching

Verify all dependencies can be fetched:

```bash
# Install/fetch dependencies
make deps

# Or using cargo directly
cargo fetch
```

**Expected Result:** All dependencies download successfully.

### 5.2 Dependency Updates

Check for outdated dependencies:

```bash
# Update dependencies
make deps/update

# Or using cargo directly
cargo update
```

**Expected Result:** Dependencies update without conflicts.

### 5.3 Security Audit

Check dependencies for security vulnerabilities:

```bash
# Audit dependencies (requires cargo-audit)
make deps/audit

# Or using cargo directly (if cargo-audit is installed)
cargo audit
```

**Expected Result:** No known security vulnerabilities.

### 5.4 Unused Dependencies

Check for unused dependencies:

```bash
# Check for unused dependencies (requires cargo-udeps)
cargo +nightly udeps
```

**Expected Result:** No unused dependencies reported.

## 6. Workspace Structure Validation

### 6.1 Workspace Configuration

Verify workspace structure matches specification:

```bash
# Check workspace members
cargo metadata --format-version 1 | jq '.workspace_members'

# Verify crate structure
find crates/ -name Cargo.toml -exec echo "=== {} ===" \; -exec cat {} \;
```

**Expected Structure:**
```
faraday/
├── Cargo.toml              # [workspace]
├── crates/
│   ├── faraday-core/       # Core library
│   ├── faraday-cli/        # CLI binary
│   ├── faraday-asbuilt/    # As-built blocks catalog
│   └── faraday-tui/        # Live data viewer
```

### 6.2 Crate Dependencies

Validate inter-crate dependencies:

```bash
# Check dependency graph
cargo tree
```

**Expected Result:** Clean dependency tree with expected relationships between crates.

## 7. Installation Validation

### 7.1 CLI Installation

Install and test CLI tool:

```bash
# Install faraday CLI globally
make install

# Verify installation
faraday --version
faraday --help
```

**Expected Result:** CLI installs successfully and shows version/help information.

### 7.2 TUI Installation

Install and test TUI tool:

```bash
# Install faraday TUI globally
make install/tui

# Verify installation
faraday-tui --version
faraday-tui --help
```

**Expected Result:** TUI installs successfully and shows version/help information.

## 8. Development Workflow Validation

### 8.1 Quick Development Check

Run the quick development workflow:

```bash
# Quick check (compile + unit tests)
make dev/quick
```

**Expected Result:** Fast feedback on basic code health.

### 8.2 Full Development Check

Run the complete development workflow:

```bash
# Full development check (format, lint, compile, test)
make dev/check
```

**Expected Result:** Complete validation passes, ready for commit.

### 8.3 Git Integration

Validate git integration features:

```bash
# Check git status
make git/status

# Prepare for commit (runs all checks)
make git/prepare
```

**Expected Result:** Git status shows clean state, all pre-commit checks pass.

## 9. Architecture Layer Validation

### 9.1 Layer Separation

Manually verify the 5-layer architecture:

1. **CLI Layer** - Check `crates/faraday-cli/src/` uses only clap for command parsing
2. **Command Layer** - Check `crates/faraday-core/src/commands/` contains high-level operations
3. **Protocol Layer** - Check `crates/faraday-core/src/protocol/` implements J1979 and UDS
4. **Transport Layer** - Check `crates/faraday-core/src/transport/` implements ISO-TP
5. **Link Layer** - Check `crates/faraday-core/src/link/` implements ELM327 and SocketCAN

### 9.2 Design Principles Validation

Verify key design principles:
- [ ] Protocol layer uses `IsoTpTransport` trait (transport-agnostic)
- [ ] All I/O operations are async
- [ ] No unwrap() in production code (except tests)
- [ ] Uses thiserror in library, anyhow in CLI
- [ ] tracing used for observability

## 10. Phase Implementation Status

### 10.1 Phase 1 (Complete) ✅

Test read-only HS-CAN functionality:

```bash
# These commands should be available and functional
faraday read-dtc --help
faraday clear-dtc --help
faraday live --help
faraday vin --help
```

### 10.2 Phase 2 (Complete) ✅

Test UDS basics and ISO-TP:

```bash
# These commands should be available and functional
faraday read-did --help
faraday session --help
```

### 10.3 Phase 3 (Partial) 🔶

Check AS-built functionality:

```bash
# Core library should be complete
cargo test --package faraday-asbuilt

# CLI commands may not be implemented yet
faraday asbuilt --help  # May not exist yet
```

### 10.4 Phase 4 (Partial) 🔶

Check write functionality:

```bash
# Protocol support should be complete in core
cargo test --package faraday-core

# CLI commands and safety systems may not be implemented
faraday asbuilt write --help    # May not exist yet
faraday asbuilt restore --help  # May not exist yet
```

## 11. Safety Features Validation

### 11.1 Write Operation Safety (When Implemented)

When write operations are available, verify:
- [ ] Mandatory snapshots before writes
- [ ] Validation against known blocks
- [ ] Block writes to programming DIDs (F1xx, F0xx)
- [ ] `--dry-run` mode required
- [ ] Double confirmation for changes
- [ ] Audit logging in `~/.local/share/faraday/audit.jsonl`

### 11.2 Operational Requirements

Document validation of:
- [ ] Engine off, ignition in KOEO requirement
- [ ] Battery voltage ≥ 12.4V check
- [ ] No active communication DTCs check

## 12. Clean-up Validation

### 12.1 Clean Build Artifacts

Test cleanup functionality:

```bash
# Clean build artifacts
make clean

# Verify artifacts are removed
ls target/ 2>/dev/null && echo "Artifacts remain" || echo "Clean successful"
```

### 12.2 Complete Clean

Test complete cleanup:

```bash
# Clean everything including cache
make clean/all

# Verify complete removal
ls target/ Cargo.lock 2>/dev/null && echo "Files remain" || echo "Complete clean successful"
```

## Validation Checklist

Use this checklist to ensure complete validation:

### Code Quality
- [ ] Formatting check passes (`make fmt/check`)
- [ ] Linting passes with no warnings (`make lint`)
- [ ] Compilation check passes (`make build/check`)

### Building
- [ ] Debug build succeeds (`make build/debug`)
- [ ] Release build succeeds (`make build`)
- [ ] All binaries created successfully
- [ ] Phase-specific builds work

### Testing
- [ ] All tests pass (`make test`)
- [ ] Unit tests pass (`make test/unit`)
- [ ] Integration tests pass (`make test/integration`)
- [ ] Phase-specific tests pass

### Documentation
- [ ] Documentation builds (`make doc`)
- [ ] All public items documented
- [ ] Documentation standards met

### Dependencies
- [ ] Dependencies fetch successfully (`make deps`)
- [ ] No security vulnerabilities (`make deps/audit`)
- [ ] No unused dependencies

### Installation
- [ ] CLI installs and works (`make install`)
- [ ] TUI installs and works (`make install/tui`)

### Workflow
- [ ] Quick development check passes (`make dev/quick`)
- [ ] Full development check passes (`make dev/check`)
- [ ] Git integration works

### Architecture
- [ ] Layer separation maintained
- [ ] Design principles followed
- [ ] Phase implementations match status

### Safety (When Applicable)
- [ ] Write operation safety features
- [ ] Operational requirements validated

### Cleanup
- [ ] Clean operations work
- [ ] No artifacts remain after cleanup

## Troubleshooting

### Common Issues

1. **Formatting Failures**: Run `make fmt` to fix formatting issues
2. **Clippy Warnings**: Address each warning individually, treating them as errors
3. **Test Failures**: Check test output for specific failure reasons
4. **Build Errors**: Ensure all dependencies are available and up-to-date
5. **Documentation Issues**: Verify all public items have proper doc comments

### Getting Help

If validation fails:
1. Check the error messages carefully
2. Ensure all prerequisites are installed
3. Try cleaning and rebuilding (`make clean && make build`)
4. Check that you're using the correct Rust version (≥1.70)
5. Refer to the project documentation in `docs/` directory

This validation process ensures the project meets all quality standards specified in README.md and CLAUDE.md before deployment or contribution.
# Faraday

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

A Rust CLI tool for communicating with the Ford Fusion 2017 SEL via OBD-II. It performs diagnostics and configuration through FORScan-compatible adapters using CAN bus protocols (HS-CAN + MS-CAN).

## 🔧 Development

### Prerequisites

- Rust 1.70 or higher
- Cargo (included with Rust)

### Using Makefile

The project includes a `Makefile` for common development tasks.

Run `make help` to see available targets.

```bash
make help
```

Common commands:

- `make build`: Build the project (dev)
- `make release`: Build the project (release)
- `make test`: Run tests with all features
- `make fmt`: Format code using rustfmt
- `make clippy`: Run clippy linter
- `make doc`: Build and open documentation
- `make clean`: Clean build artifacts
- `make check`: Check dependencies

If you prefer using `cargo` directly:

- Build: `cargo build --release`
- Test: `cargo test --all-features`
- Format: `cargo fmt`
- Lint: `cargo clippy -- -D warnings`
- Documentation: `cargo doc --no-deps`

## 📄 License

This project is licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)
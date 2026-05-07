# Faraday

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

A Rust CLI tool for communicating with the Ford Fusion 2017 SEL via OBD-II. It performs diagnostics and configuration through FORScan-compatible adapters using CAN bus protocols (HS-CAN + MS-CAN).

Named in tribute to Michael Faraday, whose work on electromagnetism underpins all electrical/CAN communication in vehicles. The project aims to partially replace FORScan for scriptable, version-controllable automotive operations while serving as a deep exercise in automotive protocols.

## 🎯 Project Goals

- **Standard diagnostics** — Read DTCs, live data, and vehicle information via SAE J1979
- **Proprietary reads** — Read "as-built" data and Ford-specific DIDs via UDS (ISO 14229)
- **Configuration writes** — Modify as-built blocks with Security Access, including mandatory snapshots and rollback capability
- **Scriptable operations** — Replace manual FORScan workflows with version-controllable CLI commands
- **Reusable library** — Produce a decoupled `faraday-core` library for other projects

## 🚗 Vehicle Support

**Target Vehicle:** Ford Fusion 2017 SEL (Brazilian/Mercosur market, assembled in Hermosillo)

**Supported Adapters:**
- Primary: Vgate vLinker FS (USB/Bluetooth variants)
- Compatible: OBDLink EX, ELS27 with STN chips

**CAN Bus Architecture:**
- **HS-CAN** (500 kbps): PCM, TCM, ABS, RCM, PSCM - pins 6/14
- **MS-CAN** (125 kbps): BCM, IPC, APIM, HVAC, DSM, PAM - pins 3/11

## 🏗️ Architecture

The project follows a strict 5-layer architecture:

```
┌─────────────────────────────────────────────────────────┐
│  CLI Layer (faraday-cli) — Command parsing with clap   │
├─────────────────────────────────────────────────────────┤
│  Command Layer — High-level operations                 │
│  ReadDTCs, ReadAsBuilt, WriteAsBuilt, etc.             │
├─────────────────────────────────────────────────────────┤
│  Protocol Layer — J1979 (OBD-II) + UDS (ISO 14229)    │
├─────────────────────────────────────────────────────────┤
│  Transport Layer — ISO-TP (ISO 15765-2) over CAN       │
├─────────────────────────────────────────────────────────┤
│  Link Layer — ELM327 AT commands and SocketCAN         │
└─────────────────────────────────────────────────────────┘
```

## 📚 Technical Documentation

Understanding Ford vehicle diagnostics requires knowledge of various automotive protocols, bus architectures, and diagnostic concepts. The following documentation provides detailed explanations of the technical terms and concepts used throughout this project:

### Core Concepts
- **[DTCs](docs/DTCs.md)** - Diagnostic Trouble Codes: standardized fault codes stored in vehicle modules
- **[As-Built](docs/AsBuilt.md)** - Configuration data that defines vehicle feature behavior and settings
- **[DIDs](docs/DIDs.md)** - Data Identifiers used to access specific data within vehicle modules

### Protocols and Standards
- **[UDS](docs/UDS.md)** - Unified Diagnostic Services (ISO 14229): advanced diagnostic protocol
- **[FORScan](docs/FORScan.md)** - Popular Ford diagnostic software and its relationship to Faraday

### Hardware and Communication
- **[Adapters](docs/Adapters.md)** - OBD-II adapters and their differences, compatibility requirements
- **[Device Detection](docs/DeviceDetection.md)** - Guide for identifying and configuring OBD-II adapter device paths
- **[HS-CAN](docs/HS-CAN.md)** - High-Speed CAN bus: powertrain modules (PCM, TCM, ABS, RCM, PSCM) via pins 6/14
- **[MS-CAN](docs/MS-CAN.md)** - Medium-Speed CAN bus: body control modules (BCM, IPC, APIM, HVAC, DSM, PAM) via pins 3/11

Each document is written specifically for the **Ford Fusion 2017 SEL** context and explains how these concepts apply to the Faraday project's implementation.

### Workspace Structure

```
faraday/
├── Cargo.toml              # [workspace]
├── crates/
│   ├── faraday-core/       # Core library: link + transport + protocol + commands
│   ├── faraday-cli/        # CLI binary (clap), produces `faraday` executable
│   ├── faraday-asbuilt/    # As-built blocks catalog (data-only)
│   └── faraday-tui/        # Live data viewer (ratatui)
└── docs/SPEC.md           # Technical specification
```

## 🛡️ Safety Features

**Write Operation Safety:**
- Mandatory snapshots before any write operation
- Validation against known blocks in `faraday-asbuilt`
- Block writes to programming DIDs (`F1xx`, `F0xx`)
- `--dry-run` mode required for real writes
- Double confirmation for configuration changes
- Audit logging in `~/.local/share/faraday/audit.jsonl`

**Operational Requirements:**
Configuration writes should only occur with:
- Engine off, ignition in KOEO (Key On Engine Off)
- Battery voltage ≥ 12.4V
- No active communication DTCs

## 🗺️ Development Roadmap

### Phase 1: Read-only HS-CAN (standard OBD-II) ✅
CLI commands: `read-dtc`, `clear-dtc`, `live <pids>`, `vin`

### Phase 2: UDS basics, full ISO-TP
CLI commands: `read-did --module <module> <did>`, `session --module <module> extended`

### Phase 3: MS-CAN + as-built reads
CLI commands: `asbuilt dump --module <module>`, `asbuilt show --module <module> --feature <feature>`

### Phase 4: Security Access + Write
CLI commands: `asbuilt write --module <module> --block <block>`, `asbuilt restore <snapshot>`

### Phase 5: Polish and ergonomics
Live data TUI, YAML profiles, structured logging, documentation

## 🚨 Important Disclaimers

- **Vehicle-specific:** Currently supports only Ford Fusion 2017 SEL
- **No firmware reprogramming:** UDS Modes 34/36 are out of scope (high risk)
- **CLI/TUI only:** No GUI planned
- **Experimental:** Use at your own risk - always create snapshots before modifications

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
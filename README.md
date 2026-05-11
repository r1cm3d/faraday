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
│   ├── faraday-tui/        # Live data viewer (ratatui)
│   └── faraday-emu/        # PTY-based ECU emulator, produces `faraday-emu` executable
└── docs/SPEC.md            # Technical specification
```

## ✏️ Phase 4 Commands — Security Access + Write

```bash
# Capture an as-built snapshot before making changes
faraday --adapter /tmp/faraday-dev asbuilt snapshot --module bcm
faraday --adapter /tmp/faraday-dev asbuilt snapshot --module bcm --output ./my_backup.json

# Preview a write without touching the vehicle (dry-run)
faraday --adapter /tmp/faraday-dev asbuilt write --module bcm --feature drl_enabled --value true --dry-run

# Write a feature value (auto-captures snapshot, prompts for confirmation)
faraday --adapter /tmp/faraday-dev asbuilt write --module bcm --feature drl_enabled --value true
faraday --adapter /tmp/faraday-dev asbuilt write --module ipc --feature speed_units --value mph --yes

# Restore all blocks from a snapshot file
faraday --adapter /tmp/faraday-dev asbuilt restore ~/.local/share/faraday/snapshots/bcm_2026-05-10T14-30-00Z.json
```

**Supported value formats:**
- Boolean features: `true`/`false`, `on`/`off`, `1`/`0`, `enabled`/`disabled`
- Enumerated features: value name (case-insensitive) or numeric key
- Numeric features: integer within the feature's defined range

Each write appends an entry to `~/.local/share/faraday/audit.jsonl`.

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
- **Status:** Complete and functional

### Phase 2: UDS basics, full ISO-TP ✅
CLI commands: `read-did --module <module> <did>`, `session --module <module> extended`
- **Status:** Complete with full UDS implementation

### Phase 3: MS-CAN + as-built reads ✅
CLI commands: `asbuilt dump --module <module>`, `asbuilt show --module <module> --feature <feature>`
- **Status:** Complete and functional
- **Core library:** ✅ `faraday-asbuilt` crate with full decoding and DID mapping
- **CLI commands:** ✅ `asbuilt dump` and `asbuilt show` implemented
- **MS-CAN support:** ✅ Automatic CAN bus switching per module

### Phase 4: Security Access + Write ✅
CLI commands: `asbuilt write`, `asbuilt snapshot`, `asbuilt restore`
- **Protocol support:** ✅ UDS 0x27 (SecurityAccess) + 0x2E (WriteDataByIdentifier)
- **Seed→key algorithm:** ✅ Ford FORScan configuration-access algorithm (requires hardware validation)
- **Feature-level writes:** ✅ `asbuilt write --module bcm --feature drl_enabled --value true`
- **Snapshots:** ✅ Auto-captured before each write; `asbuilt snapshot --module bcm`
- **Rollback:** ✅ `asbuilt restore <snapshot.json>`
- **Dry-run:** ✅ `--dry-run` shows diff without touching the vehicle
- **Audit logging:** ✅ `~/.local/share/faraday/audit.jsonl` (JSONL, one entry per operation)
- **Safety guards:** ✅ Programming DID block (`F0xx`/`F1xx`), interactive confirmation, `--yes` for scripts

### Phase 5: Polish and ergonomics ✅
Live data TUI, YAML profiles, structured logging, documentation
- **Live data TUI:** ✅ Complete (`faraday-tui` fully functional)
- **YAML profiles:** ✅ `faraday profile apply my-fusion.yml` / `faraday profile validate`
- **Structured logging:** ✅ Per-command session JSONL (`~/.local/share/faraday/sessions.jsonl`) + audit log
- **Documentation:** ✅ Complete and comprehensive

### Phase 6: Comprehensive Hidden Diagnostic TUI 🔶

Transform `faraday-tui` from a basic 5-PID viewer into a professional multi-tab diagnostic interface covering all major vehicle systems reachable via HS-CAN and MS-CAN.

**Current gap:** Phase 5 TUI exposes fewer than 5% of available diagnostic data (5 PIDs, single view, no tab navigation).

#### 6.1 Multi-Tab Interface Architecture 🔲
- Tab-based navigation (number keys `1`–`9`, arrow keys)
- Real-time status bar with connection state, battery voltage, and update rate
- Context-sensitive help overlay (`?`)
- Pause/resume (`p`), data reset (`r`), tab export (`e`)

#### 6.2 Engine & Powertrain Dashboard 🔲
Standard OBD-II extensions (PIDs `0x06`–`0x62`):
- Short/long-term fuel trim (both banks)
- Ignition timing advance
- O2 sensor voltages and fuel-air equivalence ratio
- EGR commanded position and error
- Engine fuel rate · driver demand torque · actual torque
- Ford-specific: cylinder misfire counters · engine oil life (UDS DIDs)

#### 6.3 Transmission Analytics Panel 🔲
TCM UDS reads:
- Current gear vs. commanded gear
- Transmission fluid temperature
- Torque converter slip %
- Shift solenoid status grid (color-coded)
- Line pressure

#### 6.4 Body Systems Monitor (BCM) 🔲
BCM UDS reads:
- Battery voltage under load · alternator duty cycle
- Door ajar bitmask with per-door ASCII diagram
- Lighting circuit health table
- HVAC blower actual vs. commanded speed

#### 6.5 Safety Systems Status (ABS / RCM) 🔲
ABS and RCM UDS reads:
- Wheel speeds FL/FR/RL/RR with sparklines
- Yaw rate and lateral acceleration
- Stability intervention counter
- Airbag squib continuity and seatbelt bitmask

#### 6.6 Advanced Driver Assistance (PAM) 🔲
PAM UDS reads:
- Eight ultrasonic sensor distances visualized as a top-down vehicle diagram
- Backup camera status
- Object detection confidence

#### 6.7 Climate Control (HVAC) 🔲
HVAC UDS reads:
- Driver/passenger cabin temps (dual-zone)
- Blend door actual vs. commanded %
- Evaporator temp · refrigerant pressure · AC compressor load

#### 6.8 Communication & Infotainment (APIM) 🔲
APIM UDS reads:
- GPS fix quality and satellite count
- Cellular RSSI and Bluetooth device count
- Software version string for all polled modules

#### 6.9 Vehicle Analytics & History 🔲
In-session computed from live data:
- RPM histogram (idle / cruise / high-load bands)
- Speed distribution
- Estimated fuel consumption (PID `0x5E`)
- Brake and acceleration event counts
- Persisted to `~/.local/share/faraday/analytics.jsonl` on exit

#### 6.10 System Health Monitoring 🔲
Cross-module status table:
- Last response time per module
- Consecutive timeout count
- Module voltage (where available)
- Software version

#### Technical Implementation 🔲
- New `panels/` module tree inside `faraday-tui` (one file per tab)
- `Panel` trait with `update()` / `render()` / `help_text()` interface
- Per-panel polling intervals (250 ms engine PIDs → 5 s infotainment)
- Background tokio tasks pre-fetch non-active tabs so switching is instant
- Graceful degradation: timed-out modules show `[N/A]`, TUI never panics
- Extended PID catalog in `faraday-core::protocol::j1979` (PIDs `0x06`–`0x62`)

**Navigation:** `1`–`9` jump to tabs · `←`/`→` cycle · `p` pause · `r` reset · `e` export · `?` help · `q` quit

### Next Priority: Phase 6
Implement the multi-tab TUI architecture (6.1) first, then extend PIDs in `faraday-core` (6.2), then build each panel (6.3–6.10).

## 🚨 Important Disclaimers

- **Vehicle-specific:** Currently supports only Ford Fusion 2017 SEL
- **No firmware reprogramming:** UDS Modes 34/36 are out of scope (high risk)
- **CLI/TUI only:** No GUI planned
- **Experimental:** Use at your own risk - always create snapshots before modifications

## 🔧 Development

### ECU Emulator

`faraday-emu` is a standalone process that creates a virtual serial port (PTY) and speaks the full ELM327/STN text protocol. It exercises the **complete 5-layer stack** — including `VLinkerFs`, baud rate configuration, and AT/STN command parsing — identical to a real adapter. No OBD-II adapter or vehicle connection is required.

```bash
# Terminal 1 — start the emulator (prints slave device path, creates symlink)
faraday-emu --link /tmp/faraday-dev

# Terminal 2 — connect as if it were real hardware
faraday --adapter /tmp/faraday-dev vin
faraday --adapter /tmp/faraday-dev read-dtc
faraday --adapter /tmp/faraday-dev live 0C,05,0D
faraday --adapter /tmp/faraday-dev read-did --module pcm F190
faraday --adapter /tmp/faraday-dev session --module pcm extended
```

Omit `--link` to use the raw `/dev/pts/N` path printed on stdout. The emulator removes the symlink automatically on Ctrl-C or SIGTERM.

**Simulated values:**

| Data | Value |
|---|---|
| VIN | `1FA6P8TH5H5123456` |
| Engine RPM | 800 rpm |
| Coolant temp | 65 °C |
| Stored / pending / permanent DTCs | None |
| Control module voltage | 14.26 V |

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
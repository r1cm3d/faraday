# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## General Policies

### Language Policy
- **ALWAYS write code in English** - variable names, function names, class names, files, constants
- **ALWAYS write comments in English** - (when explicitly requested)
- **ALWAYS write documentation in English**
- **ALWAYS write commit messages in English**
- **NEVER use non-English characters** in code identifiers

### Code Comments Policy
- **DO NOT add comments** unless explicitly requested
- Code should be self-documenting through clear naming and structure
- Only add comments when the user specifically asks for them

### Logging Policy

What CAN be logged:
- Non-sensitive IDs (UUIDs, generated IDs)
- Operation names and types
- Status codes and error codes
- Timestamps and durations
- Non-sensitive business metrics

**NEVER log whole objects** - always log specific, direct attributes only.

## Project Overview

`faraday` is a Rust CLI tool for communicating with the Ford Fusion 2017 SEL via OBD-II. It performs diagnostics and configuration through FORScan-compatible adapters using CAN bus protocols (HS-CAN + MS-CAN).

## Commit Guidelines

### Commit Message Format
- Use **imperative mood**: "make xyzzy do frotz" instead of "This patch makes xyzzy do frotz"
- Keep the summary line under 70-75 characters
- Reference bug entries by number and URL when fixing logged bugs
- Include SHA-1 (at least first 12 characters) when referencing commits

### Every commit must have a clear description
- Describe the problem that motivated the change
- Describe user-visible impact (crashes, lockups, performance regressions)
- Quantify optimizations and trade-offs with actual numbers
- Explain what you are actually doing in technical detail

### Commit Separation
- Separate each logical change into a separate commit
- Bug fixes and performance enhancements should be different commits
- API updates and new features using that API should be separate commits
- Each commit should make and easily understood change
- Each commit should be verifiable by reviewers on its own merits

## Development Commands

The project follows standard Rust workspace practices:

- **Build**: `cargo build` / `cargo build --release`
- **Test**: `cargo test --all-features`
- **Lint**: `cargo clippy -- -D warnings`
- **Format**: `cargo fmt`
- **Documentation**: `cargo doc --no-deps`
- **Single test**: `cargo test test_name`

Check dependencies: `cargo check`

## Architecture

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

### Layered Architecture
The system follows a strict 5-layer architecture:

1. **CLI Layer** (`faraday-cli`) - Command parsing with clap
2. **Command Layer** (`faraday-core::commands`) - High-level operations like ReadDTCs, ReadAsBuilt
3. **Protocol Layer** (`faraday-core::protocol`) - J1979 (OBD-II) and UDS (ISO 14229)
4. **Transport Layer** (`faraday-core::transport`) - ISO-TP (ISO 15765-2) over CAN
5. **Link Layer** (`faraday-core::link`) - ELM327 AT commands and SocketCAN

### Key Design Principles
- **Transport-agnostic protocol**: Protocol layer uses `IsoTpTransport` trait
- **Async-first**: Built on tokio runtime with async I/O
- **Zero unwrap in production**: Only in tests and proven invariants
- **Observability**: Every CAN frame logged via tracing
- **Error handling**: Uses thiserror in library, anyhow in CLI

## Vehicle Context

### CAN Bus Architecture
- **HS-CAN** (500 kbps): PCM, TCM, ABS, RCM, PSCM - pins 6/14
- **MS-CAN** (125 kbps): BCM, IPC, APIM, HVAC, DSM, PAM - pins 3/11

### Critical Module Addresses
Key request/response header pairs:
- PCM: `7E0`/`7E8` (HS-CAN)
- BCM: `726`/`72E` (MS-CAN)
- IPC: `720`/`728` (MS-CAN)
- Functional broadcast: `7DF` (HS-CAN)

## Safety Considerations

### Write Operations Safety
- **Mandatory snapshots** before any write operation
- Validation against known blocks in `faraday-asbuilt`
- Block writes to programming DIDs (`F1xx`, `F0xx`)
- `--dry-run` mode required for real writes
- Double confirmation for configuration changes
- Audit logging in `~/.local/share/faraday/audit.jsonl`

### Operational Requirements
Configuration writes should only occur with:
- Engine off, ignition in KOEO
- Battery voltage ≥ 12.4V
- No active communication DTCs

## Development Phases

### Phase 1: Read-only HS-CAN (standard OBD-II) ✅
CLI commands: `read-dtc`, `clear-dtc`, `live <pids>`, `vin`
- **Status:** Complete and functional

### Phase 2: UDS basics, full ISO-TP ✅
CLI commands: `read-did --module <module> <did>`, `session --module <module> extended`
- **Status:** Complete with full UDS implementation

### Phase 3: MS-CAN + as-built reads 🔶
CLI commands: `asbuilt dump --module <module>`, `asbuilt show --module <module> --feature <feature>`
- **Core library:** Complete (`faraday-asbuilt` crate)
- **CLI commands:** Need implementation
- **MS-CAN support:** Complete

### Phase 4: Security Access + Write 🔶
CLI commands: `asbuilt write --module <module> --block <block>`, `asbuilt restore <snapshot>`
- **Protocol support:** Complete
- **CLI commands:** Need implementation
- **Safety systems:** Need implementation (snapshots, audit logging)

### Phase 5: Polish and ergonomics 🔶
Live data TUI, YAML profiles, structured logging, documentation
- **TUI:** ✅ Complete (`faraday-tui`)
- **YAML profiles:** Need implementation
- **Structured logging:** Partial (need audit logging)
- **Documentation:** ✅ Complete

### Phase 6: Comprehensive Hidden Diagnostic TUI 🔶
Transform TUI from basic OBD-II viewer to professional Ford diagnostic interface
CLI commands: Enhanced `faraday-tui` with multi-tab interface and comprehensive diagnostics
- **Multi-tab interface:** Need implementation
- **Comprehensive diagnostic panels:** Need implementation
- **Advanced analytics:** Need implementation
- **Real-time system monitoring:** Need implementation

#### 6.1 Multi-Tab Interface Architecture
- **Tab-based navigation** for different vehicle systems
- **Real-time status bar** showing connection, data rates, and system health
- **Context-sensitive help** for each diagnostic parameter

#### 6.2 Engine & Powertrain Dashboard
**Gauges & Metrics:**
- Fuel trim values (short/long term)
- Individual cylinder misfire counters
- Ignition timing advance/retard
- Catalyst efficiency percentage
- Turbocharger boost pressure
- Variable valve timing positions
- EGR valve position and flow
- Engine oil life percentage

#### 6.3 Transmission Analytics Panel
**Real-time Data:**
- Gear ratios and shift patterns
- Transmission fluid temperature/pressure
- Clutch slip percentages
- Torque converter lockup status
- Line pressure modulation
- Shift solenoid status grid

#### 6.4 Body Systems Monitor
**BCM Diagnostics:**
- Battery voltage under load conditions
- Charging system performance
- Individual door lock feedback
- Window motor current draw
- Lighting circuit status
- HVAC blower performance

#### 6.5 Safety Systems Status
**ABS/ESC Module:**
- Individual wheel speed sensors
- Brake pressure distribution
- Yaw rate and lateral acceleration
- Electronic stability interventions
- Brake fluid level and pad wear

**Airbag System (RCM):**
- Crash sensor readings
- Seat occupancy detection
- Seatbelt status monitoring
- Airbag squib continuity

#### 6.6 Advanced Driver Assistance
**Parking Aid (PAM):**
- Ultrasonic sensor ranges
- Backup camera status
- Object detection confidence
- Parking trajectory visualization

#### 6.7 Comfort & Climate Control
**HVAC Diagnostics:**
- Multi-zone cabin temperatures
- Blend door positions vs commands
- Refrigerant pressure monitoring
- Air quality sensors
- Climate learning algorithms

#### 6.8 Communication & Infotainment
**SYNC/APIM Module:**
- GPS signal strength and satellite count
- Bluetooth connection quality
- Cellular modem signal strength
- Wi-Fi hotspot device management
- Software version tracking

#### 6.9 Vehicle Analytics & History
**Performance Tracking:**
- Engine operating hours by RPM range
- Fuel consumption patterns
- Brake application frequency
- Acceleration/deceleration patterns
- Trip data visualization
- Cold start frequency

#### 6.10 System Health Monitoring
**Diagnostic Infrastructure:**
- CAN bus communication errors
- Module temperature monitoring
- Power supply voltage to modules
- Memory usage in modules
- Software corruption detection
- Calibration drift tracking

#### 6.11 Technical Requirements
- **Data Collection:** Extend CommandExecutor with Ford-specific diagnostic commands
- **UI/UX:** Responsive layout with color-coded status indicators and trend visualization
- **Configuration:** YAML profiles for different diagnostic scenarios
- **Error Handling:** Graceful degradation and automatic reconnection
- **Performance:** Real-time updates with data persistence and export functionality

## Testing Strategy

- **Unit tests** with mocks at each layer
- **ISO-TP property tests** using proptest for payloads 1-4095 bytes
- **Integration tests** with mock ELM327 responses
- **Hardware-in-the-loop** testing on real vehicle
- **Test fixtures** from real bus traces in `tests/fixtures/`

## Key Dependencies

- `serialport` / `btleplug` - Hardware I/O
- `tokio` - Async runtime
- `nom` - Hex/AT response parsing
- `clap` - CLI interface
- `serde` + `serde_yaml` - Configuration profiles
- `thiserror` / `anyhow` - Error handling
- `tracing` + `tracing-subscriber` - Logging/observability

## Rust Project Guidelines

### Core Style and Formatting
- Always run `cargo fmt` before committing
- Follow standard Rust naming conventions (snake_case for functions/variables, PascalCase for types)
- Use idiomatic Rust patterns and avoid unnecessary clones
- Maximum line length: 100 characters (configured in rustfmt.toml)

### Error Handling
- Use `Result<T, E>` for operations that can fail
- Use `anyhow` or `thiserror` for error handling (this project uses both - thiserror in library, anyhow in CLI)
- Avoid unwrap() and expect() in production code except in tests or when panic is truly the correct behavior
- Provide meaningful error messages with context
- Use `?` operator for error propagation

### Documentation
- Add doc comments (///) for all public items
- **DO NOT use comments for complex functions** - keep code as clean as possible
- Run `cargo doc` to verify documentation builds correctly
- Add module-level documentation explaining the purpose of each module

### Testing
- Write unit tests for all public functions
- Place unit tests in a `tests` module at the bottom of each file
- Integration tests go in the `tests/` directory
- Run `cargo test` before committing
- Aim for meaningful test coverage, not just high percentages
- Use test fixtures defined in tests/fixtures.rs
- Mock external services in tests

### Dependencies and Features
- Keep dependencies minimal and well-justified
- Use specific version requirements, avoid wildcards
- Enable only necessary feature flags
- Run `cargo clippy` to catch common mistakes
- Run `cargo check` frequently during development

### Architecture Patterns
- Keep handlers thin - business logic belongs in services
- Use dependency injection via constructor parameters
- Main logic should be in lib.rs, not main.rs (for CLI tools)
- Keep main.rs minimal - just CLI setup and calling lib functions

### Async/Concurrency
- Use tokio runtime for async operations
- All I/O operations must be async
- Use Arc<Mutex<T>> sparingly - prefer message passing with channels
- Don't block the runtime - use spawn_blocking for CPU-intensive work

### Security
- Validate and sanitize all user input
- Use parameterized queries to prevent SQL injection
- Hash passwords with bcrypt or argon2
- Use secure random for tokens (rand crate with OsRng)

### Before Committing Checklist
1. `cargo fmt`
2. `cargo clippy -- -D warnings`
3. `cargo test --all-features`
4. `cargo doc --no-deps` (verify docs build)
5. Check for unused dependencies with cargo-udeps

## Makefile Best Practices

### Target Naming
- **Namespace all targets** using `/` as delimiter (e.g., `docker/build`)
- **Never use `:` in target names** - breaks dependencies
- Group targets by namespace into separate files (e.g., `tasks/Makefile.docker`)

### Organization
- Use `include` for modular Makefiles
- **Avoid using $(eval ...)** - leads to confusing execution paths
- Use `?=` to set default variable values
- Keep targets small and focused - delegate complex logic to shell scripts

### Standard Targets
Every project should have: `have`, `deps`, `build`, `test`, `clean`

### Best Practices
- Use target dependencies for prerequisites
- Declare non-file targets as `.PHONY`
- Use `@` prefix to suppress command echo for clean output
- Implement a self-documenting help target using `##` comments
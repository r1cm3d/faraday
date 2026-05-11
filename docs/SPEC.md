# faraday — Technical Specification

**Project:** Rust CLI for diagnostics and configuration of the Ford Fusion 2017 SEL via OBD-II
**Author:** r1cm3d
**Status:** Draft v0.1
**Last updated:** 2026-05-10

---

## 1. Overview

`faraday` is a command-line tool written in Rust for communicating with the ECU and other electronic modules of the Ford Fusion 2017 SEL (Brazilian/Mercosur market, assembled in Hermosillo) through a FORScan-compatible OBD-II adapter (HS-CAN + MS-CAN).

The name pays tribute to Michael Faraday, whose work on electromagnetism underpins all the electrical/CAN communication in the vehicle. The "Faraday cage" metaphor also resonates with the philosophy of Section 6: layered protections against bricking modules.

The tool covers three classes of operation:

1. **Standard diagnostics** — reading DTCs, live data, and vehicle information via SAE J1979.
2. **Proprietary reads** — reading "as-built" data and Ford-specific DIDs via UDS (ISO 14229).
3. **Configuration writes** — modifying as-built blocks with Security Access (Service 27), with mandatory snapshots/rollback.

### 1.1 Goals

- Partially replace FORScan for scriptable, version-controllable operations.
- Serve as a deep exercise in automotive protocols, ISO-TP and UDS implemented from scratch.
- Produce a reusable Rust library (`faraday-core`) decoupled from the CLI.
- Enable version-controlled YAML configuration profiles (config-as-code for the vehicle).

### 1.2 Non-goals

- Support for makes/models other than the Ford Fusion 2017. The architecture allows future extension, but the initial scope is strictly this vehicle.
- Module firmware reprogramming/flashing (UDS Modes 34/36). High risk, low reward, out of scope.
- Support for legacy protocols (ISO 9141-2, KWP2000). The 2017 Fusion is CAN-only.
- GUI. The presentation layer is strictly CLI/TUI.

---

## 2. Vehicle Technical Context

### 2.1 Bus Architecture

| Bus | Speed | OBD-II Pins | Main modules |
|-----|-------|-------------|--------------|
| HS-CAN | 500 kbps | 6 (CAN-H), 14 (CAN-L) | PCM, TCM, ABS, RCM, PSCM |
| MS-CAN | 125 kbps | 3 (CAN-H), 11 (CAN-L) | BCM, IPC, APIM (SYNC), HVAC, DSM, PAM |

### 2.2 Header Table (11-bit CAN Addresses)

| Header (req) | Header (resp) | Module | Bus |
|--------------|---------------|--------|-----|
| `7E0` | `7E8` | PCM (Powertrain Control Module) | HS-CAN |
| `7E1` | `7E9` | TCM (Transmission Control Module) | HS-CAN |
| `7E2` | `7EA` | ABS | HS-CAN |
| `7E3` | `7EB` | RCM (Restraints Control / Airbag) | HS-CAN |
| `726` | `72E` | BCM (Body Control Module) | MS-CAN |
| `720` | `728` | IPC (Instrument Panel Cluster) | MS-CAN |
| `7D0` | `7D8` | APIM (SYNC) | MS-CAN |
| `733` | `73B` | PAM (Parking Aid Module) | MS-CAN |
| `727` | `72F` | DSM (Driver Seat Module) | MS-CAN |
| `7DF` | varies | Functional broadcast (J1979) | HS-CAN |

### 2.3 OBD-II Adapter

**Primary Target:** Vgate vLinker FS (USB/Bluetooth variants) — FORScan-recommended adapter with automatic HS-CAN/MS-CAN switching.

**Key specifications:**
- 32-bit processor with up to 3Mbps transmission speeds
- Automatic electronic HS-CAN/MS-CAN switching (no manual toggle)
- STN1170/STN2120 chipset (proprietary commands, not standard ELM327 AT)
- FEPS 18V programming voltage support
- USB serial and Bluetooth variants available

**Alternative compatible adapters:** OBDLink EX, ELS27 with STN chips. Host communication via USB serial (Phase 1) or Bluetooth (later phases).

---

## 3. Software Architecture

### 3.1 Layers

```
┌──────────────────────────────────────────────────────────┐
│  CLI (clap) — faraday                                     │
│  Commands: read-dtc, live, asbuilt-read, asbuilt-write…  │
├──────────────────────────────────────────────────────────┤
│  Command layer — faraday-core::commands                   │
│  High-level operations: ReadDTCs, ReadAsBuilt, …          │
├──────────────────────────────────────────────────────────┤
│  Protocol layer — faraday-core::protocol                  │
│  J1979 (standard OBD-II) + UDS (ISO 14229)                │
├──────────────────────────────────────────────────────────┤
│  Transport layer — faraday-core::transport                │
│  ISO-TP (ISO 15765-2) over CAN frames                     │
├──────────────────────────────────────────────────────────┤
│  Link layer — faraday-core::link                          │
│  ELM327 AT commands  |  SocketCAN (Linux)                 │
├──────────────────────────────────────────────────────────┤
│  Physical I/O — serialport / btleplug / socketcan         │
└──────────────────────────────────────────────────────────┘
```

### 3.2 Workspace Crates

```
faraday/                    # Workspace root
├── Cargo.toml              # [workspace]
├── crates/
│   ├── faraday-core/       # Lib: link + transport + protocol + commands
│   ├── faraday-cli/        # Bin: CLI (clap), binary name `faraday`
│   ├── faraday-asbuilt/    # Lib: as-built blocks catalog (data-only)
│   └── faraday-tui/        # Optional bin: live data viewer (ratatui)
└── SPEC.md
```

### 3.3 Expected External Dependencies

| Crate | Use | Layer |
|-------|-----|-------|
| `serialport` | Serial I/O with USB ELM327 | Physical |
| `btleplug` | Bluetooth I/O with BT ELM327 | Physical |
| `socketcan` | Native Linux CAN I/O (future) | Physical |
| `tokio` | Async runtime | Cross-cutting |
| `nom` | Hex/AT response parser | Link |
| `clap` | CLI parsing | CLI |
| `serde` + `serde_yaml` | Configuration profiles | CLI |
| `ratatui` | Live data TUI | TUI |
| `thiserror` / `anyhow` | Error handling | Cross-cutting |
| `tracing` + `tracing-subscriber` | Observability | Cross-cutting |

### 3.4 Design Principles

- **Transport-agnostic protocol.** The protocol layer (J1979/UDS) takes an `IsoTpTransport` trait and knows nothing about ELM327 or SocketCAN. Enables mock-based testing and migration to SocketCAN without rewriting the protocol.
- **Async-first.** `tokio::io::AsyncRead/Write` for serial. Tester Present runs in the background as a separate task.
- **Errors are data.** `thiserror` in the library, `anyhow` in the CLI. UDS NRCs (Negative Response Codes) are first-class types, not strings.
- **No `unwrap` in production code.** Only in tests and on invariants proven by construction.
- **Zero-copy where it makes sense.** ISO-TP frame parsing uses slices, not Vec.
- **Observability from day one.** Every CAN frame sent/received logged at `tracing::trace!`. Protocol decisions at `debug!`.

---

## 4. Phased Roadmap

### Phase 1 — Read-only HS-CAN, standard OBD-II

**Scope:** read DTCs, live data, and VIN without writing anything to the vehicle.

**Deliverables:**
- `faraday-core` crate with `link::elm327` and `protocol::j1979` modules.
- ISO-TP single-frame only (up to 7 bytes of payload).
- Implemented services: Mode 01 (live), Mode 03 (DTCs), Mode 04 (clear), Mode 07 (pending), Mode 09 (VIN/CalID), Mode 0A (permanent).
- CLI commands: `faraday read-dtc`, `faraday clear-dtc`, `faraday live <pids>`, `faraday vin`.
- DTC parser (2 bytes → P/C/B/U + 4 hex digits).
- Initial Mode 01 PID catalog: `04`, `05`, `0C`, `0D`, `0F`, `10`, `11`, `2F`, `42`, `46`, `5C`.
- Test suite with mock transport.

**Acceptance criteria:** run `faraday live 0C,0D,05` in a 5Hz loop on the real vehicle for 10 minutes without errors.

### Phase 2 — UDS basics, full ISO-TP, DID reads

**Scope:** speak UDS to any HS-CAN module and read arbitrary DIDs.

**Deliverables:**
- Full multi-frame ISO-TP (First Frame, Consecutive Frames, Flow Control).
- UDS services implemented: 10 (DiagnosticSessionControl), 22 (ReadDataByIdentifier), 3E (TesterPresent).
- Background TesterPresent task every 2s while extended session is active.
- NRC (negative response code) parser.
- CLI: `faraday read-did --module pcm 0xF190`, `faraday session --module pcm extended`.

**Acceptance criteria:** read DID `F190` (VIN) from the PCM via UDS (not via Mode 09) successfully. Hold an extended session for 60s without timeout.

### Phase 3 — MS-CAN + as-built reads ✅ Complete

**Scope:** read configuration from BCM and IPC.

**Deliverables:**
- MS-CAN switching support in the link layer (adapter-specific AT command).
- `faraday-asbuilt` crate with mapping of known BCM/IPC blocks for the 2017 Fusion (data tables in Rust or embedded TOML).
- Bit decoder → semantic features (e.g. byte 3 bit 2 of block 726-01 → DRL enable).
- CLI: `faraday asbuilt dump --module bcm`, `faraday asbuilt show --module bcm --feature drl`.
- Output formats: raw hex, structured YAML, diff between dumps.

**Acceptance criteria:** a full BCM dump produces readable, idempotent YAML (re-dumping yields a byte-identical file).

### Phase 4 — Security Access + Write ✅ Complete

**Scope:** modify configurations safely.

**Deliverables:**
- Ford seed→key algorithm in `faraday-core::protocol::seed_key` — XOR mask `0xB3CA_4057` for configuration access level 0x01 (requires hardware validation, see §9).
- UDS Service 27 (SecurityAccess) integrated into `CommandExecutor::write_asbuilt_block`.
- UDS Service 2E (WriteDataByIdentifier) with `security_access_granted` guard.
- `AsBuiltEncoder` in `faraday-asbuilt::encoder` — bit-level inverse of `AsBuiltDecoder`.
- Snapshot persistence in `faraday-asbuilt::snapshot` (serde_json, save/load).
- **Mandatory snapshot before any write** — auto-saved to `~/.local/share/faraday/snapshots/`.
- Audit logging in `~/.local/share/faraday/audit.jsonl` (JSONL, one entry per operation).
- `faraday asbuilt restore <snapshot>` command for rollback.
- Dry-run mode (`--dry-run`) — shows diff and logs `dry_run: true` without writing.
- Interactive confirmation prompt (`--yes` to skip, for CI/scripts).
- CLI: `faraday asbuilt write --module bcm --feature drl_enabled --value true`.
- CLI: `faraday asbuilt snapshot --module bcm [--output <path>]`.
- Programming DID guard — rejects writes to `F0xx`/`F1xx` DIDs at the command layer.

**Acceptance criteria:** enable a reversible feature in a controlled environment (e.g. DRL enable/disable on BCM), validate the physical behavior, then restore from snapshot via `faraday asbuilt restore`.

#### 4.1 Seed→Key Algorithm

The Ford configuration-access seed→key algorithm for access level 0x01 (4-byte seed):

```
key = seed XOR 0xB3CA_4057
```

Implemented in `crates/faraday-core/src/protocol/seed_key.rs`. The XOR mask is the
commonly documented value for Ford Fusion configuration access found in FORScan/CyanLabs
community research. **Requires hardware validation** against a real ECU before trusting
for production writes.

#### 4.2 Snapshot Format

Snapshots are JSON files written by `faraday-asbuilt::snapshot::save_snapshot`.
Default path: `~/.local/share/faraday/snapshots/<module>_<timestamp>.json`.

```json
{
  "timestamp": "2026-05-10T14:30:00Z",
  "vehicle_vin": "3FADP0L33HR123456",
  "blocks": [
    {
      "id": { "module": "726", "id": "01" },
      "description": "BCM Configuration Block 01 - Lighting and DRL",
      "did": 1793,
      "data": [0, 0, 0, 4, 0, 0, 0, 0],
      "features": []
    }
  ]
}
```

#### 4.3 Audit Log Schema

One JSON object per line in `~/.local/share/faraday/audit.jsonl`:

```json
{"timestamp":"2026-05-10T14:30:05Z","operation":"write","module":"Bcm","did":1793,"before_hex":"0000000000000000","after_hex":"0000000400000000","dry_run":false,"result":"ok"}
```

Fields: `timestamp` (ISO 8601), `operation` (`write`|`restore`), `module` (Debug name),
`did` (decimal), `before_hex`, `after_hex` (uppercase hex), `dry_run` (bool),
`result` (`ok` | `error: <message>`).

### Phase 5 — Polish and ergonomics

**Deliverables:**
- Live data TUI with `ratatui` (gauges, RPM/speed sparklines). ✅
- Versioned YAML profiles: `faraday profile apply my-fusion.yml`. ✅
- Structured session logging in JSONL (`~/.local/share/faraday/sessions.jsonl`). ✅

### Phase 6 — Comprehensive Hidden Diagnostic TUI

**Scope:** Transform `faraday-tui` from a basic 5-PID viewer into a professional multi-tab diagnostic interface covering all vehicle systems reachable via HS-CAN and MS-CAN.

**Motivation:** The Phase 5 TUI exposes fewer than 5% of the diagnostic information the Ford Fusion 2017 makes available. Phase 6 closes this gap, matching the diagnostic depth of tools like FORScan in a scriptable, keyboard-driven terminal interface.

#### 6.1 Multi-Tab Interface Architecture

Replace the single-view layout with a tab bar at the top of the screen. Each tab owns a dedicated `Panel` type that carries its own data model and rendering logic.

**Tab index and key bindings:**

| Key | Tab |
|-----|-----|
| `1` | Engine & Powertrain |
| `2` | Transmission |
| `3` | Body Systems (BCM) |
| `4` | Safety (ABS/RCM) |
| `5` | ADAS / Parking |
| `6` | Climate (HVAC) |
| `7` | Infotainment (APIM) |
| `8` | Vehicle Analytics |
| `9` | System Health |
| `←`/`→` | Cycle tabs |
| `p` | Pause/resume data collection |
| `r` | Reset history buffers |
| `e` | Export current tab data to JSONL |
| `?` | Toggle context-sensitive help overlay |
| `q`/`Esc` | Quit |

**Status bar (always visible):** connection state · active tab · data-point count · current update rate · battery voltage (PID `0x42`).

#### 6.2 Extended PID Catalog

Phase 6 adds the following standard OBD-II PIDs to `faraday-core::protocol::j1979`:

| PID | Name | Unit | Formula |
|-----|------|------|---------|
| `0x06` | Short-term fuel trim bank 1 | % | `(A − 128) × 100 / 128` |
| `0x07` | Long-term fuel trim bank 1 | % | `(A − 128) × 100 / 128` |
| `0x08` | Short-term fuel trim bank 2 | % | `(A − 128) × 100 / 128` |
| `0x09` | Long-term fuel trim bank 2 | % | `(A − 128) × 100 / 128` |
| `0x0E` | Timing advance | ° before TDC | `A / 2 − 64` |
| `0x13` | O2 sensors present | bitmask | — |
| `0x14` | O2 bank 1 sensor 1 voltage | V / % | `A × 0.005` / `(B − 128) × 100/128` |
| `0x15` | O2 bank 1 sensor 2 voltage | V / % | same |
| `0x2C` | EGR commanded | % | `A × 100 / 255` |
| `0x2D` | EGR error | % | `(A − 128) × 100 / 128` |
| `0x44` | Fuel-air equivalence ratio | λ | `(256A + B) × 2 / 65536` |
| `0x4D` | Engine runtime with MIL on | min | `256A + B` |
| `0x4E` | Engine runtime since codes cleared | min | `256A + B` |
| `0x5A` | Relative throttle position | % | `A × 100 / 255` |
| `0x5E` | Engine fuel rate | L/h | `(256A + B) × 0.05` |
| `0x61` | Driver demand torque | % | `A − 125` |
| `0x62` | Actual engine torque | % | `A − 125` |

All new PIDs follow the same `Pid(u8)` newtype pattern and extend `interpret_value`/`get_pid_data_length` in `j1979.rs`.

#### 6.3 Ford-Specific UDS DID Queries

Many Phase 6 panels require module-specific reads via UDS service `0x22`. These are issued through `CommandExecutor::read_did`, already implemented in Phase 2. The new work is organizing queries into per-panel fetch functions.

Representative DIDs (addresses subject to hardware validation):

| Module | DID | Description |
|--------|-----|-------------|
| TCM `7E1` | `0xDD01` | Transmission fluid temperature |
| TCM `7E1` | `0xDD02` | Current gear / commanded gear |
| TCM `7E1` | `0xDD03` | Torque converter slip |
| BCM `726` | `0x4001` | Battery voltage under load |
| BCM `726` | `0x4002` | Alternator duty cycle |
| BCM `726` | `0x4010` | Door ajar bitmask |
| ABS `7E2` | `0xC001` | Wheel speed FL/FR/RL/RR |
| ABS `7E2` | `0xC002` | Yaw rate · lateral acceleration |
| PAM `733` | `0xE001` | Ultrasonic sensor distances (8 sensors) |
| HVAC | `0xB001` | Blend door positions |
| HVAC | `0xB002` | Evaporator temp · refrigerant pressure |
| APIM `7D0` | `0xA001` | GPS fix quality / satellite count |
| APIM `7D0` | `0xA002` | Cellular RSSI |
| APIM `7D0` | `0xA010` | Module software version string |
| PCM `7E0` | `0x1900` | Engine oil life percentage |
| PCM `7E0` | `0x1901` | Individual cylinder misfire counters |

**Note:** All DID addresses marked with module addresses above require hardware validation on the real vehicle. The values listed match community-documented FORScan DIDs but may differ in firmware revisions.

#### 6.4 Panel Specifications

**Tab 1 — Engine & Powertrain**

Metrics: RPM · speed · coolant temp · engine load · throttle position · MAF · intake air temp · fuel trim (STFT/LTFT both banks) · timing advance · O2 sensor voltages · EGR command/error · fuel-air ratio · fuel rate · engine torque · oil temp · oil life · cylinder misfire counters.

Layout: left column gauges (RPM/speed/load) + right column sparklines (fuel trim trend) + bottom grid for misfire counters per cylinder.

**Tab 2 — Transmission**

Metrics (via TCM UDS DIDs): fluid temperature · current gear vs. commanded gear · torque converter slip % · line pressure · shift solenoid status grid.

Layout: top row gauges + solenoid status grid (color-coded: green = energised, gray = off) + fluid temp sparkline.

**Tab 3 — Body Systems (BCM)**

Metrics (via BCM UDS DIDs): battery voltage under load · alternator duty cycle · door ajar bitmask (individual door indicators) · window motor status · lighting circuit health · HVAC blower actual vs. commanded.

Layout: battery/charging gauges + door diagram (ASCII art with per-door status) + table for lighting circuits.

**Tab 4 — Safety Systems (ABS / RCM)**

Metrics (via ABS/RCM UDS DIDs): wheel speeds (FL/FR/RL/RR) · yaw rate · lateral acceleration · steering angle · stability intervention counter · airbag squib continuity bitmask · seatbelt bitmask.

Layout: four-corner wheel speed display + yaw/lateral sparklines + safety bitmask grid.

**Tab 5 — ADAS / Parking**

Metrics (via PAM UDS DIDs): eight ultrasonic sensor distances (front/rear four each) + backup camera status + object detection confidence.

Layout: top-down vehicle silhouette (ASCII) with distance bars radiating from front/rear bumpers.

**Tab 6 — Climate Control (HVAC)**

Metrics (via HVAC UDS DIDs): driver/passenger cabin temps · blend door actual vs. commanded % · evaporator temp · refrigerant pressure · AC compressor load · blower actual speed.

Layout: two-zone temperature display + pressure gauge + blend door bars.

**Tab 7 — Infotainment (APIM)**

Metrics (via APIM UDS DIDs): GPS fix quality · satellite count · cellular RSSI · Bluetooth device count · software version string.

Layout: signal strength indicators + version table.

**Tab 8 — Vehicle Analytics**

In-session computed metrics: RPM histogram (idle / cruise / high-load bands) · speed distribution · estimated fuel consumption (from PID `0x5E`) · brake events (inferred from speed drops > 10 km/h/s) · acceleration events (> 0.3 g).

Persistence: appended to `~/.local/share/faraday/analytics.jsonl` when the TUI exits.

**Tab 9 — System Health**

Cross-module status table: for each known module — last response time · consecutive timeout count · reported voltage (if available) · software version. Serves as a connectivity overview.

#### 6.5 Data Architecture in `faraday-tui`

```
faraday-tui/src/
├── main.rs          # CLI args, terminal setup, event loop (tab navigation added)
├── app.rs           # App struct owns all panel states + ActiveTab enum
├── ui.rs            # draw() dispatches to per-tab render fn
├── panels/
│   ├── mod.rs
│   ├── engine.rs    # EnginePanel data + render
│   ├── transmission.rs
│   ├── body.rs
│   ├── safety.rs
│   ├── adas.rs
│   ├── climate.rs
│   ├── infotainment.rs
│   ├── analytics.rs
│   └── health.rs
└── widgets/
    ├── mod.rs
    ├── sparkline_ext.rs   # Multi-series sparkline helper
    ├── status_grid.rs     # Bitmask → colored cell grid
    └── vehicle_diagram.rs # ASCII vehicle top-down + distance bars
```

Each panel implements a `Panel` trait:

```rust
pub trait Panel {
    fn title(&self) -> &str;
    fn update(&mut self, executor: &mut CommandExecutor<impl IsoTpTransport>) -> impl Future<Output = ()>;
    fn render(&self, f: &mut Frame, area: Rect);
    fn help_text(&self) -> &str;
}
```

The `App` struct holds `Vec<Box<dyn Panel>>` and an `active_tab: usize` index. The event loop calls `panels[active_tab].update()` every tick and `ui::draw()` on every frame.

#### 6.6 Polling Strategy

Not all data changes at the same rate. The update loop uses per-panel intervals:

| Panel | Interval |
|-------|----------|
| Engine (PIDs) | 250 ms |
| Transmission (UDS) | 500 ms |
| Body, Safety, ADAS, Climate | 1 s |
| Infotainment, Analytics, Health | 5 s |

Background tokio tasks collect data for non-active tabs at their native interval so switching tabs shows fresh data immediately.

#### 6.7 Graceful Degradation

If a module does not respond within one timeout:
- The panel shows `[N/A]` in place of unavailable fields.
- The System Health tab marks the module as `TIMEOUT`.
- Collection retries with exponential back-off (1 s → 2 s → 4 s, capped at 30 s).
- No panic. The TUI remains fully operational for responding modules.

#### 6.8 Acceptance Criteria

- All 9 tabs navigate correctly with number keys and arrow keys.
- Engine tab displays ≥ 15 distinct metrics in real-time.
- Switching tabs shows data for the new tab within one tick of its polling interval.
- A module that stops responding degrades gracefully without crashing the TUI.
- Vehicle Analytics tab writes a valid JSONL record on exit.
- All new PIDs round-trip correctly in the existing mock-transport unit tests.

---

## 5. Supported UDS Command Model

| Service ID | Name | Phase | Risk |
|------------|------|-------|------|
| `10` | DiagnosticSessionControl | 2 | Low |
| `11` | ECUReset | — | Medium (not implemented initially) |
| `14` | ClearDiagnosticInformation | 1 (via Mode 04) | Low |
| `19` | ReadDTCInformation | 2 | Low |
| `22` | ReadDataByIdentifier | 2 | Low |
| `27` | SecurityAccess | 4 | High |
| `2E` | WriteDataByIdentifier | 4 | **Critical** |
| `2F` | InputOutputControlByIdentifier | — | Medium (out of scope) |
| `31` | RoutineControl | — | Variable (out of scope) |
| `3E` | TesterPresent | 2 | None |

---

## 6. Safety Considerations

### 6.1 Brick Risk

Incorrect writes to as-built data can leave modules in an unusable state. Mandatory mitigations before Phase 4:

- Full snapshot of the target module before any write.
- Validation that the byte being written belongs to a known block (lookup in `faraday-asbuilt`).
- Block writes to programming DIDs (`F1xx`, `F0xx` reserved by Ford).
- `--dry-run` mode active by default; real writes require explicit `--commit`.
- Logs persisted in `~/.local/share/faraday/audit.jsonl`.

### 6.2 Bus Off

Malformed CAN frames can cause Bus Off in modules. Mitigations:

- Every ISO-TP frame construction validates size and PCI byte before sending.
- Rate limiting at the link layer: max 100 frames/second on send.
- Detection of transmission errors via ELM327 responses (`CAN ERROR`, `BUS BUSY`) with exponential backoff.

### 6.3 Operation with Engine Running

Writes should only occur with:

- Engine off, ignition in KOEO (Key On Engine Off).
- Battery voltage ≥ 12.4V (verifiable via PID `42`).
- No active communication DTC (offline module).

The CLI validates these preconditions and aborts if they aren't met, except with `--force`.

---

## 7. Profile Model (Phase 5)

Example YAML profile:

```yaml
# my-fusion.yml
vehicle:
  vin: "3FA6P0H7XHR123456"
  model: "Fusion 2017 SEL"

modules:
  bcm:
    drl_enabled: true
    auto_lock_on_drive: true
    unlock_beeps: 2
  ipc:
    show_digital_speedometer: true
    welcome_animation: true
```

Commands:

- `faraday profile validate my-fusion.yml` — checks every module/feature/value against the known block catalog without connecting to a vehicle.
- `faraday profile apply my-fusion.yml` — validates, then writes each feature in order; auto-snapshots each affected block and appends audit log entries.
- `faraday profile apply my-fusion.yml --dry-run` — shows the diff without writing.

The CLI resolves each semantic feature to the byte/bit in the corresponding as-built block, snapshots, writes, and validates.

### Session Logging

Every CLI invocation appends one JSON object to `~/.local/share/faraday/sessions.jsonl`:

```json
{"timestamp":"2026-05-10T14:30:05Z","command":"profile apply examples/my-fusion.yml","adapter":"/dev/ttyUSB0","duration_ms":1243,"result":"ok"}
```

Fields: `timestamp` (ISO 8601), `command` (human-readable label), `adapter` (device path), `duration_ms` (wall time), `result` (`ok` | `error: <message>`). Session logging failures are non-fatal warnings.

---

## 8. Testing

### 8.1 Layered Strategy

- **Unit tests** at each layer with mocks of the layer below.
- **ISO-TP property tests** with `proptest`: round-trip of payloads from 1 to 4095 bytes.
- **Integration tests** with a mock ELM327 simulating realistic CAN responses.
- **Hardware-in-the-loop** (manual, not CI): full test battery on the real vehicle at the end of each phase.

### 8.2 Fixtures

Capture real bus traces from the vehicle and store them in `tests/fixtures/` for regression testing.

---

## 9. Implementation Decisions

**Adapter Model:** Confirmed vLinker FS as primary target adapter. Uses proprietary STN commands for MS-CAN switching rather than standard ELM327 AT commands.

**Phase 1 Communication:** USB serial only. Bluetooth support deferred to later phases for reduced complexity.

**Remaining Open Questions:**
- 2017 Fusion seed→key algorithm: the XOR mask `0xB3CA_4057` is implemented but requires hardware validation on a real ECU before trusting for production writes. A captured seed/key pair from the vehicle would allow a definitive unit test.
- Versioning policy for `faraday-asbuilt`: data is derived from community reverse engineering — attribution and licensing.

---

## 10. References

- ISO 15765-2:2016 — Road vehicles — Diagnostic communication over CAN — Network layer.
- ISO 14229-1:2020 — Road vehicles — UDS — Specification and requirements.
- SAE J1979 — E/E Diagnostic Test Modes.
- ELM Electronics — ELM327DSJ datasheet.
- CyanLabs Forum — Ford as-built community documentation.
- FORScan Forum — DID and module documentation.

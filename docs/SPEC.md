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

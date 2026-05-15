# ADR-007: PTY-Based `faraday-emu` for Hardware-Free Integration Testing

**Status:** Accepted

---

## Context

Testing `faraday` end-to-end requires a physical OBD-II adapter connected to a running
vehicle. This makes automated and developer-workstation testing impractical. A software
substitute that allows `VLinkerFs` to connect via a real serial path (not a mock) would
enable full-stack testing without hardware.

Alternatives considered:

**Option A — Mock `LinkLayer` only:** Replace `VLinkerFs` with a `MockLinkLayer` in tests.
Tests the protocol and transport layers but not the AT-command parsing in `VLinkerFs`.

**Option B — PTY-based emulator crate:** Create a separate binary (`faraday-emu`) that
opens a PTY (pseudo-terminal), exposes it as `/tmp/faraday-dev`, and responds to the
STN1170 AT-command protocol. `VLinkerFs` connects to `/tmp/faraday-dev` identically to
how it would connect to `/dev/ttyUSB0`.

---

## Decision

Implement **Option B — `faraday-emu` PTY emulator**. The crate uses the `nix` crate for
PTY creation (`nix::pty::openpty`), creates a symlink at `/tmp/faraday-dev`, and runs an
`async` handler loop that parses incoming STN AT commands and replies with realistic
simulated ECU data (VIN, RPM, DTCs, as-built blocks).

The `Makefile` provides `make tui/emu` which starts `faraday-emu` in the background and
connects `faraday-tui` to the emulator PTY.

`faraday-emu` has its own inline unit tests for `ecu.rs` (data generation) and
`handler.rs` (AT-command parsing).

---

## Consequences

**Positive:**
- The complete `VLinkerFs` → `IsoTp` → protocol → command stack is exercisable without
  any hardware. All AT-command parsing and frame construction in `VLinkerFs` is tested.
- Realistic simulated data (varying RPM, temperatures, DTCs) makes TUI panel rendering
  testable visually.
- `faraday-emu` is a first-class crate in the workspace — it builds, lints, and tests as
  part of `cargo test --all-features`.

**Negative:**
- The emulator is Linux-only (`nix::pty` requires POSIX PTY support; macOS support is
  partial, Windows has no PTY equivalent).
- Emulated ECU behaviour is hand-crafted — it may not reflect real Ford Fusion protocol
  quirks (e.g., specific NRC codes, timing constraints, bus arbitration delays).
- The PTY symlink at `/tmp/faraday-dev` is a global resource; running multiple emulator
  instances simultaneously would conflict.

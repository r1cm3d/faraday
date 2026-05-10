# Phase 3 Manual Validation Guide

This guide covers two tracks for validating `asbuilt dump` and `asbuilt show`.

---

## Track A — Simulator (faraday-emu)

### Prerequisites

```bash
cargo build
```

### Steps

**1. Start the emulator.** It prints the PTY slave path on stdout.

```bash
cargo run -p faraday-emu
# Output example: Listening on /dev/pts/3
```

Keep it running and open a second terminal for the steps below.

**2. Dump BCM as-built configuration.**

```bash
faraday --adapter /dev/pts/3 asbuilt dump --module bcm
```

Expected output (YAML):

```yaml
blocks:
  - id: 726-01
    description: BCM Configuration Block 01 - Lighting and DRL
    did: '0x0701'
    data: 0000000004030000
    features:
      - name: drl_enabled
        description: Daytime Running Lights Enable
        value: DRL Enabled
        raw: 1
      - name: auto_headlights
        description: Automatic Headlight Control
        value: Manual Headlights Only
        raw: 0
      - name: welcome_lighting
        description: Welcome Lighting Duration
        value: 3 seconds
        raw: 3
  - id: 726-02
    ...
```

**3. Show a single feature.**

```bash
faraday --adapter /dev/pts/3 asbuilt show --module bcm --feature drl_enabled
```

Expected:

```
drl_enabled: DRL Enabled (raw: 1)
```

**4. Dump IPC configuration.**

```bash
faraday --adapter /dev/pts/3 asbuilt dump --module ipc
```

Expected: blocks `720-01` and `720-02`. Block `720-01` must show `show_digital_speedometer: Digital Speedometer Shown`. Block `720-02` must show `welcome_animation: Animation Enabled` and `gauge_brightness: Level 6`.

**5. Verify YAML idempotency (acceptance criterion from SPEC.md).**

```bash
faraday --adapter /dev/pts/3 asbuilt dump --module bcm > a.yaml
faraday --adapter /dev/pts/3 asbuilt dump --module bcm > b.yaml
diff a.yaml b.yaml
echo "Exit code: $?"   # must print 0
```

**6. Confirm error on unknown feature.**

```bash
faraday --adapter /dev/pts/3 asbuilt show --module bcm --feature nonexistent
echo "Exit code: $?"   # must be non-zero
```

Expected: error message `unknown feature: nonexistent`.

**7. Confirm error for unsupported module.**

```bash
faraday --adapter /dev/pts/3 asbuilt dump --module tcm
echo "Exit code: $?"   # must be non-zero
```

Expected: error message about no as-built blocks defined.

---

## Track B — Physical Device (Ford Fusion 2017 SEL + vLinker FS)

### Prerequisites

- vLinker FS adapter plugged into OBD-II port.
  - HS-CAN: pins 6 and 14.
  - MS-CAN: pins 3 and 11.
- Ignition: KOEO (Key On Engine Off). Engine must be off.
- Battery voltage ≥ 12.4 V. Check with a multimeter if unsure.
- Adapter appears as `/dev/ttyUSB0` (or set `FARADAY_ADAPTER=/dev/ttyUSBX`).
- CLI installed: `cargo install --path crates/faraday-cli`.

### Steps

**1. Verify connectivity with a known-good command.**

```bash
faraday vin
```

Expected: 17-character VIN starting with `1FA`. If this fails, debug the adapter connection before proceeding.

**2. Dump BCM as-built over MS-CAN.**

```bash
faraday asbuilt dump --module bcm
```

Expected: YAML with blocks `726-01` and `726-02`. Raw `data` hex bytes reflect actual vehicle configuration and will differ from the simulator. Feature values (enabled/disabled) reflect the vehicle's current settings.

**3. Show DRL status.**

```bash
faraday asbuilt show --module bcm --feature drl_enabled
```

Expected: either `DRL Enabled` or `DRL Disabled`. Cross-check by looking at block `726-01` byte 3 bit 2 in the `data` hex field of the dump output.

**4. Dump IPC configuration over MS-CAN.**

```bash
faraday asbuilt dump --module ipc
```

Expected: blocks `720-01` and `720-02` with speed and temperature unit settings matching the current dashboard configuration.

**5. Verify YAML idempotency on real data.**

```bash
faraday asbuilt dump --module bcm > hw1.yaml
faraday asbuilt dump --module bcm > hw2.yaml
diff hw1.yaml hw2.yaml
echo "Exit code: $?"   # must print 0
```

**6. (Optional) Cross-validate against FORScan.**

- Open FORScan → BCM → As-Built tab.
- Read block `726-01` raw hex.
- Compare byte-for-byte against the `data` field in `hw1.yaml`. They must be identical.

### Known constraints

- **MS-CAN switching**: the vLinker FS automatically switches to the 125 kbps MS-CAN bus when communicating with BCM/IPC addresses. If reads time out, verify the adapter firmware version and that MS-CAN wiring (pins 3/11) is connected.
- **Extended session**: `read_asbuilt_block` automatically enters UDS extended diagnostic session (service `0x10 0x03`) before reading each DID. No manual session setup is needed.
- **No active DTCs**: communication DTCs on the MS-CAN bus can prevent extended session entry. Clear them with `faraday clear-dtc --module bcm` if reads fail with `Conditions not correct (0x22)`.

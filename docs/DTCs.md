# Diagnostic Trouble Codes (DTCs)

Diagnostic Trouble Codes (DTCs) are standardized codes used to identify and communicate vehicle system faults detected by the electronic control modules (ECMs). In the Ford Fusion 2017 SEL, DTCs are stored in various modules and can be read through the OBD-II port using standard SAE J1979 protocols.

## DTC Format

DTCs follow a standardized 5-character format:

- **First character** (Letter): System category
  - `P` - Powertrain (engine, transmission)
  - `B` - Body (lighting, climate, security)
  - `C` - Chassis (ABS, suspension, steering)
  - `U` - Network/Communication (CAN bus issues)

- **Second character** (Number): Code type
  - `0` - Generic (SAE standard)
  - `1` - Manufacturer specific
  - `2` - Generic (SAE standard)
  - `3` - Manufacturer specific

- **Characters 3-5** (Numbers): Specific fault identifier

### Example DTCs
- `P0171` - System Too Lean (Bank 1)
- `B1342` - ECU Internal Failure
- `C1095` - ABS Hydraulic Pump Motor Circuit Failure
- `U0100` - Lost Communication with ECM/PCM

## Reading DTCs in Faraday

DTCs can be retrieved using several SAE J1979 modes:

### Mode 03: Read Stored DTCs
```bash
faraday read-dtc
```
Returns currently stored DTCs that have triggered the Malfunction Indicator Light (MIL).

### Mode 07: Read Pending DTCs
```bash
faraday read-dtc --pending
```
Returns DTCs that have been detected but haven't yet triggered the MIL (typically require 2+ detection cycles).

### Mode 0A: Read Permanent DTCs
```bash
faraday read-dtc --permanent
```
Returns DTCs that cannot be cleared until the fault is properly repaired and the system completes its drive cycle requirements.

## Clearing DTCs

DTCs can be cleared using Mode 04:

```bash
faraday clear-dtc
```

**Warning:** Clearing DTCs also resets:
- Readiness monitors
- Freeze frame data
- O2 sensor test results
- Other diagnostic data

## Ford-Specific Considerations

### Module Distribution
In the 2017 Fusion, DTCs are distributed across multiple modules:

- **PCM** (7E0/7E8): Powertrain codes (P-codes)
- **BCM** (726/72E): Body control codes (B-codes)
- **ABS** (7E2/7EA): Chassis codes (C-codes)
- **IPC** (720/728): Instrument cluster codes
- **RCM** (7E3/7EB): Restraint system codes

### Enhanced DTCs
Ford uses enhanced DTC formats beyond standard J1979:
- Sub-DTCs for more specific fault isolation
- Occurrence counters
- Environmental data (temperature, voltage at time of fault)
- Module-specific diagnostic data

These enhanced features require UDS (ISO 14229) communication rather than standard OBD-II modes.

## Safety Considerations

- Never ignore DTCs related to safety systems (ABS, airbags, steering)
- Some DTCs may indicate conditions that could cause vehicle damage if driven
- Communication DTCs (U-codes) may indicate bus failures affecting multiple systems
- Always address the root cause rather than just clearing codes

## Related Documentation

- [UDS.md](UDS.md) - For advanced DTC reading via UDS Service 19
- [HS-CAN.md](HS-CAN.md) - Modules accessible via high-speed CAN
- [MS-CAN.md](MS-CAN.md) - Modules accessible via medium-speed CAN

## References

- SAE J1979 - E/E Diagnostic Test Modes
- ISO 14229-1 - Unified Diagnostic Services (UDS)
- Ford Service Manual - 2017 Fusion Diagnostic Procedures
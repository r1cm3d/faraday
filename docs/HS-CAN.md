# High-Speed CAN Bus (HS-CAN)

The High-Speed CAN bus in the Ford Fusion 2017 SEL operates at 500 kbps and connects critical powertrain and safety systems. HS-CAN provides the primary diagnostic interface accessible through standard OBD-II protocols and is where most emissions-related and safety-critical modules reside.

## Technical Specifications

### Physical Layer
- **Speed**: 500 kbps (500,000 bits per second)
- **Topology**: Linear bus with 120Ω termination resistors
- **Voltage**: Differential signaling (CAN-H and CAN-L)
- **OBD-II Pins**: Pin 6 (CAN-H), Pin 14 (CAN-L)
- **Wire Gauge**: Typically 22 AWG twisted pair
- **Maximum Length**: ~40 meters (limited by signal integrity at 500 kbps)

### Protocol Details
- **Frame Format**: CAN 2.0A (11-bit identifiers) and CAN 2.0B (29-bit identifiers)
- **Arbitration**: CSMA/CD with priority-based access
- **Error Detection**: CRC, frame check, acknowledgment, bit monitoring
- **Message Types**: Data frames, remote frames, error frames, overload frames

## Connected Modules

### Primary HS-CAN Modules in 2017 Fusion

| Module | Full Name | CAN ID (Req/Resp) | Primary Function |
|--------|-----------|-------------------|------------------|
| **PCM** | Powertrain Control Module | 7E0/7E8 | Engine control, emissions |
| **TCM** | Transmission Control Module | 7E1/7E9 | Transmission control |
| **ABS** | Anti-lock Braking System | 7E2/7EA | Brake control, stability |
| **RCM** | Restraints Control Module | 7E3/7EB | Airbag, seatbelt systems |
| **PSCM** | Power Steering Control Module | 7E4/7EC | Electric power steering |

### Module Characteristics

#### PCM (Powertrain Control Module) - 7E0/7E8
**Primary diagnostic module for Faraday Phase 1**

**Responsibilities:**
- Engine control (fuel injection, ignition timing, idle control)
- Emissions monitoring (O2 sensors, catalytic converter)
- Diagnostic trouble code management
- Readiness monitors for emissions testing
- Vehicle identification (VIN, calibration IDs)

**Diagnostic Capabilities:**
- Full OBD-II Mode 01-0A support
- UDS extended diagnostics
- Live data streaming (RPM, load, temperatures, pressures)
- Freeze frame data on DTC storage
- Readiness monitor status

**Faraday Access:**
```bash
# Basic PCM diagnostics
faraday read-dtc --module pcm
faraday live 0C,0D,05 --module pcm
faraday vin --module pcm

# Extended UDS access
faraday session --module pcm extended
faraday read-did --module pcm 0xF190  # VIN via UDS
```

#### TCM (Transmission Control Module) - 7E1/7E9
**Automatic transmission control and diagnostics**

**Responsibilities:**
- Shift point determination
- Torque converter lockup control
- Transmission fluid temperature monitoring
- Gear position sensing
- Adaptive learning for shift quality

**Common Diagnostic Parameters:**
- Transmission fluid temperature
- Gear selection and actual gear
- Shift solenoid status
- Torque converter slip
- Adaptive shift pressures

#### ABS (Anti-lock Braking System) - 7E2/7EA
**Brake system safety and control**

**Responsibilities:**
- Anti-lock brake control during emergency braking
- Electronic stability control (ESC)
- Traction control system (TCS)
- Brake assist functionality
- Wheel speed sensor monitoring

**Safety Considerations:**
- Never clear ABS DTCs without proper brake system inspection
- Module contains safety-critical software
- Some parameters require vehicle to be stationary

#### RCM (Restraints Control Module) - 7E3/7EB
**Passive safety systems**

**Responsibilities:**
- Airbag deployment logic
- Seatbelt pretensioner control
- Crash sensor monitoring
- Occupant detection
- Rollover detection

**Critical Safety Notes:**
- **Never attempt configuration changes** to RCM
- Contains safety-critical algorithms
- Modification could disable airbag deployment
- Requires specialized tools for any service

#### PSCM (Power Steering Control Module) - 7E4/7EC
**Electric power steering assist**

**Responsibilities:**
- Steering assist torque calculation
- Vehicle speed compensation
- Return-to-center functionality
- Diagnostic monitoring of steering system

## Bus Arbitration and Message Priority

### CAN Arbitration
Messages on HS-CAN compete for bus access using identifier-based priority:
- **Lower CAN ID = Higher Priority**
- **7DF** (Functional broadcast) has highest priority for diagnostics
- **7E0-7E7** (PCM region) has very high priority
- **Non-conflicting transmission** through CSMA/CD protocol

### Message Types on HS-CAN

#### Standard OBD-II Messages (J1979)
- **7DF → broadcast**: Functional request to all modules
- **7E0 → PCM**: Direct request to powertrain module
- **7E8 ← PCM**: Response from powertrain module

#### UDS Messages (ISO 14229)
- **Extended sessions**: 7E0 → PCM with session control
- **Data requests**: Read/write DIDs for configuration
- **Security access**: Seed/key authentication for protected functions

#### Periodic Messages
- **Engine RPM**: Transmitted at ~100ms intervals
- **Vehicle speed**: Broadcast for multiple module consumption
- **Engine temperature**: Regular updates for thermal management

## Diagnostic Access Methods

### Phase 1: Standard OBD-II (J1979)
```bash
# Read engine DTCs
faraday read-dtc --bus hs-can

# Live data from multiple modules
faraday live 0C,0D,05,42 --rate 5Hz

# Clear all DTCs on HS-CAN
faraday clear-dtc --bus hs-can
```

### Phase 2: UDS Extended Diagnostics
```bash
# Extended session with PCM
faraday session --module pcm extended

# Read detailed module information
faraday read-did --module pcm 0xF187,0xF188,0xF18C

# Advanced DTC information
faraday read-dtc --module pcm --detailed --snapshot-data
```

## Bus Loading and Performance

### Typical HS-CAN Utilization
- **Idle**: ~15-25% bus utilization
- **Normal driving**: ~30-45% utilization
- **Diagnostic active**: +5-15% additional load
- **Maximum sustained**: <80% (safety margin)

### Performance Considerations
- **Diagnostic impact**: Faraday adds minimal bus load (~1-3%)
- **Priority respect**: Diagnostic messages have lower priority than operational messages
- **Rate limiting**: Faraday limits request rate to prevent bus flooding
- **Error handling**: Automatic retry with exponential backoff on bus errors

## Network Topology

### Physical Bus Layout
```
         PCM ────┬──── TCM ────┬──── ABS ────┬──── RCM ────┬──── PSCM
         7E0     │     7E1     │     7E2     │     7E3     │     7E4
                 │             │             │             │
           ┌─────┴─────────────┴─────────────┴─────────────┴─────┐
           │                HS-CAN Bus                            │
           │            500 kbps, Pins 6/14                      │
           └──────────────────┬───────────────────────────────────┘
                              │
                        OBD-II Connector
                         Faraday Access
```

### Bus Termination
- **120Ω resistors** at both ends of the bus
- **Proper termination** essential for signal integrity at 500 kbps
- **Bus integrity check** via resistance measurement (should read ~60Ω between CAN-H and CAN-L with ignition off)

## Troubleshooting HS-CAN Issues

### Communication Problems
```bash
# Test HS-CAN connectivity
faraday test-bus --bus hs-can

# Check specific module response
faraday ping --module pcm --timeout 1000

# Scan for responsive modules
faraday scan-modules --bus hs-can
```

### Common Error Conditions

#### Bus Off
**Cause**: Module detected too many transmission errors
**Symptoms**: Module becomes unresponsive to diagnostics
**Resolution**: Clear DTCs, cycle ignition, check bus integrity

#### No Response
**Cause**: Module in sleep mode, communication fault, or bus issue
**Diagnosis**:
```bash
# Check bus voltage and termination
faraday test-bus --voltage-check --termination-check

# Verify other modules responding
faraday scan-modules --exclude pcm
```

#### Slow Response
**Cause**: High bus utilization, multiple active sessions
**Solution**: Limit concurrent diagnostic sessions, check bus load

### Signal Quality Issues
- **Voltage levels**: CAN-H should be ~3.5V, CAN-L should be ~1.5V (recessive state)
- **Differential voltage**: Should be ~2V between CAN-H and CAN-L during transmission
- **Termination resistance**: Should measure ~60Ω between pins 6 and 14

## Safety Considerations

### Critical Module Awareness
- **Never modify safety-critical modules** (RCM, ABS) without proper training
- **PCM modifications** can affect emissions compliance
- **Always create snapshots** before any configuration changes

### Bus Integrity Protection
- **Rate limiting**: Faraday prevents bus flooding
- **Error detection**: Automatic detection of bus-off conditions
- **Graceful degradation**: Fails safely when communication errors occur

### Vehicle State Requirements
For safe HS-CAN diagnostics:
- **Engine state**: Can be running or off (depends on operation)
- **Ignition**: Must be in RUN or KOEO position
- **Battery voltage**: ≥12V for reliable communication
- **No active U-codes**: Communication DTCs can indicate bus issues

## Integration with Other Buses

### Gateway Function
Some HS-CAN modules serve as gateways to other network segments:
- **PCM** may bridge to powertrain-specific networks
- **ABS** connects to wheel speed sensor networks
- **Cross-bus communication** through designated gateway modules

### Message Translation
Certain data flows between HS-CAN and MS-CAN:
- **Vehicle speed** from HS-CAN (PCM) to MS-CAN (IPC display)
- **Engine status** from HS-CAN to MS-CAN for body control logic
- **Diagnostic requests** may be bridged between buses

## Related Documentation

- [MS-CAN.md](MS-CAN.md) - Medium-speed CAN bus (body control modules)
- [Adapters.md](Adapters.md) - Adapters supporting HS-CAN access
- [UDS.md](UDS.md) - Advanced diagnostics protocol over HS-CAN
- [DTCs.md](DTCs.md) - Diagnostic trouble codes from HS-CAN modules

## References

- ISO 11898-1:2015 - Road vehicles — Controller area network (CAN) — Part 1: Data link layer and physical signalling
- SAE J1979:2017 - E/E Diagnostic Test Modes
- Ford Service Manual - 2017 Fusion Network Architecture
- ISO 15765-2:2016 - Diagnostic communication over CAN — Network layer services
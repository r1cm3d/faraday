# Medium-Speed CAN Bus (MS-CAN)

The Medium-Speed CAN bus in the Ford Fusion 2017 SEL operates at 125 kbps and connects body control, comfort, and infotainment modules. MS-CAN is where most user-facing features are controlled, including lighting, climate, instrument displays, and convenience functions. This bus requires specialized adapters with automatic switching capabilities.

## Technical Specifications

### Physical Layer
- **Speed**: 125 kbps (125,000 bits per second)
- **Topology**: Linear bus with 120Ω termination resistors
- **Voltage**: Differential signaling (CAN-H and CAN-L)
- **OBD-II Pins**: Pin 3 (CAN-H), Pin 11 (CAN-L)
- **Wire Gauge**: Typically 22 AWG twisted pair
- **Maximum Length**: ~500 meters (longer distances possible at lower speeds)

### Protocol Details
- **Frame Format**: CAN 2.0A (11-bit identifiers) primarily
- **Arbitration**: CSMA/CD with priority-based access
- **Error Detection**: CRC, frame check, acknowledgment, bit monitoring
- **Ford Extensions**: Proprietary message formats and timing requirements

## Connected Modules

### Primary MS-CAN Modules in 2017 Fusion

| Module | Full Name | CAN ID (Req/Resp) | Primary Function |
|--------|-----------|-------------------|------------------|
| **BCM** | Body Control Module | 726/72E | Body electrical, lighting, security |
| **IPC** | Instrument Panel Cluster | 720/728 | Gauges, displays, warnings |
| **APIM** | Accessory Protocol Interface Module | 7D0/7D8 | SYNC infotainment system |
| **HVAC** | Heating, Ventilation, Air Conditioning | 733/73B | Climate control |
| **DSM** | Driver Seat Module | 727/72F | Power seat controls |
| **PAM** | Parking Aid Module | 731/739 | Parking sensors, cameras |

### Module Characteristics

#### BCM (Body Control Module) - 726/72E
**Primary target for as-built configuration in Faraday**

**Responsibilities:**
- Exterior lighting control (headlights, taillights, turn signals)
- Interior lighting and welcome/goodbye sequences
- Power window and door lock control
- Security system and remote keyless entry
- Wiper and washer control
- Horn and chime control

**As-Built Configuration Examples:**
- Daytime Running Lights (DRL) enable/disable
- Auto-lock when shifting to Drive
- Number of unlock beeps
- Welcome lighting duration and pattern
- Auto-down window behavior

**Faraday Access:**
```bash
# Read BCM as-built configuration
faraday asbuilt dump --module bcm

# Modify DRL setting
faraday asbuilt write --module bcm --feature drl_enabled --value true

# Read specific as-built block
faraday read-did --module bcm 0x726A
```

#### IPC (Instrument Panel Cluster) - 720/728
**Dashboard display and gauge control**

**Responsibilities:**
- Speedometer, tachometer, fuel gauge display
- Warning lamp control (MIL, ABS, airbag, etc.)
- Digital display content (odometer, trip, DTE)
- Audio chimes and warning sounds
- Language and units selection

**Configurable Features:**
- Digital speedometer display enable
- Welcome animation on startup
- Gauge sweep behavior
- Warning chime volume levels
- Display brightness and contrast

**Common DIDs:**
- `0x720A-720F`: Display configuration blocks
- `0x7211`: Language settings
- `0x7212`: Units (metric/imperial)

#### APIM (Accessory Protocol Interface Module) - 7D0/7D8
**SYNC infotainment system controller**

**Responsibilities:**
- SYNC 3 infotainment system operation
- Bluetooth connectivity and phone integration
- Navigation system control
- Audio system management
- Wi-Fi hotspot functionality (if equipped)

**Configuration Areas:**
- Region and language settings
- Feature enable/disable flags
- Hardware configuration parameters
- Software version management

**Note**: APIM modifications are complex and can affect warranty; proceed with extreme caution.

#### HVAC (Climate Control) - 733/73B
**Automatic climate control system**

**Responsibilities:**
- Temperature control and regulation
- Fan speed and distribution control
- A/C compressor control
- Rear window defrost
- Auto-climate logic

**Configurable Parameters:**
- Default temperature settings
- Auto-climate behavior
- Fan curve adjustments
- Defrost logic timing

#### DSM (Driver Seat Module) - 727/72F
**Power seat positioning and memory**

**Responsibilities:**
- Power seat position control
- Seat memory storage and recall
- Lumbar and side bolster adjustment
- Heated/cooled seat control (if equipped)

#### PAM (Parking Aid Module) - 731/739
**Parking assistance systems**

**Responsibilities:**
- Ultrasonic parking sensor monitoring
- Reverse camera interface
- Active park assist (if equipped)
- Cross-traffic alert

## Bus Loading and Performance

### Typical MS-CAN Utilization
- **Key off**: ~5-10% (modules in sleep/wake cycles)
- **Ignition on**: ~20-35% utilization
- **Active features**: +10-20% (climate, audio, navigation)
- **Diagnostic active**: +3-8% additional load

### Performance Characteristics
- **Lower priority**: MS-CAN has lower urgency than HS-CAN
- **Comfort features**: Most functions are convenience rather than safety
- **Sleep modes**: Modules enter low-power states to preserve battery
- **Wake-up events**: CAN traffic can wake sleeping modules

## Diagnostic Access Challenges

### Access Complexity
Unlike HS-CAN's standardized OBD-II access, MS-CAN requires:
- **Specialized adapters**: Standard ELM327 cannot access MS-CAN
- **Bus switching**: Adapters must switch from HS-CAN (pins 6/14) to MS-CAN (pins 3/11)
- **Ford-specific protocols**: Proprietary extensions beyond standard UDS
- **Module-specific timing**: Different response time requirements

### Adapter Requirements
```bash
# Verify adapter can access MS-CAN
faraday test-bus --bus ms-can

# Expected output:
# MS-CAN (pins 3/11): OK - BCM responding (726/72E)
# MS-CAN (pins 3/11): OK - IPC responding (720/728)
```

### Module Wake-Up
MS-CAN modules may be in sleep mode:
```bash
# Wake up modules before diagnosis
faraday wake-modules --bus ms-can

# Scan for active modules
faraday scan-modules --bus ms-can --timeout 5000
```

## As-Built Configuration Access

### Reading Configuration
```bash
# Read all as-built blocks from BCM
faraday asbuilt dump --module bcm --output bcm_config.yml

# Read specific feature status
faraday asbuilt show --module bcm --feature auto_lock_on_drive
faraday asbuilt show --module ipc --feature digital_speedometer
```

### Writing Configuration
**⚠️ Requires Security Access and Extreme Caution**

```bash
# Enable DRL (with all safety checks)
faraday asbuilt write --module bcm --block 726-01 --byte 3 --bit 2 --value 1 --commit

# Apply configuration profile
faraday profile apply my-fusion-config.yml --dry-run  # Preview first
faraday profile apply my-fusion-config.yml --commit   # Apply changes
```

### Example Configuration Profile
```yaml
# my-fusion-config.yml
vehicle:
  vin: "3FA6P0H7XHR123456"
  model: "Fusion 2017 SEL"

modules:
  bcm:
    drl_enabled: true
    auto_lock_on_drive: true
    unlock_beeps: 2
    welcome_lighting: true
    welcome_duration: 30  # seconds

  ipc:
    show_digital_speedometer: true
    welcome_animation: true
    gauge_sweep_on_startup: true
    metric_units: false
```

## Network Topology

### Physical Bus Layout
```
    BCM ────┬──── IPC ────┬──── APIM ───┬──── HVAC ───┬──── DSM ────┬──── PAM
    726     │     720     │     7D0     │     733     │     727     │     731
            │             │             │             │             │
      ┌─────┴─────────────┴─────────────┴─────────────┴─────────────┴─────┐
      │                     MS-CAN Bus                                     │
      │                125 kbps, Pins 3/11                                │
      └───────────────────────┬────────────────────────────────────────────┘
                              │
                        OBD-II Connector
                      Faraday Access Point
```

### Gateway Connections
MS-CAN modules may communicate with HS-CAN through:
- **BCM as gateway**: Some cross-bus message routing
- **Dedicated gateway modules**: In some configurations
- **Direct bridging**: For critical safety information

## Module Sleep and Wake Management

### Sleep Behavior
MS-CAN modules implement sophisticated power management:
- **Timed sleep**: Enter sleep mode after inactivity timeout
- **Deep sleep**: Minimal power consumption, slower wake-up
- **Network wake**: Can be awakened by CAN traffic

### Wake-Up Strategies
```bash
# Standard wake-up sequence
faraday wake-modules --bus ms-can --method standard

# Force wake with broadcast message
faraday wake-modules --bus ms-can --method broadcast --force

# Module-specific wake
faraday wake --module bcm --timeout 2000
```

### Power Management Impact
- **Battery drain**: Active diagnostics prevent sleep modes
- **Session management**: Limit diagnostic session duration
- **Graceful disconnect**: Allow modules to return to sleep

## Security and Access Control

### Security Levels
MS-CAN modules typically have:
- **Open access**: Basic information and status
- **Protected access**: Configuration and as-built data
- **Secure access**: Critical features and programming

### Authentication Requirements
```bash
# Extended session required for most configuration access
faraday session --module bcm extended

# Security access for write operations
faraday session --module bcm extended --secure

# Check security status
faraday security-status --module bcm
```

## Common MS-CAN Operations

### Feature Discovery
```bash
# Scan available features
faraday asbuilt scan --module bcm
faraday asbuilt scan --module ipc

# List configurable parameters
faraday asbuilt list-features --module bcm --category lighting
```

### Configuration Management
```bash
# Create baseline snapshot
faraday asbuilt snapshot --module bcm --name baseline_$(date +%Y%m%d)

# Show configuration differences
faraday asbuilt diff --baseline baseline_20260507 --current

# Restore previous configuration
faraday asbuilt restore --snapshot baseline_20260507
```

### Bulk Operations
```bash
# Configure multiple features
faraday asbuilt batch-write --config multi-feature.yml --commit

# Apply organization standard
faraday asbuilt apply-template --template corporate_fleet.yml
```

## Troubleshooting MS-CAN Issues

### Common Problems

#### No Module Response
```bash
# Check physical connections
faraday test-bus --bus ms-can --voltage-check

# Verify adapter switching capability
faraday test-adapter --bus-switching-test
```

#### Partial Module Access
```bash
# Scan responsive modules
faraday scan-modules --bus ms-can --detailed

# Test specific module
faraday ping --module bcm --retries 3
```

#### Configuration Write Failures
```bash
# Check security access status
faraday security-status --module bcm

# Verify vehicle state
faraday check-preconditions --for-writing

# Review audit log
faraday audit-log --recent --module bcm
```

### Signal Quality Issues
- **Lower frequency**: 125 kbps more susceptible to interference
- **Longer distances**: MS-CAN wiring spans more of the vehicle
- **Multiple modules**: More potential failure points

## Safety Considerations

### Feature Dependencies
Many MS-CAN features interact:
- **Lighting sequences**: BCM controls multiple lighting zones
- **Security system**: BCM coordinates locks, alarm, immobilizer
- **Display warnings**: IPC shows status from multiple modules

### Safe Modification Practices
1. **Always snapshot**: Create full backup before changes
2. **Test incrementally**: Change one feature at a time
3. **Verify functionality**: Test all affected systems
4. **Document changes**: Maintain change log
5. **Plan rollback**: Know how to restore original configuration

### Module Interdependencies
```
BCM ←→ IPC: Warning lamp control, chimes
BCM ←→ APIM: Button inputs, display requests
IPC ←→ APIM: Display coordination
HVAC ←→ BCM: Climate status indicators
```

## Related Documentation

- [HS-CAN.md](HS-CAN.md) - High-speed CAN bus (powertrain modules)
- [AsBuilt.md](AsBuilt.md) - As-built configuration concepts
- [Adapters.md](Adapters.md) - Adapters supporting MS-CAN access
- [FORScan.md](FORScan.md) - FORScan MS-CAN compatibility

## References

- Ford Service Manual - 2017 Fusion Body Control System
- ISO 11898-1:2015 - Controller area network (CAN) specification
- Ford TSB 19-2109 - Body Control Module Configuration Procedures
- CyanLabs Forum - Ford MS-CAN Module Documentation
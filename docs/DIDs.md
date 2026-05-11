# Data Identifiers (DIDs)

Data Identifiers (DIDs) are standardized 16-bit addresses used in the UDS (Unified Diagnostic Services) protocol to access specific data within electronic control modules (ECMs). In the Ford Fusion 2017 SEL, DIDs provide access to everything from vehicle identification numbers to complex as-built configuration data.

## DID Structure

DIDs are 2-byte (16-bit) hexadecimal values ranging from `0x0000` to `0xFFFF`:

- **Format**: 4 hexadecimal digits (e.g., `0xF190`, `0x726A`)
- **Byte order**: Big-endian (most significant byte first)
- **Access**: Via UDS Service 22 (ReadDataByIdentifier) and Service 2E (WriteDataByIdentifier)

## Standard DID Ranges

### ISO 14229 Defined Ranges

| Range | Purpose | Access |
|-------|---------|---------|
| `0x0000-0x00FF` | Manufacturer specific | Various |
| `0x0100-0xEFFF` | Manufacturer specific | Various |
| `0xF000-0xF0FF` | Network Configuration DIDs | Read-only |
| `0xF100-0xF1FF` | Vehicle manufacturer specific | **Programming** |
| `0xF200-0xF2FF` | Vehicle manufacturer specific | Various |
| `0xF300-0xF3FF` | Network Configuration DIDs | Read-only |
| `0xF400-0xF4FF` | Vehicle manufacturer specific | Various |
| `0xF500-0xF5FF` | Reserved | - |
| `0xF600-0xF6FF` | Vehicle manufacturer specific | Various |
| `0xF700-0xF7FF` | Reserved | - |
| `0xF800-0xF8FF` | Vehicle manufacturer specific | Various |
| `0xF900-0xF9FF` | WWH-OBD DIDs | Read-only |
| `0xFA00-0xFAFF` | Vehicle manufacturer specific | Various |
| `0xFB00-0xFBFF` | Reserved | - |
| `0xFC00-0xFCFF` | Reserved | - |
| `0xFD00-0xFDFF` | Reserved | - |
| `0xFE00-0xFEFF` | Reserved | - |
| `0xFF00-0xFFFF` | ISO standard DIDs | Read-only |

### ⚠️ Critical: Programming DIDs (`F1xx` and `F0xx`)

**NEVER write to programming DIDs** - these control module firmware and can render modules inoperable:
- `F0xx` range: Flash programming related
- `F1xx` range: Module programming and calibration data

Faraday blocks all write attempts to these ranges for safety.

## Common Ford Fusion 2017 DIDs

### Standard Information DIDs

| DID | Name | Module | Description |
|-----|------|--------|-------------|
| `F190` | VIN | All | Vehicle Identification Number |
| `F187` | Vehicle Part Number | All | Module part number |
| `F188` | Software Version | All | Software version |
| `F189` | Software Calibration ID | All | Calibration identifier |
| `F18A` | Software Calibration Version | All | Calibration version |
| `F18C` | ECU Serial Number | All | Module serial number |
| `F191` | ECU Hardware Number | All | Hardware part number |
| `F192` | ECU Software Number | All | Software part number |
| `F193` | System Name | All | ECU system name |
| `F194` | Programming Date | All | Last programming date |

### Module-Specific DIDs

#### PCM (7E0/7E8)
| DID | Description |
|-----|-------------|
| `7E01` | Engine runtime parameters |
| `7E02` | Fuel system data |
| `7E03` | Emission system status |

#### BCM (726/72E)
| DID | Description |
|-----|-------------|
| `0201` | TPMS tire pressures — front axle / rear axle (1 byte each = PSI, per-axle; FORScan: `726-02-01`) |
| `726A-726F` | As-built configuration blocks |
| `7270` | Feature enable/disable flags |

#### IPC (720/728)
| DID | Description |
|-----|-------------|
| `0401` | TPMS display units — `0x04`=PSI · `0x08`=kPa · `0x0C`=Bar (FORScan: `720-04-01`) |
| `720A-720F` | Display configuration blocks |
| `7211` | Language settings |
| `7212` | Units settings (metric/imperial) |

## Reading DIDs with Faraday

### Basic DID Read
```bash
# Read VIN from PCM
faraday read-did --module pcm 0xF190

# Read software version from BCM
faraday read-did --module bcm 0xF188
```

### As-Built Block Access
```bash
# Read as-built block from BCM
faraday read-did --module bcm 0x726A

# Read multiple blocks
faraday read-did --module bcm 0x726A,0x726B,0x726C
```

### Module Information Discovery
```bash
# Read all standard information DIDs
faraday read-did --module pcm --info-dids

# Read module identification
faraday read-did --module bcm 0xF187,0xF188,0xF18C
```

## Writing DIDs (Advanced)

**⚠️ Extreme Caution Required**: Writing DIDs can modify critical vehicle configuration.

### Prerequisites
1. **Extended diagnostic session** (UDS Service 10)
2. **Security access** (UDS Service 27) with valid seed/key
3. **Proper vehicle state** (KOEO, battery voltage, no comm DTCs)
4. **Mandatory snapshot** before any write operation

### Write Example
```bash
# Write as-built block (with all safety checks)
faraday write-did --module bcm 0x726A --data "01234567" --commit
```

## Error Handling

### Common Negative Response Codes (NRCs)
- `0x13` - Incorrect Message Length or Invalid Format
- `0x22` - Conditions Not Correct
- `0x31` - Request Out of Range
- `0x33` - Security Access Denied
- `0x72` - General Programming Failure

### UDS Session Requirements
Some DIDs require specific diagnostic sessions:
- **Default session**: Basic information DIDs
- **Extended session**: As-built and configuration DIDs
- **Programming session**: Reserved for factory/dealer tools

## Module-Specific Considerations

### Security Levels
Different modules have varying security requirements:
- **PCM**: High security for powertrain parameters
- **BCM**: Medium security for body functions
- **IPC**: Low security for display settings

### Session Management
Reading multiple DIDs efficiently:
1. Establish extended diagnostic session
2. Authenticate with security access if needed
3. Read multiple DIDs in sequence
4. Maintain session with TesterPresent
5. Return to default session when complete

## Safety and Validation

### Faraday Safety Mechanisms
- **DID range validation**: Block writes to programming ranges
- **Known DID database**: Only access documented, safe DIDs
- **Pre-write snapshots**: Automatic backup before modifications
- **Format validation**: Verify data length and format
- **Module compatibility**: Check DID availability per module

### Best Practices
- Always read before writing to understand current state
- Use `--dry-run` mode to preview changes
- Verify changes with subsequent reads
- Maintain audit logs of all modifications

## Related Documentation

- [AsBuilt.md](AsBuilt.md) - How DIDs access as-built configuration blocks
- [UDS.md](UDS.md) - UDS services that use DIDs
- [HS-CAN.md](HS-CAN.md) - Modules accessible via high-speed CAN
- [MS-CAN.md](MS-CAN.md) - Modules accessible via medium-speed CAN

## References

- ISO 14229-1:2020 - Unified Diagnostic Services
- Ford Service Manual - 2017 Fusion Diagnostic Data Identifiers
- SAE J1979 - E/E Diagnostic Test Modes
- CyanLabs Forum - Ford DID documentation
# OBD-II Adapters for Ford Diagnostics

OBD-II adapters serve as the interface between diagnostic software and vehicle CAN buses. For Ford vehicles like the 2017 Fusion, the adapter must support both HS-CAN and MS-CAN protocols with automatic switching capabilities. This document covers adapter compatibility, differences, and selection criteria for use with Faraday.

## Adapter Requirements for Ford Vehicles

### Essential Capabilities
- **Dual CAN bus support**: Access to both HS-CAN (500 kbps) and MS-CAN (125 kbps)
- **Automatic switching**: Protocol switching without manual intervention
- **STN chipset**: Superior to standard ELM327 for Ford applications
- **ISO-TP support**: Multi-frame message handling for UDS communication
- **Programming voltage**: 18V FEPS capability for advanced operations

### Ford-Specific Challenges
Unlike generic OBD-II, Ford vehicles require:
- **MS-CAN access**: Body control modules on separate 125 kbps bus
- **Proprietary protocols**: Ford-specific extensions beyond standard J1979
- **Complex timing**: Precise frame timing for multi-ECU communication
- **Security access**: Seed/key authentication for configuration changes

## Primary Adapter: Vgate vLinker FS

**Manufacturer**: Vgate Technology Co.
**Model**: vLinker FS (USB and Bluetooth variants)
**Status**: Primary recommendation for Faraday

### Technical Specifications

| Feature | Specification |
|---------|---------------|
| **Chipset** | STN1170/STN2120 |
| **Processor** | 32-bit ARM |
| **CAN Speed** | Up to 3 Mbps transmission |
| **Bus Support** | HS-CAN + MS-CAN automatic switching |
| **Programming Voltage** | 18V FEPS support |
| **Interfaces** | USB 2.0, Bluetooth 4.0 |
| **Power** | Bus-powered (USB) or vehicle 12V |
| **Operating Temperature** | -20°C to +70°C |

### Key Advantages

#### Automatic Bus Switching
The vLinker FS automatically detects and switches between CAN buses:
- **HS-CAN detection**: Monitors pins 6/14 for 500 kbps activity
- **MS-CAN fallback**: Switches to pins 3/11 for 125 kbps when needed
- **No manual commands**: Transparent switching without user intervention
- **Protocol preservation**: Maintains session state across switches

#### STN Chipset Benefits
- **Advanced AT commands**: Extended command set beyond ELM327
- **Better error handling**: More robust communication error recovery
- **Higher throughput**: Faster data transfer rates
- **Multi-protocol**: Simultaneous protocol support

#### FORScan Optimization
Specifically designed for FORScan compatibility:
- **Certified compatibility**: Official FORScan recommendation
- **Optimized firmware**: Tailored for Ford diagnostic requirements
- **Regular updates**: Firmware updates for new Ford models

### Connection Options

#### USB Variant (Recommended for Faraday)
- **Interface**: USB-A male to OBD-II female
- **Power**: Bus-powered from computer
- **Latency**: ~1ms communication latency
- **Reliability**: Most stable connection method

```bash
# Faraday USB connection
faraday --adapter usb --device /dev/ttyUSB0 read-dtc
```

#### Bluetooth Variant
- **Interface**: Bluetooth 4.0 Low Energy
- **Range**: ~10 meters line-of-sight
- **Power**: Vehicle 12V powered
- **Latency**: ~10-50ms depending on stack

```bash
# Faraday Bluetooth connection
faraday --adapter bluetooth --device "vLinker FS" read-dtc
```

## Alternative Compatible Adapters

### OBDLink EX (ScanTool.net)

**Manufacturer**: ScanTool.net
**Model**: OBDLink EX
**Status**: Compatible alternative

#### Specifications
- **Chipset**: STN1170
- **Interfaces**: Wi-Fi, Bluetooth, USB
- **Bus Support**: HS-CAN + MS-CAN (manual switching)
- **Programming Voltage**: None (12V only)
- **Certification**: OBD-II compliant, FCC certified

#### Advantages
- **Multi-interface**: Wi-Fi, Bluetooth, and USB in one device
- **Professional build quality**: Robust construction
- **Wide compatibility**: Works with many diagnostic applications
- **Regular firmware updates**: Active development support

#### Limitations
- **Manual bus switching**: Requires AT commands to switch between CAN buses
- **No programming voltage**: Cannot support 18V FEPS operations
- **Higher cost**: More expensive than vLinker FS

#### Faraday Configuration
```bash
# Manual MS-CAN switch required
faraday --adapter obdlink --manual-switching read-did --module bcm 0xF190
```

### ELS27 Professional Adapters

**Various manufacturers**: Multiple suppliers
**Chipset**: STN2120/STN2220
**Status**: Professional-grade compatible

#### Specifications
- **Chipset**: STN2120 or STN2220 (latest generation)
- **Bus Support**: Full HS-CAN + MS-CAN automatic switching
- **Programming Voltage**: 18V FEPS support
- **Interfaces**: USB, Ethernet, CAN-FD ready
- **Certifications**: Professional automotive compliance

#### Advantages
- **Professional grade**: Built for commercial diagnostic applications
- **Latest STN chipset**: Most advanced protocol support
- **Full programming support**: Complete FEPS voltage capability
- **Future-proof**: CAN-FD and upcoming protocol support

#### Disadvantages
- **High cost**: Professional pricing ($200-500)
- **Overkill for hobbyist use**: More capability than needed
- **Availability**: Limited distribution channels

### Incompatible: Standard ELM327 Adapters

**Why ELM327 doesn't work for Ford:**

#### Technical Limitations
- **Single bus only**: Cannot access MS-CAN (pins 3/11)
- **Limited to HS-CAN**: Only PCM, TCM, ABS accessible
- **No BCM/IPC access**: Body control modules unreachable
- **Missing protocols**: Lacks Ford-specific extensions
- **Poor error handling**: Cannot handle complex Ford communication

#### Module Accessibility Comparison

| Module | ELM327 | STN Adapters |
|--------|--------|--------------|
| PCM | ✅ | ✅ |
| TCM | ✅ | ✅ |
| ABS | ✅ | ✅ |
| RCM | ✅ | ✅ |
| BCM | ❌ | ✅ |
| IPC | ❌ | ✅ |
| APIM | ❌ | ✅ |
| HVAC | ❌ | ✅ |

**Result**: ELM327 can only access ~40% of Ford modules

## Adapter Selection Guide

### For Faraday Development and Testing
**Recommendation**: Vgate vLinker FS (USB)
- **Rationale**: Best balance of capability, cost, and reliability
- **Use case**: Development, testing, occasional diagnostics
- **Cost**: ~$50-70

### For Professional/Commercial Use
**Recommendation**: ELS27 with STN2220
- **Rationale**: Professional reliability and full capability
- **Use case**: Commercial diagnostics, frequent use
- **Cost**: ~$200-400

### For Mixed Tool Environment
**Recommendation**: OBDLink EX
- **Rationale**: Works with multiple diagnostic applications
- **Use case**: FORScan + Faraday + other tools
- **Cost**: ~$100-150

## Physical Connection Details

### OBD-II Connector Pinout (Ford Fusion 2017)

```
    1  2  3  4  5  6  7  8
   ┌─────────────────────────┐
   │     3     6           │
   │                       │
   │ 4    11   14    16    │
   └─────────────────────────┘
   9  10 11 12 13 14 15 16
```

| Pin | Function | Signal |
|-----|----------|--------|
| 3 | MS-CAN High | CAN-H (125 kbps) |
| 4 | Chassis Ground | Ground |
| 6 | HS-CAN High | CAN-H (500 kbps) |
| 11 | MS-CAN Low | CAN-L (125 kbps) |
| 14 | HS-CAN Low | CAN-L (500 kbps) |
| 16 | Battery Positive | +12V |

### Bus Architecture

```
┌─────────────┐    ┌─────────────┐
│   HS-CAN    │    │   MS-CAN    │
│  500 kbps   │    │  125 kbps   │
│  Pins 6/14  │    │  Pins 3/11  │
├─────────────┤    ├─────────────┤
│     PCM     │    │     BCM     │
│     TCM     │    │     IPC     │
│     ABS     │    │    APIM     │
│     RCM     │    │    HVAC     │
│    PSCM     │    │     DSM     │
└─────────────┘    │     PAM     │
                   └─────────────┘
```

## Adapter Configuration in Faraday

### Automatic Detection
```bash
# Faraday auto-detects compatible adapters
faraday scan-adapters

# Output:
# Found: vLinker FS (USB) - /dev/ttyUSB0
# Found: OBDLink EX (Bluetooth) - vLinker_12345
```

### Manual Configuration
```bash
# Specify adapter explicitly
faraday --adapter vlinker-usb --device /dev/ttyUSB0 read-dtc

# With custom timeout
faraday --adapter vlinker-usb --device /dev/ttyUSB0 --timeout 5000 session --module bcm extended
```

### Adapter-Specific Settings
```yaml
# ~/.config/faraday/config.yml
adapters:
  vlinker-usb:
    device: /dev/ttyUSB0
    baud_rate: 38400
    timeout_ms: 2000

  obdlink-bt:
    device: "OBDLink_EX_12345"
    timeout_ms: 5000
    manual_switching: true
```

## Troubleshooting Common Adapter Issues

### Connection Problems
```bash
# Test adapter connectivity
faraday test-adapter --verbose

# Check permissions (Linux)
sudo usermod -a -G dialout $USER
# (logout/login required)
```

### Bus Communication Issues
```bash
# Verify both buses accessible
faraday test-buses --adapter vlinker-usb

# Expected output:
# HS-CAN (pins 6/14): OK - PCM responding
# MS-CAN (pins 3/11): OK - BCM responding
```

### Performance Optimization
```bash
# Reduce latency for USB adapters
echo 1 | sudo tee /sys/bus/usb-serial/devices/ttyUSB0/latency_timer

# Increase buffer sizes for high-throughput operations
faraday --buffer-size 8192 live 0C,0D,05 --rate 10Hz
```

## Future Adapter Support

### Planned Faraday Enhancements
- **Native SocketCAN**: Direct Linux CAN interface support
- **Wi-Fi adapters**: Network-based diagnostic interfaces
- **CAN-FD ready**: Support for next-generation protocols
- **Multi-adapter**: Simultaneous adapter support

### Emerging Technologies
- **Ethernet-based diagnostics**: DoIP (Diagnostics over IP)
- **Wireless CAN**: Bluetooth Low Energy CAN adapters
- **Cloud integration**: Remote diagnostic capabilities

## Related Documentation

- [HS-CAN.md](HS-CAN.md) - High-speed CAN bus details
- [MS-CAN.md](MS-CAN.md) - Medium-speed CAN bus details
- [FORScan.md](FORScan.md) - FORScan adapter compatibility

## References

- Vgate Technology - vLinker FS Documentation
- ScanTool.net - OBDLink EX Specifications
- OBD Solutions - STN Chipset Technical Reference
- ISO 15765-2:2016 - Diagnostic communication over CAN
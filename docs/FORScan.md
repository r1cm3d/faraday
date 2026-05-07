# FORScan Software and Compatibility

FORScan is specialized diagnostic software developed for Ford, Lincoln, Mazda, and Volvo vehicles. It provides advanced diagnostic capabilities beyond standard OBD-II scanners, including access to Ford-specific modules, as-built configuration, and dealer-level functions. Faraday aims to provide scriptable, version-controllable alternatives to common FORScan operations.

## What is FORScan?

FORScan is a Windows-based diagnostic application that communicates with vehicle modules through compatible OBD-II adapters. It was developed by the Russian team at FORScan LLC and has become the de facto standard for Ford enthusiasts and independent technicians.

### Key FORScan Capabilities
- **Complete module access**: All Ford ECUs including BCM, PCM, IPC, APIM
- **As-built configuration**: Read and modify vehicle configuration blocks
- **Advanced diagnostics**: Detailed DTC information with freeze frame data
- **Live data streaming**: Real-time parameter monitoring
- **Module programming**: Software updates and calibration (license required)
- **Special functions**: DPF regeneration, key programming, module reset

## FORScan vs Faraday

### Similarities
Both tools provide:
- Ford-specific diagnostic capabilities
- As-built configuration access
- Support for FORScan-compatible adapters
- Advanced DTC reading beyond standard OBD-II
- Module-specific communication protocols

### Key Differences

| Feature | FORScan | Faraday |
|---------|---------|---------|
| **Platform** | Windows GUI | Cross-platform CLI/TUI |
| **Interface** | Point-and-click | Command-line driven |
| **Automation** | Manual operations | Scriptable workflows |
| **Version Control** | Not supported | YAML profiles in git |
| **Programming** | Full flash capabilities | Configuration only |
| **Cost** | Free (basic) + paid licenses | Open source |
| **Target Users** | Technicians, enthusiasts | Developers, power users |

## Faraday's Relationship to FORScan

### Complementary Use Cases

**FORScan is better for:**
- Initial vehicle exploration and discovery
- One-off diagnostic sessions
- Module programming and firmware updates
- Complex troubleshooting with guided workflows
- Users comfortable with GUI interfaces

**Faraday is better for:**
- Automated testing and monitoring
- Configuration management as code
- Integration into larger automation systems
- Batch operations across multiple vehicles
- Development and research workflows

### Data Compatibility

Faraday uses FORScan-compatible data sources:
- **Adapter protocols**: Same AT commands and communication methods
- **As-built definitions**: Compatible block/byte/bit addressing
- **Module addressing**: Identical CAN header mappings
- **DID definitions**: Same data identifier usage

This means configurations discovered with FORScan can be implemented in Faraday, and vice versa.

## FORScan-Compatible Adapters

Both FORScan and Faraday work with the same adapter ecosystem:

### Primary Adapter: Vgate vLinker FS
**Why FORScan recommends it:**
- Automatic HS-CAN/MS-CAN switching
- 32-bit processor with high-speed communication
- STN1170/STN2120 chipset with proprietary commands
- FEPS 18V programming voltage support
- USB and Bluetooth connectivity options

**Faraday compatibility:**
- Full support for diagnostic operations
- Automatic bus switching capabilities
- Same AT command set for configuration

### Alternative Compatible Adapters

#### OBDLink EX (ScanTool.net)
- **Chipset**: STN1170
- **Features**: Wi-Fi, Bluetooth, USB connectivity
- **Compatibility**: Full diagnostic support
- **Programming**: Limited (no FEPS voltage)

#### ELS27 (Various manufacturers)
- **Chipset**: STN2120/STN2220
- **Features**: Professional-grade, multiple interfaces
- **Compatibility**: Complete FORScan compatibility
- **Programming**: Full support including FEPS

#### NOT Compatible: Standard ELM327
- **Limitation**: Cannot access MS-CAN bus
- **Issue**: No automatic bus switching
- **Result**: Limited to HS-CAN modules only (PCM, TCM, ABS)

## Protocol Compatibility

### Communication Layers

Both FORScan and Faraday use identical protocol stacks:

```
┌─────────────────────────────────────────────────────────┐
│  Application Layer                                       │
│  FORScan GUI ←→ Faraday CLI                             │
├─────────────────────────────────────────────────────────┤
│  UDS + J1979 (identical protocols)                      │
├─────────────────────────────────────────────────────────┤
│  ISO-TP over CAN (same frame structure)                 │
├─────────────────────────────────────────────────────────┤
│  STN/ELM327 AT Commands (compatible command set)        │
├─────────────────────────────────────────────────────────┤
│  Physical Interface (USB/Bluetooth/Serial)              │
└─────────────────────────────────────────────────────────┘
```

### Shared AT Commands
- `AT Z` - Reset adapter
- `AT SP 6` - Set protocol to ISO 15765-4 CAN (11-bit ID, 500 kbaud)
- `AT SP 7` - Set protocol to ISO 15765-4 CAN (29-bit ID, 500 kbaud)
- `AT SH [header]` - Set CAN header for requests
- `AT STN***` - STN-specific commands for advanced features

## Migration Strategies

### From FORScan to Faraday

#### As-Built Configuration
1. **Document current state** with FORScan
2. **Export configuration** (manual documentation)
3. **Create Faraday profile** in YAML format
4. **Validate with read operations** before writes
5. **Implement with Faraday** using `--dry-run` first

#### Workflow Examples

**FORScan approach:**
1. Open FORScan → Vehicle Selection → Connect
2. Navigate to BCM → As-Built → Block 726-01
3. Modify byte 3, bit 2 → Enable DRL
4. Write to vehicle → Test functionality

**Equivalent Faraday approach:**
```bash
# Create configuration profile
cat > my-fusion.yml << EOF
modules:
  bcm:
    drl_enabled: true
EOF

# Apply configuration
faraday profile apply my-fusion.yml --commit
```

### Hybrid Workflows

**Discovery with FORScan, Implementation with Faraday:**
1. Use FORScan to explore unknown configurations
2. Document successful changes
3. Implement in Faraday for repeatability
4. Version control the configuration

**Example: Custom lighting sequence**
1. **FORScan**: Experiment with BCM welcome lighting settings
2. **Document**: Record working block/byte/bit combinations
3. **Faraday**: Create YAML profile for the configuration
4. **Git**: Version control the lighting profile
5. **Deploy**: Apply to multiple vehicles consistently

## FORScan Community Resources

Faraday benefits from the extensive FORScan community:

### Knowledge Bases
- **FORScan Official Forum**: Module definitions and procedures
- **CyanLabs**: Ford diagnostic resources and documentation
- **F150Forum.com**: Platform-specific as-built modifications
- **Reddit r/FORScan**: Community troubleshooting and discoveries

### As-Built Databases
Community-maintained databases that both FORScan and Faraday reference:
- Module address mappings
- Block/byte/bit feature definitions
- Known safe modification ranges
- Year/model compatibility matrices

## Limitations and Scope

### What FORScan Can Do That Faraday Cannot
- **Module programming**: Firmware updates and calibration flashing
- **Key programming**: Anti-theft system key registration
- **Special functions**: DPF regeneration, injector coding
- **Guided troubleshooting**: Step-by-step diagnostic procedures
- **Real-time graphing**: Advanced data visualization

### What Faraday Can Do That FORScan Cannot
- **Automation**: Unattended batch operations
- **Version control**: Git-managed configuration profiles
- **CI/CD integration**: Automated testing and deployment
- **Cross-platform**: Linux/macOS support
- **Scriptable**: Integration with larger automation systems

## Future Compatibility

### Planned Faraday Enhancements
- **FORScan profile import**: Convert FORScan configurations to YAML
- **Shared adapter sessions**: Coordinate with running FORScan instances
- **Extended live data**: TUI matching FORScan's real-time displays
- **Community integration**: Direct access to as-built databases

### Ecosystem Integration
- **Data exchange**: Standard formats for configuration sharing
- **Adapter sharing**: Protocol for multi-tool adapter access
- **Community contributions**: Collaborative as-built database maintenance

## Safety Considerations

Both tools require similar safety practices:

- **Vehicle state**: KOEO (Key On Engine Off) for configuration changes
- **Battery condition**: ≥12.4V for reliable communication
- **Backup strategy**: Always snapshot before modifications
- **Validation**: Test changes in safe environments first
- **Rollback plan**: Maintain ability to restore original configuration

## Related Documentation

- [Adapters.md](Adapters.md) - Detailed adapter compatibility information
- [AsBuilt.md](AsBuilt.md) - As-built configuration concepts
- [UDS.md](UDS.md) - Protocol details shared with FORScan

## References

- FORScan Official Documentation - https://forscan.org/
- CyanLabs Ford Resources - https://cyanlabs.net/
- FORScan Community Forum - https://forscan.org/forum/
- STN Chip Documentation - OBDSolutions.com
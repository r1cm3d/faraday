# As-Built Configuration Data

As-built data represents the factory configuration stored in vehicle electronic control modules (ECMs). This data defines how various features and systems behave, essentially serving as the "DNA" of the vehicle's electronic personality. In the Ford Fusion 2017 SEL, as-built data controls everything from lighting patterns to comfort features.

## What is As-Built Data?

As-built data consists of binary configuration blocks stored in non-volatile memory within each ECM. These blocks contain bit-packed settings that enable, disable, or modify the behavior of vehicle features.

### Structure
- **Module-specific blocks**: Each module (BCM, IPC, PCM, etc.) has its own set of as-built blocks
- **Block addressing**: Identified by module address + block number (e.g., `726-01` for BCM block 1)
- **Bit-level configuration**: Individual bits within bytes control specific features
- **Checksums**: Blocks include validation data to prevent corruption

## Common As-Built Applications

### Body Control Module (BCM - 726)
Controls body-related features:
- Daytime Running Lights (DRL) enable/disable
- Auto-lock behavior when shifting to Drive
- Number of beeps when locking/unlocking
- Welcome/goodbye lighting sequences
- Window auto-down behavior

### Instrument Panel Cluster (IPC - 720)
Controls display and gauge behavior:
- Digital speedometer display
- Warning chime volumes
- Language settings
- Welcome animation enable/disable
- Gauge sweep on startup

### Powertrain Control Module (PCM - 7E0)
Engine and transmission parameters:
- Idle speed targets
- Shift firmness
- Speed limiter settings
- Emission control parameters

## Reading As-Built Data

As-built blocks are read using UDS Service 22 (ReadDataByIdentifier) with specific Data Identifiers (DIDs):

```bash
# Read specific block
faraday read-did --module bcm 0x726-01

# Dump all blocks from a module
faraday asbuilt dump --module bcm

# Show human-readable feature status
faraday asbuilt show --module bcm --feature drl
```

## Modifying As-Built Data

**⚠️ Critical Safety Warning**: Modifying as-built data can disable safety systems, void warranties, or render modules inoperable. Always create snapshots before making changes.

### Prerequisites for Safe Modification
1. **Engine off, ignition in KOEO** (Key On Engine Off)
2. **Battery voltage ≥ 12.4V**
3. **No active communication DTCs**
4. **Mandatory snapshot creation** before any write

### Write Process
1. **Security Access**: Authenticate with the module using Service 27
2. **Snapshot**: Create timestamped backup of current configuration
3. **Validation**: Verify the target block/byte/bit is known and safe
4. **Write**: Use UDS Service 2E (WriteDataByIdentifier)
5. **Verification**: Read back and confirm the change

```bash
# Example: Enable DRL (with safety checks)
faraday asbuilt write --module bcm --block 726-01 --byte 3 --bit 2 --value 1 --commit

# Restore from snapshot if needed
faraday asbuilt restore snapshot_20260507_143022.yml
```

## Safety Mechanisms in Faraday

### Mandatory Protections
- **Automatic snapshots**: Every write operation creates a timestamped backup
- **Block validation**: Only known, documented blocks can be modified
- **Programming DID blocking**: Writes to `F1xx` and `F0xx` DIDs are forbidden
- **Dry-run mode**: `--dry-run` shows what would be changed without writing
- **Double confirmation**: Interactive prompts for destructive operations
- **Audit logging**: All operations logged to `~/.local/share/faraday/audit.jsonl`

### Operational Safeguards
- Pre-flight checks for proper vehicle state
- Rate limiting to prevent bus flooding
- Automatic detection of write failures
- Rollback capability for failed configurations

## Module-Specific Considerations

### BCM (Body Control Module)
- Most user-facing features
- Generally safe to modify comfort/convenience settings
- Some blocks control security systems (exercise caution)

### IPC (Instrument Panel Cluster)
- Display and warning settings
- Language and unit preferences
- Usually safe for cosmetic changes

### PCM (Powertrain Control Module)
- **High risk**: Controls engine operation
- Changes can affect emissions compliance
- May trigger DTCs if incompatible with hardware

### APIM (SYNC Module)
- Infotainment system configuration
- Region-specific settings
- Software version dependencies

## Data Sources and Validation

As-built data knowledge comes from:
- **FORScan community databases**
- **Ford service documentation**
- **Reverse engineering efforts**
- **Real vehicle validation**

The `faraday-asbuilt` crate contains curated, validated block definitions specific to the 2017 Fusion.

## Profile-Based Configuration

Future Faraday versions will support YAML configuration profiles:

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

Applied with: `faraday profile apply my-fusion.yml --commit`

## Related Documentation

- [DIDs.md](DIDs.md) - Data identifiers used to access as-built blocks
- [UDS.md](UDS.md) - Protocol details for reading and writing
- [FORScan.md](FORScan.md) - Relationship to FORScan software

## References

- ISO 14229-1 - Unified Diagnostic Services
- FORScan Forum - Community as-built documentation
- CyanLabs - Ford diagnostic resources
- Ford Service Manual - 2017 Fusion Module Configuration
# Unified Diagnostic Services (UDS) - ISO 14229

Unified Diagnostic Services (UDS) is the standardized diagnostic protocol defined by ISO 14229 for communication with automotive electronic control units (ECUs). In the Ford Fusion 2017 SEL, UDS enables advanced diagnostics, configuration, and programming beyond basic OBD-II capabilities.

## Protocol Overview

UDS operates as an application layer protocol running over ISO-TP (ISO 15765-2) transport, providing standardized services for:

- **Diagnostic session management**
- **Security access control**
- **Data reading and writing**
- **Routine execution**
- **File transfer and programming**
- **Error memory management**

## UDS Services Supported by Faraday

| Service ID | Name | Description | Phase |
|------------|------|-------------|--------|
| `0x10` | DiagnosticSessionControl | Establish diagnostic sessions | 2 |
| `0x14` | ClearDiagnosticInformation | Clear stored DTCs | 1 |
| `0x19` | ReadDTCInformation | Advanced DTC reading | 2 |
| `0x22` | ReadDataByIdentifier | Read data via DIDs | 2 |
| `0x27` | SecurityAccess | Authentication for protected operations | 4 |
| `0x2E` | WriteDataByIdentifier | Write data via DIDs | 4 |
| `0x3E` | TesterPresent | Keep session alive | 2 |

### Out of Scope (High Risk)
- `0x34/0x36/0x37` - Request Download/Transfer/Exit (firmware flashing)
- `0x31` - RoutineControl (can trigger dangerous operations)
- `0x11` - ECUReset (unnecessary for configuration tasks)

## Diagnostic Sessions

UDS defines different access levels through diagnostic sessions:

### Session Types

#### Default Session (0x01)
- **Purpose**: Normal vehicle operation
- **Access**: Limited to basic diagnostic information
- **Timeout**: N/A (permanent)
- **Security**: None required

```bash
faraday session --module pcm default
```

#### Extended Session (0x02)
- **Purpose**: Enhanced diagnostics and configuration
- **Access**: As-built data, detailed DTCs, non-volatile memory
- **Timeout**: 5 seconds (requires TesterPresent)
- **Security**: May require security access for some operations

```bash
faraday session --module pcm extended
```

#### Programming Session (0x03)
- **Purpose**: Module programming and calibration
- **Access**: Flash memory, bootloader functions
- **Status**: **Not implemented** (out of scope for Faraday)

### Session Management

#### Automatic TesterPresent
When in extended session, Faraday automatically sends TesterPresent (0x3E) every 2 seconds to prevent session timeout:

```rust
// Background task keeps session alive
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(2)).await;
        transport.send_tester_present().await?;
    }
});
```

## Security Access (Service 0x27)

Security Access implements a seed/key challenge-response mechanism to protect critical vehicle functions.

### Process Flow
1. **Request Seed** (`0x27 0x01`): ECU provides random seed value
2. **Compute Key**: Apply Ford-specific algorithm to generate response key
3. **Send Key** (`0x27 0x02`): Submit computed key for validation
4. **Access Granted**: ECU unlocks protected functions

### Ford Seed/Key Algorithm
Ford uses proprietary algorithms that vary by:
- **Module type** (PCM vs BCM vs IPC)
- **Model year and platform**
- **Security level** (different algorithms for different access levels)

```bash
# Faraday handles seed/key automatically
faraday session --module bcm extended --secure
```

### Security Considerations
- **Limited attempts**: Failed attempts trigger lockout periods
- **Session-based**: Security access expires when session ends
- **Audit logging**: All security access attempts logged for compliance

## Negative Response Codes (NRCs)

UDS defines standardized error codes for failed requests:

### Common NRCs in Faraday Context

| NRC | Name | Description | Common Cause |
|-----|------|-------------|--------------|
| `0x10` | General Reject | Request rejected | Malformed message |
| `0x11` | Service Not Supported | Service unavailable | Wrong session/module |
| `0x12` | Sub-Function Not Supported | Invalid sub-function | Incorrect parameter |
| `0x13` | Incorrect Message Length | Wrong data length | Protocol error |
| `0x21` | Busy Repeat Request | ECU busy | Retry required |
| `0x22` | Conditions Not Correct | Prerequisites not met | Wrong vehicle state |
| `0x31` | Request Out Of Range | Invalid parameter | Bad DID/data |
| `0x33` | Security Access Denied | Authentication failed | Wrong key/no access |
| `0x35` | Invalid Key | Key validation failed | Algorithm error |
| `0x36` | Exceeded Number Of Attempts | Too many failed attempts | Security lockout |
| `0x37` | Required Time Delay Not Expired | Must wait before retry | Cooldown period |
| `0x72` | General Programming Failure | Programming operation failed | Write error |
| `0x78` | Request Correctly Received But Response Pending | Long operation in progress | Wait for completion |

### Error Handling in Faraday

```rust
match response {
    UdsResponse::PositiveResponse(data) => {
        // Process successful response
    },
    UdsResponse::NegativeResponse(nrc) => {
        match nrc {
            Nrc::SecurityAccessDenied => {
                // Attempt re-authentication
            },
            Nrc::ConditionsNotCorrect => {
                // Check vehicle state
            },
            _ => return Err(UdsError::from(nrc)),
        }
    }
}
```

## Protocol Implementation Details

### Message Structure
UDS messages follow this format over ISO-TP:

```
┌─────────────┬─────────────┬──────────────────┐
│ Service ID  │ Sub-function│     Data         │
│   1 byte    │   0-1 byte  │   0-4093 bytes   │
└─────────────┴─────────────┴──────────────────┘
```

### Timing Parameters
- **P2 Timeout**: 50ms (standard response timeout)
- **P2* Timeout**: 5000ms (extended response timeout after 0x78 NRC)
- **S3 Timeout**: 5000ms (session timeout without TesterPresent)

### Multi-Frame Handling
UDS messages larger than 7 bytes use ISO-TP multi-frame protocol:
- **First Frame (FF)**: Contains message length and first data bytes
- **Flow Control (FC)**: Receiver controls transmission timing
- **Consecutive Frames (CF)**: Continue data transmission

## Faraday UDS Implementation

### Transport Abstraction
```rust
#[async_trait]
pub trait IsoTpTransport {
    async fn send_request(&self, data: &[u8]) -> Result<Vec<u8>, TransportError>;
    async fn start_session(&self, session_type: SessionType) -> Result<(), UdsError>;
    async fn send_tester_present(&self) -> Result<(), UdsError>;
}
```

### Command Examples

#### Read DTC Information
```bash
# Read all stored DTCs with status and snapshot data
faraday read-dtc --module pcm --detailed

# Read DTC by status mask
faraday read-dtc --module bcm --status-mask 0x0A
```

#### Data Identifier Operations
```bash
# Read VIN using UDS (alternative to Mode 09)
faraday read-did --module pcm 0xF190

# Write as-built block (requires security access)
faraday write-did --module bcm 0x726A --data "01234567" --commit
```

#### Session Management
```bash
# Extended session with automatic TesterPresent
faraday session --module ipc extended --duration 300s

# Secure session for configuration changes
faraday session --module bcm extended --secure
```

## Module-Specific UDS Behavior

### PCM (Powertrain Control Module)
- **High security**: Multiple security levels
- **Extended timeouts**: Some operations take >5 seconds
- **Strict prerequisites**: Engine state requirements

### BCM (Body Control Module)
- **Medium security**: Single security level
- **Fast response**: Typically <100ms
- **Feature-rich**: Most as-built configuration here

### IPC (Instrument Panel Cluster)
- **Low security**: Limited security access needed
- **Display-focused**: Primarily cosmetic changes
- **Session-sensitive**: Some DIDs only in extended session

## Safety Integration

### Pre-flight Checks
Before UDS operations, Faraday validates:
- Vehicle in KOEO (Key On Engine Off)
- Battery voltage ≥ 12.4V
- No active communication DTCs
- Module responding to ping

### Operation Safeguards
- **Mandatory snapshots** before writes
- **Rollback capability** for failed operations
- **Rate limiting** to prevent bus flooding
- **Session cleanup** on errors or completion

### Audit Trail
All UDS operations logged with:
- Timestamp and operator
- Module and service details
- Request/response data
- Success/failure status

## Related Documentation

- [DIDs.md](DIDs.md) - Data identifiers accessed via Service 0x22/0x2E
- [AsBuilt.md](AsBuilt.md) - Configuration data modified through UDS
- [DTCs.md](DTCs.md) - Enhanced DTC reading via Service 0x19

## References

- ISO 14229-1:2020 - Unified Diagnostic Services - Specification and requirements
- ISO 14229-2:2016 - Unified Diagnostic Services - Session layer services
- ISO 15765-2:2016 - Road vehicles — Diagnostic communication over CAN — Network layer services
- Ford Service Manual - 2017 Fusion UDS Implementation Guide
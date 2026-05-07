/*!
Core library for OBD-II diagnostics and Ford-specific protocols.

This library provides a layered architecture for communicating with Ford vehicle
ECUs through OBD-II adapters, with specific support for the 2017 Ford Fusion SEL.

# Architecture

The library is organized into five layers:

1. **Commands** - High-level operations (ReadDTCs, ReadAsBuilt, etc.)
2. **Protocol** - J1979 (OBD-II) and UDS (ISO 14229) implementations
3. **Transport** - ISO-TP (ISO 15765-2) over CAN frames
4. **Link** - Adapter-specific communication (vLinker FS, ELM327, etc.)
5. **Physical I/O** - Serial port, Bluetooth, or SocketCAN

# Safety Considerations

This library handles automotive diagnostic protocols that can potentially
modify vehicle behavior. All write operations must be performed with
appropriate safety measures including:

- Mandatory snapshots before writes
- Validation against known configuration blocks
- Dry-run modes for testing
- Comprehensive audit logging
*/

pub mod commands;
pub mod link;
pub mod protocol;
pub mod transport;

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;
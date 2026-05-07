/*!
Automotive diagnostic protocol implementations.

Supports:
- SAE J1979 (standard OBD-II)
- ISO 14229 (UDS - Unified Diagnostic Services)

All protocols operate over the ISO-TP transport layer and are independent
of the underlying CAN hardware.
*/

pub mod j1979;
pub mod uds;
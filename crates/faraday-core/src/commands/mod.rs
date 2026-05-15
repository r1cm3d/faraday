/*!
High-level command implementations.

This module provides the command layer that combines protocol operations
into user-facing commands like ReadDTCs, ReadAsBuilt, etc. Each command
handles the complete workflow including module connection, error handling,
and data interpretation.
*/

use crate::transport::IsoTpTransport;

pub mod asbuilt;
pub mod diagnostics;
pub mod live_data;
pub mod vehicle_info;
pub mod write_asbuilt;

/// Top-level executor that exposes all faraday commands over a given transport.
pub struct CommandExecutor<T: IsoTpTransport> {
    transport: T,
}

impl<T: IsoTpTransport> CommandExecutor<T> {
    /// Wraps an existing transport in a new `CommandExecutor`.
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    /// Returns a mutable reference to the underlying transport.
    pub fn transport(&mut self) -> &mut T {
        &mut self.transport
    }
}

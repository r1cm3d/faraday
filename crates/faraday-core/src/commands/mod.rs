/*!
High-level command implementations.

This module provides the command layer that combines protocol operations
into user-facing commands like ReadDTCs, ReadAsBuilt, etc. Each command
handles the complete workflow including module connection, error handling,
and data interpretation.
*/

use crate::{
    transport::IsoTpTransport,
};

pub mod diagnostics;
pub mod live_data;
pub mod vehicle_info;

pub struct CommandExecutor<T: IsoTpTransport> {
    transport: T,
}

impl<T: IsoTpTransport> CommandExecutor<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
        }
    }

    pub fn transport(&mut self) -> &mut T {
        &mut self.transport
    }
}
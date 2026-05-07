/*!
Link layer implementations for various OBD-II adapters.

This module provides adapters for communicating with different OBD-II hardware:
- vLinker FS (primary target)
- ELM327-compatible adapters
- SocketCAN (future Linux native support)

All adapters implement the `LinkLayer` trait which provides a uniform interface
for the transport layer to send and receive CAN frames.
*/

use crate::{CanFrame, Result};
use async_trait::async_trait;

pub mod vlinker;

#[async_trait]
pub trait LinkLayer: Send + Sync {
    async fn connect(&mut self) -> Result<()>;

    async fn disconnect(&mut self) -> Result<()>;

    async fn send_frame(&mut self, frame: &CanFrame) -> Result<()>;

    async fn receive_frame(&mut self) -> Result<CanFrame>;

    async fn set_can_bus(&mut self, bus: crate::CanBus) -> Result<()>;

    fn is_connected(&self) -> bool;
}
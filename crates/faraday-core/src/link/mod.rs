/*!
Link layer implementations for various OBD-II adapters.

This module provides adapters for communicating with different OBD-II hardware:
- vLinker FS (primary target)
- ELM327-compatible adapters
- SocketCAN (future Linux native support)

All adapters implement the `LinkLayer` trait which provides a uniform interface
for the transport layer to send and receive CAN frames.
*/

use crate::{CanBus, CanFrame, Result};
use async_trait::async_trait;

pub mod vlinker;

/// Hardware abstraction for a physical OBD-II adapter.
#[async_trait]
pub trait LinkLayer: Send + Sync {
    /// Opens the adapter connection and initializes the CAN bus.
    async fn connect(&mut self) -> Result<()>;

    /// Closes the adapter connection gracefully.
    async fn disconnect(&mut self) -> Result<()>;

    /// Transmits a single raw CAN frame.
    async fn send_frame(&mut self, frame: &CanFrame) -> Result<()>;

    /// Waits for and returns the next incoming CAN frame.
    async fn receive_frame(&mut self) -> Result<CanFrame>;

    /// Switches the adapter to the specified CAN bus (HS or MS).
    async fn set_can_bus(&mut self, bus: CanBus) -> Result<()>;

    /// Returns `true` if the adapter is currently connected and ready.
    fn is_connected(&self) -> bool;
}

#[async_trait]
impl LinkLayer for Box<dyn LinkLayer> {
    async fn connect(&mut self) -> Result<()> {
        (**self).connect().await
    }

    async fn disconnect(&mut self) -> Result<()> {
        (**self).disconnect().await
    }

    async fn send_frame(&mut self, frame: &CanFrame) -> Result<()> {
        (**self).send_frame(frame).await
    }

    async fn receive_frame(&mut self) -> Result<CanFrame> {
        (**self).receive_frame().await
    }

    async fn set_can_bus(&mut self, bus: CanBus) -> Result<()> {
        (**self).set_can_bus(bus).await
    }

    fn is_connected(&self) -> bool {
        (**self).is_connected()
    }
}

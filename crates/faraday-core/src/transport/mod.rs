/*!
ISO-TP (ISO 15765-2) transport layer implementation.

Provides reliable transport of larger messages over CAN frames supporting:
- Single frames (up to 7 bytes payload)
- Multi-frame sequences with flow control
- Segmentation and reassembly
- Error detection and timeout handling
*/

use crate::{CanBus, CanId, Result};
use async_trait::async_trait;

pub mod isotp;

/// Async ISO-TP transport abstraction used by the protocol layer.
#[async_trait]
pub trait IsoTpTransport: Send + Sync {
    /// Sends `data` as an ISO-TP message with the given `request_id`.
    async fn send(&mut self, request_id: CanId, data: &[u8]) -> Result<()>;

    /// Waits for and reassembles an ISO-TP response addressed to `response_id`.
    async fn receive(&mut self, request_id: CanId, response_id: CanId) -> Result<Vec<u8>>;

    /// Sends `data` and returns the first matching response — the typical request/response cycle.
    async fn request_response(
        &mut self,
        request_id: CanId,
        response_id: CanId,
        data: &[u8],
    ) -> Result<Vec<u8>>;

    /// Adjusts the response timeout for subsequent operations.
    async fn set_timeout(&mut self, timeout: std::time::Duration);

    /// Switches the adapter to the given CAN bus (HS or MS). Default implementation is a no-op.
    async fn set_can_bus(&mut self, _bus: CanBus) -> Result<()> {
        Ok(())
    }
}

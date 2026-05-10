/*!
ISO-TP (ISO 15765-2) transport layer implementation.

Provides reliable transport of larger messages over CAN frames supporting:
- Single frames (up to 7 bytes payload)
- Multi-frame sequences with flow control
- Segmentation and reassembly
- Error detection and timeout handling
*/

use crate::{CanId, Result};
use async_trait::async_trait;

pub mod isotp;

#[async_trait]
pub trait IsoTpTransport: Send + Sync {
    async fn send(&mut self, request_id: CanId, data: &[u8]) -> Result<()>;

    async fn receive(&mut self, request_id: CanId, response_id: CanId) -> Result<Vec<u8>>;

    async fn request_response(&mut self, request_id: CanId, response_id: CanId, data: &[u8]) -> Result<Vec<u8>>;

    async fn set_timeout(&mut self, timeout: std::time::Duration);
}
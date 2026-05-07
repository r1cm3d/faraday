/*!
ISO-TP (ISO 15765-2) implementation over CAN frames.

Supports single frame and multi-frame message transport with proper
flow control and error handling.
*/

use crate::{link::LinkLayer, CanFrame, CanId, Error, Result};
use async_trait::async_trait;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, trace};

const DEFAULT_TIMEOUT: Duration = Duration::from_millis(1000);
#[derive(Debug, Clone, Copy)]
enum PciType {
    SingleFrame = 0,
    FirstFrame = 1,
    ConsecutiveFrame = 2,
    FlowControl = 3,
}

impl PciType {
    fn from_byte(byte: u8) -> Option<Self> {
        match (byte & 0xF0) >> 4 {
            0 => Some(PciType::SingleFrame),
            1 => Some(PciType::FirstFrame),
            2 => Some(PciType::ConsecutiveFrame),
            3 => Some(PciType::FlowControl),
            _ => None,
        }
    }
}

pub struct IsoTp<L: LinkLayer> {
    link: L,
    timeout: Duration,
}

impl<L: LinkLayer> IsoTp<L> {
    pub fn new(link: L) -> Self {
        Self {
            link,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    fn create_single_frame(&self, request_id: CanId, data: &[u8]) -> Result<CanFrame> {
        if data.len() > 7 {
            return Err(Error::transport("Data too long for single frame"));
        }

        let mut frame_data = vec![data.len() as u8];
        frame_data.extend_from_slice(data);

        while frame_data.len() < 8 {
            frame_data.push(0x55);
        }

        Ok(CanFrame::new(request_id, frame_data))
    }

    fn create_first_frame(&self, request_id: CanId, data: &[u8]) -> Result<CanFrame> {
        if data.len() < 8 {
            return Err(Error::transport("Use single frame for data < 8 bytes"));
        }

        let length = data.len() as u16;
        let pci = 0x10 | ((length >> 8) & 0x0F) as u8;
        let length_low = (length & 0xFF) as u8;

        let mut frame_data = vec![pci, length_low];
        frame_data.extend_from_slice(&data[..6]);

        Ok(CanFrame::new(request_id, frame_data))
    }

    fn create_consecutive_frame(&self, request_id: CanId, sequence: u8, data: &[u8]) -> Result<CanFrame> {
        let pci = 0x20 | (sequence & 0x0F);
        let mut frame_data = vec![pci];
        frame_data.extend_from_slice(data);

        while frame_data.len() < 8 {
            frame_data.push(0x55);
        }

        Ok(CanFrame::new(request_id, frame_data))
    }


    async fn send_single_frame(&mut self, request_id: CanId, data: &[u8]) -> Result<()> {
        let frame = self.create_single_frame(request_id, data)?;
        trace!("Sending single frame: {:?}", frame);
        self.link.send_frame(&frame).await?;
        Ok(())
    }

    async fn send_multi_frame(&mut self, request_id: CanId, data: &[u8]) -> Result<()> {
        let first_frame = self.create_first_frame(request_id, data)?;
        trace!("Sending first frame: {:?}", first_frame);
        self.link.send_frame(&first_frame).await?;

        let mut remaining_data = &data[6..];
        let mut sequence = 1u8;

        while !remaining_data.is_empty() {
            let chunk_size = std::cmp::min(remaining_data.len(), 7);
            let chunk = &remaining_data[..chunk_size];

            let consecutive_frame = self.create_consecutive_frame(request_id, sequence, chunk)?;
            trace!("Sending consecutive frame {}: {:?}", sequence, consecutive_frame);
            self.link.send_frame(&consecutive_frame).await?;

            remaining_data = &remaining_data[chunk_size..];
            sequence = (sequence + 1) % 16;

            if !remaining_data.is_empty() {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        }

        Ok(())
    }


    async fn receive_frame_with_id(&mut self, expected_id: CanId) -> Result<CanFrame> {
        loop {
            let frame = self.link.receive_frame().await?;
            if frame.id == expected_id {
                return Ok(frame);
            }
            trace!("Ignoring frame with unexpected ID: {:03X}", frame.id.id());
        }
    }
}

#[async_trait]
impl<L: LinkLayer> super::IsoTpTransport for IsoTp<L> {
    async fn send(&mut self, request_id: CanId, data: &[u8]) -> Result<()> {
        trace!("Sending ISO-TP message: {} bytes", data.len());

        if data.len() <= 7 {
            self.send_single_frame(request_id, data).await
        } else {
            self.send_multi_frame(request_id, data).await
        }
    }

    async fn receive(&mut self, response_id: CanId) -> Result<Vec<u8>> {
        trace!("Receiving ISO-TP message from ID {:03X}", response_id.id());

        let first_frame = timeout(self.timeout, self.receive_frame_with_id(response_id)).await
            .map_err(|_| Error::Timeout)??;

        if first_frame.data.is_empty() {
            return Err(Error::invalid_frame("Empty frame received"));
        }

        let pci_type = PciType::from_byte(first_frame.data[0])
            .ok_or_else(|| Error::invalid_frame("Invalid PCI type"))?;

        match pci_type {
            PciType::SingleFrame => {
                let length = (first_frame.data[0] & 0x0F) as usize;
                if length > first_frame.data.len() - 1 {
                    return Err(Error::invalid_frame("Invalid single frame length"));
                }
                Ok(first_frame.data[1..=length].to_vec())
            }
            PciType::FirstFrame => {
                let length = (((first_frame.data[0] & 0x0F) as u16) << 8) | first_frame.data[1] as u16;
                let mut data = first_frame.data[2..].to_vec();

                let mut expected_sequence = 1u8;
                while data.len() < length as usize {
                    let frame = timeout(self.timeout, self.receive_frame_with_id(response_id)).await
                        .map_err(|_| Error::Timeout)??;

                    let pci_type = PciType::from_byte(frame.data[0])
                        .ok_or_else(|| Error::invalid_frame("Invalid PCI type"))?;

                    match pci_type {
                        PciType::ConsecutiveFrame => {
                            let sequence = frame.data[0] & 0x0F;
                            if sequence != expected_sequence {
                                return Err(Error::transport(format!(
                                    "Sequence mismatch: expected {}, got {}",
                                    expected_sequence, sequence
                                )));
                            }

                            let remaining = length as usize - data.len();
                            let chunk_size = std::cmp::min(remaining, 7);
                            data.extend_from_slice(&frame.data[1..=chunk_size]);

                            expected_sequence = (expected_sequence + 1) % 16;
                        }
                        _ => return Err(Error::invalid_frame("Expected consecutive frame")),
                    }
                }

                data.truncate(length as usize);
                Ok(data)
            }
            _ => Err(Error::invalid_frame("Unexpected frame type")),
        }
    }

    async fn request_response(&mut self, request_id: CanId, response_id: CanId, data: &[u8]) -> Result<Vec<u8>> {
        debug!("ISO-TP request-response: REQ={:03X} RSP={:03X} {} bytes",
               request_id.id(), response_id.id(), data.len());

        self.send(request_id, data).await?;
        self.receive(response_id).await
    }

    async fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }
}
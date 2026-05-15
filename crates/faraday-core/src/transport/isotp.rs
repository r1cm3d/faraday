/*!
ISO-TP (ISO 15765-2) implementation over CAN frames.

Supports single frame and multi-frame message transport with proper
flow control and error handling.
*/

use crate::{link::LinkLayer, CanBus, CanFrame, CanId, Error, Result};
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

/// ISO-TP (ISO 15765-2) transport implemented over a [`LinkLayer`].
pub struct IsoTp<L: LinkLayer> {
    link: L,
    timeout: Duration,
}

impl<L: LinkLayer> IsoTp<L> {
    /// Wraps a link-layer adapter in a new `IsoTp` transport with the default 1-second timeout.
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

    fn create_consecutive_frame(
        &self,
        request_id: CanId,
        sequence: u8,
        data: &[u8],
    ) -> Result<CanFrame> {
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
            trace!(
                "Sending consecutive frame {}: {:?}",
                sequence,
                consecutive_frame
            );
            self.link.send_frame(&consecutive_frame).await?;

            remaining_data = &remaining_data[chunk_size..];
            sequence = (sequence + 1) % 16;

            if !remaining_data.is_empty() {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        }

        Ok(())
    }

    fn create_flow_control_frame(&self, request_id: CanId) -> CanFrame {
        CanFrame::new(
            request_id,
            vec![0x30, 0x00, 0x00, 0x55, 0x55, 0x55, 0x55, 0x55],
        )
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

    async fn receive(&mut self, request_id: CanId, response_id: CanId) -> Result<Vec<u8>> {
        trace!("Receiving ISO-TP message from ID {:03X}", response_id.id());

        let first_frame = timeout(self.timeout, self.receive_frame_with_id(response_id))
            .await
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
                let length =
                    (((first_frame.data[0] & 0x0F) as u16) << 8) | first_frame.data[1] as u16;
                let mut data = first_frame.data[2..].to_vec();

                let fc_frame = self.create_flow_control_frame(request_id);
                self.link.send_frame(&fc_frame).await?;
                trace!("Sent Flow Control frame to {:03X}", request_id.id());

                let mut expected_sequence = 1u8;
                while data.len() < length as usize {
                    let frame = timeout(self.timeout, self.receive_frame_with_id(response_id))
                        .await
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

    async fn request_response(
        &mut self,
        request_id: CanId,
        response_id: CanId,
        data: &[u8],
    ) -> Result<Vec<u8>> {
        debug!(
            "ISO-TP request-response: REQ={:03X} RSP={:03X} {} bytes",
            request_id.id(),
            response_id.id(),
            data.len()
        );

        self.send(request_id, data).await?;
        self.receive(request_id, response_id).await
    }

    async fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    async fn set_can_bus(&mut self, bus: CanBus) -> Result<()> {
        self.link.set_can_bus(bus).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::IsoTpTransport;
    use crate::{CanBus, CanFrame, CanId};
    use async_trait::async_trait;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    struct MockLinkLayer {
        sent_frames: Arc<Mutex<Vec<CanFrame>>>,
        recv_queue: Arc<Mutex<VecDeque<CanFrame>>>,
    }

    impl MockLinkLayer {
        fn new(recv_frames: Vec<CanFrame>) -> Self {
            Self {
                sent_frames: Arc::new(Mutex::new(Vec::new())),
                recv_queue: Arc::new(Mutex::new(VecDeque::from(recv_frames))),
            }
        }
    }

    #[async_trait]
    impl crate::link::LinkLayer for MockLinkLayer {
        async fn connect(&mut self) -> crate::Result<()> {
            Ok(())
        }
        async fn disconnect(&mut self) -> crate::Result<()> {
            Ok(())
        }

        async fn send_frame(&mut self, frame: &CanFrame) -> crate::Result<()> {
            self.sent_frames.lock().unwrap().push(frame.clone());
            Ok(())
        }

        async fn receive_frame(&mut self) -> crate::Result<CanFrame> {
            self.recv_queue
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| crate::Error::link("mock recv queue exhausted"))
        }

        async fn set_can_bus(&mut self, _bus: CanBus) -> crate::Result<()> {
            Ok(())
        }

        fn is_connected(&self) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn receive_multi_frame_sends_flow_control_and_reassembles() {
        let request_id = CanId::new(0x7E0);
        let response_id = CanId::new(0x7E8);

        // 13-byte payload 0x01..=0x0D split across a FirstFrame + one ConsecutiveFrame
        let first_frame = CanFrame::new(
            response_id,
            vec![0x10, 0x0D, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
        );
        let consec_frame = CanFrame::new(
            response_id,
            vec![0x21, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D],
        );

        let mock = MockLinkLayer::new(vec![first_frame, consec_frame]);
        let sent_ref = mock.sent_frames.clone();

        let mut transport = IsoTp::new(mock);
        let result: Vec<u8> = transport.receive(request_id, response_id).await.unwrap();

        assert_eq!(result, (1u8..=13).collect::<Vec<u8>>());

        let sent = sent_ref.lock().unwrap();
        assert_eq!(sent.len(), 1, "expected exactly one FC frame sent");
        assert_eq!(sent[0].id, request_id, "FC must be addressed to request_id");
        assert_eq!(
            sent[0].data,
            vec![0x30, 0x00, 0x00, 0x55, 0x55, 0x55, 0x55, 0x55],
            "FC must be ContinueToSend with block_size=0 and STmin=0"
        );
    }

    proptest::proptest! {
        #[test]
        fn single_frame_receive_no_fc_sent(
            payload in proptest::collection::vec(proptest::num::u8::ANY, 1usize..=7usize)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let request_id = CanId::new(0x7E0);
                let response_id = CanId::new(0x7E8);

                let len = payload.len() as u8;
                let mut frame_data = vec![len];
                frame_data.extend_from_slice(&payload);
                while frame_data.len() < 8 {
                    frame_data.push(0x55);
                }

                let mock = MockLinkLayer::new(vec![CanFrame::new(response_id, frame_data)]);
                let sent_ref = mock.sent_frames.clone();

                let mut transport = IsoTp::new(mock);
                let result: Vec<u8> = transport.receive(request_id, response_id).await.unwrap();

                assert_eq!(result, payload);
                assert!(sent_ref.lock().unwrap().is_empty(), "no FC for single-frame");
            });
        }
    }
}

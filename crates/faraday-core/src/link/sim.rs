use crate::{CanBus, CanFrame, CanId, Error, Result};
use async_trait::async_trait;
use std::collections::VecDeque;

pub struct SimulatedEcu {
    response_queue: VecDeque<CanFrame>,
}

impl SimulatedEcu {
    pub fn new() -> Self {
        Self {
            response_queue: VecDeque::new(),
        }
    }

    fn response_id_for(request_id: CanId) -> CanId {
        match request_id.id() {
            0x7DF | 0x7E0 => CanId::new(0x7E8),
            0x7E1 => CanId::new(0x7E9),
            0x7E2 => CanId::new(0x7EA),
            0x7E3 => CanId::new(0x7EB),
            0x7E4 => CanId::new(0x7EC),
            0x726 => CanId::new(0x72E),
            0x720 => CanId::new(0x728),
            0x7D0 => CanId::new(0x7D8),
            0x733 => CanId::new(0x73B),
            0x727 => CanId::new(0x72F),
            _ => CanId::new(0x7E8),
        }
    }

    fn pid_bytes(pid: u8) -> Vec<u8> {
        match pid {
            0x04 => vec![0x5A],       // engine load ~35%
            0x05 => vec![0x69],       // coolant 65°C
            0x0C => vec![0x0C, 0x80], // 800 rpm
            0x0D => vec![0x00],       // 0 km/h
            0x0F => vec![0x45],       // intake 29°C
            0x10 => vec![0x00, 0x1E], // MAF 0.30 g/s
            0x11 => vec![0x00],       // throttle 0%
            0x2F => vec![0xB3],       // fuel ~70%
            0x42 => vec![0x37, 0xB8], // 14.26V
            0x46 => vec![0x37],       // ambient 15°C
            0x5C => vec![0x69],       // oil 65°C
            _ => vec![0x00],
        }
    }

    fn generate_response(payload: &[u8]) -> Option<Vec<u8>> {
        if payload.is_empty() {
            return None;
        }
        match payload[0] {
            0x01 => {
                let mut resp = vec![0x41];
                for &pid in &payload[1..] {
                    resp.push(pid);
                    resp.extend(Self::pid_bytes(pid));
                }
                Some(resp)
            }
            0x03 => Some(vec![0x43, 0x00]),
            0x04 => Some(vec![0x44]),
            0x07 => Some(vec![0x47, 0x00]),
            0x09 if payload.len() >= 2 && payload[1] == 0x02 => {
                let mut resp = vec![0x49, 0x02, 0x01];
                resp.extend_from_slice(b"1FA6P8TH5H5123456");
                Some(resp)
            }
            0x0A => Some(vec![0x4A, 0x00]),
            0x10 if payload.len() >= 2 => {
                Some(vec![0x50, payload[1], 0x00, 0x19, 0x01, 0xF4])
            }
            0x22 if payload.len() >= 3 => {
                let did = ((payload[1] as u16) << 8) | payload[2] as u16;
                let mut resp = vec![0x62, payload[1], payload[2]];
                match did {
                    0xF190 => resp.extend_from_slice(b"1FA6P8TH5H5123456"),
                    0xF191 => resp.extend_from_slice(b"HU5T-14G370-AA"),
                    0xF194 => resp.extend_from_slice(b"HU5T-14G370-AAB"),
                    0xF18A => resp.extend_from_slice(b"FORD"),
                    0xF18B => resp.extend_from_slice(b"170101"),
                    0xF18C => resp.extend_from_slice(b"SIM000001"),
                    _ => resp.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]),
                }
                Some(resp)
            }
            0x3E => Some(vec![0x7E, 0x00]),
            _ => None,
        }
    }

    fn enqueue_iso_tp(&mut self, response_id: CanId, data: &[u8]) {
        if data.len() <= 7 {
            let mut frame = vec![data.len() as u8];
            frame.extend_from_slice(data);
            while frame.len() < 8 {
                frame.push(0x55);
            }
            self.response_queue.push_back(CanFrame::new(response_id, frame));
        } else {
            let total = data.len() as u16;
            let mut first = vec![0x10 | ((total >> 8) & 0x0F) as u8, (total & 0xFF) as u8];
            first.extend_from_slice(&data[..6]);
            self.response_queue.push_back(CanFrame::new(response_id, first));

            let mut offset = 6;
            let mut seq = 1u8;
            while offset < data.len() {
                let end = (offset + 7).min(data.len());
                let mut frame = vec![0x20 | (seq & 0x0F)];
                frame.extend_from_slice(&data[offset..end]);
                while frame.len() < 8 {
                    frame.push(0x55);
                }
                self.response_queue.push_back(CanFrame::new(response_id, frame));
                offset = end;
                seq = (seq + 1) % 16;
            }
        }
    }
}

impl Default for SimulatedEcu {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl super::LinkLayer for SimulatedEcu {
    async fn connect(&mut self) -> Result<()> {
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        Ok(())
    }

    async fn send_frame(&mut self, frame: &CanFrame) -> Result<()> {
        if frame.data.is_empty() {
            return Ok(());
        }
        let pci_type = (frame.data[0] & 0xF0) >> 4;
        if pci_type == 3 {
            return Ok(());
        }
        if pci_type == 0 {
            let len = (frame.data[0] & 0x0F) as usize;
            if len > 0 && frame.data.len() >= 1 + len {
                let payload = &frame.data[1..1 + len];
                if let Some(response) = Self::generate_response(payload) {
                    let resp_id = Self::response_id_for(frame.id);
                    self.enqueue_iso_tp(resp_id, &response);
                }
            }
        }
        Ok(())
    }

    async fn receive_frame(&mut self) -> Result<CanFrame> {
        self.response_queue
            .pop_front()
            .ok_or_else(|| Error::link("no simulated response available"))
    }

    async fn set_can_bus(&mut self, _bus: CanBus) -> Result<()> {
        Ok(())
    }

    fn is_connected(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::link::LinkLayer;

    #[tokio::test]
    async fn responds_to_mode01_rpm_request() {
        let mut ecu = SimulatedEcu::new();
        let req = CanFrame::new(CanId::new(0x7DF), vec![0x02, 0x01, 0x0C, 0x55, 0x55, 0x55, 0x55, 0x55]);
        ecu.send_frame(&req).await.unwrap();
        let resp = ecu.receive_frame().await.unwrap();
        assert_eq!(resp.id, CanId::new(0x7E8));
        assert_eq!(resp.data[0], 0x04); // SF length 4
        assert_eq!(resp.data[1], 0x41); // positive response to 0x01
        assert_eq!(resp.data[2], 0x0C); // PID echo
        assert_eq!(resp.data[3..5], [0x0C, 0x80]); // 800 rpm
    }

    #[tokio::test]
    async fn responds_to_mode03_with_no_dtcs() {
        let mut ecu = SimulatedEcu::new();
        let req = CanFrame::new(CanId::new(0x7DF), vec![0x01, 0x03, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55]);
        ecu.send_frame(&req).await.unwrap();
        let resp = ecu.receive_frame().await.unwrap();
        assert_eq!(resp.data[1], 0x43);
        assert_eq!(resp.data[2], 0x00);
    }

    #[tokio::test]
    async fn responds_to_vin_request_as_multiframe() {
        let mut ecu = SimulatedEcu::new();
        let req = CanFrame::new(CanId::new(0x7DF), vec![0x02, 0x09, 0x02, 0x55, 0x55, 0x55, 0x55, 0x55]);
        ecu.send_frame(&req).await.unwrap();
        let ff = ecu.receive_frame().await.unwrap();
        assert_eq!((ff.data[0] & 0xF0) >> 4, 1); // first frame
        let total_len = (((ff.data[0] & 0x0F) as u16) << 8) | ff.data[1] as u16;
        assert_eq!(total_len, 20); // [0x49, 0x02, 0x01] + 17 VIN bytes
    }

    #[tokio::test]
    async fn flow_control_does_not_enqueue_response() {
        let mut ecu = SimulatedEcu::new();
        let fc = CanFrame::new(CanId::new(0x7DF), vec![0x30, 0x00, 0x00, 0x55, 0x55, 0x55, 0x55, 0x55]);
        ecu.send_frame(&fc).await.unwrap();
        assert!(ecu.receive_frame().await.is_err());
    }
}

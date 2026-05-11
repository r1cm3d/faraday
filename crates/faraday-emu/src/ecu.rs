use faraday_core::{CanFrame, CanId};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::collections::VecDeque;

struct SimState {
    rng: SmallRng,
}

impl SimState {
    fn new() -> Self {
        Self {
            rng: SmallRng::from_entropy(),
        }
    }

    fn jitter(&mut self, base: f64, deviation: f64) -> f64 {
        base + self.rng.gen_range(-deviation..=deviation)
    }
}

pub struct SimulatedEcu {
    response_queue: VecDeque<CanFrame>,
    state: SimState,
}

impl SimulatedEcu {
    pub fn new() -> Self {
        Self {
            response_queue: VecDeque::new(),
            state: SimState::new(),
        }
    }

    pub fn process_frame(&mut self, frame: &CanFrame) {
        if frame.data.is_empty() {
            return;
        }
        let pci_type = (frame.data[0] & 0xF0) >> 4;
        if pci_type == 3 {
            return;
        }
        if pci_type == 0 {
            let len = (frame.data[0] & 0x0F) as usize;
            if len > 0 && frame.data.len() > len {
                let payload: Vec<u8> = frame.data[1..1 + len].to_vec();
                let request_id = frame.id.id();
                if let Some(response) = self.generate_response(request_id, &payload) {
                    let resp_id = Self::response_id_for(frame.id);
                    self.enqueue_iso_tp(resp_id, &response);
                }
            }
        }
    }

    pub fn pop_response(&mut self) -> Option<CanFrame> {
        self.response_queue.pop_front()
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
            0x703 => CanId::new(0x70B),
            _ => CanId::new(0x7E8),
        }
    }

    fn pid_bytes(&mut self, pid: u8) -> Vec<u8> {
        match pid {
            0x04 => {
                let load = self.state.jitter(12.0, 3.0).clamp(0.0, 100.0);
                vec![(load * 255.0 / 100.0) as u8]
            }
            0x05 => {
                let temp = self.state.jitter(92.0, 2.0).clamp(-40.0, 215.0);
                vec![(temp + 40.0) as u8]
            }
            0x06 => {
                let stft = self.state.jitter(1.5, 3.0).clamp(-100.0, 99.2);
                vec![((stft * 128.0 / 100.0) + 128.0).clamp(0.0, 255.0) as u8]
            }
            0x07 => {
                let ltft = self.state.jitter(0.8, 1.5).clamp(-100.0, 99.2);
                vec![((ltft * 128.0 / 100.0) + 128.0).clamp(0.0, 255.0) as u8]
            }
            0x08 => {
                let stft = self.state.jitter(2.0, 3.0).clamp(-100.0, 99.2);
                vec![((stft * 128.0 / 100.0) + 128.0).clamp(0.0, 255.0) as u8]
            }
            0x09 => {
                let ltft = self.state.jitter(1.2, 1.5).clamp(-100.0, 99.2);
                vec![((ltft * 128.0 / 100.0) + 128.0).clamp(0.0, 255.0) as u8]
            }
            0x0C => {
                let rpm = self.state.jitter(800.0, 60.0).clamp(0.0, 16383.75);
                let raw = (rpm * 4.0) as u16;
                vec![(raw >> 8) as u8, (raw & 0xFF) as u8]
            }
            0x0D => vec![0x00],
            0x0E => {
                let timing = self.state.jitter(12.0, 3.0).clamp(-64.0, 63.5);
                vec![((timing + 64.0) * 2.0) as u8]
            }
            0x0F => {
                let temp = self.state.jitter(32.0, 4.0).clamp(-40.0, 215.0);
                vec![(temp + 40.0) as u8]
            }
            0x10 => {
                let maf = self.state.jitter(4.0, 1.0).clamp(0.0, 655.35);
                let raw = (maf * 100.0) as u16;
                vec![(raw >> 8) as u8, (raw & 0xFF) as u8]
            }
            0x11 => {
                let throttle = self.state.jitter(3.0, 2.0).clamp(0.0, 100.0);
                vec![(throttle * 255.0 / 100.0) as u8]
            }
            0x14 => {
                let v = self.state.jitter(0.65, 0.1).clamp(0.0, 1.275);
                vec![(v / 0.005) as u8]
            }
            0x15 => {
                let v = self.state.jitter(0.70, 0.1).clamp(0.0, 1.275);
                vec![(v / 0.005) as u8]
            }
            0x2C => {
                let egr = self.state.jitter(5.0, 3.0).clamp(0.0, 100.0);
                vec![(egr * 255.0 / 100.0) as u8]
            }
            0x2D => {
                let err = self.state.jitter(0.0, 2.0).clamp(-100.0, 99.2);
                vec![((err * 128.0 / 100.0) + 128.0).clamp(0.0, 255.0) as u8]
            }
            0x2F => vec![0xB3],
            0x42 => {
                let v = self.state.jitter(14.1, 0.15).clamp(0.0, 65.535);
                let raw = (v * 1000.0) as u16;
                vec![(raw >> 8) as u8, (raw & 0xFF) as u8]
            }
            0x46 => vec![0x37],
            0x5A => {
                let throttle = self.state.jitter(3.0, 2.0).clamp(0.0, 100.0);
                vec![(throttle * 255.0 / 100.0) as u8]
            }
            0x5C => {
                let temp = self.state.jitter(90.0, 4.0).clamp(-40.0, 215.0);
                vec![(temp + 40.0) as u8]
            }
            0x5E => {
                let rate = self.state.jitter(1.0, 0.3).clamp(0.0, 3276.75);
                let raw = (rate / 0.05) as u16;
                vec![(raw >> 8) as u8, (raw & 0xFF) as u8]
            }
            0x61 => {
                let torque = self.state.jitter(5.0, 5.0).clamp(-125.0, 130.0);
                vec![(torque + 125.0) as u8]
            }
            0x62 => {
                let torque = self.state.jitter(8.0, 5.0).clamp(-125.0, 130.0);
                vec![(torque + 125.0) as u8]
            }
            _ => vec![0x00],
        }
    }

    fn generate_response(&mut self, request_id: u32, payload: &[u8]) -> Option<Vec<u8>> {
        if payload.is_empty() {
            return None;
        }
        match payload[0] {
            0x01 => {
                let mut resp = vec![0x41];
                let pids: Vec<u8> = payload[1..].to_vec();
                for pid in pids {
                    resp.push(pid);
                    let bytes = self.pid_bytes(pid);
                    resp.extend(bytes);
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
            0x10 if payload.len() >= 2 => Some(vec![0x50, payload[1], 0x00, 0x19, 0x01, 0xF4]),
            0x22 if payload.len() >= 3 => {
                let did = ((payload[1] as u16) << 8) | payload[2] as u16;
                let data = match request_id {
                    0x7DF | 0x7E0 => self.did_pcm(did),
                    0x7E1 => self.did_tcm(did),
                    0x7E2 => self.did_abs(did),
                    0x7E3 => self.did_rcm(did),
                    0x726 => self.did_bcm(did),
                    0x720 => self.did_ipc(did),
                    0x7D0 => self.did_apim(did),
                    0x733 => self.did_pam(did),
                    0x703 => self.did_hvac(did),
                    _ => vec![0x00; 4],
                };
                let mut resp = vec![0x62, payload[1], payload[2]];
                resp.extend_from_slice(&data);
                Some(resp)
            }
            0x3E => Some(vec![0x7E, 0x00]),
            _ => None,
        }
    }

    fn did_pcm(&mut self, did: u16) -> Vec<u8> {
        match did {
            0xF190 => b"1FA6P8TH5H5123456".to_vec(),
            0xF191 => b"HU5T-14G370-AA".to_vec(),
            0xF194 => b"HU5T-14G370-AAB".to_vec(),
            0xF18A => b"FORD".to_vec(),
            0xF18B => b"170101".to_vec(),
            0xF18C => b"SIM000001".to_vec(),
            0xE001 => vec![0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00],
            0x1900 => vec![75u8],
            0x1901 => {
                let cyl1: u16 = if self.state.rng.gen_bool(0.05) {
                    self.state.rng.gen_range(1..=3)
                } else {
                    0
                };
                let mut data = Vec::with_capacity(8);
                for count in [cyl1, 0u16, 0u16, 0u16] {
                    data.push((count >> 8) as u8);
                    data.push((count & 0xFF) as u8);
                }
                data
            }
            0xF101 => vec![0x01],
            _ => vec![0x00; 4],
        }
    }

    fn did_tcm(&mut self, did: u16) -> Vec<u8> {
        match did {
            0xDD01 => {
                let temp = self.state.jitter(90.0, 3.0).clamp(-40.0, 215.0);
                vec![(temp + 40.0) as u8]
            }
            0xDD02 => vec![0x00, 0x00],
            0xDD03 => {
                let slip = self.state.jitter(1.0, 0.8).clamp(0.0, 6553.5);
                let raw = (slip * 10.0) as u16;
                vec![(raw >> 8) as u8, (raw & 0xFF) as u8]
            }
            0xF101 => vec![0x01],
            _ => vec![0x00; 4],
        }
    }

    fn did_abs(&mut self, did: u16) -> Vec<u8> {
        match did {
            0xC001 => {
                let mut data = Vec::with_capacity(8);
                for _ in 0..4 {
                    let raw = (self.state.rng.gen_range(0.0f64..=0.3) * 10.0) as u16;
                    data.push((raw >> 8) as u8);
                    data.push((raw & 0xFF) as u8);
                }
                data
            }
            0xC002 => {
                let yaw = self.state.jitter(0.0, 0.3);
                let lat = self.state.jitter(0.0, 0.02);
                let yaw_raw = (yaw * 10.0) as i16;
                let lat_raw = (lat * 100.0) as i16;
                vec![
                    (yaw_raw >> 8) as u8,
                    (yaw_raw & 0xFF) as u8,
                    (lat_raw >> 8) as u8,
                    (lat_raw & 0xFF) as u8,
                ]
            }
            0xF101 => vec![0x01],
            _ => vec![0x00; 4],
        }
    }

    fn did_rcm(&self, did: u16) -> Vec<u8> {
        match did {
            0xD001 => vec![0xFF],
            0xD002 => vec![0x01],
            0xF101 => vec![0x01],
            _ => vec![0x00; 4],
        }
    }

    fn did_bcm(&mut self, did: u16) -> Vec<u8> {
        match did {
            0x0201 => vec![0x23, 0x23],
            0x0701 => vec![0x00, 0x00, 0x00, 0x04, 0x03, 0x00, 0x00, 0x00],
            0x0702 => vec![0x00, 0x08, 0x01, 0x00, 0x00, 0x02, 0x00, 0x00],
            0x4001 => {
                let v = self.state.jitter(14.1, 0.15).clamp(0.0, 65.535);
                let raw = (v * 1000.0) as u16;
                vec![(raw >> 8) as u8, (raw & 0xFF) as u8]
            }
            0x4002 => {
                let duty = self.state.jitter(70.0, 5.0).clamp(0.0, 100.0);
                vec![(duty * 255.0 / 100.0) as u8]
            }
            0x4010 => vec![0x00],
            0xF101 => vec![0x01],
            _ => vec![0x00; 4],
        }
    }

    fn did_ipc(&self, did: u16) -> Vec<u8> {
        match did {
            0x0101 => vec![0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00],
            0x0102 => vec![0x80, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00],
            0x0401 => vec![0x08],
            0xF101 => vec![0x01],
            _ => vec![0x00; 4],
        }
    }

    fn did_apim(&mut self, did: u16) -> Vec<u8> {
        match did {
            0xA001 => {
                let sats: u8 = self.state.rng.gen_range(8..=12);
                vec![0x01, sats]
            }
            0xA002 => {
                let rssi = self.state.jitter(-68.0, 4.0).clamp(-120.0, 0.0);
                vec![rssi as i8 as u8]
            }
            0xA010 => b"3.4.19200\0".to_vec(),
            0xF101 => vec![0x01],
            _ => vec![0x00; 4],
        }
    }

    fn did_pam(&mut self, did: u16) -> Vec<u8> {
        match did {
            0xE001 => {
                let mut data = Vec::with_capacity(16);
                for _ in 0..8 {
                    let dist: u16 = self.state.rng.gen_range(200..=500);
                    data.push((dist >> 8) as u8);
                    data.push((dist & 0xFF) as u8);
                }
                data
            }
            0xF101 => vec![0x01],
            _ => vec![0x00; 4],
        }
    }

    fn did_hvac(&mut self, did: u16) -> Vec<u8> {
        match did {
            0xB001 => {
                let blend_d = self.state.jitter(50.0, 5.0).clamp(0.0, 100.0);
                let blend_p = self.state.jitter(50.0, 5.0).clamp(0.0, 100.0);
                let cabin_d = self.state.jitter(22.0, 1.0).clamp(-40.0, 215.0);
                let cabin_p = self.state.jitter(22.0, 1.0).clamp(-40.0, 215.0);
                vec![
                    (blend_d * 255.0 / 100.0) as u8,
                    (blend_p * 255.0 / 100.0) as u8,
                    (cabin_d + 40.0) as u8,
                    (cabin_p + 40.0) as u8,
                ]
            }
            0xB002 => {
                let evap = self.state.jitter(6.0, 2.0).clamp(-40.0, 215.0);
                let pressure = self.state.jitter(1050.0, 100.0).clamp(0.0, 6553.5);
                let compressor = self.state.jitter(40.0, 8.0).clamp(0.0, 100.0);
                let p_raw = (pressure * 10.0) as u16;
                vec![
                    (evap + 40.0) as u8,
                    (p_raw >> 8) as u8,
                    (p_raw & 0xFF) as u8,
                    (compressor * 255.0 / 100.0) as u8,
                ]
            }
            0xF101 => vec![0x01],
            _ => vec![0x00; 4],
        }
    }

    fn enqueue_iso_tp(&mut self, response_id: CanId, data: &[u8]) {
        if data.len() <= 7 {
            let mut frame = vec![data.len() as u8];
            frame.extend_from_slice(data);
            while frame.len() < 8 {
                frame.push(0x55);
            }
            self.response_queue
                .push_back(CanFrame::new(response_id, frame));
        } else {
            let total = data.len() as u16;
            let mut first = vec![0x10 | ((total >> 8) & 0x0F) as u8, (total & 0xFF) as u8];
            first.extend_from_slice(&data[..6]);
            self.response_queue
                .push_back(CanFrame::new(response_id, first));

            let mut offset = 6;
            let mut seq = 1u8;
            while offset < data.len() {
                let end = (offset + 7).min(data.len());
                let mut frame = vec![0x20 | (seq & 0x0F)];
                frame.extend_from_slice(&data[offset..end]);
                while frame.len() < 8 {
                    frame.push(0x55);
                }
                self.response_queue
                    .push_back(CanFrame::new(response_id, frame));
                offset = end;
                seq = (seq + 1) % 16;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use faraday_core::CanId;

    #[test]
    fn responds_to_mode01_rpm_request() {
        let mut ecu = SimulatedEcu::new();
        let req = CanFrame::new(
            CanId::new(0x7DF),
            vec![0x02, 0x01, 0x0C, 0x55, 0x55, 0x55, 0x55, 0x55],
        );
        ecu.process_frame(&req);
        let resp = ecu.pop_response().unwrap();
        assert_eq!(resp.id, CanId::new(0x7E8));
        assert_eq!(resp.data[0], 0x04);
        assert_eq!(resp.data[1], 0x41);
        assert_eq!(resp.data[2], 0x0C);
        let raw_rpm = ((resp.data[3] as u16) << 8) | resp.data[4] as u16;
        assert!(
            raw_rpm >= 2960 && raw_rpm <= 3440,
            "RPM raw {raw_rpm} outside expected idle range"
        );
    }

    #[test]
    fn responds_to_mode03_with_no_dtcs() {
        let mut ecu = SimulatedEcu::new();
        let req = CanFrame::new(
            CanId::new(0x7DF),
            vec![0x01, 0x03, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55],
        );
        ecu.process_frame(&req);
        let resp = ecu.pop_response().unwrap();
        assert_eq!(resp.data[1], 0x43);
        assert_eq!(resp.data[2], 0x00);
    }

    #[test]
    fn responds_to_vin_request_as_multiframe() {
        let mut ecu = SimulatedEcu::new();
        let req = CanFrame::new(
            CanId::new(0x7DF),
            vec![0x02, 0x09, 0x02, 0x55, 0x55, 0x55, 0x55, 0x55],
        );
        ecu.process_frame(&req);
        let ff = ecu.pop_response().unwrap();
        assert_eq!((ff.data[0] & 0xF0) >> 4, 1);
        let total_len = (((ff.data[0] & 0x0F) as u16) << 8) | ff.data[1] as u16;
        assert_eq!(total_len, 20);
    }

    #[test]
    fn responds_to_bcm_asbuilt_block_01() {
        let mut ecu = SimulatedEcu::new();
        let req = CanFrame::new(
            CanId::new(0x726),
            vec![0x03, 0x22, 0x07, 0x01, 0x55, 0x55, 0x55, 0x55],
        );
        ecu.process_frame(&req);
        let resp = ecu.pop_response().unwrap();
        assert_eq!(resp.id, CanId::new(0x72E));
        assert_eq!(
            (resp.data[0] & 0xF0) >> 4,
            1,
            "should be first frame (multi-frame)"
        );
        let total_len = (((resp.data[0] & 0x0F) as u16) << 8) | resp.data[1] as u16;
        assert_eq!(total_len, 11);
    }

    #[test]
    fn responds_to_ipc_asbuilt_block_01() {
        let mut ecu = SimulatedEcu::new();
        let req = CanFrame::new(
            CanId::new(0x720),
            vec![0x03, 0x22, 0x01, 0x01, 0x55, 0x55, 0x55, 0x55],
        );
        ecu.process_frame(&req);
        let resp = ecu.pop_response().unwrap();
        assert_eq!(resp.id, CanId::new(0x728));
        assert_eq!(
            (resp.data[0] & 0xF0) >> 4,
            1,
            "should be first frame (multi-frame)"
        );
        let total_len = (((resp.data[0] & 0x0F) as u16) << 8) | resp.data[1] as u16;
        assert_eq!(total_len, 11);
    }

    #[test]
    fn flow_control_does_not_enqueue_response() {
        let mut ecu = SimulatedEcu::new();
        let fc = CanFrame::new(
            CanId::new(0x7DF),
            vec![0x30, 0x00, 0x00, 0x55, 0x55, 0x55, 0x55, 0x55],
        );
        ecu.process_frame(&fc);
        assert!(ecu.pop_response().is_none());
    }

    #[test]
    fn responds_to_tcm_fluid_temp() {
        let mut ecu = SimulatedEcu::new();
        let req = CanFrame::new(
            CanId::new(0x7E1),
            vec![0x03, 0x22, 0xDD, 0x01, 0x55, 0x55, 0x55, 0x55],
        );
        ecu.process_frame(&req);
        let resp = ecu.pop_response().unwrap();
        assert_eq!(resp.id, CanId::new(0x7E9));
        assert_eq!(resp.data[1], 0x62);
        assert_eq!(resp.data[2], 0xDD);
        assert_eq!(resp.data[3], 0x01);
        let raw_temp = resp.data[4] as f64 - 40.0;
        assert!(raw_temp >= 80.0 && raw_temp <= 100.0);
    }

    #[test]
    fn responds_to_abs_wheel_speeds() {
        let mut ecu = SimulatedEcu::new();
        let req = CanFrame::new(
            CanId::new(0x7E2),
            vec![0x03, 0x22, 0xC0, 0x01, 0x55, 0x55, 0x55, 0x55],
        );
        ecu.process_frame(&req);
        let ff = ecu.pop_response().unwrap();
        assert_eq!(resp_id_from_first_frame(&ff), 0x7EA);
        assert_eq!((ff.data[0] & 0xF0) >> 4, 1, "should be multi-frame");
        let total_len = (((ff.data[0] & 0x0F) as u16) << 8) | ff.data[1] as u16;
        assert_eq!(total_len, 11); // 3 header + 8 data bytes
    }

    #[test]
    fn responds_to_pam_sensors() {
        let mut ecu = SimulatedEcu::new();
        let req = CanFrame::new(
            CanId::new(0x733),
            vec![0x03, 0x22, 0xE0, 0x01, 0x55, 0x55, 0x55, 0x55],
        );
        ecu.process_frame(&req);
        let ff = ecu.pop_response().unwrap();
        assert_eq!(resp_id_from_first_frame(&ff), 0x73B);
        let total_len = (((ff.data[0] & 0x0F) as u16) << 8) | ff.data[1] as u16;
        assert_eq!(total_len, 19); // 3 header + 16 sensor bytes
    }

    #[test]
    fn responds_to_hvac_heartbeat() {
        let mut ecu = SimulatedEcu::new();
        let req = CanFrame::new(
            CanId::new(0x703),
            vec![0x03, 0x22, 0xF1, 0x01, 0x55, 0x55, 0x55, 0x55],
        );
        ecu.process_frame(&req);
        let resp = ecu.pop_response().unwrap();
        assert_eq!(resp.id, CanId::new(0x70B));
        assert_eq!(resp.data[1], 0x62);
        assert_eq!(resp.data[4], 0x01);
    }

    #[test]
    fn responds_to_bcm_tpms_pressures() {
        let mut ecu = SimulatedEcu::new();
        let req = CanFrame::new(
            CanId::new(0x726),
            vec![0x03, 0x22, 0x02, 0x01, 0x55, 0x55, 0x55, 0x55],
        );
        ecu.process_frame(&req);
        let resp = ecu.pop_response().unwrap();
        assert_eq!(resp.id, CanId::new(0x72E));
        assert_eq!(resp.data[1], 0x62);
        assert_eq!(resp.data[2], 0x02);
        assert_eq!(resp.data[3], 0x01);
        assert_eq!(resp.data[4], 0x23);
        assert_eq!(resp.data[5], 0x23);
    }

    fn resp_id_from_first_frame(frame: &CanFrame) -> u32 {
        frame.id.id()
    }
}

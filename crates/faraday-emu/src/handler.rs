use crate::ecu::SimulatedEcu;
use faraday_core::{CanFrame, CanId};

pub enum StnCommand {
    Atz,
    Sti,
    Stslbrp,
    Stfap,
    StcpHsCan,
    StcpMsCan,
    Stpx { can_id: u32, data: Vec<u8> },
    Strx,
    Unknown,
}

pub struct EcuHandler {
    ecu: SimulatedEcu,
}

impl EcuHandler {
    pub fn new() -> Self {
        Self {
            ecu: SimulatedEcu::new(),
        }
    }

    pub fn parse_command(line: &str) -> StnCommand {
        let upper = line.trim().to_uppercase();
        match upper.as_str() {
            "ATZ" => StnCommand::Atz,
            "STI" => StnCommand::Sti,
            "STSLBRP" => StnCommand::Stslbrp,
            "STFAP" => StnCommand::Stfap,
            "STCP 24" => StnCommand::StcpHsCan,
            "STCP 25" => StnCommand::StcpMsCan,
            "STRX" => StnCommand::Strx,
            _ if upper.starts_with("STPX") => parse_stpx(&upper),
            _ => StnCommand::Unknown,
        }
    }

    pub fn handle(&mut self, cmd: StnCommand) -> Vec<u8> {
        match cmd {
            StnCommand::Atz => b"ELM327 v1.5\r\n>".to_vec(),
            StnCommand::Sti => b"OBDII to RS232 Interpreter\r\n>".to_vec(),
            StnCommand::Stslbrp | StnCommand::Stfap | StnCommand::StcpHsCan | StnCommand::StcpMsCan => {
                b"OK\r\n>".to_vec()
            }
            StnCommand::Stpx { can_id, data } => {
                let frame = CanFrame::new(CanId::new(can_id), data);
                self.ecu.process_frame(&frame);
                b"OK\r\n>".to_vec()
            }
            StnCommand::Strx => match self.ecu.pop_response() {
                None => b"\r\n>".to_vec(),
                Some(f) => {
                    format!("{:03X},{}\r\n>", f.id.id(), hex::encode_upper(&f.data)).into_bytes()
                }
            },
            StnCommand::Unknown => b"?\r\n>".to_vec(),
        }
    }
}

fn parse_stpx(line: &str) -> StnCommand {
    let body = line
        .trim_start_matches("STPX")
        .trim()
        .trim_start_matches("H:");
    let parts: Vec<&str> = body.splitn(3, ',').collect();
    if parts.len() < 3 {
        return StnCommand::Unknown;
    }
    let Ok(can_id) = u32::from_str_radix(parts[0].trim(), 16) else {
        return StnCommand::Unknown;
    };
    let Ok(data) = hex::decode(parts[2].trim()) else {
        return StnCommand::Unknown;
    };
    StnCommand::Stpx { can_id, data }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_atz() {
        assert!(matches!(EcuHandler::parse_command("ATZ"), StnCommand::Atz));
    }

    #[test]
    fn parse_stpx_rpm_request() {
        let cmd = EcuHandler::parse_command("STPX H:7DF,0,02010C55555555");
        assert!(matches!(cmd, StnCommand::Stpx { can_id: 0x7DF, .. }));
    }

    #[test]
    fn stpx_then_strx_returns_frame() {
        let mut h = EcuHandler::new();
        h.handle(EcuHandler::parse_command("STPX H:7DF,0,02010C55555555"));
        let resp = h.handle(EcuHandler::parse_command("STRX"));
        let s = String::from_utf8(resp).unwrap();
        assert!(s.starts_with("7E8,"));
        assert!(s.ends_with('>'));
    }

    #[test]
    fn strx_with_empty_queue_returns_empty_prompt() {
        let mut h = EcuHandler::new();
        assert_eq!(h.handle(StnCommand::Strx), b"\r\n>");
    }
}

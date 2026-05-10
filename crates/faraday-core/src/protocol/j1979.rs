/*!
SAE J1979 (OBD-II) protocol implementation.

Provides standard diagnostic services including:
- Mode 01: Request current powertrain diagnostic data
- Mode 03: Request stored diagnostic trouble codes
- Mode 04: Clear diagnostic trouble codes
- Mode 07: Request pending diagnostic trouble codes
- Mode 09: Request vehicle information
- Mode 0A: Request permanent diagnostic trouble codes
*/

use crate::{transport::IsoTpTransport, CanId, Error, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

const FUNCTIONAL_REQUEST_ID: CanId = CanId(0x7DF);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pid(pub u8);

impl Pid {
    pub const ENGINE_LOAD: Pid = Pid(0x04);
    pub const COOLANT_TEMP: Pid = Pid(0x05);
    pub const ENGINE_RPM: Pid = Pid(0x0C);
    pub const VEHICLE_SPEED: Pid = Pid(0x0D);
    pub const INTAKE_TEMP: Pid = Pid(0x0F);
    pub const MAF_RATE: Pid = Pid(0x10);
    pub const THROTTLE_POS: Pid = Pid(0x11);
    pub const FUEL_TANK_LEVEL: Pid = Pid(0x2F);
    pub const CONTROL_MODULE_VOLTAGE: Pid = Pid(0x42);
    pub const AMBIENT_TEMP: Pid = Pid(0x46);
    pub const ENGINE_OIL_TEMP: Pid = Pid(0x5C);
}

impl From<u8> for Pid {
    fn from(value: u8) -> Self {
        Pid(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Dtc {
    pub code: String,
    pub description: String,
}

impl Dtc {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 2 {
            return Err(Error::invalid_data("DTC must be 2 bytes"));
        }

        let first_byte = bytes[0];
        let second_byte = bytes[1];

        let prefix = match (first_byte & 0xC0) >> 6 {
            0 => "P",
            1 => "C",
            2 => "B",
            3 => "U",
            _ => unreachable!(),
        };

        let first_digit = (first_byte & 0x30) >> 4;
        let second_digit = first_byte & 0x0F;
        let third_fourth_digits = second_byte;

        let code = format!(
            "{}{:X}{:X}{:02X}",
            prefix, first_digit, second_digit, third_fourth_digits
        );

        Ok(Dtc {
            code: code.clone(),
            description: format!("Diagnostic Trouble Code {}", code),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidValue {
    pub pid: Pid,
    pub raw_value: Vec<u8>,
    pub interpreted_value: Option<f64>,
    pub unit: Option<String>,
}

impl PidValue {
    pub fn new(pid: Pid, raw_value: Vec<u8>) -> Self {
        let (interpreted_value, unit) = Self::interpret_value(pid, &raw_value);
        Self {
            pid,
            raw_value,
            interpreted_value,
            unit,
        }
    }

    fn interpret_value(pid: Pid, raw_value: &[u8]) -> (Option<f64>, Option<String>) {
        match pid {
            Pid::ENGINE_LOAD => {
                if !raw_value.is_empty() {
                    let value = (raw_value[0] as f64 * 100.0) / 255.0;
                    (Some(value), Some("%".to_string()))
                } else {
                    (None, None)
                }
            }
            Pid::COOLANT_TEMP | Pid::INTAKE_TEMP | Pid::AMBIENT_TEMP => {
                if !raw_value.is_empty() {
                    let value = raw_value[0] as f64 - 40.0;
                    (Some(value), Some("°C".to_string()))
                } else {
                    (None, None)
                }
            }
            Pid::ENGINE_RPM => {
                if raw_value.len() >= 2 {
                    let value = ((raw_value[0] as u16) << 8 | raw_value[1] as u16) as f64 / 4.0;
                    (Some(value), Some("rpm".to_string()))
                } else {
                    (None, None)
                }
            }
            Pid::VEHICLE_SPEED => {
                if !raw_value.is_empty() {
                    let value = raw_value[0] as f64;
                    (Some(value), Some("km/h".to_string()))
                } else {
                    (None, None)
                }
            }
            Pid::THROTTLE_POS => {
                if !raw_value.is_empty() {
                    let value = (raw_value[0] as f64 * 100.0) / 255.0;
                    (Some(value), Some("%".to_string()))
                } else {
                    (None, None)
                }
            }
            Pid::MAF_RATE => {
                if raw_value.len() >= 2 {
                    let value = ((raw_value[0] as u16) << 8 | raw_value[1] as u16) as f64 / 100.0;
                    (Some(value), Some("g/s".to_string()))
                } else {
                    (None, None)
                }
            }
            Pid::FUEL_TANK_LEVEL => {
                if !raw_value.is_empty() {
                    let value = (raw_value[0] as f64 * 100.0) / 255.0;
                    (Some(value), Some("%".to_string()))
                } else {
                    (None, None)
                }
            }
            Pid::CONTROL_MODULE_VOLTAGE => {
                if raw_value.len() >= 2 {
                    let value = ((raw_value[0] as u16) << 8 | raw_value[1] as u16) as f64 / 1000.0;
                    (Some(value), Some("V".to_string()))
                } else {
                    (None, None)
                }
            }
            Pid::ENGINE_OIL_TEMP => {
                if !raw_value.is_empty() {
                    let value = raw_value[0] as f64 - 40.0;
                    (Some(value), Some("°C".to_string()))
                } else {
                    (None, None)
                }
            }
            _ => (None, None),
        }
    }
}

pub struct J1979<'a, T: IsoTpTransport> {
    transport: &'a mut T,
}

impl<'a, T: IsoTpTransport> J1979<'a, T> {
    pub fn new(transport: &'a mut T) -> Self {
        Self { transport }
    }

    pub async fn read_live_data(
        &mut self,
        module_response_id: CanId,
        pids: &[Pid],
    ) -> Result<Vec<PidValue>> {
        let mut request = vec![0x01];
        for pid in pids {
            request.push(pid.0);
        }

        trace!("Reading live data for PIDs: {:?}", pids);

        let response = self
            .transport
            .request_response(FUNCTIONAL_REQUEST_ID, module_response_id, &request)
            .await?;

        self.parse_mode_01_response(&response, pids)
    }

    pub async fn read_stored_dtcs(&mut self, module_response_id: CanId) -> Result<Vec<Dtc>> {
        debug!("Reading stored DTCs");

        let request = vec![0x03];
        let response = self
            .transport
            .request_response(FUNCTIONAL_REQUEST_ID, module_response_id, &request)
            .await?;

        self.parse_dtc_response(&response)
    }

    pub async fn clear_dtcs(&mut self, module_response_id: CanId) -> Result<()> {
        debug!("Clearing DTCs");

        let request = vec![0x04];
        let response = self
            .transport
            .request_response(FUNCTIONAL_REQUEST_ID, module_response_id, &request)
            .await?;

        if !response.is_empty() && response[0] == 0x44 {
            debug!("DTCs cleared successfully");
            Ok(())
        } else {
            Err(Error::protocol("Failed to clear DTCs"))
        }
    }

    pub async fn read_pending_dtcs(&mut self, module_response_id: CanId) -> Result<Vec<Dtc>> {
        debug!("Reading pending DTCs");

        let request = vec![0x07];
        let response = self
            .transport
            .request_response(FUNCTIONAL_REQUEST_ID, module_response_id, &request)
            .await?;

        self.parse_dtc_response(&response)
    }

    pub async fn read_vin(&mut self, module_response_id: CanId) -> Result<String> {
        debug!("Reading VIN");

        let request = vec![0x09, 0x02];
        let response = self
            .transport
            .request_response(FUNCTIONAL_REQUEST_ID, module_response_id, &request)
            .await?;

        if response.len() >= 3 && response[0] == 0x49 && response[1] == 0x02 {
            let vin_bytes = &response[3..];
            let vin = String::from_utf8(vin_bytes.to_vec())
                .map_err(|_| Error::invalid_data("Invalid VIN encoding"))?;
            debug!("VIN: {}", vin);
            Ok(vin.trim_end_matches('\0').to_string())
        } else {
            Err(Error::protocol("Invalid VIN response"))
        }
    }

    pub async fn read_permanent_dtcs(&mut self, module_response_id: CanId) -> Result<Vec<Dtc>> {
        debug!("Reading permanent DTCs");

        let request = vec![0x0A];
        let response = self
            .transport
            .request_response(FUNCTIONAL_REQUEST_ID, module_response_id, &request)
            .await?;

        self.parse_dtc_response(&response)
    }

    fn parse_mode_01_response(&self, response: &[u8], pids: &[Pid]) -> Result<Vec<PidValue>> {
        if response.len() < 2 || response[0] != 0x41 {
            return Err(Error::protocol("Invalid Mode 01 response"));
        }

        let mut values = Vec::new();
        let mut offset = 1;

        for &pid in pids {
            if offset >= response.len() {
                break;
            }

            if response[offset] == pid.0 {
                offset += 1;
                let data_length = self.get_pid_data_length(pid);

                if offset + data_length <= response.len() {
                    let raw_value = response[offset..offset + data_length].to_vec();
                    values.push(PidValue::new(pid, raw_value));
                    offset += data_length;
                } else {
                    break;
                }
            } else {
                offset += 1;
            }
        }

        Ok(values)
    }

    fn parse_dtc_response(&self, response: &[u8]) -> Result<Vec<Dtc>> {
        if response.len() < 2 {
            return Err(Error::protocol("Invalid DTC response"));
        }

        let num_dtcs = response[1] as usize;
        let mut dtcs = Vec::new();

        let mut offset = 2;
        for _ in 0..num_dtcs {
            if offset + 2 <= response.len() {
                let dtc_bytes = &response[offset..offset + 2];
                if let Ok(dtc) = Dtc::from_bytes(dtc_bytes) {
                    dtcs.push(dtc);
                }
                offset += 2;
            }
        }

        Ok(dtcs)
    }

    fn get_pid_data_length(&self, pid: Pid) -> usize {
        match pid {
            Pid::ENGINE_LOAD
            | Pid::COOLANT_TEMP
            | Pid::VEHICLE_SPEED
            | Pid::INTAKE_TEMP
            | Pid::THROTTLE_POS
            | Pid::FUEL_TANK_LEVEL
            | Pid::AMBIENT_TEMP
            | Pid::ENGINE_OIL_TEMP => 1,

            Pid::ENGINE_RPM | Pid::MAF_RATE | Pid::CONTROL_MODULE_VOLTAGE => 2,

            _ => 1,
        }
    }
}

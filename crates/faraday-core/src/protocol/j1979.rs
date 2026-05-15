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

/// An OBD-II Parameter Identifier (PID) byte used in Mode 01/02 requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pid(pub u8);

impl Pid {
    /// Calculated engine load (0–100 %).
    pub const ENGINE_LOAD: Pid = Pid(0x04);
    /// Engine coolant temperature (−40 to +215 °C).
    pub const COOLANT_TEMP: Pid = Pid(0x05);
    /// Short-term fuel trim, bank 1 (−100 to +99.2 %).
    pub const SHORT_FUEL_TRIM_B1: Pid = Pid(0x06);
    /// Long-term fuel trim, bank 1 (−100 to +99.2 %).
    pub const LONG_FUEL_TRIM_B1: Pid = Pid(0x07);
    /// Short-term fuel trim, bank 2 (−100 to +99.2 %).
    pub const SHORT_FUEL_TRIM_B2: Pid = Pid(0x08);
    /// Long-term fuel trim, bank 2 (−100 to +99.2 %).
    pub const LONG_FUEL_TRIM_B2: Pid = Pid(0x09);
    /// Engine speed (0–16 383.75 RPM).
    pub const ENGINE_RPM: Pid = Pid(0x0C);
    /// Vehicle speed (0–255 km/h).
    pub const VEHICLE_SPEED: Pid = Pid(0x0D);
    /// Ignition timing advance relative to TDC (−64 to +63.5 °).
    pub const TIMING_ADVANCE: Pid = Pid(0x0E);
    /// Intake air temperature (−40 to +215 °C).
    pub const INTAKE_TEMP: Pid = Pid(0x0F);
    /// Mass air-flow rate (0–655.35 g/s).
    pub const MAF_RATE: Pid = Pid(0x10);
    /// Absolute throttle position (0–100 %).
    pub const THROTTLE_POS: Pid = Pid(0x11);
    /// Oxygen sensors present (bitmask).
    pub const O2_SENSORS_PRESENT: Pid = Pid(0x13);
    /// O2 sensor voltage — bank 1, sensor 1 (0–1.275 V).
    pub const O2_B1S1_VOLTAGE: Pid = Pid(0x14);
    /// O2 sensor voltage — bank 1, sensor 2 (0–1.275 V).
    pub const O2_B1S2_VOLTAGE: Pid = Pid(0x15);
    /// Commanded EGR valve position (0–100 %).
    pub const EGR_COMMANDED: Pid = Pid(0x2C);
    /// EGR error (−100 to +99.2 %).
    pub const EGR_ERROR: Pid = Pid(0x2D);
    /// Fuel tank level input (0–100 %).
    pub const FUEL_TANK_LEVEL: Pid = Pid(0x2F);
    /// Control module supply voltage (0–65.535 V).
    pub const CONTROL_MODULE_VOLTAGE: Pid = Pid(0x42);
    /// Commanded equivalence ratio (0–2).
    pub const FUEL_AIR_EQUIV_RATIO: Pid = Pid(0x44);
    /// Ambient air temperature (−40 to +215 °C).
    pub const AMBIENT_TEMP: Pid = Pid(0x46);
    /// Time run with MIL on (minutes).
    pub const RUNTIME_MIL_ON: Pid = Pid(0x4D);
    /// Time since diagnostic trouble codes were cleared (minutes).
    pub const RUNTIME_SINCE_CLEAR: Pid = Pid(0x4E);
    /// Relative throttle position (0–100 %).
    pub const REL_THROTTLE_POS: Pid = Pid(0x5A);
    /// Engine oil temperature (−40 to +215 °C).
    pub const ENGINE_OIL_TEMP: Pid = Pid(0x5C);
    /// Engine fuel rate (0–3 276.75 L/h).
    pub const ENGINE_FUEL_RATE: Pid = Pid(0x5E);
    /// Driver's demand engine torque (−125 to +130 %).
    pub const DRIVER_DEMAND_TORQUE: Pid = Pid(0x61);
    /// Actual engine torque (−125 to +130 %).
    pub const ACTUAL_ENGINE_TORQUE: Pid = Pid(0x62);
}

impl From<u8> for Pid {
    fn from(value: u8) -> Self {
        Pid(value)
    }
}

/// A decoded Diagnostic Trouble Code.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Dtc {
    /// Five-character DTC string in the form `P0300` / `C0001` / `B0001` / `U0100`.
    pub code: String,
    /// Human-readable description derived from the code.
    pub description: String,
}

impl Dtc {
    /// Decodes a two-byte DTC payload from an OBD-II response into a `Dtc`.
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
        let description = format!("Diagnostic Trouble Code {}", code);

        Ok(Dtc { code, description })
    }
}

/// A decoded OBD-II PID measurement with optional engineering-unit interpretation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidValue {
    /// The PID that was queried.
    pub pid: Pid,
    /// The raw bytes returned by the ECU.
    pub raw_value: Vec<u8>,
    /// The scaled, physical-unit value (e.g. RPM, °C, %) if interpretation is defined.
    pub interpreted_value: Option<f64>,
    /// The engineering unit string (e.g. `"RPM"`, `"°C"`, `"%"`).
    pub unit: Option<String>,
}

impl PidValue {
    /// Constructs a `PidValue` by interpreting `raw_value` for the given `pid`.
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
            Pid::ENGINE_LOAD | Pid::FUEL_TANK_LEVEL => {
                if !raw_value.is_empty() {
                    let value = (raw_value[0] as f64 * 100.0) / 255.0;
                    (Some(value), Some("%".to_string()))
                } else {
                    (None, None)
                }
            }
            Pid::COOLANT_TEMP | Pid::INTAKE_TEMP | Pid::AMBIENT_TEMP | Pid::ENGINE_OIL_TEMP => {
                if !raw_value.is_empty() {
                    let value = raw_value[0] as f64 - 40.0;
                    (Some(value), Some("°C".to_string()))
                } else {
                    (None, None)
                }
            }
            Pid::SHORT_FUEL_TRIM_B1
            | Pid::LONG_FUEL_TRIM_B1
            | Pid::SHORT_FUEL_TRIM_B2
            | Pid::LONG_FUEL_TRIM_B2
            | Pid::EGR_ERROR => {
                if !raw_value.is_empty() {
                    let value = (raw_value[0] as f64 - 128.0) * 100.0 / 128.0;
                    (Some(value), Some("%".to_string()))
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
            Pid::TIMING_ADVANCE => {
                if !raw_value.is_empty() {
                    let value = raw_value[0] as f64 / 2.0 - 64.0;
                    (Some(value), Some("°".to_string()))
                } else {
                    (None, None)
                }
            }
            Pid::THROTTLE_POS | Pid::REL_THROTTLE_POS | Pid::EGR_COMMANDED => {
                if !raw_value.is_empty() {
                    let value = (raw_value[0] as f64 * 100.0) / 255.0;
                    (Some(value), Some("%".to_string()))
                } else {
                    (None, None)
                }
            }
            Pid::O2_SENSORS_PRESENT => {
                if !raw_value.is_empty() {
                    (Some(raw_value[0] as f64), None)
                } else {
                    (None, None)
                }
            }
            Pid::O2_B1S1_VOLTAGE | Pid::O2_B1S2_VOLTAGE => {
                if raw_value.len() >= 2 {
                    let voltage = raw_value[0] as f64 * 0.005;
                    (Some(voltage), Some("V".to_string()))
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
            Pid::FUEL_AIR_EQUIV_RATIO => {
                if raw_value.len() >= 4 {
                    let value =
                        ((raw_value[0] as u32) << 8 | raw_value[1] as u32) as f64 * 2.0 / 65536.0;
                    (Some(value), Some("λ".to_string()))
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
            Pid::RUNTIME_MIL_ON | Pid::RUNTIME_SINCE_CLEAR => {
                if raw_value.len() >= 2 {
                    let value = ((raw_value[0] as u16) << 8 | raw_value[1] as u16) as f64;
                    (Some(value), Some("min".to_string()))
                } else {
                    (None, None)
                }
            }
            Pid::ENGINE_FUEL_RATE => {
                if raw_value.len() >= 2 {
                    let value = ((raw_value[0] as u16) << 8 | raw_value[1] as u16) as f64 * 0.05;
                    (Some(value), Some("L/h".to_string()))
                } else {
                    (None, None)
                }
            }
            Pid::DRIVER_DEMAND_TORQUE | Pid::ACTUAL_ENGINE_TORQUE => {
                if !raw_value.is_empty() {
                    let value = raw_value[0] as f64 - 125.0;
                    (Some(value), Some("%".to_string()))
                } else {
                    (None, None)
                }
            }
            _ => (None, None),
        }
    }
}

/// Low-level J1979 (OBD-II) protocol driver.
pub struct J1979<'a, T: IsoTpTransport> {
    transport: &'a mut T,
}

impl<'a, T: IsoTpTransport> J1979<'a, T> {
    /// Wraps an existing transport in a new `J1979` driver instance.
    pub fn new(transport: &'a mut T) -> Self {
        Self { transport }
    }

    /// Requests live Mode 01 data for `pids` from the given response CAN ID.
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

    /// Reads stored (confirmed) DTCs via Mode 03 from the given response CAN ID.
    pub async fn read_stored_dtcs(&mut self, module_response_id: CanId) -> Result<Vec<Dtc>> {
        debug!("Reading stored DTCs");

        let request = vec![0x03];
        let response = self
            .transport
            .request_response(FUNCTIONAL_REQUEST_ID, module_response_id, &request)
            .await?;

        self.parse_dtc_response(&response)
    }

    /// Sends Mode 04 to clear all DTCs from the module.
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

    /// Reads pending (not yet confirmed) DTCs via Mode 07.
    pub async fn read_pending_dtcs(&mut self, module_response_id: CanId) -> Result<Vec<Dtc>> {
        debug!("Reading pending DTCs");

        let request = vec![0x07];
        let response = self
            .transport
            .request_response(FUNCTIONAL_REQUEST_ID, module_response_id, &request)
            .await?;

        self.parse_dtc_response(&response)
    }

    /// Reads the Vehicle Identification Number via Mode 09 PID 02.
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

    /// Reads permanent DTCs via Mode 0A (cannot be cleared by Mode 04).
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
            | Pid::ENGINE_OIL_TEMP
            | Pid::SHORT_FUEL_TRIM_B1
            | Pid::LONG_FUEL_TRIM_B1
            | Pid::SHORT_FUEL_TRIM_B2
            | Pid::LONG_FUEL_TRIM_B2
            | Pid::TIMING_ADVANCE
            | Pid::O2_SENSORS_PRESENT
            | Pid::EGR_COMMANDED
            | Pid::EGR_ERROR
            | Pid::REL_THROTTLE_POS
            | Pid::DRIVER_DEMAND_TORQUE
            | Pid::ACTUAL_ENGINE_TORQUE => 1,

            Pid::ENGINE_RPM
            | Pid::MAF_RATE
            | Pid::CONTROL_MODULE_VOLTAGE
            | Pid::O2_B1S1_VOLTAGE
            | Pid::O2_B1S2_VOLTAGE
            | Pid::RUNTIME_MIL_ON
            | Pid::RUNTIME_SINCE_CLEAR
            | Pid::ENGINE_FUEL_RATE => 2,

            Pid::FUEL_AIR_EQUIV_RATIO => 4,

            _ => 1,
        }
    }
}

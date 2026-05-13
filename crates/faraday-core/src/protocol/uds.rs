/*!
ISO 14229 (UDS - Unified Diagnostic Services) implementation.

Supports services including:
- 0x10: DiagnosticSessionControl
- 0x22: ReadDataByIdentifier
- 0x27: SecurityAccess
- 0x2E: WriteDataByIdentifier
- 0x3E: TesterPresent

UDS operates at a higher level than J1979 and provides direct access
to manufacturer-specific diagnostic data and functions.
*/

use crate::{transport::IsoTpTransport, CanId, Error, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, trace, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagnosticSession {
    Default = 0x01,
    Programming = 0x02,
    Extended = 0x03,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DataIdentifier(pub u16);

impl DataIdentifier {
    pub const VIN: DataIdentifier = DataIdentifier(0xF190);
    pub const ECU_SOFTWARE_NUMBER: DataIdentifier = DataIdentifier(0xF194);
    pub const ECU_HARDWARE_NUMBER: DataIdentifier = DataIdentifier(0xF191);
    pub const SUPPLIER_IDENTIFIER: DataIdentifier = DataIdentifier(0xF18A);
    pub const ECU_MANUFACTURING_DATE: DataIdentifier = DataIdentifier(0xF18B);
    pub const ECU_SERIAL_NUMBER: DataIdentifier = DataIdentifier(0xF18C);

    pub const PROGRAMMING_DID_PREFIX: u8 = 0xF0;
    pub const IDENTIFICATION_DID_PREFIX: u8 = 0xF1;
}

impl From<u16> for DataIdentifier {
    fn from(value: u16) -> Self {
        DataIdentifier(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecurityAccessType {
    RequestSeed = 0x01,
    SendKey = 0x02,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegativeResponse {
    pub service: u8,
    pub nrc: u8,
    pub description: String,
}

impl NegativeResponse {
    pub fn new(service: u8, nrc: u8) -> Self {
        let description = match nrc {
            0x10 => "General reject",
            0x11 => "Service not supported",
            0x12 => "Sub-function not supported",
            0x13 => "Incorrect message length or invalid format",
            0x14 => "Response too long",
            0x21 => "Busy repeat request",
            0x22 => "Conditions not correct",
            0x24 => "Request sequence error",
            0x25 => "No response from subnet component",
            0x26 => "Failure prevents execution of requested action",
            0x31 => "Request out of range",
            0x33 => "Security access denied",
            0x35 => "Invalid key",
            0x36 => "Exceeded number of attempts",
            0x37 => "Required time delay not expired",
            0x78 => "Request correctly received - response pending",
            _ => "Unknown error code",
        };

        Self {
            service,
            nrc,
            description: description.to_string(),
        }
    }

    pub fn is_response_pending(&self) -> bool {
        self.nrc == 0x78
    }
}

pub struct Uds<'a, T: IsoTpTransport> {
    transport: &'a mut T,
    current_session: DiagnosticSession,
    security_access_granted: bool,
    tester_present_task: Option<tokio::task::JoinHandle<()>>,
}

impl<'a, T: IsoTpTransport> Uds<'a, T> {
    pub fn new(transport: &'a mut T) -> Self {
        Self {
            transport,
            current_session: DiagnosticSession::Default,
            security_access_granted: false,
            tester_present_task: None,
        }
    }

    pub async fn diagnostic_session_control(
        &mut self,
        request_id: CanId,
        response_id: CanId,
        session: DiagnosticSession,
    ) -> Result<()> {
        debug!("Setting diagnostic session to {:?}", session);

        let request = vec![0x10, session as u8];
        let response = self
            .transport
            .request_response(request_id, response_id, &request)
            .await?;

        self.check_positive_response(&response, 0x10, 2)?;

        self.current_session = session;

        if session == DiagnosticSession::Extended {
            self.start_tester_present(request_id).await;
        } else {
            self.stop_tester_present().await;
        }

        debug!("Diagnostic session set to {:?}", session);
        Ok(())
    }

    pub async fn read_data_by_identifier(
        &mut self,
        request_id: CanId,
        response_id: CanId,
        did: DataIdentifier,
    ) -> Result<Vec<u8>> {
        debug!("Reading DID {:04X}", did.0);

        let request = vec![0x22, (did.0 >> 8) as u8, (did.0 & 0xFF) as u8];
        let response = self
            .transport
            .request_response(request_id, response_id, &request)
            .await?;

        if response.len() < 3 {
            return Err(Error::protocol("Response too short"));
        }

        self.check_positive_response(&response, 0x22, 3)?;

        let returned_did = ((response[1] as u16) << 8) | (response[2] as u16);
        if returned_did != did.0 {
            return Err(Error::protocol(format!(
                "DID mismatch: requested {:04X}, got {:04X}",
                did.0, returned_did
            )));
        }

        let data = response[3..].to_vec();
        debug!("Read DID {:04X}: {} bytes", did.0, data.len());
        Ok(data)
    }

    pub async fn security_access_request_seed(
        &mut self,
        request_id: CanId,
        response_id: CanId,
        access_type: u8,
    ) -> Result<Vec<u8>> {
        debug!(
            "Requesting security access seed for type {:02X}",
            access_type
        );

        let request = vec![0x27, access_type];
        let response = self
            .transport
            .request_response(request_id, response_id, &request)
            .await?;

        self.check_positive_response(&response, 0x27, 2)?;

        let seed = response[2..].to_vec();
        debug!("Received seed: {} bytes", seed.len());
        Ok(seed)
    }

    pub async fn security_access_send_key(
        &mut self,
        request_id: CanId,
        response_id: CanId,
        access_type: u8,
        key: &[u8],
    ) -> Result<()> {
        debug!("Sending security access key for type {:02X}", access_type);

        let mut request = vec![0x27, access_type + 1];
        request.extend_from_slice(key);

        let response = self
            .transport
            .request_response(request_id, response_id, &request)
            .await?;

        self.check_positive_response(&response, 0x27, 2)?;

        self.security_access_granted = true;
        debug!("Security access granted");
        Ok(())
    }

    pub async fn write_data_by_identifier(
        &mut self,
        request_id: CanId,
        response_id: CanId,
        did: DataIdentifier,
        data: &[u8],
    ) -> Result<()> {
        if !self.security_access_granted {
            return Err(Error::protocol(
                "Security access required for write operations",
            ));
        }

        debug!("Writing DID {:04X}: {} bytes", did.0, data.len());

        let mut request = vec![0x2E, (did.0 >> 8) as u8, (did.0 & 0xFF) as u8];
        request.extend_from_slice(data);

        let response = self
            .transport
            .request_response(request_id, response_id, &request)
            .await?;

        self.check_positive_response(&response, 0x2E, 3)?;

        let returned_did = ((response[1] as u16) << 8) | (response[2] as u16);
        if returned_did != did.0 {
            return Err(Error::protocol(format!(
                "DID mismatch: requested {:04X}, got {:04X}",
                did.0, returned_did
            )));
        }

        debug!("Successfully wrote DID {:04X}", did.0);
        Ok(())
    }

    pub async fn tester_present(&mut self, request_id: CanId, response_id: CanId) -> Result<()> {
        trace!("Sending tester present");

        let request = vec![0x3E, 0x00];
        let response = self
            .transport
            .request_response(request_id, response_id, &request)
            .await?;

        self.check_positive_response(&response, 0x3E, 2)?;
        Ok(())
    }

    async fn start_tester_present(&mut self, _request_id: CanId) {
        self.stop_tester_present().await;

        debug!("Starting tester present background task");
    }

    async fn stop_tester_present(&mut self) {
        if let Some(task) = self.tester_present_task.take() {
            task.abort();
            debug!("Stopped tester present background task");
        }
    }

    fn check_positive_response(
        &self,
        response: &[u8],
        expected_service: u8,
        min_length: usize,
    ) -> Result<()> {
        if response.len() < min_length {
            return Err(Error::protocol("Response too short"));
        }

        if response[0] == 0x7F {
            if response.len() >= 3 {
                let service = response[1];
                let nrc = response[2];
                let neg_response = NegativeResponse::new(service, nrc);

                if neg_response.is_response_pending() {
                    warn!("Received response pending, may need longer timeout");
                }

                return Err(Error::UdsNegativeResponse {
                    service,
                    code: nrc,
                    description: neg_response.description,
                });
            } else {
                return Err(Error::protocol("Invalid negative response format"));
            }
        }

        let expected_response = expected_service + 0x40;
        if response[0] != expected_response {
            return Err(Error::protocol(format!(
                "Unexpected response service: expected {:02X}, got {:02X}",
                expected_response, response[0]
            )));
        }

        Ok(())
    }

    pub fn current_session(&self) -> DiagnosticSession {
        self.current_session
    }

    pub fn is_security_access_granted(&self) -> bool {
        self.security_access_granted
    }
}

impl<'a, T: IsoTpTransport> Drop for Uds<'a, T> {
    fn drop(&mut self) {
        if let Some(task) = self.tester_present_task.take() {
            task.abort();
        }
    }
}

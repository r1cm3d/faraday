use super::CommandExecutor;
use crate::{
    protocol::{
        j1979::J1979,
        uds::{DataIdentifier, DiagnosticSession, Uds},
    },
    transport::IsoTpTransport,
    Module, Result,
};
use tracing::info;

impl<T: IsoTpTransport> CommandExecutor<T> {
    async fn read_string_did(
        &mut self,
        request_id: crate::CanId,
        response_id: crate::CanId,
        did: DataIdentifier,
    ) -> Result<String> {
        let mut uds = Uds::new(&mut self.transport);
        let data = uds
            .read_data_by_identifier(request_id, response_id, did)
            .await?;
        String::from_utf8(data)
            .map(|s| s.trim_end_matches('\0').to_string())
            .map_err(|_| crate::Error::invalid_data("Invalid string encoding"))
    }

    pub async fn read_vin(&mut self) -> Result<String> {
        info!("Reading VIN via J1979");
        let mut j1979 = J1979::new(&mut self.transport);
        j1979.read_vin(Module::Pcm.response_id()).await
    }

    pub async fn read_vin_uds(&mut self, module: Module) -> Result<String> {
        info!("Reading VIN via UDS from {:?}", module);
        self.read_string_did(
            module.request_id(),
            module.response_id(),
            DataIdentifier::VIN,
        )
        .await
    }

    pub async fn read_ecu_info(&mut self, module: Module) -> Result<EcuInfo> {
        info!("Reading ECU information from {:?}", module);

        {
            let mut uds = Uds::new(&mut self.transport);
            uds.diagnostic_session_control(
                module.request_id(),
                module.response_id(),
                DiagnosticSession::Extended,
            )
            .await?;
        }

        let req = module.request_id();
        let resp = module.response_id();

        let software_number = self
            .read_string_did(req, resp, DataIdentifier::ECU_SOFTWARE_NUMBER)
            .await
            .ok();
        let hardware_number = self
            .read_string_did(req, resp, DataIdentifier::ECU_HARDWARE_NUMBER)
            .await
            .ok();
        let supplier_id = self
            .read_string_did(req, resp, DataIdentifier::SUPPLIER_IDENTIFIER)
            .await
            .ok();
        let manufacturing_date = self
            .read_string_did(req, resp, DataIdentifier::ECU_MANUFACTURING_DATE)
            .await
            .ok();
        let serial_number = self
            .read_string_did(req, resp, DataIdentifier::ECU_SERIAL_NUMBER)
            .await
            .ok();

        Ok(EcuInfo {
            module,
            software_number,
            hardware_number,
            supplier_id,
            manufacturing_date,
            serial_number,
        })
    }

    pub async fn read_data_by_identifier(
        &mut self,
        request_id: crate::CanId,
        response_id: crate::CanId,
        did: DataIdentifier,
    ) -> Result<Vec<u8>> {
        let mut uds = Uds::new(&mut self.transport);
        uds.read_data_by_identifier(request_id, response_id, did)
            .await
    }

    pub async fn diagnostic_session_control(
        &mut self,
        request_id: crate::CanId,
        response_id: crate::CanId,
        session: DiagnosticSession,
    ) -> Result<()> {
        let mut uds = Uds::new(&mut self.transport);
        uds.diagnostic_session_control(request_id, response_id, session)
            .await
    }
}

#[derive(Debug, Clone)]
pub struct EcuInfo {
    pub module: Module,
    pub software_number: Option<String>,
    pub hardware_number: Option<String>,
    pub supplier_id: Option<String>,
    pub manufacturing_date: Option<String>,
    pub serial_number: Option<String>,
}

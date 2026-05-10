use super::CommandExecutor;
use crate::{
    protocol::{
        seed_key,
        uds::{DataIdentifier, DiagnosticSession, Uds},
    },
    transport::IsoTpTransport,
    Module, Result,
};
use tracing::info;

impl<T: IsoTpTransport> CommandExecutor<T> {
    /// Write a raw as-built configuration block to a module via UDS DID.
    ///
    /// Switches the CAN bus, enters extended session, acquires security access
    /// using the Ford seed→key algorithm, then writes the DID. Programming
    /// DIDs (0xF0xx, 0xF1xx) are blocked as a safety measure.
    pub async fn write_asbuilt_block(
        &mut self,
        module: Module,
        did: u16,
        data: &[u8],
    ) -> Result<()> {
        let prefix = (did >> 8) as u8;
        if prefix == 0xF0 || prefix == 0xF1 {
            return Err(crate::Error::unsupported(format!(
                "writing programming DID {:04X} is blocked for safety",
                did
            )));
        }

        info!("Writing as-built block DID {:04X} to {:?}", did, module);

        self.transport.set_can_bus(module.bus()).await?;

        let req = module.request_id();
        let resp = module.response_id();

        {
            let mut uds = Uds::new(&mut self.transport);
            uds.diagnostic_session_control(req, resp, DiagnosticSession::Extended)
                .await?;
        }

        let seed = {
            let mut uds = Uds::new(&mut self.transport);
            uds.security_access_request_seed(req, resp, 0x01).await?
        };

        let key = seed_key::compute_key(&seed, 0x01)?;

        {
            let mut uds = Uds::new(&mut self.transport);
            uds.security_access_send_key(req, resp, 0x01, &key).await?;
            uds.write_data_by_identifier(req, resp, DataIdentifier(did), data)
                .await?;
        }

        info!("Successfully wrote DID {:04X} to {:?}", did, module);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CanBus, CanId};
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    struct MockTransport {
        bus_calls: Arc<Mutex<Vec<CanBus>>>,
        request_log: Arc<Mutex<Vec<Vec<u8>>>>,
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                bus_calls: Arc::new(Mutex::new(Vec::new())),
                request_log: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl IsoTpTransport for MockTransport {
        async fn send(&mut self, _req: CanId, _data: &[u8]) -> crate::Result<()> {
            Ok(())
        }

        async fn receive(&mut self, _req: CanId, _resp: CanId) -> crate::Result<Vec<u8>> {
            Ok(vec![])
        }

        async fn request_response(
            &mut self,
            _req: CanId,
            _resp: CanId,
            data: &[u8],
        ) -> crate::Result<Vec<u8>> {
            self.request_log.lock().unwrap().push(data.to_vec());
            match data[0] {
                0x10 => Ok(vec![0x50, data[1], 0x00, 0x19, 0x01, 0xF4]),
                0x27 if data[1] == 0x01 => Ok(vec![0x67, 0x01, 0x00, 0x00, 0x00, 0x00]),
                0x27 if data[1] == 0x02 => Ok(vec![0x67, 0x02]),
                0x2E => {
                    let did_hi = data[1];
                    let did_lo = data[2];
                    Ok(vec![0x6E, did_hi, did_lo])
                }
                _ => Ok(vec![0x7F, data[0], 0x11]),
            }
        }

        async fn set_timeout(&mut self, _timeout: std::time::Duration) {}

        async fn set_can_bus(&mut self, bus: CanBus) -> crate::Result<()> {
            self.bus_calls.lock().unwrap().push(bus);
            Ok(())
        }
    }

    #[tokio::test]
    async fn blocks_programming_dids() {
        let mut executor = CommandExecutor::new(MockTransport::new());
        let result = executor
            .write_asbuilt_block(Module::Bcm, 0xF101, &[0x01])
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("blocked"));
    }

    #[tokio::test]
    async fn switches_to_mscan_for_bcm() {
        let transport = MockTransport::new();
        let bus_calls = transport.bus_calls.clone();
        let mut executor = CommandExecutor::new(transport);

        executor
            .write_asbuilt_block(Module::Bcm, 0x0701, &[0xAB, 0xCD])
            .await
            .unwrap();

        assert_eq!(bus_calls.lock().unwrap()[0], CanBus::MsCan);
    }

    #[tokio::test]
    async fn security_access_precedes_write() {
        let transport = MockTransport::new();
        let request_log = transport.request_log.clone();
        let mut executor = CommandExecutor::new(transport);

        executor
            .write_asbuilt_block(Module::Bcm, 0x0701, &[0x00])
            .await
            .unwrap();

        let log = request_log.lock().unwrap();
        let services: Vec<u8> = log.iter().map(|r| r[0]).collect();

        let write_pos = services.iter().rposition(|&s| s == 0x2E).unwrap();
        let last_sa_pos = services.iter().rposition(|&s| s == 0x27).unwrap();

        assert!(
            last_sa_pos < write_pos,
            "security access must precede write"
        );
    }
}

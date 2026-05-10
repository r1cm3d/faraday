/*!
As-built configuration read commands.

Reads raw block data from vehicle modules via UDS ReadDataByIdentifier,
with automatic CAN bus switching (HS-CAN vs MS-CAN) per module.
*/

use super::CommandExecutor;
use crate::{
    protocol::uds::{DataIdentifier, DiagnosticSession, Uds},
    transport::IsoTpTransport,
    Module, Result,
};
use tracing::info;

impl<T: IsoTpTransport> CommandExecutor<T> {
    /// Read a raw as-built configuration block from a module via UDS DID.
    ///
    /// Switches the CAN bus to the correct speed for the module, enters
    /// extended diagnostic session, then reads the given DID.
    pub async fn read_asbuilt_block(&mut self, module: Module, did: u16) -> Result<Vec<u8>> {
        info!("Reading as-built block DID {:04X} from {:?}", did, module);

        self.transport.set_can_bus(module.bus()).await?;

        {
            let mut uds = Uds::new(&mut self.transport);
            uds.diagnostic_session_control(
                module.request_id(),
                module.response_id(),
                DiagnosticSession::Extended,
            )
            .await?;
        }

        let mut uds = Uds::new(&mut self.transport);
        uds.read_data_by_identifier(
            module.request_id(),
            module.response_id(),
            DataIdentifier(did),
        )
        .await
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
        block_data: Vec<u8>,
    }

    impl MockTransport {
        fn new(block_data: Vec<u8>) -> Self {
            Self {
                bus_calls: Arc::new(Mutex::new(Vec::new())),
                block_data,
            }
        }
    }

    #[async_trait]
    impl IsoTpTransport for MockTransport {
        async fn send(&mut self, _req: CanId, _data: &[u8]) -> Result<()> {
            Ok(())
        }

        async fn receive(&mut self, _req: CanId, _resp: CanId) -> Result<Vec<u8>> {
            Ok(self.block_data.clone())
        }

        async fn request_response(
            &mut self,
            _req: CanId,
            _resp: CanId,
            data: &[u8],
        ) -> Result<Vec<u8>> {
            if data[0] == 0x10 {
                return Ok(vec![0x50, data[1], 0x00, 0x19, 0x01, 0xF4]);
            }
            if data[0] == 0x22 {
                let mut resp = vec![0x62, data[1], data[2]];
                resp.extend_from_slice(&self.block_data);
                return Ok(resp);
            }
            Ok(vec![0x7F, data[0], 0x11])
        }

        async fn set_timeout(&mut self, _timeout: std::time::Duration) {}

        async fn set_can_bus(&mut self, bus: CanBus) -> Result<()> {
            self.bus_calls.lock().unwrap().push(bus);
            Ok(())
        }
    }

    #[tokio::test]
    async fn read_asbuilt_block_switches_to_mscan_for_bcm() {
        let raw_data = vec![0x00, 0x00, 0x00, 0x04, 0x03, 0x00, 0x00, 0x00];
        let transport = MockTransport::new(raw_data.clone());
        let bus_calls = transport.bus_calls.clone();
        let mut executor = CommandExecutor::new(transport);

        let result = executor
            .read_asbuilt_block(Module::Bcm, 0x0701)
            .await
            .unwrap();

        assert_eq!(result, raw_data);
        assert_eq!(bus_calls.lock().unwrap()[0], CanBus::MsCan);
    }

    #[tokio::test]
    async fn read_asbuilt_block_uses_hscan_for_pcm() {
        let raw_data = vec![0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00];
        let transport = MockTransport::new(raw_data.clone());
        let bus_calls = transport.bus_calls.clone();
        let mut executor = CommandExecutor::new(transport);

        let result = executor
            .read_asbuilt_block(Module::Pcm, 0xE001)
            .await
            .unwrap();

        assert_eq!(result, raw_data);
        assert_eq!(bus_calls.lock().unwrap()[0], CanBus::HsCan);
    }
}

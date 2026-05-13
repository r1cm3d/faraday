use super::CommandExecutor;
use crate::{
    protocol::j1979::{Dtc, J1979},
    transport::IsoTpTransport,
    DtcKind, Module, Result,
};
use tracing::info;

impl<T: IsoTpTransport> CommandExecutor<T> {
    pub async fn read_stored_dtcs(&mut self, module: Module) -> Result<Vec<Dtc>> {
        info!("Reading stored DTCs from {:?}", module);
        let mut j1979 = J1979::new(&mut self.transport);
        j1979.read_stored_dtcs(module.response_id()).await
    }

    pub async fn read_pending_dtcs(&mut self, module: Module) -> Result<Vec<Dtc>> {
        info!("Reading pending DTCs from {:?}", module);
        let mut j1979 = J1979::new(&mut self.transport);
        j1979.read_pending_dtcs(module.response_id()).await
    }

    pub async fn read_permanent_dtcs(&mut self, module: Module) -> Result<Vec<Dtc>> {
        info!("Reading permanent DTCs from {:?}", module);
        let mut j1979 = J1979::new(&mut self.transport);
        j1979.read_permanent_dtcs(module.response_id()).await
    }

    pub async fn clear_dtcs(&mut self, module: Module) -> Result<()> {
        info!("Clearing DTCs from {:?}", module);
        let mut j1979 = J1979::new(&mut self.transport);
        j1979.clear_dtcs(module.response_id()).await
    }

    pub async fn read_dtcs(&mut self, module: Module, kind: DtcKind) -> Result<Vec<Dtc>> {
        match kind {
            DtcKind::Stored => self.read_stored_dtcs(module).await,
            DtcKind::Pending => self.read_pending_dtcs(module).await,
            DtcKind::Permanent => self.read_permanent_dtcs(module).await,
        }
    }
}

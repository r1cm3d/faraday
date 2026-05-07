use anyhow::Result;
use faraday_core::{
    commands::CommandExecutor,
    link::vlinker::VLinkerFs,
    transport::isotp::IsoTp,
};

pub mod clear_dtc;
pub mod live;
pub mod read_did;
pub mod read_dtc;
pub mod session;
pub mod vin;

pub async fn create_executor(adapter_path: String) -> Result<CommandExecutor<IsoTp<VLinkerFs>>> {
    let vlinker = VLinkerFs::with_port_name(&adapter_path)?;
    let isotp = IsoTp::new(vlinker);
    Ok(CommandExecutor::new(isotp))
}
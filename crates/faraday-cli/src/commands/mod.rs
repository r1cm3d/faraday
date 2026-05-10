use anyhow::Result;
use faraday_core::{
    commands::CommandExecutor,
    link::{vlinker::VLinkerFs, LinkLayer},
    transport::isotp::IsoTp,
};

pub mod asbuilt;
pub mod clear_dtc;
pub mod live;
pub mod read_did;
pub mod read_dtc;
pub mod session;
pub mod vin;

pub async fn create_executor(
    adapter_path: String,
) -> Result<CommandExecutor<IsoTp<Box<dyn LinkLayer>>>> {
    let mut vlinker = VLinkerFs::with_port_name(&adapter_path)?;
    vlinker.connect().await?;
    Ok(CommandExecutor::new(IsoTp::new(Box::new(vlinker))))
}

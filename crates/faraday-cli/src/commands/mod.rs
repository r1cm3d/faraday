use anyhow::Result;
use faraday_core::{
    commands::CommandExecutor,
    link::{sim::SimulatedEcu, vlinker::VLinkerFs, LinkLayer},
    transport::isotp::IsoTp,
};

pub mod clear_dtc;
pub mod live;
pub mod read_did;
pub mod read_dtc;
pub mod session;
pub mod vin;

pub async fn create_executor(
    adapter_path: String,
    emulate: bool,
) -> Result<CommandExecutor<IsoTp<Box<dyn LinkLayer>>>> {
    let link: Box<dyn LinkLayer> = if emulate {
        Box::new(SimulatedEcu::new())
    } else {
        let mut vlinker = VLinkerFs::with_port_name(&adapter_path)?;
        vlinker.connect().await?;
        Box::new(vlinker)
    };
    Ok(CommandExecutor::new(IsoTp::new(link)))
}

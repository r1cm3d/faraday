use anyhow::Result;
use faraday_core::Module;
use crate::{cli::VinMethod, output::OutputFormatter};

pub async fn execute(adapter_path: String, method: VinMethod) -> Result<()> {
    let mut executor = super::create_executor(adapter_path).await?;
    let mut formatter = OutputFormatter::new(false);

    let vin = match method {
        VinMethod::J1979 => {
            formatter.print_info("Reading VIN via SAE J1979...")?;
            executor.read_vin().await?
        }
        VinMethod::Uds => {
            formatter.print_info("Reading VIN via UDS...")?;
            executor.read_vin_uds(Module::Pcm).await?
        }
    };

    formatter.print_vin(&vin)?;

    Ok(())
}
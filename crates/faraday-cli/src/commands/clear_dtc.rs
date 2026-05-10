use crate::{cli::ModuleArg, output::OutputFormatter};
use anyhow::Result;
use faraday_core::Module;

pub async fn execute(adapter_path: String, module: ModuleArg) -> Result<()> {
    let mut executor = super::create_executor(adapter_path).await?;
    let mut formatter = OutputFormatter::new(false);
    let module: Module = module.into();

    formatter.print_info(&format!("Clearing DTCs from {:?}...", module))?;

    executor.clear_dtcs(module).await?;

    formatter.print_success("DTCs cleared successfully")?;

    Ok(())
}

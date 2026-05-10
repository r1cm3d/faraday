use anyhow::Result;
use faraday_core::Module;
use crate::{cli::ModuleArg, output::OutputFormatter};

pub async fn execute(
    adapter_path: String,
    emulate: bool,
    module: ModuleArg,
    stored: bool,
    pending: bool,
    permanent: bool,
) -> Result<()> {
    let mut executor = super::create_executor(adapter_path, emulate).await?;
    let mut formatter = OutputFormatter::new(false);
    let module: Module = module.into();

    let read_all = !stored && !pending && !permanent;

    if stored || read_all {
        formatter.print_header("Stored DTCs")?;
        let dtcs = executor.read_stored_dtcs(module).await?;
        formatter.print_dtcs(&dtcs)?;
    }

    if pending || read_all {
        formatter.print_header("Pending DTCs")?;
        let dtcs = executor.read_pending_dtcs(module).await?;
        formatter.print_dtcs(&dtcs)?;
    }

    if permanent || read_all {
        formatter.print_header("Permanent DTCs")?;
        let dtcs = executor.read_permanent_dtcs(module).await?;
        formatter.print_dtcs(&dtcs)?;
    }

    Ok(())
}
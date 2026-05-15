//! DTC read command.

use crate::{cli::ModuleArg, output::OutputFormatter};
use anyhow::Result;
use faraday_core::{DtcKind, Module};

pub async fn execute(
    adapter_path: String,
    module: ModuleArg,
    stored: bool,
    pending: bool,
    permanent: bool,
) -> Result<()> {
    let mut executor = super::create_executor(adapter_path).await?;
    let mut formatter = OutputFormatter::new(false);
    let module: Module = module.into();

    let read_all = !stored && !pending && !permanent;

    let categories = [
        (stored || read_all, "Stored DTCs", DtcKind::Stored),
        (pending || read_all, "Pending DTCs", DtcKind::Pending),
        (permanent || read_all, "Permanent DTCs", DtcKind::Permanent),
    ];

    for (should_read, header, kind) in categories {
        if !should_read {
            continue;
        }
        formatter.print_header(header)?;
        let dtcs = executor.read_dtcs(module, kind).await?;
        formatter.print_dtcs(&dtcs)?;
    }

    Ok(())
}

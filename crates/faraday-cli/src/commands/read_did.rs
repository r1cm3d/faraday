use anyhow::Result;
use faraday_core::{protocol::uds::DataIdentifier, Module};
use crate::{cli::ModuleArg, output::OutputFormatter};

pub async fn execute(
    adapter_path: String,
    emulate: bool,
    module: ModuleArg,
    did_string: String,
) -> Result<()> {
    let mut executor = super::create_executor(adapter_path, emulate).await?;
    let mut formatter = OutputFormatter::new(false);
    let module: Module = module.into();

    let did_value = u16::from_str_radix(did_string.trim_start_matches("0x"), 16)
        .map_err(|_| anyhow::anyhow!("Invalid DID: {}", did_string))?;

    let did = DataIdentifier(did_value);

    formatter.print_info(&format!("Reading DID {:04X} from {:?}...", did_value, module))?;

    let data = executor.read_data_by_identifier(module.request_id(), module.response_id(), did).await?;

    formatter.print_did_data(did_value, &data)?;

    Ok(())
}
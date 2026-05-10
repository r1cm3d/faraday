use anyhow::{anyhow, Result};
use faraday_asbuilt::{decoder::AsBuiltDecoder, AsBuiltBlock, FeatureValue};
use faraday_core::Module;

use crate::{
    cli::{AsBuiltAction, ModuleArg},
    output::OutputFormatter,
};

pub async fn execute(adapter_path: String, action: AsBuiltAction) -> Result<()> {
    match action {
        AsBuiltAction::Dump { module } => dump(adapter_path, module).await,
        AsBuiltAction::Show { module, feature } => show(adapter_path, module, feature).await,
    }
}

async fn dump(adapter_path: String, module_arg: ModuleArg) -> Result<()> {
    let module: Module = module_arg.into();
    let blocks = known_blocks(&module_arg);

    if blocks.is_empty() {
        return Err(anyhow!(
            "no as-built blocks defined for module {:?}",
            module
        ));
    }

    let mut executor = super::create_executor(adapter_path).await?;
    let mut populated: Vec<(AsBuiltBlock, Vec<FeatureValue>)> = Vec::new();

    for mut block in blocks {
        let raw = executor.read_asbuilt_block(module, block.did).await?;
        block.data = raw;
        let features = AsBuiltDecoder::decode_block(&block)
            .map_err(|e| anyhow!("decode error for {}: {}", block.id, e))?;
        populated.push((block, features));
    }

    let mut formatter = OutputFormatter::new(false);
    formatter.print_asbuilt_dump(&populated)?;
    Ok(())
}

async fn show(adapter_path: String, module_arg: ModuleArg, feature_name: String) -> Result<()> {
    let module: Module = module_arg.into();
    let blocks = known_blocks(&module_arg);

    if blocks.is_empty() {
        return Err(anyhow!(
            "no as-built blocks defined for module {:?}",
            module
        ));
    }

    let mut executor = super::create_executor(adapter_path).await?;

    for mut block in blocks {
        let raw = executor.read_asbuilt_block(module, block.did).await?;
        block.data = raw;
        let features = AsBuiltDecoder::decode_block(&block)
            .map_err(|e| anyhow!("decode error for {}: {}", block.id, e))?;

        if let Some(fv) = features.iter().find(|fv| fv.feature.name == feature_name) {
            let mut formatter = OutputFormatter::new(false);
            formatter.print_asbuilt_feature(fv)?;
            return Ok(());
        }
    }

    Err(anyhow!("unknown feature: {}", feature_name))
}

fn known_blocks(module: &ModuleArg) -> Vec<AsBuiltBlock> {
    match module {
        ModuleArg::Bcm => faraday_asbuilt::bcm::get_known_blocks(),
        ModuleArg::Ipc => faraday_asbuilt::ipc::get_known_blocks(),
        ModuleArg::Pcm => faraday_asbuilt::pcm::get_known_blocks(),
        _ => vec![],
    }
}

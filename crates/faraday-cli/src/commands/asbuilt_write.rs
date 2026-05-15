//! As-built write command with safety checks.

use anyhow::{anyhow, bail, Result};
use chrono::Utc;
use faraday_asbuilt::{
    encoder::AsBuiltEncoder, snapshot as snap, AsBuiltBlock, AsBuiltSnapshot, Feature, FeatureType,
};
use faraday_core::Module;
use std::path::PathBuf;

use crate::{
    audit::{AuditEntry, AuditLogger, AuditOperation},
    cli::ModuleArg,
    commands::create_executor,
    output::OutputFormatter,
};

pub async fn write(
    adapter_path: String,
    module_arg: ModuleArg,
    feature_name: String,
    value_str: String,
    dry_run: bool,
    yes: bool,
) -> Result<()> {
    let module: Module = module_arg.into();
    let blocks = known_blocks(&module_arg);

    let (mut block, feature) = find_feature(&blocks, &feature_name)
        .ok_or_else(|| anyhow!("unknown feature: {}", feature_name))?;

    let mut executor = create_executor(adapter_path).await?;
    let current_data = executor.read_asbuilt_block(module, block.did).await?;
    block.data = current_data.clone();

    let raw_value = parse_value(&feature, &value_str)?;
    let mut new_data = current_data.clone();
    AsBuiltEncoder::encode_feature(&mut new_data, &feature, raw_value)
        .map_err(|e| anyhow!("encode error: {}", e))?;

    let mut fmt = OutputFormatter::new(false);
    fmt.print_header(&format!("Write {} on module {:?}", feature_name, module))?;
    println!("  Before : {}", hex::encode_upper(&current_data));
    println!("  After  : {}", hex::encode_upper(&new_data));
    println!("  Feature: {} = {}", feature_name, value_str);

    if !yes && !dry_run && !confirm("Write to vehicle?")? {
        println!("Aborted.");
        return Ok(());
    }

    let timestamp = current_timestamp();
    let snapshot_path = snapshot_output(module_arg, &timestamp);
    let snapshot = AsBuiltSnapshot {
        timestamp: timestamp.clone(),
        vehicle_vin: None,
        blocks: vec![block.clone()],
    };
    snap::save_snapshot(&snapshot_path, &snapshot)
        .map_err(|e| anyhow!("failed to save snapshot: {}", e))?;
    fmt.print_info(&format!("Snapshot saved: {}", snapshot_path.display()))?;

    if dry_run {
        fmt.print_info("[DRY RUN] no changes written to vehicle")?;
        append_audit(
            AuditOperation::Write,
            module,
            block.did,
            &current_data,
            &new_data,
            true,
            "ok",
        )?;
        return Ok(());
    }

    let result = executor
        .write_asbuilt_block(module, block.did, &new_data)
        .await;

    match &result {
        Ok(_) => {
            append_audit(
                AuditOperation::Write,
                module,
                block.did,
                &current_data,
                &new_data,
                false,
                "ok",
            )?;
            fmt.print_success("Write complete")?;
        }
        Err(e) => {
            let msg = format!("error: {}", e);
            append_audit(
                AuditOperation::Write,
                module,
                block.did,
                &current_data,
                &new_data,
                false,
                &msg,
            )?;
            return Err(anyhow!("write failed: {}", e));
        }
    }

    Ok(())
}

pub async fn snapshot(
    adapter_path: String,
    module_arg: ModuleArg,
    output: Option<PathBuf>,
) -> Result<()> {
    let module: Module = module_arg.into();
    let blocks = known_blocks(&module_arg);

    if blocks.is_empty() {
        bail!("no as-built blocks defined for module {:?}", module);
    }

    let mut executor = create_executor(adapter_path).await?;
    let mut populated: Vec<AsBuiltBlock> = Vec::new();

    for mut block in blocks {
        let raw = executor.read_asbuilt_block(module, block.did).await?;
        block.data = raw;
        populated.push(block);
    }

    let timestamp = current_timestamp();
    let path = output.unwrap_or_else(|| snapshot_output(module_arg, &timestamp));
    let snap_data = AsBuiltSnapshot {
        timestamp,
        vehicle_vin: None,
        blocks: populated,
    };

    snap::save_snapshot(&path, &snap_data)
        .map_err(|e| anyhow!("failed to save snapshot: {}", e))?;

    let mut fmt = OutputFormatter::new(false);
    fmt.print_success(&format!("Snapshot saved: {}", path.display()))?;
    Ok(())
}

pub async fn restore(adapter_path: String, snapshot_path: PathBuf, yes: bool) -> Result<()> {
    let loaded = snap::load_snapshot(&snapshot_path)
        .map_err(|e| anyhow!("failed to load snapshot: {}", e))?;

    let mut fmt = OutputFormatter::new(false);
    fmt.print_header(&format!(
        "Restoring snapshot from {}",
        snapshot_path.display()
    ))?;
    println!("  Timestamp : {}", loaded.timestamp);
    if let Some(vin) = &loaded.vehicle_vin {
        println!("  VIN       : {}", vin);
    }
    println!("  Blocks    : {}", loaded.blocks.len());

    if !yes && !confirm("Restore all blocks?")? {
        println!("Aborted.");
        return Ok(());
    }

    let mut executor = create_executor(adapter_path).await?;

    for block in &loaded.blocks {
        let module = module_from_can_id(&block.id.module)
            .ok_or_else(|| anyhow!("unknown module CAN ID: {}", block.id.module))?;

        let current = executor.read_asbuilt_block(module, block.did).await?;

        let result = executor
            .write_asbuilt_block(module, block.did, &block.data)
            .await;

        match &result {
            Ok(_) => {
                append_audit(
                    AuditOperation::Restore,
                    module,
                    block.did,
                    &current,
                    &block.data,
                    false,
                    "ok",
                )?;
                fmt.print_success(&format!("Restored block {}", block.id))?;
            }
            Err(e) => {
                let msg = format!("error: {}", e);
                append_audit(
                    AuditOperation::Restore,
                    module,
                    block.did,
                    &current,
                    &block.data,
                    false,
                    &msg,
                )?;
                return Err(anyhow!("restore failed for {}: {}", block.id, e));
            }
        }
    }

    fmt.print_success("Restore complete")?;
    Ok(())
}

/// Searches `blocks` for a feature with the given `name` and returns the owning block and feature.
pub(crate) fn find_feature(blocks: &[AsBuiltBlock], name: &str) -> Option<(AsBuiltBlock, Feature)> {
    for block in blocks {
        if let Some(feature) = block.features.iter().find(|f| f.name == name) {
            return Some((block.clone(), feature.clone()));
        }
    }
    None
}

/// Parses a raw string value into a `u8` according to the feature's type constraints.
pub(crate) fn parse_value(feature: &Feature, raw: &str) -> Result<u8> {
    match &feature.feature_type {
        FeatureType::Boolean { .. } => match raw.to_ascii_lowercase().as_str() {
            "true" | "1" | "on" | "enabled" | "yes" => Ok(1),
            "false" | "0" | "off" | "disabled" | "no" => Ok(0),
            _ => Err(anyhow!(
                "invalid boolean value '{}': use true/false/on/off/1/0",
                raw
            )),
        },
        FeatureType::Enumerated { values } => {
            if let Some((&k, _)) = values.iter().find(|(_, v)| v.eq_ignore_ascii_case(raw)) {
                return Ok(k);
            }
            raw.parse::<u8>().map_err(|_| {
                let options: Vec<String> =
                    values.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
                anyhow!(
                    "invalid value '{}': options are {}",
                    raw,
                    options.join(", ")
                )
            })
        }
        FeatureType::Numeric { min, max, .. } => {
            let v: u8 = raw
                .parse()
                .map_err(|_| anyhow!("'{}' is not a valid number", raw))?;
            if v < *min || v > *max {
                bail!("value {} out of range [{}, {}]", v, min, max);
            }
            Ok(v)
        }
    }
}

fn append_audit(
    operation: AuditOperation,
    module: Module,
    did: u16,
    before: &[u8],
    after: &[u8],
    dry_run: bool,
    result: &str,
) -> Result<()> {
    let logger = AuditLogger::new()?;
    logger.log(&AuditEntry {
        timestamp: current_timestamp(),
        operation,
        module: format!("{:?}", module),
        did,
        before_hex: hex::encode_upper(before),
        after_hex: hex::encode_upper(after),
        dry_run,
        result: result.to_string(),
    })
}

fn confirm(prompt: &str) -> Result<bool> {
    use std::io::{self, Write};
    print!("{} [y/N] ", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}

fn current_timestamp() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn snapshot_output(module_arg: ModuleArg, timestamp: &str) -> PathBuf {
    let filename = format!(
        "{}_{}.json",
        module_name(module_arg),
        timestamp.replace(':', "-")
    );
    crate::audit::snapshot_dir().join(filename)
}

fn module_name(module_arg: ModuleArg) -> &'static str {
    match module_arg {
        ModuleArg::Bcm => "bcm",
        ModuleArg::Ipc => "ipc",
        ModuleArg::Pcm => "pcm",
        ModuleArg::Tcm => "tcm",
        ModuleArg::Abs => "abs",
        ModuleArg::Rcm => "rcm",
        ModuleArg::Pscm => "pscm",
        ModuleArg::Apim => "apim",
        ModuleArg::Pam => "pam",
        ModuleArg::Dsm => "dsm",
    }
}

fn module_from_can_id(can_id_str: &str) -> Option<Module> {
    match can_id_str.to_ascii_uppercase().as_str() {
        "726" => Some(Module::Bcm),
        "720" => Some(Module::Ipc),
        "7E0" => Some(Module::Pcm),
        "7E1" => Some(Module::Tcm),
        "760" => Some(Module::Abs),
        "737" => Some(Module::Rcm),
        "730" => Some(Module::Pscm),
        "7D0" => Some(Module::Apim),
        "736" => Some(Module::Pam),
        "754" => Some(Module::Dsm),
        _ => None,
    }
}

/// Returns the catalog of known as-built blocks for the given module.
pub(crate) fn known_blocks(module: &ModuleArg) -> Vec<AsBuiltBlock> {
    match module {
        ModuleArg::Bcm => faraday_asbuilt::bcm::get_known_blocks(),
        ModuleArg::Ipc => faraday_asbuilt::ipc::get_known_blocks(),
        ModuleArg::Pcm => faraday_asbuilt::pcm::get_known_blocks(),
        _ => vec![],
    }
}

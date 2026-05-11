use anyhow::{anyhow, bail, Result};
use std::path::PathBuf;

use crate::{
    cli::{ModuleArg, ProfileAction},
    output::OutputFormatter,
    profile::Profile,
};

pub async fn execute(adapter_path: String, action: ProfileAction) -> Result<()> {
    match action {
        ProfileAction::Apply { file, dry_run, yes } => {
            apply(adapter_path, file, dry_run, yes).await
        }
        ProfileAction::Validate { file } => validate(file),
    }
}

pub fn validate(path: PathBuf) -> Result<()> {
    let profile = Profile::load(&path)?;

    let mut errors: Vec<String> = Vec::new();
    let mut ok_count = 0usize;

    for (module_str, features) in &profile.modules {
        let module_arg = match parse_module_arg(module_str) {
            Some(m) => m,
            None => {
                errors.push(format!("unknown module: {}", module_str));
                continue;
            }
        };

        let blocks = super::asbuilt_write::known_blocks(&module_arg);

        for (feature_name, value) in features {
            let value_str = yaml_value_to_str(value);
            match super::asbuilt_write::find_feature(&blocks, feature_name) {
                Some((_, feature)) => {
                    match super::asbuilt_write::parse_value(&feature, &value_str) {
                        Ok(_) => {
                            println!("  ok  {}.{} = {}", module_str, feature_name, value_str);
                            ok_count += 1;
                        }
                        Err(e) => {
                            errors.push(format!(
                                "{}.{} = {}: {}",
                                module_str, feature_name, value_str, e
                            ));
                        }
                    }
                }
                None => {
                    errors.push(format!("unknown feature: {}.{}", module_str, feature_name));
                }
            }
        }
    }

    if errors.is_empty() {
        println!("\nProfile valid ({} features)", ok_count);
        Ok(())
    } else {
        for e in &errors {
            eprintln!("error: {}", e);
        }
        bail!("profile validation failed ({} errors)", errors.len());
    }
}

pub async fn apply(adapter_path: String, path: PathBuf, dry_run: bool, yes: bool) -> Result<()> {
    let profile = Profile::load(&path)?;

    validate(path.clone())?;

    let total: usize = profile.modules.values().map(|f| f.len()).sum();

    let mut fmt = OutputFormatter::new(false);
    fmt.print_header(&format!("Profile apply: {}", path.display()))?;

    if let Some(v) = &profile.vehicle {
        if let Some(vin) = &v.vin {
            println!("  VIN  : {}", vin);
        }
        if let Some(model) = &v.model {
            println!("  Model: {}", model);
        }
    }
    println!("  Features to write: {}", total);

    if !yes && !dry_run && !confirm(&format!("Apply {} feature(s) to vehicle?", total))? {
        println!("Aborted.");
        return Ok(());
    }

    let mut written = 0usize;
    let mut failed = 0usize;

    for (module_str, features) in &profile.modules {
        let module_arg = parse_module_arg(module_str)
            .ok_or_else(|| anyhow!("unknown module: {}", module_str))?;

        for (feature_name, value) in features {
            let value_str = yaml_value_to_str(value);
            let result = super::asbuilt_write::write(
                adapter_path.clone(),
                module_arg,
                feature_name.clone(),
                value_str.clone(),
                dry_run,
                true,
            )
            .await;

            match result {
                Ok(_) => written += 1,
                Err(e) => {
                    eprintln!("failed {}.{}: {}", module_str, feature_name, e);
                    failed += 1;
                }
            }
        }
    }

    println!("\nSummary: {} written, {} failed", written, failed);

    if failed > 0 {
        bail!("{} feature(s) failed to write", failed);
    }
    Ok(())
}

fn parse_module_arg(s: &str) -> Option<ModuleArg> {
    match s.to_ascii_lowercase().as_str() {
        "pcm" => Some(ModuleArg::Pcm),
        "tcm" => Some(ModuleArg::Tcm),
        "abs" => Some(ModuleArg::Abs),
        "rcm" => Some(ModuleArg::Rcm),
        "pscm" => Some(ModuleArg::Pscm),
        "bcm" => Some(ModuleArg::Bcm),
        "ipc" => Some(ModuleArg::Ipc),
        "apim" => Some(ModuleArg::Apim),
        "pam" => Some(ModuleArg::Pam),
        "dsm" => Some(ModuleArg::Dsm),
        _ => None,
    }
}

fn yaml_value_to_str(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::String(s) => s.clone(),
        other => format!("{:?}", other),
    }
}

fn confirm(prompt: &str) -> Result<bool> {
    use std::io::{self, Write};
    print!("{} [y/N] ", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}

use anyhow::{Context, Result};
use serde::Deserialize;
use std::{collections::HashMap, path::Path};

#[derive(Debug, Deserialize)]
pub struct Profile {
    pub vehicle: Option<VehicleInfo>,
    pub modules: HashMap<String, HashMap<String, serde_yaml::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct VehicleInfo {
    pub vin: Option<String>,
    pub model: Option<String>,
}

impl Profile {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read profile: {}", path.display()))?;
        serde_yaml::from_str(&content)
            .with_context(|| format!("failed to parse profile: {}", path.display()))
    }
}

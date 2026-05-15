//! YAML vehicle profile loading.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::{collections::HashMap, path::Path};

/// A YAML profile describing desired vehicle configuration.
#[derive(Debug, Deserialize)]
pub struct Profile {
    /// Optional vehicle identification block.
    pub vehicle: Option<VehicleInfo>,
    /// Per-module feature overrides keyed by module name then feature name.
    pub modules: HashMap<String, HashMap<String, serde_yaml::Value>>,
}

/// Vehicle identification fields embedded in a YAML profile.
#[derive(Debug, Deserialize)]
pub struct VehicleInfo {
    /// Expected VIN — used to guard against applying a profile to the wrong vehicle.
    pub vin: Option<String>,
    /// Human-readable model description (informational only).
    pub model: Option<String>,
}

impl Profile {
    /// Reads and deserializes a YAML profile from `path`.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read profile: {}", path.display()))?;
        serde_yaml::from_str(&content)
            .with_context(|| format!("failed to parse profile: {}", path.display()))
    }
}

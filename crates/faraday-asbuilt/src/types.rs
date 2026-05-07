use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockId {
    pub module: String,
    pub id: String,
}

impl BlockId {
    pub fn new(module: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            module: module.into(),
            id: id.into(),
        }
    }
}

impl std::fmt::Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.module, self.id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsBuiltBlock {
    pub id: BlockId,
    pub description: String,
    pub data: Vec<u8>,
    pub features: Vec<Feature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub name: String,
    pub description: String,
    pub bit_position: BitPosition,
    pub feature_type: FeatureType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BitPosition {
    pub byte: usize,
    pub bit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureType {
    Boolean { true_description: String, false_description: String },
    Enumerated { values: HashMap<u8, String> },
    Numeric { min: u8, max: u8, unit: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureValue {
    pub feature: Feature,
    pub raw_value: u8,
    pub interpreted_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsBuiltSnapshot {
    pub timestamp: String,
    pub vehicle_vin: Option<String>,
    pub blocks: Vec<AsBuiltBlock>,
}
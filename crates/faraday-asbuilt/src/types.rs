//! Core data types for as-built block catalogs and feature decoding.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Uniquely identifies an as-built block by module CAN ID and block index.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockId {
    /// CAN ID of the ECU module (e.g. `"726"` for BCM).
    pub module: String,
    /// Block index within the module (e.g. `"01"`).
    pub id: String,
}

impl BlockId {
    /// Creates a new [`BlockId`] from the given module CAN ID and block index.
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

/// One as-built block read from a vehicle ECU, along with its feature catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsBuiltBlock {
    /// Logical identifier for this block (module + index).
    pub id: BlockId,
    /// Human-readable description of the block's purpose.
    pub description: String,
    /// UDS Data Identifier used to read and write this block.
    pub did: u16,
    /// Raw bytes returned by the ECU for this block.
    pub data: Vec<u8>,
    /// Catalog of features encoded in this block.
    pub features: Vec<Feature>,
}

/// A configurable feature encoded as a bit field within an as-built block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    /// Short machine-readable name used for CLI lookup (e.g. `"drl_enabled"`).
    pub name: String,
    /// Human-readable description of the feature.
    pub description: String,
    /// Location of the bit field within the block data.
    pub bit_position: BitPosition,
    /// Type and valid values for this feature.
    pub feature_type: FeatureType,
}

/// Location of a feature's bit field within a byte array.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BitPosition {
    /// Zero-based byte index within the block data.
    pub byte: usize,
    /// Bit index within the byte (0 = LSB, 7 = MSB).
    pub bit: usize,
}

/// Describes the valid encoding for a feature's bit field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureType {
    /// A single-bit on/off flag.
    Boolean {
        true_description: String,
        false_description: String,
    },
    /// A multi-bit field mapping raw values to named variants.
    Enumerated { values: HashMap<u8, String> },
    /// A numeric field with an inclusive valid range and optional unit.
    Numeric {
        min: u8,
        max: u8,
        unit: Option<String>,
    },
}

/// A decoded feature value along with its source feature definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureValue {
    /// The feature definition that produced this value.
    pub feature: Feature,
    /// Raw bit-field value extracted from the block data.
    pub raw_value: u8,
    /// Human-readable interpretation of the raw value.
    pub interpreted_value: String,
}

/// A point-in-time snapshot of one or more as-built blocks read from a vehicle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsBuiltSnapshot {
    /// ISO 8601 timestamp when the snapshot was taken.
    pub timestamp: String,
    /// Vehicle VIN, if available at snapshot time.
    pub vehicle_vin: Option<String>,
    /// As-built blocks included in this snapshot.
    pub blocks: Vec<AsBuiltBlock>,
}

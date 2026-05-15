/*!
Body Control Module (BCM) as-built configuration blocks for Ford Fusion 2017 SEL.

The BCM controls various convenience and comfort features including
lighting, locking, and other body functions.
*/

use crate::{AsBuiltBlock, BitPosition, BlockId, Feature, FeatureType};
use std::collections::HashMap;

/// Returns all known BCM as-built blocks with their feature catalog.
pub fn get_known_blocks() -> Vec<AsBuiltBlock> {
    vec![create_block_726_01(), create_block_726_02()]
}

fn create_block_726_01() -> AsBuiltBlock {
    AsBuiltBlock {
        id: BlockId::new("726", "01"),
        description: "BCM Configuration Block 01 - Lighting and DRL".to_string(),
        did: 0x0701,
        data: vec![0; 8], // Placeholder data
        features: vec![
            Feature {
                name: "drl_enabled".to_string(),
                description: "Daytime Running Lights Enable".to_string(),
                bit_position: BitPosition { byte: 3, bit: 2 },
                feature_type: FeatureType::Boolean {
                    true_description: "DRL Enabled".to_string(),
                    false_description: "DRL Disabled".to_string(),
                },
            },
            Feature {
                name: "auto_headlights".to_string(),
                description: "Automatic Headlight Control".to_string(),
                bit_position: BitPosition { byte: 2, bit: 4 },
                feature_type: FeatureType::Boolean {
                    true_description: "Auto Headlights Enabled".to_string(),
                    false_description: "Manual Headlights Only".to_string(),
                },
            },
            Feature {
                name: "welcome_lighting".to_string(),
                description: "Welcome Lighting Duration".to_string(),
                bit_position: BitPosition { byte: 4, bit: 0 },
                feature_type: FeatureType::Numeric {
                    min: 0,
                    max: 7,
                    unit: Some("seconds".to_string()),
                },
            },
        ],
    }
}

fn create_block_726_02() -> AsBuiltBlock {
    let mut unlock_beeps = HashMap::new();
    unlock_beeps.insert(0, "No Beeps".to_string());
    unlock_beeps.insert(1, "Single Beep".to_string());
    unlock_beeps.insert(2, "Double Beep".to_string());
    unlock_beeps.insert(3, "Triple Beep".to_string());

    AsBuiltBlock {
        id: BlockId::new("726", "02"),
        description: "BCM Configuration Block 02 - Security and Convenience".to_string(),
        did: 0x0702,
        data: vec![0; 8], // Placeholder data
        features: vec![
            Feature {
                name: "auto_lock_on_drive".to_string(),
                description: "Auto Lock When Driving".to_string(),
                bit_position: BitPosition { byte: 1, bit: 3 },
                feature_type: FeatureType::Boolean {
                    true_description: "Auto Lock Enabled".to_string(),
                    false_description: "Auto Lock Disabled".to_string(),
                },
            },
            Feature {
                name: "unlock_beeps".to_string(),
                description: "Number of Beeps on Unlock".to_string(),
                bit_position: BitPosition { byte: 2, bit: 0 },
                feature_type: FeatureType::Enumerated {
                    values: unlock_beeps,
                },
            },
            Feature {
                name: "remote_start_enabled".to_string(),
                description: "Remote Start Feature".to_string(),
                bit_position: BitPosition { byte: 5, bit: 1 },
                feature_type: FeatureType::Boolean {
                    true_description: "Remote Start Enabled".to_string(),
                    false_description: "Remote Start Disabled".to_string(),
                },
            },
        ],
    }
}

/// Returns the BCM block whose index matches `id`, or `None` if not found.
pub fn get_block_by_id(id: &str) -> Option<AsBuiltBlock> {
    get_known_blocks()
        .into_iter()
        .find(|block| block.id.id == id)
}

/// Returns a flat list of every feature defined across all BCM blocks.
pub fn get_all_features() -> Vec<Feature> {
    get_known_blocks()
        .into_iter()
        .flat_map(|block| block.features)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_known_blocks() {
        let blocks = get_known_blocks();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].id.to_string(), "726-01");
        assert_eq!(blocks[1].id.to_string(), "726-02");
    }

    #[test]
    fn test_get_block_by_id() {
        let block = get_block_by_id("01").unwrap();
        assert_eq!(block.id.to_string(), "726-01");
        assert!(block.description.contains("Lighting"));
    }

    #[test]
    fn test_block_726_01_features() {
        let block = create_block_726_01();
        assert_eq!(block.features.len(), 3);

        let drl_feature = &block.features[0];
        assert_eq!(drl_feature.name, "drl_enabled");
        assert_eq!(drl_feature.bit_position.byte, 3);
        assert_eq!(drl_feature.bit_position.bit, 2);
    }

    #[test]
    fn test_block_dids() {
        let blocks = get_known_blocks();
        assert_eq!(blocks[0].did, 0x0701);
        assert_eq!(blocks[1].did, 0x0702);
    }
}

/*!
Powertrain Control Module (PCM) as-built configuration blocks for Ford Fusion 2017 SEL.

The PCM controls engine and transmission parameters. Most PCM configuration
is not user-modifiable and relates to emissions compliance and performance.
*/

use crate::{AsBuiltBlock, BlockId, Feature, FeatureType, BitPosition};
use std::collections::HashMap;

pub fn get_known_blocks() -> Vec<AsBuiltBlock> {
    vec![
        create_block_7e0_01(),
    ]
}

fn create_block_7e0_01() -> AsBuiltBlock {
    let mut fuel_types = HashMap::new();
    fuel_types.insert(0, "Regular Gasoline".to_string());
    fuel_types.insert(1, "Premium Gasoline".to_string());
    fuel_types.insert(2, "Flex Fuel (E85)".to_string());

    AsBuiltBlock {
        id: BlockId::new("7E0", "01"),
        description: "PCM Configuration Block 01 - Engine Parameters".to_string(),
        data: vec![0; 8], // Placeholder data
        features: vec![
            Feature {
                name: "fuel_type".to_string(),
                description: "Fuel Type Configuration".to_string(),
                bit_position: BitPosition { byte: 2, bit: 0 },
                feature_type: FeatureType::Enumerated {
                    values: fuel_types,
                },
            },
            Feature {
                name: "auto_start_stop".to_string(),
                description: "Auto Start-Stop Feature".to_string(),
                bit_position: BitPosition { byte: 4, bit: 3 },
                feature_type: FeatureType::Boolean {
                    true_description: "Auto Start-Stop Enabled".to_string(),
                    false_description: "Auto Start-Stop Disabled".to_string(),
                },
            },
            Feature {
                name: "idle_speed_offset".to_string(),
                description: "Idle Speed Offset".to_string(),
                bit_position: BitPosition { byte: 6, bit: 0 },
                feature_type: FeatureType::Numeric {
                    min: 0,
                    max: 15,
                    unit: Some("rpm x 10".to_string()),
                },
            },
        ],
    }
}

pub fn get_block_by_id(id: &str) -> Option<AsBuiltBlock> {
    get_known_blocks()
        .into_iter()
        .find(|block| block.id.id == id)
}

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
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].id.to_string(), "7E0-01");
    }

    #[test]
    fn test_get_block_by_id() {
        let block = get_block_by_id("01").unwrap();
        assert_eq!(block.id.to_string(), "7E0-01");
        assert!(block.description.contains("Engine Parameters"));
    }

    #[test]
    fn test_block_7e0_01_features() {
        let block = create_block_7e0_01();
        assert_eq!(block.features.len(), 3);

        let fuel_type_feature = &block.features[0];
        assert_eq!(fuel_type_feature.name, "fuel_type");
        assert_eq!(fuel_type_feature.bit_position.byte, 2);
        assert_eq!(fuel_type_feature.bit_position.bit, 0);
    }
}
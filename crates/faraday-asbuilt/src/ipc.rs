/*!
Instrument Panel Cluster (IPC) as-built configuration blocks for Ford Fusion 2017 SEL.

The IPC controls the instrument cluster display, gauges, warning lights,
and various display preferences.
*/

use crate::{AsBuiltBlock, BitPosition, BlockId, Feature, FeatureType};
use std::collections::HashMap;

/// Returns all known IPC as-built blocks with their feature catalog.
pub fn get_known_blocks() -> Vec<AsBuiltBlock> {
    vec![
        create_block_720_01(),
        create_block_720_02(),
        create_block_720_04(),
    ]
}

fn create_block_720_01() -> AsBuiltBlock {
    let mut speed_units = HashMap::new();
    speed_units.insert(0, "km/h".to_string());
    speed_units.insert(1, "mph".to_string());

    let mut temp_units = HashMap::new();
    temp_units.insert(0, "Celsius".to_string());
    temp_units.insert(1, "Fahrenheit".to_string());

    AsBuiltBlock {
        id: BlockId::new("720", "01"),
        description: "IPC Configuration Block 01 - Display Settings".to_string(),
        did: 0x0101,
        data: vec![0; 8],
        features: vec![
            Feature {
                name: "show_digital_speedometer".to_string(),
                description: "Digital Speedometer Display".to_string(),
                bit_position: BitPosition { byte: 2, bit: 1 },
                feature_type: FeatureType::Boolean {
                    true_description: "Digital Speedometer Shown".to_string(),
                    false_description: "Analog Only".to_string(),
                },
            },
            Feature {
                name: "speed_units".to_string(),
                description: "Speed Display Units".to_string(),
                bit_position: BitPosition { byte: 1, bit: 0 },
                feature_type: FeatureType::Enumerated {
                    values: speed_units,
                },
            },
            Feature {
                name: "temperature_units".to_string(),
                description: "Temperature Display Units".to_string(),
                bit_position: BitPosition { byte: 1, bit: 2 },
                feature_type: FeatureType::Enumerated { values: temp_units },
            },
            Feature {
                name: "show_compass".to_string(),
                description: "Compass Display".to_string(),
                bit_position: BitPosition { byte: 3, bit: 4 },
                feature_type: FeatureType::Boolean {
                    true_description: "Compass Enabled".to_string(),
                    false_description: "Compass Disabled".to_string(),
                },
            },
        ],
    }
}

fn create_block_720_02() -> AsBuiltBlock {
    let mut brightness_levels = HashMap::new();
    for i in 0..=7 {
        brightness_levels.insert(i, format!("Level {}", i + 1));
    }

    AsBuiltBlock {
        id: BlockId::new("720", "02"),
        description: "IPC Configuration Block 02 - Lighting and Animation".to_string(),
        did: 0x0102,
        data: vec![0; 8],
        features: vec![
            Feature {
                name: "welcome_animation".to_string(),
                description: "Startup Welcome Animation".to_string(),
                bit_position: BitPosition { byte: 0, bit: 7 },
                feature_type: FeatureType::Boolean {
                    true_description: "Animation Enabled".to_string(),
                    false_description: "Animation Disabled".to_string(),
                },
            },
            Feature {
                name: "gauge_brightness".to_string(),
                description: "Gauge Cluster Brightness".to_string(),
                bit_position: BitPosition { byte: 2, bit: 0 },
                feature_type: FeatureType::Enumerated {
                    values: brightness_levels,
                },
            },
            Feature {
                name: "show_ambient_temp".to_string(),
                description: "Ambient Temperature Display".to_string(),
                bit_position: BitPosition { byte: 1, bit: 5 },
                feature_type: FeatureType::Boolean {
                    true_description: "Ambient Temp Shown".to_string(),
                    false_description: "Ambient Temp Hidden".to_string(),
                },
            },
            Feature {
                name: "auto_dim_cluster".to_string(),
                description: "Auto Dim Cluster at Night".to_string(),
                bit_position: BitPosition { byte: 4, bit: 2 },
                feature_type: FeatureType::Boolean {
                    true_description: "Auto Dim Enabled".to_string(),
                    false_description: "Manual Brightness Only".to_string(),
                },
            },
        ],
    }
}

fn create_block_720_04() -> AsBuiltBlock {
    let mut units = HashMap::new();
    units.insert(4u8, "psi".to_string());
    units.insert(8u8, "kpa".to_string());
    units.insert(12u8, "bar".to_string());

    AsBuiltBlock {
        id: BlockId::new("720", "04"),
        description: "IPC Configuration Block 04 - TPMS Display Settings".to_string(),
        did: 0x0401,
        data: vec![0; 1],
        features: vec![Feature {
            name: "tpms_display_units".to_string(),
            description: "Tire Pressure Display Units".to_string(),
            bit_position: BitPosition { byte: 0, bit: 0 },
            feature_type: FeatureType::Enumerated { values: units },
        }],
    }
}

/// Returns the IPC block whose index matches `id`, or `None` if not found.
pub fn get_block_by_id(id: &str) -> Option<AsBuiltBlock> {
    get_known_blocks()
        .into_iter()
        .find(|block| block.id.id == id)
}

/// Returns a flat list of every feature defined across all IPC blocks.
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
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].id.to_string(), "720-01");
        assert_eq!(blocks[1].id.to_string(), "720-02");
        assert_eq!(blocks[2].id.to_string(), "720-04");
    }

    #[test]
    fn test_get_block_by_id() {
        let block = get_block_by_id("01").unwrap();
        assert_eq!(block.id.to_string(), "720-01");
        assert!(block.description.contains("Display Settings"));
    }

    #[test]
    fn test_block_720_01_features() {
        let block = create_block_720_01();
        assert_eq!(block.features.len(), 4);

        let digital_speedo_feature = &block.features[0];
        assert_eq!(digital_speedo_feature.name, "show_digital_speedometer");
        assert_eq!(digital_speedo_feature.bit_position.byte, 2);
        assert_eq!(digital_speedo_feature.bit_position.bit, 1);
    }

    #[test]
    fn test_block_dids() {
        let blocks = get_known_blocks();
        assert_eq!(blocks[0].did, 0x0101);
        assert_eq!(blocks[1].did, 0x0102);
        assert_eq!(blocks[2].did, 0x0401);
    }

    #[test]
    fn test_block_720_04_tpms_feature() {
        let block = create_block_720_04();
        assert_eq!(block.features.len(), 1);
        assert_eq!(block.features[0].name, "tpms_display_units");
        assert_eq!(block.features[0].bit_position.byte, 0);
    }
}

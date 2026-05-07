use crate::{AsBuiltBlock, Feature, FeatureType, FeatureValue, Result, Error, BitPosition};

pub struct AsBuiltDecoder;

impl AsBuiltDecoder {
    pub fn decode_block(block: &AsBuiltBlock) -> Result<Vec<FeatureValue>> {
        let mut values = Vec::new();

        for feature in &block.features {
            let raw_value = Self::extract_bit_value(&block.data, feature.bit_position)?;
            let interpreted_value = Self::interpret_value(&feature.feature_type, raw_value);

            values.push(FeatureValue {
                feature: feature.clone(),
                raw_value,
                interpreted_value,
            });
        }

        Ok(values)
    }

    pub fn decode_feature(data: &[u8], feature: &Feature) -> Result<FeatureValue> {
        let raw_value = Self::extract_bit_value(data, feature.bit_position)?;
        let interpreted_value = Self::interpret_value(&feature.feature_type, raw_value);

        Ok(FeatureValue {
            feature: feature.clone(),
            raw_value,
            interpreted_value,
        })
    }

    fn extract_bit_value(data: &[u8], position: BitPosition) -> Result<u8> {
        if position.byte >= data.len() {
            return Err(Error::data_too_short(position.byte + 1, data.len()));
        }

        if position.bit > 7 {
            return Err(Error::invalid_bit_position(position.byte, position.bit));
        }

        let byte_value = data[position.byte];
        let bit_value = (byte_value >> position.bit) & 1;

        Ok(bit_value)
    }

    fn interpret_value(feature_type: &FeatureType, raw_value: u8) -> String {
        match feature_type {
            FeatureType::Boolean { true_description, false_description } => {
                if raw_value != 0 {
                    true_description.clone()
                } else {
                    false_description.clone()
                }
            }
            FeatureType::Enumerated { values } => {
                values
                    .get(&raw_value)
                    .cloned()
                    .unwrap_or_else(|| format!("Unknown value: {}", raw_value))
            }
            FeatureType::Numeric { min, max, unit } => {
                let clamped_value = raw_value.clamp(*min, *max);
                if let Some(unit) = unit {
                    format!("{} {}", clamped_value, unit)
                } else {
                    clamped_value.to_string()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_extract_bit_value() {
        let data = vec![0b10101010, 0b11110000];

        assert_eq!(AsBuiltDecoder::extract_bit_value(&data, BitPosition { byte: 0, bit: 0 }).unwrap(), 0);
        assert_eq!(AsBuiltDecoder::extract_bit_value(&data, BitPosition { byte: 0, bit: 1 }).unwrap(), 1);
        assert_eq!(AsBuiltDecoder::extract_bit_value(&data, BitPosition { byte: 0, bit: 7 }).unwrap(), 1);

        assert_eq!(AsBuiltDecoder::extract_bit_value(&data, BitPosition { byte: 1, bit: 0 }).unwrap(), 0);
        assert_eq!(AsBuiltDecoder::extract_bit_value(&data, BitPosition { byte: 1, bit: 4 }).unwrap(), 1);
    }

    #[test]
    fn test_interpret_boolean() {
        let feature_type = FeatureType::Boolean {
            true_description: "Enabled".to_string(),
            false_description: "Disabled".to_string(),
        };

        assert_eq!(AsBuiltDecoder::interpret_value(&feature_type, 1), "Enabled");
        assert_eq!(AsBuiltDecoder::interpret_value(&feature_type, 0), "Disabled");
    }

    #[test]
    fn test_interpret_enumerated() {
        let mut values = HashMap::new();
        values.insert(0, "Off".to_string());
        values.insert(1, "On".to_string());
        values.insert(2, "Auto".to_string());

        let feature_type = FeatureType::Enumerated { values };

        assert_eq!(AsBuiltDecoder::interpret_value(&feature_type, 0), "Off");
        assert_eq!(AsBuiltDecoder::interpret_value(&feature_type, 2), "Auto");
        assert_eq!(AsBuiltDecoder::interpret_value(&feature_type, 5), "Unknown value: 5");
    }
}
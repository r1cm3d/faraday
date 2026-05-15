//! Bit-field encoder: writes feature values back into raw as-built block data.

use crate::{BitPosition, Error, Feature, FeatureType, Result};

/// Stateless encoder that writes feature values into raw as-built block data.
pub struct AsBuiltEncoder;

impl AsBuiltEncoder {
    /// Validates `raw_value` against `feature`'s type constraints and writes it into `data`.
    pub fn encode_feature(data: &mut [u8], feature: &Feature, raw_value: u8) -> Result<()> {
        Self::validate_value(&feature.feature_type, raw_value)?;
        Self::set_bit_value(data, feature.bit_position, raw_value)
    }

    /// Sets or clears the single bit at `position` in `data` according to `value & 1`.
    pub fn set_bit_value(data: &mut [u8], position: BitPosition, value: u8) -> Result<()> {
        if position.byte >= data.len() {
            return Err(Error::data_too_short(position.byte + 1, data.len()));
        }
        if position.bit > 7 {
            return Err(Error::invalid_bit_position(position.byte, position.bit));
        }
        if value & 1 == 1 {
            data[position.byte] |= 1 << position.bit;
        } else {
            data[position.byte] &= !(1u8 << position.bit);
        }
        Ok(())
    }

    fn validate_value(feature_type: &FeatureType, value: u8) -> Result<()> {
        match feature_type {
            FeatureType::Boolean { .. } => {
                if value > 1 {
                    return Err(Error::invalid_block(format!(
                        "boolean feature expects 0 or 1, got {}",
                        value
                    )));
                }
            }
            FeatureType::Enumerated { values } => {
                if !values.contains_key(&value) {
                    return Err(Error::invalid_block(format!(
                        "enumerated feature has no variant {}",
                        value
                    )));
                }
            }
            FeatureType::Numeric { min, max, .. } => {
                if value < *min || value > *max {
                    return Err(Error::invalid_block(format!(
                        "numeric value {} out of range [{}, {}]",
                        value, min, max
                    )));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BitPosition;
    use std::collections::HashMap;

    #[test]
    fn set_bit_value_sets_bit_to_one() {
        let mut data = vec![0b0000_0000u8, 0b0000_0000u8];
        AsBuiltEncoder::set_bit_value(&mut data, BitPosition { byte: 0, bit: 3 }, 1).unwrap();
        assert_eq!(data[0], 0b0000_1000);
    }

    #[test]
    fn set_bit_value_clears_bit_to_zero() {
        let mut data = vec![0b1111_1111u8];
        AsBuiltEncoder::set_bit_value(&mut data, BitPosition { byte: 0, bit: 5 }, 0).unwrap();
        assert_eq!(data[0], 0b1101_1111);
    }

    #[test]
    fn set_then_read_roundtrips() {
        let mut data = vec![0u8; 4];
        let pos = BitPosition { byte: 2, bit: 6 };
        AsBuiltEncoder::set_bit_value(&mut data, pos, 1).unwrap();
        assert_eq!((data[2] >> 6) & 1, 1);

        AsBuiltEncoder::set_bit_value(&mut data, pos, 0).unwrap();
        assert_eq!((data[2] >> 6) & 1, 0);
    }

    #[test]
    fn validate_boolean_rejects_value_above_one() {
        let feature_type = FeatureType::Boolean {
            true_description: "On".to_string(),
            false_description: "Off".to_string(),
        };
        let mut data = vec![0u8];
        let feature = Feature {
            name: "test".to_string(),
            description: "test".to_string(),
            bit_position: BitPosition { byte: 0, bit: 0 },
            feature_type,
        };
        assert!(AsBuiltEncoder::encode_feature(&mut data, &feature, 2).is_err());
    }

    #[test]
    fn validate_enumerated_rejects_unknown_variant() {
        let mut values = HashMap::new();
        values.insert(0u8, "Off".to_string());
        values.insert(1u8, "On".to_string());
        let feature_type = FeatureType::Enumerated { values };
        let mut data = vec![0u8];
        let feature = Feature {
            name: "test".to_string(),
            description: "test".to_string(),
            bit_position: BitPosition { byte: 0, bit: 0 },
            feature_type,
        };
        assert!(AsBuiltEncoder::encode_feature(&mut data, &feature, 5).is_err());
    }

    #[test]
    fn validate_numeric_rejects_out_of_range() {
        let feature_type = FeatureType::Numeric {
            min: 1,
            max: 5,
            unit: None,
        };
        let mut data = vec![0u8];
        let feature = Feature {
            name: "test".to_string(),
            description: "test".to_string(),
            bit_position: BitPosition { byte: 0, bit: 0 },
            feature_type,
        };
        assert!(AsBuiltEncoder::encode_feature(&mut data, &feature, 0).is_err());
        assert!(AsBuiltEncoder::encode_feature(&mut data, &feature, 6).is_err());
    }
}

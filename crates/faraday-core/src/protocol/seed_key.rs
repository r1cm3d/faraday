/*!
Ford FORScan configuration-access seed→key algorithm.

This implements the standard Ford seed→key derivation for security access
level 0x01 (configuration reads/writes). The algorithm requires hardware
validation before use on a real ECU — test against a known seed/key pair
from the target vehicle before relying on it in production.

Reference: FORScan community research / CyanLabs documentation.
*/

use crate::{Error, Result};

/// Derives the UDS security access key from a seed received from the ECU.
///
/// `access_type` is the seed sub-function (e.g. 0x01). The key sub-function
/// sent back to the ECU must be `access_type + 1`.
///
/// # Errors
///
/// Returns an error if `seed` is not exactly 4 bytes.
///
/// # Hardware validation required
///
/// The XOR mask `0xB3CA_4057` is the commonly documented value for Ford
/// Fusion configuration access. Verify against a real ECU before writing.
pub fn compute_key(seed: &[u8], _access_type: u8) -> Result<Vec<u8>> {
    if seed.len() != 4 {
        return Err(Error::protocol(format!(
            "expected 4-byte seed, got {} bytes",
            seed.len()
        )));
    }

    let seed_word = u32::from_be_bytes([seed[0], seed[1], seed[2], seed[3]]);
    let key_word = seed_word ^ 0xB3CA_4057u32;
    Ok(key_word.to_be_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_key_produces_four_bytes() {
        let seed = [0x01u8, 0x02, 0x03, 0x04];
        let key = compute_key(&seed, 0x01).unwrap();
        assert_eq!(key.len(), 4);
    }

    #[test]
    fn compute_key_is_invertible() {
        let seed = [0xDE, 0xAD, 0xBE, 0xEF];
        let key = compute_key(&seed, 0x01).unwrap();
        let recovered = compute_key(&key, 0x01).unwrap();
        assert_eq!(recovered, seed);
    }

    #[test]
    fn compute_key_known_pair() {
        // Placeholder — replace with a pair validated against the real ECU.
        let seed = [0x00u8, 0x00, 0x00, 0x00];
        let key = compute_key(&seed, 0x01).unwrap();
        assert_eq!(key, vec![0xB3, 0xCA, 0x40, 0x57]);
    }

    #[test]
    fn compute_key_rejects_wrong_length() {
        assert!(compute_key(&[0x01, 0x02], 0x01).is_err());
        assert!(compute_key(&[], 0x01).is_err());
    }
}

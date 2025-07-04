use crate::HmacSha256;
use derive_jserror::JsError;
use hmac::Mac;
use thiserror::Error;
use wasm_bindgen::prelude::*;

const HMAC_KEY_VERSION_0A: &[u8; 32] = include_bytes!("v3/switch_hmac_key_version_0A.bin");
const HMAC_KEY_VERSION_0B: &[u8; 32] = include_bytes!("v3/switch_hmac_key_version_0B.bin");
const HMAC_KEY_VERSION_0C: &[u8; 32] = include_bytes!("v3/switch_hmac_key_version_0C.bin");
const HMAC_KEY_VERSION_0D: &[u8; 32] = include_bytes!("v3/switch_hmac_key_version_0D.bin");

#[derive(Error, JsError, Debug)]
#[allow(missing_docs)]
pub enum V3Error {
    #[error("The inquiry number has encoded an unknown version: {0}")]
    UnknownVersion(u8),
}

/// Calculate the master key for the parental control using the v3 algorithm. The inquire number
/// cannot be bigger than 10 digits and the date must be valid (there are some loose checks).
///
/// Remember that the given master key must be presented with the correct amount of leading zeroes
/// to always have 8 digits.
///
/// Only works on Switch (from 1.0.0 to 7.0.1).
#[wasm_bindgen]
pub fn calculate_v3_master_key(inquiry_number: u64, day: u8, month: u8) -> Result<u64, V3Error> {
    assert!(inquiry_number <= 9_999_999_999);

    assert!(day > 0);
    assert!(day <= 31);

    assert!(month > 0);
    assert!(month <= 12);

    let version = (inquiry_number / 100_000_000) % 100;

    let hmac_key = match version {
        0x0A => HMAC_KEY_VERSION_0A,
        0x0B => HMAC_KEY_VERSION_0B,
        0x0C => HMAC_KEY_VERSION_0C,
        0x0D => HMAC_KEY_VERSION_0D,

        _ => return Err(V3Error::UnknownVersion(version as u8)),
    };

    let input = format!("{inquiry_number:0>10}");

    #[allow(clippy::expect_used)]
    let mut hmac = HmacSha256::new_from_slice(hmac_key).expect("Invalid lenght of the key");

    hmac.update(input.as_bytes());

    #[allow(clippy::expect_used)]
    let hash: [u8; 8] = hmac.finalize().into_bytes()[0..8]
        .try_into()
        .expect("The HMAC hash is always long enough");

    let output = u64::from_le_bytes(hash) & 0x0000FFFFFFFFFFFF;

    Ok(output % 100000000)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: u8 = 5;
    const MONTH: u8 = 8;

    #[test]
    fn version_0a() {
        assert_eq!(
            calculate_v3_master_key(1034567890, DAY, MONTH).unwrap(),
            3593035
        );
    }

    #[test]
    fn version_0b() {
        assert_eq!(
            calculate_v3_master_key(1134567890, DAY, MONTH).unwrap(),
            97972487
        );
    }

    #[test]
    fn version_0c() {
        assert_eq!(
            calculate_v3_master_key(1234567890, DAY, MONTH).unwrap(),
            99348932
        );
    }

    #[test]
    fn version_0d() {
        assert_eq!(
            calculate_v3_master_key(1334567890, DAY, MONTH).unwrap(),
            99964632
        );
    }
}

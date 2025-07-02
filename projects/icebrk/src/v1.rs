use hmac::{Hmac, Mac};
use sha2::Sha256;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

const HMAC_KEY_REGION_0: &[u8; 32] = include_bytes!("v1/3ds_hmac_key_0.bin");
const HMAC_KEY_REGION_1: &[u8; 32] = include_bytes!("v1/3ds_hmac_key_1.bin");
const HMAC_KEY_REGION_2: &[u8; 32] = include_bytes!("v1/3ds_hmac_key_2.bin");

#[derive(Error, Debug)]
pub enum V1Error {
    #[error("The inquiry number has encoded an unknown region: {0}")]
    UnknownRegion(u8),
}

/// TODO: DOCS: Include "only on 3DS"
pub fn calculate_v1_master_key(inquiry_number: u64, day: u8, month: u8) -> Result<u32, V1Error> {
    let region = inquiry_number / 1000000000;

    let hmac_key = match region {
        0 => HMAC_KEY_REGION_0,
        1 => HMAC_KEY_REGION_1,
        2 => HMAC_KEY_REGION_2,

        _ => return Err(V1Error::UnknownRegion(region as u8)),
    };

    // The month and day with a leading zero when the number is not two digits long
    // and the inquiry number (also padded with zeroes)
    let input = format!("{month:0>2}{day:0>2}{:0>10}", inquiry_number);

    #[allow(clippy::expect_used)]
    let mut hmac = HmacSha256::new_from_slice(hmac_key).expect("Invalid lenght of the key");

    hmac.update(input.as_bytes());

    #[allow(clippy::expect_used)]
    let hash: [u8; 4] = hmac.finalize().into_bytes()[0..4]
        .try_into()
        .expect("The HMAC hash is always long enough");

    Ok(u32::from_le_bytes(hash) % 100000)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: u8 = 5;
    const MONTH: u8 = 8;

    #[test]
    fn region_0() {
        assert_eq!(
            calculate_v1_master_key(123456789, DAY, MONTH).unwrap(),
            3741
        );
    }

    #[test]
    fn region_1() {
        assert_eq!(
            calculate_v1_master_key(1123456789, DAY, MONTH).unwrap(),
            93328
        );
    }

    #[test]
    fn region_2() {
        assert_eq!(
            calculate_v1_master_key(2123456789, DAY, MONTH).unwrap(),
            10129
        );
    }
}

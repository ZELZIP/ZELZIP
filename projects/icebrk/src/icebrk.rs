// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Implementation of the different algorithms used on Nintendo consoles to generate the parental control master key.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use wasm_bindgen::prelude::*;

type HmacSha256 = Hmac<Sha256>;

mod v0;
mod v1;
mod v2;
mod v3;

/// Generic enum for a few platforms by Nintendo.
#[wasm_bindgen]
#[derive(PartialEq)]
pub enum Platform {
    /// The Nintendo Wii platform.
    Wii,

    /// The Nintendo DSi platform.
    Dsi,

    /// The Nintendo 3DS platform.
    The3ds,

    /// The Nintendo Wii U platform.
    WiiU,

    /// The Nintendo Switch platform
    Switch,
}

pub use v0::calculate_v0_master_key;
pub use v1::{V1Error, calculate_v1_master_key};
pub use v2::{V2Error, calculate_v2_master_key};
pub use v3::{V3Error, calculate_v3_master_key};

fn calculate_master_key_shared_v1_and_v2(
    hmac_key: &[u8; 32],
    inquiry_number: u64,
    day: u8,
    month: u8,
    big_endian: bool,
) -> u32 {
    // The month and day with a leading zero when the number is not two digits long
    // and the inquiry number (also padded with zeroes)
    let input = format!("{month:0>2}{day:0>2}{inquiry_number:0>10}");

    #[allow(clippy::expect_used)]
    let mut hmac = HmacSha256::new_from_slice(hmac_key).expect("Invalid lenght of the key");

    hmac.update(input.as_bytes());

    #[allow(clippy::expect_used)]
    let hash: [u8; 4] = hmac.finalize().into_bytes()[0..4]
        .try_into()
        .expect("The HMAC hash is always long enough");

    let output = if big_endian {
        u32::from_be_bytes(hash)
    } else {
        u32::from_le_bytes(hash)
    };

    output % 100000
}

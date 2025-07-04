// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use crate::Platform;
use wasm_bindgen::prelude::*;

const CRC_INIT_VALUE: u32 = 0xFFFFFFFF;
const CRC_XOROUT: u32 = 0xAAAA;

const fn build_crc_algorithm(polynomial: u32) -> crc::Algorithm<u32> {
    crc::Algorithm::<u32> {
        width: 32,
        poly: polynomial,
        init: CRC_INIT_VALUE,
        refin: true,
        refout: true,
        xorout: CRC_XOROUT,

        // These values are only relevant for the
        // [CRC RevEng catalog](https://reveng.sourceforge.io/crc-catalogue/all.htm)
        check: 0,
        residue: 0,
    }
}

const CRC_POLYNOMIAL_WII_AND_DSI: u32 = 0x04C11DB7;
const CRC_POLYNOMIAL_WIIU_AND_3DS: u32 = 0x04C65DB7;

const CRC_ALGORITHM_WII_AND_DSI: crc::Algorithm<u32> =
    build_crc_algorithm(CRC_POLYNOMIAL_WII_AND_DSI);

const CRC_ALGORITHM_WIIU_AND_3DS: crc::Algorithm<u32> =
    build_crc_algorithm(CRC_POLYNOMIAL_WIIU_AND_3DS);

const CRC_ADDOUT_WII_AND_DSI: u32 = 0x14C1;
const CRC_ADDOUT_WIIU_AND_3DS: u32 = 0x1657;

const fn get_crc(platform: Platform) -> (&'static crc::Algorithm<u32>, u32) {
    match platform {
        Platform::Wii | Platform::Dsi => (&CRC_ALGORITHM_WII_AND_DSI, CRC_ADDOUT_WII_AND_DSI),
        Platform::WiiU | Platform::The3ds => (&CRC_ALGORITHM_WIIU_AND_3DS, CRC_ADDOUT_WIIU_AND_3DS),

        Platform::Switch => panic!(
            "The version 0 of the parental control master key algorithm is not available on the Nintendo Switch Platform"
        ),
    }
}

/// Calculate the master key for the parental control using the v0 algorithm. The inquire number
/// cannot be bigger than 8 digits and the date must be valid (there are some loose checks).
///
/// Remember that the given master key must be presented with the correct amount of leading zeroes
/// to always have 5 digits.
///
/// Only works on Wii, DSi, 3DS (from 1.0.0 to 6.3.0) and Wii U (from 1.0.0 to 4.1.0).
#[wasm_bindgen]
pub fn calculate_v0_master_key(platform: Platform, inquiry_number: u32, day: u8, month: u8) -> u32 {
    assert!(inquiry_number <= 99_999_999);

    assert!(day > 0);
    assert!(day <= 31);

    assert!(month > 0);
    assert!(month <= 12);

    let (algorithm, addout) = get_crc(platform);

    // The month and day with a leading zero when the number is not two digits long
    // and the last four digits of the inquiry number (also padded with zeroes)
    let input = format!("{month:0>2}{day:0>2}{:0>4}", inquiry_number % 10000);

    let crc = crc::Crc::<u32>::new(algorithm);
    let checksum = (crc.checksum(input.as_bytes())) + addout;

    checksum % 100000
}

#[cfg(test)]
mod tests {
    use super::*;

    const INQUIRY_NUMBER: u32 = 84293062;
    const DAY: u8 = 5;
    const MONTH: u8 = 8;

    #[test]
    fn wii_platform() {
        assert_eq!(
            calculate_v0_master_key(Platform::Wii, INQUIRY_NUMBER, DAY, MONTH),
            66150
        );
    }

    #[test]
    fn dsi_platform() {
        assert_eq!(
            calculate_v0_master_key(Platform::Dsi, INQUIRY_NUMBER, DAY, MONTH),
            66150
        );
    }

    #[test]
    fn wiiu_platform() {
        assert_eq!(
            calculate_v0_master_key(Platform::WiiU, INQUIRY_NUMBER, DAY, MONTH),
            87902
        );
    }

    #[test]
    fn the_3ds_platform() {
        assert_eq!(
            calculate_v0_master_key(Platform::The3ds, INQUIRY_NUMBER, DAY, MONTH),
            87902
        );
    }
}

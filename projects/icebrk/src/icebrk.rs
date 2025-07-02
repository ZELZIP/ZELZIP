// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Implementation of the different algorithms used on Nintendo consoles to generate the parental control master key.

use wasm_bindgen::prelude::*;

mod v0;
mod v1;

/// Generic enum for a few platforms by Nintendo.
#[wasm_bindgen]
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
pub use v1::calculate_v1_master_key;

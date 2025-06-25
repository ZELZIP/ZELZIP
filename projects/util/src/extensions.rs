// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

#[cfg(feature = "std")]
mod read;

#[cfg(feature = "std")]
mod write;

#[cfg(feature = "alloc")]
mod string;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "std")] {
        pub use read::ReadEx;
        pub use write::WriteEx;
    }
}

#[cfg(feature = "alloc")]
pub use string::StringEx;

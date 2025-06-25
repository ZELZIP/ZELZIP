// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Crate to parse binary formats used on the
//! [Nintendo](https://en.wikipedia.org/wiki/Nintendo) [Wii](https://en.wikipedia.org/wiki/Wii), [DSi](https://en.wikipedia.org/wiki/Nintendo_DSi), [3DS family](https://en.wikipedia.org/wiki/Nintendo_3DS) and [Wii U](https://en.wikipedia.org/wiki/Wii_U) consoles and
//! [NUS (Nintendo Update Server)](https://wiibrew.org/wiki/NUS) and [iQue](https://en.wikipedia.org/wiki/IQue) platforms.

pub mod certificate_chain;
pub mod signed_blob_header;
pub mod ticket;
pub mod title_id;
pub mod title_metadata;
pub mod wad;
pub mod wii_common_key;

pub use certificate_chain::CertificateChain;
pub use ticket::{CryptographicMethod, PreSwitchTicket};
pub use title_metadata::{
    TitleMetadata, TitleMetadataContentEntryKind, content_selector::ContentSelector,
};
pub use wad::Wad;

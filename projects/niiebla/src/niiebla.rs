//! Crate to parse binary formats used on the
//! [Nintendo](https://en.wikipedia.org/wiki/Nintendo) [Wii](https://en.wikipedia.org/wiki/Wii), [DSi](https://en.wikipedia.org/wiki/Nintendo_DSi), [3DS family](https://en.wikipedia.org/wiki/Nintendo_3DS) and [Wii U](https://en.wikipedia.org/wiki/Wii_U) consoles and
//! [NUS (Nintendo Update Server)](https://wiibrew.org/wiki/NUS) and [iQUe](https://en.wikipedia.org/wiki/IQue) platforms.

mod signed_blob_header;
pub use signed_blob_header::{SignedBlobHeader, SignedBlobHeaderSignature};

mod wad;
pub use wad::Wad;

mod ticket;
pub use ticket::*;

mod title_metadata;
pub use title_metadata::{
    TitleMetadata, TitleMetadataContentEntry, TitleMetadataContentEntryKind, TitleMetadataRegion,
};

mod wii_common_key;
pub use wii_common_key::WiiCommonKeyKind;

mod title_id;

mod certificate_chain;
pub use certificate_chain::{Certificate, CertificateChain, CertificateKey};

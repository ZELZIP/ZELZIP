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

pub use wad::Wad;

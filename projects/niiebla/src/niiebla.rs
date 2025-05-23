pub mod certificate_chain;
pub mod common_key;
pub mod ticket;
pub mod wad;

use std::string::FromUtf8Error;

pub(crate) fn align_offset(value: u64, boundary: u64) -> u64 {
    value + (boundary - (value % boundary)) % boundary
}

pub(crate) fn string_from_null_terminated_bytes(bytes: &[u8]) -> Result<String, FromUtf8Error> {
    let string_end = bytes
        .iter()
        .position(|&char| char == b'\0')
        // Fallback to use all the bytes
        .unwrap_or(bytes.len());

    String::from_utf8(bytes[0..string_end].to_vec())
}

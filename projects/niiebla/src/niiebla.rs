pub mod aes;
pub mod certificate_chain;
pub mod common_key;
pub mod signed_blob_header;
pub mod ticket;
pub mod title_id;
pub mod title_metadata;
pub mod wad;

use std::io;
use std::io::Write;
use std::string::FromUtf8Error;

pub(crate) fn align_to_boundary(value: u64, boundary: u64) -> u64 {
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

pub trait Dump {
    fn dump<W: Write>(&self, writer: &mut W) -> io::Result<()>;
}

pub trait WriteEx: io::Write {
    fn write_zeroed(&mut self, number_of_zeroes: usize) -> io::Result<()> {
        self.write_all(&vec![0; number_of_zeroes])?;

        Ok(())
    }
}

impl<W: Write> WriteEx for W {}

pub mod aes;
pub mod certificate_chain;
pub mod common_key;
pub mod signed_blob_header;
pub mod ticket;
pub mod title_id;
pub mod title_metadata;
pub mod wad;

use byteorder::WriteBytesExt;
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

pub(crate) trait WriteEx: io::Write {
    fn write_zeroed(&mut self, number_of_zeroes: usize) -> io::Result<()> {
        self.write_all(&vec![0; number_of_zeroes])?;

        Ok(())
    }

    fn write_as_c_string(&mut self, string: &str) -> io::Result<()> {
        self.write_all(string.as_bytes())?;
        self.write_all(&[0])?;

        Ok(())
    }

    fn write_as_c_string_padded(&mut self, string: &str, padding: usize) -> io::Result<()> {
        assert!(string.len() < padding);

        self.write_as_c_string(string)?;
        self.write_zeroed(padding - string.len() - 1)?;

        Ok(())
    }

    fn write_bool(&mut self, value: bool) -> io::Result<()> {
        self.write_u8(value as u8)?;

        Ok(())
    }
}

impl<W: Write> WriteEx for W {}

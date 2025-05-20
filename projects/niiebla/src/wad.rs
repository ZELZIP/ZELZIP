pub mod installable;

use crate::wad::installable::{InstallableWad, InstallableWadError};
use std::io;
use std::io::Read;
use std::io::Seek;
use thiserror::Error;

const INSTALLABLE_WAD_MAGIC_NUMBERS: [u8; 8] = [0x00, 0x00, 0x00, 0x20, 0x49, 0x73, 0x00, 0x00];

#[derive(Debug)]
pub enum Wad {
    Installable { installable: InstallableWad },
    // TODO(IMPLEMENT) Support for backup wads
}

#[derive(Error, Debug)]
pub enum WadError {
    #[error("An IO error has occurred: {0}")]
    IoError(#[from] io::Error),

    #[error("An error has occurred while parsing an installable Wad: {0}")]
    InstallableWadError(#[from] InstallableWadError),

    #[error("Unknown WAD format")]
    UnknownWadFormatError,
}

impl Wad {
    pub fn from_reader<T: Read + Seek>(buffer: &mut T) -> Result<Wad, WadError> {
        let mut magic_numbers_buffer = [0; 8];
        buffer.read_exact(&mut magic_numbers_buffer)?;

        // Keep the cursor in place for the read file parsing
        buffer.rewind()?;

        match magic_numbers_buffer {
            INSTALLABLE_WAD_MAGIC_NUMBERS => Ok(Wad::Installable {
                installable: unsafe { InstallableWad::from_reader(buffer)? },
            }),

            _ => Err(WadError::UnknownWadFormatError),
        }
    }
}

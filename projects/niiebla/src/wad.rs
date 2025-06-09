//! Implementation of the binary file format used by Nintendo to store titles without discs.

pub mod installable;

use crate::wad::installable::{InstallableWad, InstallableWadError};
use std::io;
use std::io::Read;
use std::io::Seek;
use thiserror::Error;

const INSTALLABLE_WAD_MAGIC_NUMBERS: [u8; 8] = [0x00, 0x00, 0x00, 0x20, 0x49, 0x73, 0x00, 0x00];

/// Represent the different kinds of WAD files that are known to have been used on the Nintendo
/// Wii.
#[derive(Debug)]
pub enum Wad {
    /// WAD that stores the data needed to install a title into the system.
    Installable(InstallableWad),

    // TODO(IMPLEMENT) Support for backup wads
    //   - Remember to also add a `try_backup` function
    /// Kind of WAD that was used to store encrypted data safely into the SD card, used to store
    /// channels and downloadable content (DLCs).
    BackUp,
}

#[derive(Error, Debug)]
pub enum WadError {
    #[error("An IO error has occurred: {0}")]
    IoError(#[from] io::Error),

    #[error("An error has occurred while parsing an installable Wad: {0}")]
    InstallableWadParseError(#[from] InstallableWadError),

    #[error("Unknown WAD format")]
    UnknownWadFormatError,

    #[error("The found WAD format was not the wanted one")]
    UndesiredWadFormat,
}

impl Wad {
    /// Create a new [Wad] by parsing a stream.
    pub fn new<T: Read + Seek>(stream: &mut T) -> Result<Self, WadError> {
        let mut magic_numbers_buffer = [0; 8];
        stream.read_exact(&mut magic_numbers_buffer)?;

        // Keep the cursor in the correct place for the file parsing
        stream.rewind()?;

        match magic_numbers_buffer {
            INSTALLABLE_WAD_MAGIC_NUMBERS => {
                Ok(Self::Installable(unsafe { InstallableWad::new(stream)? }))
            }

            _ => Err(WadError::UnknownWadFormatError),
        }
    }

    /// Like [Self::new] but treats any format of WAD except the Installable ones as an
    /// error.
    pub fn try_new_installable<T: Read + Seek>(stream: &mut T) -> Result<InstallableWad, WadError> {
        match Self::new(stream)? {
            Self::Installable(installable_wad) => Ok(installable_wad),

            _ => Err(WadError::UndesiredWadFormat),
        }
    }
}

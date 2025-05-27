pub mod installable;

use crate::wad::installable::{InstallableWad, InstallableWadError};
use std::io;
use std::io::Read;
use std::io::Seek;
use thiserror::Error;

const INSTALLABLE_WAD_MAGIC_NUMBERS: [u8; 8] = [0x00, 0x00, 0x00, 0x20, 0x49, 0x73, 0x00, 0x00];

#[derive(Debug)]
pub enum Wad {
    Installable(InstallableWad),
    // TODO(IMPLEMENT) Support for backup wads
    //   - Remember to also add a `try_backup` function
    BackUp,
}

#[derive(Error, Debug)]
pub enum WadError {
    #[error("An IO error has occurred: {0}")]
    IoError(#[from] io::Error),

    #[error("An error has occurred while parsing an installable Wad: {0}")]
    InstallableWadError(#[from] InstallableWadError),

    #[error("Unknown WAD format")]
    UnknownWadFormatError,

    #[error("The found WAD format was not the wanted one")]
    UndesiredWadFormat,
}

impl Wad {
    pub fn from_reader<T: Read + Seek>(reader: &mut T) -> Result<Wad, WadError> {
        let mut magic_numbers_buffer = [0; 8];
        reader.read_exact(&mut magic_numbers_buffer)?;

        // Keep the cursor in the correct place for the file parsing
        reader.rewind()?;

        match magic_numbers_buffer {
            INSTALLABLE_WAD_MAGIC_NUMBERS => Ok(Wad::Installable(unsafe {
                InstallableWad::from_reader(reader)?
            })),

            _ => Err(WadError::UnknownWadFormatError),
        }
    }

    /// Like [Self::from_reader] but treats any format of WAD except the Installable ones as an
    /// error.
    pub fn try_new_installable<T: Read + Seek>(reader: &mut T) -> Result<InstallableWad, WadError> {
        match Wad::from_reader(reader)? {
            Wad::Installable(installable_wad) => Ok(installable_wad),

            _ => Err(WadError::UndesiredWadFormat),
        }
    }
}

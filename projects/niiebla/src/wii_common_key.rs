use byteorder::WriteBytesExt;
use std::io;
use std::io::Write;
use thiserror::Error;

/// Kinds of encryption keys used on the Nintendo Wii.
#[derive(Debug)]
pub enum WiiCommonKeyKind {
    /// Key used in most retail consoles.
    Normal,

    /// Key used on consoles with Korea set as its internal region (KOR).
    Korean,

    /// Key used on the virtual Wii console (vWii) inside the Nintendo Wii U.
    WiiUvWii,
}

#[derive(Error, Debug)]
pub enum CommonKeyKindError {
    #[error("Unknown common key index: {0}")]
    UnknownCommonKeyIndex(u8),
}

impl WiiCommonKeyKind {
    /// Get a common key given its "common key index" (identifier).
    pub const fn new(identifier: u8) -> Result<Self, CommonKeyKindError> {
        Ok(match identifier {
            0 => Self::Normal,
            1 => Self::Korean,
            2 => Self::WiiUvWii,

            identifier => return Err(CommonKeyKindError::UnknownCommonKeyIndex(identifier)),
        })
    }

    /// Get the identifier associated with the given common key.
    pub fn dump_identifier<T: Write>(&self, writer: &mut T) -> io::Result<()> {
        writer.write_u8(match self {
            Self::Normal => 0,
            Self::Korean => 1,
            Self::WiiUvWii => 2,
        })?;

        Ok(())
    }

    /// Get the bytes of the correct kind of common key.
    pub const fn bytes(&self) -> [u8; 16] {
        match self {
            Self::Normal => [
                0xeb, 0xe4, 0x2a, 0x22, 0x5e, 0x85, 0x93, 0xe4, 0x48, 0xd9, 0xc5, 0x45, 0x73, 0x81,
                0xaa, 0xf7,
            ],
            Self::Korean => [
                0x63, 0xb8, 0x2b, 0xb4, 0xf4, 0x61, 0x4e, 0x2e, 0x13, 0xf2, 0xfe, 0xfb, 0xba, 0x4c,
                0x9b, 0x7e,
            ],
            Self::WiiUvWii => [
                0x30, 0xbf, 0xc7, 0x6e, 0x7c, 0x19, 0xaf, 0xbb, 0x23, 0x16, 0x33, 0x30, 0xce, 0xd7,
                0xc2, 0x8d,
            ],
        }
    }
}

use byteorder::{BigEndian, ReadBytesExt};
use std::io::{self, Read, Seek};
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Debug)]
pub struct SignedBlobHeader {
    pub signature_kind: SignedBlobHeaderSignatureKind,
    pub signature: [u8; 256],
}

#[derive(Error, Debug)]
pub enum SignedBlobHeaderError {
    #[error("An IO error has occurred: {0}")]
    IoError(#[from] io::Error),

    #[error("Converting into UTF-8 failed: {0}")]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("Unknown signature kind: {0:#X}")]
    UnknownSignatureKind(u32),
}

impl SignedBlobHeader {
    /// Create a new signed blob header.
    ///
    /// # Safety
    /// The given buffer is assumed to be from an signed blob header,
    /// the current position of the Seek pointer is taken as the start.
    pub unsafe fn from_reader<T: Read + Seek>(
        reader: &mut T,
    ) -> Result<SignedBlobHeader, SignedBlobHeaderError> {
        let signature_kind =
            SignedBlobHeaderSignatureKind::from_identifier(reader.read_u32::<BigEndian>()?)?;

        let mut signature = [0; 256];
        reader.read_exact(&mut signature)?;

        // Skip 60 bytes of padding
        reader.seek_relative(60)?;

        Ok(SignedBlobHeader {
            signature_kind,
            signature,
        })
    }
}

#[derive(Debug)]
pub enum SignedBlobHeaderSignatureKind {
    Rsa2048,
}

impl SignedBlobHeaderSignatureKind {
    fn from_identifier(
        identifier: u32,
    ) -> Result<SignedBlobHeaderSignatureKind, SignedBlobHeaderError> {
        const SIGNATURE_KIND_IDENTIFIER_RSA_2048: u32 = 0x10001;

        Ok(match identifier {
            SIGNATURE_KIND_IDENTIFIER_RSA_2048 => SignedBlobHeaderSignatureKind::Rsa2048,

            bytes => return Err(SignedBlobHeaderError::UnknownSignatureKind(bytes)),
        })
    }
}

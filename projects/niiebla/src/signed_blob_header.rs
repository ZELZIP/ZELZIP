use crate::WriteEx;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Seek};
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Debug)]
pub enum SignedBlobHeaderSignatureKind {
    Rsa2048,
}

impl SignedBlobHeaderSignatureKind {
    const SIGNATURE_KIND_IDENTIFIER_RSA_2048: u32 = 0x10001;

    fn from_identifier(
        identifier: u32,
    ) -> Result<SignedBlobHeaderSignatureKind, SignedBlobHeaderError> {
        Ok(match identifier {
            SignedBlobHeaderSignatureKind::SIGNATURE_KIND_IDENTIFIER_RSA_2048 => {
                SignedBlobHeaderSignatureKind::Rsa2048
            }

            bytes => return Err(SignedBlobHeaderError::UnknownSignatureKind(bytes)),
        })
    }

    fn dump<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u32::<BigEndian>(match self {
            SignedBlobHeaderSignatureKind::Rsa2048 => {
                SignedBlobHeaderSignatureKind::SIGNATURE_KIND_IDENTIFIER_RSA_2048
            }
        })?;

        Ok(())
    }
}

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

    pub fn dump<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        self.signature_kind.dump(writer)?;
        writer.write_all(&self.signature)?;
        writer.write_zeroed(60)?;

        Ok(())
    }
}

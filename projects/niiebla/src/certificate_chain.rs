//! Implementation of the binary file format used by Nintendo to store certificate chains.

use crate::signed_blob_header::{SignedBlobHeader, SignedBlobHeaderError};
use byteorder::{BE, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Seek, Write};
use std::string::FromUtf8Error;
use thiserror::Error;
use util::StreamPin;
use util::WriteEx;

#[derive(Debug)]
/// A set of certificates.
pub struct CertificateChain {
    /// The set of cetificates.
    pub certificates: Vec<Certificate>,
}

impl CertificateChain {
    /// Create a new certificate chain.
    pub fn new<T: Read + Seek>(
        stream: T,
        number_of_certificates: usize,
    ) -> Result<Self, CertificateChainError> {
        let mut stream = StreamPin::new(stream)?;
        let mut certificates = Vec::new();

        for _ in 0..number_of_certificates {
            certificates.push(Certificate::new(&mut stream)?);
            stream.align_position(64)?;
        }

        Ok(Self { certificates })
    }

    /// Dump the certificate chain into a stream.
    pub fn dump<T: Write + Seek>(&self, stream: T) -> io::Result<()> {
        let mut stream = StreamPin::new(stream)?;

        for certificate in &self.certificates {
            certificate.dump(&mut stream)?;
            stream.align_zeroed(64)?;
        }

        Ok(())
    }

    /// Get the sizes of the certificate chain in bytes.
    pub fn size(&self) -> u32 {
        self.certificates
            .iter()
            .fold(0, |accumulator, current| accumulator + current.size())
    }
}

#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum CertificateChainError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Unknown signature kind: {0:#X}")]
    UnknownSignatureKind(u32),

    #[error("Unknown key kind: {0:#X}")]
    UnknownKeyKind(u32),

    #[error("Converting into UTF-8 failed: {0}")]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("Unable to parse the signed blob header: {0}")]
    SignedBlobHeaderError(#[from] SignedBlobHeaderError),
}

#[derive(Debug, Clone)]
/// A single certificate.
pub struct Certificate {
    /// Header with data to prove the authenticity that this data
    /// has being created by an authorized entity.
    pub signed_blob_header: SignedBlobHeader,

    /// The name of the certificate.
    pub identity: String,

    /// The key stored inside the certificate.
    pub key: CertificateKey,
}

impl Certificate {
    /// Create a new certificate.
    pub fn new<T: Read + Seek>(mut stream: T) -> Result<Self, CertificateChainError> {
        let signed_blob_header = SignedBlobHeader::new(&mut stream)?;

        let key_value_kind_identifier = stream.read_u32::<BE>()?;

        let identity = util::read_string!(stream, 64)?;

        let key = CertificateKey {
            id: stream.read_u32::<BE>()?,
            value: CertificateKeyValue::new(key_value_kind_identifier, &mut stream)?,
        };

        Ok(Self {
            signed_blob_header,
            identity,
            key,
        })
    }

    /// Dump the certificate chain into a stream.
    pub fn dump<T: Write + Seek>(&self, mut stream: T) -> io::Result<()> {
        self.signed_blob_header.dump(&mut stream)?;

        self.key.value.dump_kind_identifier(&mut stream)?;
        stream.write_bytes_padded(self.identity.as_bytes(), 64)?;
        stream.write_u32::<BE>(self.key.id)?;
        self.key.value.dump_value(&mut stream)?;

        Ok(())
    }

    /// Get the sizes of the certificate in bytes.
    pub fn size(&self) -> u32 {
        let size = match self.key.value {
            CertificateKeyValue::Rsa4096(_) => 512,
            CertificateKeyValue::Rsa2048(_) => 256,
            CertificateKeyValue::EccB223(_) => 60,
        } + self.signed_blob_header.size()
            + 72;

        util::align_to_boundary(size as u64, 64) as u32
    }
}

#[derive(Debug, Clone)]
/// The public key stored inside a certificate.
pub struct CertificateKey {
    /// The ID of the certificate.
    pub id: u32,

    /// The public key data itself.
    pub value: CertificateKeyValue,
}

/// The public key data stored inside a certificate.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum CertificateKeyValue {
    /// The key is stored as RSA-4096 data.
    Rsa4096(Box<[u8; 512 + 4]>),

    /// The key is stored as RSA-2048 data.
    Rsa2048(Box<[u8; 256 + 4]>),

    /// The key is stored as ECCB223 data.
    EccB223(Box<[u8; 60]>),
}

impl CertificateKeyValue {
    fn new<T: Read + Seek>(identifier: u32, mut stream: T) -> Result<Self, CertificateChainError> {
        let public_key = match identifier {
            0 => {
                let buf = util::read_exact!(stream, 512 + 4)?;
                Self::Rsa4096(Box::new(buf))
            }
            1 => {
                let buf = util::read_exact!(stream, 256 + 4)?;
                Self::Rsa2048(Box::new(buf))
            }
            2 => {
                let buf = util::read_exact!(stream, 60)?;
                Self::EccB223(Box::new(buf))
            }

            _ => return Err(CertificateChainError::UnknownKeyKind(identifier)),
        };

        Ok(public_key)
    }

    fn dump_kind_identifier<T: Write>(&self, mut stream: T) -> io::Result<()> {
        stream.write_u32::<BE>(match self {
            Self::Rsa4096(_) => 0,
            Self::Rsa2048(_) => 1,
            Self::EccB223(_) => 2,
        })?;

        Ok(())
    }

    fn dump_value<T: Write>(&self, mut stream: T) -> io::Result<()> {
        match self {
            Self::Rsa4096(value) => stream.write_all(value.as_slice())?,
            Self::Rsa2048(value) => stream.write_all(value.as_slice())?,
            Self::EccB223(value) => stream.write_all(value.as_slice())?,
        }

        Ok(())
    }
}

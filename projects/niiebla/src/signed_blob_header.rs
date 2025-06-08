use byteorder::{BE, ReadBytesExt, WriteBytesExt};
use std::boxed::Box;
use std::io::{self, Read, Seek, Write};
use std::string::{FromUtf8Error, String};
use thiserror::Error;
use util::{StreamPin, WriteEx};

/// Blob placed at the start of some binary data to denote the entity that issued them.
#[derive(Debug, Clone)]
pub struct SignedBlobHeader {
    /// Signature of the blob.
    pub signature: SignedBlobHeaderSignature,

    /// Issuer of the signature.
    pub issuer: String,
}

impl SignedBlobHeader {
    /// Create a new [SignedBlobHeader] by parsing an stream.
    pub fn new<T: Read + Seek>(stream: &mut T) -> Result<Self, SignedBlobHeaderError> {
        let mut stream = StreamPin::new(stream)?;

        let signature = SignedBlobHeaderSignature::new(&mut stream)?;
        stream.align_position(64)?;

        let issuer = util::read_string!(stream, 64)?;

        Ok(Self { signature, issuer })
    }

    /// Dump the signed blob header..
    pub fn dump<T: Write + Seek>(&self, stream: &mut T) -> io::Result<()> {
        let mut stream = StreamPin::new(stream)?;

        self.signature.dump(&mut stream)?;
        stream.align_zeroed(64)?;
        stream.write_bytes_padded(self.issuer.as_bytes(), 64)?;

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum SignedBlobHeaderError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Unknown signature kind: {0:#X}")]
    UnknownSignatureKind(u32),

    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] FromUtf8Error),
}

/// Signature in different cryptography formats.
#[derive(Debug, Clone)]
pub enum SignedBlobHeaderSignature {
    /// RSA-4096 PKCS#1 v1.5 with SHA-1.
    Rsa4096Sha1(Box<[u8; 512]>),

    /// RSA-2048 PKCS#1 v1.5 with SHA-1.
    Rsa2048Sha1(Box<[u8; 256]>),

    /// ECDSA with SHA-1.
    EcdsaSha1(Box<[u8; 60]>),

    /// RSA-4096 PKCS#1 v1.5 with SHA-256.
    Rsa4096Sha256(Box<[u8; 512]>),

    /// RSA-2048 PKCS#1 v1.5 with SHA-256.
    Rsa2048Sha256(Box<[u8; 256]>),

    /// ECDSA with SHA-256.
    EcdsaSha256(Box<[u8; 60]>),

    /// HMAC-SHA1-160
    HmacSha1(Box<[u8; 20]>),
}

impl SignedBlobHeaderSignature {
    fn new<T: Read>(stream: &mut T) -> Result<Self, SignedBlobHeaderError> {
        Ok(match stream.read_u32::<BE>()? {
            0x010000 => {
                let buf = util::read_exact!(stream, 512)?;
                Self::Rsa4096Sha1(Box::new(buf))
            }

            0x010001 => {
                let buf = util::read_exact!(stream, 256)?;
                Self::Rsa2048Sha1(Box::new(buf))
            }

            0x010002 => {
                let buf = util::read_exact!(stream, 60)?;
                Self::EcdsaSha1(Box::new(buf))
            }

            0x010003 => {
                let buf = util::read_exact!(stream, 512)?;
                Self::Rsa4096Sha256(Box::new(buf))
            }

            0x010004 => {
                let buf = util::read_exact!(stream, 256)?;
                Self::Rsa2048Sha256(Box::new(buf))
            }

            0x010005 => {
                let buf = util::read_exact!(stream, 60)?;
                Self::EcdsaSha256(Box::new(buf))
            }

            0x010006 => {
                let buf = util::read_exact!(stream, 20)?;
                Self::HmacSha1(Box::new(buf))
            }

            kind => return Err(SignedBlobHeaderError::UnknownSignatureKind(kind)),
        })
    }

    fn dump<T: Write>(&self, stream: &mut T) -> io::Result<()> {
        match self {
            Self::Rsa4096Sha1(data) => {
                stream.write_u32::<BE>(0x010000)?;
                stream.write_all(data.as_slice())?;
            }

            Self::Rsa2048Sha1(data) => {
                stream.write_u32::<BE>(0x010001)?;
                stream.write_all(data.as_slice())?;
            }

            Self::EcdsaSha1(data) => {
                stream.write_u32::<BE>(0x010002)?;
                stream.write_all(data.as_slice())?;
            }

            Self::Rsa4096Sha256(data) => {
                stream.write_u32::<BE>(0x010003)?;
                stream.write_all(data.as_slice())?;
            }

            Self::Rsa2048Sha256(data) => {
                stream.write_u32::<BE>(0x010004)?;
                stream.write_all(data.as_slice())?;
            }

            Self::EcdsaSha256(data) => {
                stream.write_u32::<BE>(0x010005)?;
                stream.write_all(data.as_slice())?;
            }

            Self::HmacSha1(data) => {
                stream.write_u32::<BE>(0x010006)?;
                stream.write_all(data.as_slice())?;
            }
        }

        Ok(())
    }
}

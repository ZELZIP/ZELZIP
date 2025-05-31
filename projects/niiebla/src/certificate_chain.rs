use crate::WriteEx;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::string::FromUtf8Error;
use thiserror::Error;

fn seek_to_relative_boundary<T: Seek>(
    seeker: &mut T,
    absolute_start: u64,
    boundary: u64,
) -> Result<(), io::Error> {
    let relative_position = seeker.stream_position()? - absolute_start;

    seeker.seek(SeekFrom::Start(
        absolute_start + crate::align_to_boundary(relative_position, boundary),
    ))?;

    Ok(())
}

// Boxing the values as Clippy suggest just makes everything harder
// for little to none benefit
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Copy)]
pub enum CertificateSignature {
    Rsa4096([u8; 512]),
    Rsa2048([u8; 256]),
    ElipticCurve([u8; 60]),
}

impl CertificateSignature {
    const SIGNATURE_IDENTIER_RSA_4096: u32 = 0x00010000;
    const SIGNATURE_IDENTIER_RSA_2048: u32 = 0x00010001;
    const SIGNATURE_IDENTIER_ELIPTIC_CURVE: u32 = 0x00010002;

    /// Create a new certificate signature.
    ///
    /// # Safety
    /// The given buffer is assumed to be from a certificate signature,
    /// the current position of the Seek pointer is taken as the start (the first byte of the
    /// signature kind identifier.
    unsafe fn from_reader<T: Read + Seek>(
        reader: &mut T,
    ) -> Result<CertificateSignature, CertificateChainError> {
        let mut signature = match reader.read_u32::<BigEndian>()? {
            Self::SIGNATURE_IDENTIER_RSA_4096 => CertificateSignature::Rsa4096([0; 512]),
            Self::SIGNATURE_IDENTIER_RSA_2048 => CertificateSignature::Rsa2048([0; 256]),
            Self::SIGNATURE_IDENTIER_ELIPTIC_CURVE => CertificateSignature::ElipticCurve([0; 60]),

            identifier => return Err(CertificateChainError::UnknownSignatureKind(identifier)),
        };

        // TODO(IMPROVE): This has more boilerplate that I would like
        match signature {
            CertificateSignature::Rsa4096(ref mut value) => {
                reader.read_exact(value)?;
            }

            CertificateSignature::Rsa2048(ref mut value) => {
                reader.read_exact(value)?;
            }

            CertificateSignature::ElipticCurve(ref mut value) => {
                reader.read_exact(value)?;
            }
        }

        Ok(signature)
    }

    pub fn dump<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u32::<BigEndian>(match self {
            Self::Rsa4096(_) => Self::SIGNATURE_IDENTIER_RSA_4096,
            Self::Rsa2048(_) => Self::SIGNATURE_IDENTIER_RSA_2048,
            Self::ElipticCurve(_) => Self::SIGNATURE_IDENTIER_ELIPTIC_CURVE,
        })?;

        match self {
            CertificateSignature::Rsa4096(signature) => writer.write_all(signature)?,
            CertificateSignature::Rsa2048(signature) => writer.write_all(signature)?,
            CertificateSignature::ElipticCurve(signature) => writer.write_all(signature)?,
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CertificateKey {
    pub id: u32,
    pub value: CertificateKeyValue,
}

// Boxing the values as Clippy suggest just makes everything harder
// for little to none benefit
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Copy)]
pub enum CertificateKeyValue {
    Rsa4096([u8; 512 + 4]),
    Rsa2048([u8; 256 + 4]),
    EccB223([u8; 60]),
}

impl CertificateKeyValue {
    const SIGNATURE_IDENTIER_RSA_4096: u32 = 0x00000000;
    const SIGNATURE_IDENTIER_RSA_2048: u32 = 0x00000001;
    const SIGNATURE_IDENTIER_ELIPTIC_CURVE: u32 = 0x00000002;

    /// Create a new certificate key..
    ///
    /// # Safety
    /// The given buffer is assumed to be from a certificate key,
    /// the current position of the Seek pointer is taken as the start.
    unsafe fn from_reader<T: Read + Seek>(
        idenfitier: u32,
        reader: &mut T,
    ) -> Result<CertificateKeyValue, CertificateChainError> {
        let mut signature = match idenfitier {
            Self::SIGNATURE_IDENTIER_RSA_4096 => CertificateKeyValue::Rsa4096([0; 512 + 4]),
            Self::SIGNATURE_IDENTIER_RSA_2048 => CertificateKeyValue::Rsa2048([0; 256 + 4]),
            Self::SIGNATURE_IDENTIER_ELIPTIC_CURVE => CertificateKeyValue::EccB223([0; 60]),

            identifier => return Err(CertificateChainError::UnknownKeyKind(identifier)),
        };

        match signature {
            CertificateKeyValue::Rsa4096(ref mut value) => {
                reader.read_exact(value)?;
            }

            CertificateKeyValue::Rsa2048(ref mut value) => {
                reader.read_exact(value)?;
            }

            CertificateKeyValue::EccB223(ref mut value) => {
                reader.read_exact(value)?;
            }
        }

        Ok(signature)
    }

    pub fn dump_kind_identifier<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u32::<BigEndian>(match self {
            CertificateKeyValue::Rsa4096(_) => Self::SIGNATURE_IDENTIER_RSA_4096,
            CertificateKeyValue::Rsa2048(_) => Self::SIGNATURE_IDENTIER_RSA_2048,
            CertificateKeyValue::EccB223(_) => Self::SIGNATURE_IDENTIER_ELIPTIC_CURVE,
        })?;

        Ok(())
    }

    pub fn dump_value<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            CertificateKeyValue::Rsa4096(signature) => writer.write_all(signature)?,
            CertificateKeyValue::Rsa2048(signature) => writer.write_all(signature)?,
            CertificateKeyValue::EccB223(signature) => writer.write_all(signature)?,
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Certificate {
    pub signature: CertificateSignature,
    pub issuer: String,
    pub identity: String,
    pub key: CertificateKey,
}

impl Certificate {
    /// Create a new certificate.
    ///
    /// # Safety
    /// The given buffer is assumed to be from a certificate,
    /// the current position of the Seek pointer is taken as the start.
    pub unsafe fn from_reader<T: Read + Seek>(
        reader: &mut T,
    ) -> Result<Certificate, CertificateChainError> {
        unsafe {
            let start_position = reader.stream_position()?;
            let signature = CertificateSignature::from_reader(reader)?;

            // Fix aligment after signature
            seek_to_relative_boundary(reader, start_position, 0x40)?;

            let mut issuer_bytes = [0; 64];
            reader.read_exact(&mut issuer_bytes)?;
            let issuer = crate::string_from_null_terminated_bytes(&issuer_bytes)?;

            let key_value_kind_identifier = reader.read_u32::<BigEndian>()?;

            let mut identity_bytes = [0; 64];
            reader.read_exact(&mut identity_bytes)?;
            let identity = crate::string_from_null_terminated_bytes(&identity_bytes)?;

            let key = CertificateKey {
                id: reader.read_u32::<BigEndian>()?,
                value: CertificateKeyValue::from_reader(key_value_kind_identifier, reader)?,
            };

            Ok(Certificate {
                signature,
                issuer,
                identity,
                key,
            })
        }
    }

    pub fn new_dummy() -> Certificate {
        Certificate {
            signature: CertificateSignature::Rsa4096([0; 512]),
            issuer: String::from(""),
            identity: String::from(""),
            key: CertificateKey {
                id: 0,
                value: CertificateKeyValue::Rsa4096([0; 512 + 4]),
            },
        }
    }

    pub fn dump<W: Write + Seek>(&self, writer: &mut W) -> io::Result<()> {
        let start_position = writer.stream_position()?;

        self.signature.dump(writer)?;
        seek_to_relative_boundary(writer, start_position, 0x40)?;

        writer.write_as_c_string_padded(&self.issuer, 64)?;
        self.key.value.dump_kind_identifier(writer)?;
        writer.write_as_c_string_padded(&self.identity, 64)?;
        writer.write_u32::<BigEndian>(self.key.id)?;
        self.key.value.dump_value(writer)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct CertificateChain {
    pub certificates: Vec<Certificate>,
}

#[derive(Error, Debug)]
pub enum CertificateChainError {
    #[error("An IO error has occurred: {0}")]
    IoError(#[from] io::Error),

    #[error("Unknown signature kind: {0:#X}")]
    UnknownSignatureKind(u32),

    #[error("Unknown key kind: {0:#X}")]
    UnknownKeyKind(u32),

    #[error("Converting into UTF-8 failed: {0}")]
    FromUtf8Error(#[from] FromUtf8Error),
}

impl CertificateChain {
    /// Create a new certificate chain.
    ///
    /// # Safety
    /// The given buffer is assumed to be from a certificate chain,
    /// the current position of the Seek pointer is taken as the start.
    /// The given number of certificates is trusted to be right.
    pub unsafe fn from_reader<T: Read + Seek>(
        reader: &mut T,
        number_of_certificates: usize,
    ) -> Result<CertificateChain, CertificateChainError> {
        unsafe {
            let start_position = reader.stream_position()?;
            let mut certificates = Vec::new();

            for _ in 0..number_of_certificates {
                certificates.push(Certificate::from_reader(reader)?);

                // TODO: Put this into a extension trait
                seek_to_relative_boundary(reader, start_position, 0x40)?;
            }

            Ok(CertificateChain { certificates })
        }
    }

    pub fn dump<W: Write + Seek>(&self, writer: &mut W) -> io::Result<()> {
        let start_position = writer.stream_position()?;

        for certificate in &self.certificates {
            certificate.dump(writer)?;
            seek_to_relative_boundary(writer, start_position, 0x40)?;
        }

        // TODO(FIX ME): This is ugly... Too bad!
        let position = writer.stream_position()?;
        writer.seek(SeekFrom::Start(
            crate::align_to_boundary(position, 0x40) - 1,
        ))?;
        writer.write_zeroed(1)?;

        Ok(())
    }
}

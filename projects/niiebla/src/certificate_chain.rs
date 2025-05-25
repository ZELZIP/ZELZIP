use byteorder::{BigEndian, ReadBytesExt};
use std::io::{self, Read, Seek, SeekFrom};
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
    /// Create a new certificate signature.
    ///
    /// # Safety
    /// The given buffer is assumed to be from a certificate signature,
    /// the current position of the Seek pointer is taken as the start (the first byte of the
    /// signature kind identifier.
    unsafe fn from_reader<T: Read + Seek>(
        reader: &mut T,
    ) -> Result<CertificateSignature, CertificateChainError> {
        const SIGNATURE_IDENTIER_RSA_4096: u32 = 0x00010000;
        const SIGNATURE_IDENTIER_RSA_2048: u32 = 0x00010001;
        const SIGNATURE_IDENTIER_ELIPTIC_CURVE: u32 = 0x00010002;

        let mut signature = match reader.read_u32::<BigEndian>()? {
            SIGNATURE_IDENTIER_RSA_4096 => CertificateSignature::Rsa4096([0; 512]),
            SIGNATURE_IDENTIER_RSA_2048 => CertificateSignature::Rsa2048([0; 256]),
            SIGNATURE_IDENTIER_ELIPTIC_CURVE => CertificateSignature::ElipticCurve([0; 60]),

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
    /// Create a new certificate key..
    ///
    /// # Safety
    /// The given buffer is assumed to be from a certificate key,
    /// the current position of the Seek pointer is taken as the start.
    unsafe fn from_reader<T: Read + Seek>(
        idenfitier: u32,
        reader: &mut T,
    ) -> Result<CertificateKeyValue, CertificateChainError> {
        const SIGNATURE_IDENTIER_RSA_4096: u32 = 0x00000000;
        const SIGNATURE_IDENTIER_RSA_2048: u32 = 0x00000001;
        const SIGNATURE_IDENTIER_ELIPTIC_CURVE: u32 = 0x00000002;

        let mut signature = match idenfitier {
            SIGNATURE_IDENTIER_RSA_4096 => CertificateKeyValue::Rsa4096([0; 512 + 4]),
            SIGNATURE_IDENTIER_RSA_2048 => CertificateKeyValue::Rsa2048([0; 256 + 4]),
            SIGNATURE_IDENTIER_ELIPTIC_CURVE => CertificateKeyValue::EccB223([0; 60]),

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
        let start_position = reader.stream_position()?;
        let mut certificates = Vec::new();

        for _ in 0..number_of_certificates {
            certificates.push(Certificate::from_reader(reader)?);

            seek_to_relative_boundary(reader, start_position, 0x40)?;
        }

        Ok(CertificateChain { certificates })
    }
}

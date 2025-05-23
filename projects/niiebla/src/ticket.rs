use crate::certificate_chain::{CertificateChain, CertificateChainError};
use crate::common_key::{CommonKeyKind, CommonKeyKindError};
use aes::cipher::{block_padding::NoPadding, BlockDecryptMut, KeyIvInit};
use byteorder::{BigEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::Bytes;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TicketError {
    #[error("An IO error has occurred: {0}")]
    IoError(#[from] io::Error),

    #[error("Unknown signature kind: {0:#X}")]
    UnknownSignatureKind(u32),

    #[error("Converting into UTF-8 failed: {0}")]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("Unknown ticket version: {0:#X}")]
    UnknownTicketVersion(u8),

    #[error("Invalid title export flag value: {0:#X}")]
    InvalidTitleExportFlag(u8),

    #[error("Unknown common key: {0:#X}")]
    UnknownCommonKey(u8),

    #[error("An error has occurred while handling the common key: {0}")]
    CommonKeyError(#[from] CommonKeyKindError),

    #[error("Unknown limit entry type: {0:#X}")]
    UnknownLimitEntryType(u32),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TicketSignatureKind {
    Rsa2048,
}

impl TicketSignatureKind {
    fn from_identifier(identifier: u32) -> Result<TicketSignatureKind, TicketError> {
        const SIGNATURE_KIND_IDENTIFIER_RSA_2048: u32 = 0x10001;

        Ok(match identifier {
            SIGNATURE_KIND_IDENTIFIER_RSA_2048 => TicketSignatureKind::Rsa2048,

            bytes => return Err(TicketError::UnknownSignatureKind(bytes)),
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TicketVersion {
    Version0,
    Version1,
}

impl TicketVersion {
    fn from_number(version_number: u8) -> Result<TicketVersion, TicketError> {
        Ok(match version_number {
            0 => TicketVersion::Version0,
            1 => TicketVersion::Version1,

            version => return Err(TicketError::UnknownTicketVersion(version)),
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TicketLimitEntry {
    NoLimit,
    TimeLimit { minutes: u32 },
    LaunchLimit { number_of_launches: u32 },
}

impl TicketLimitEntry {
    fn new(kind: u32, associated_value: u32) -> Result<TicketLimitEntry, TicketError> {
        Ok(match kind {
            0 | 3 => TicketLimitEntry::NoLimit,

            1 => TicketLimitEntry::TimeLimit {
                minutes: associated_value,
            },

            2 => TicketLimitEntry::LaunchLimit {
                number_of_launches: associated_value,
            },

            kind => return Err(TicketError::UnknownLimitEntryType(kind)),
        })
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct Ticket {
    pub signature_kind: TicketSignatureKind,

    #[serde_as(as = "Bytes")]
    pub signature: [u8; 256],

    pub signature_issuer: String,

    #[serde_as(as = "Bytes")]
    pub ecdh_data: [u8; 60],

    pub ticket_version: TicketVersion,
    pub encrypted_title_key: [u8; 16],
    pub ticket_id: [u8; 8],
    pub console_id: u32,
    pub title_id: [u8; 8],
    pub title_version: u16,
    pub permitted_titles_mask: u32,
    pub permit_mask: u32,
    pub is_title_export_allowed: bool,
    pub common_key_kind: CommonKeyKind,

    #[serde_as(as = "Bytes")]
    pub content_access_permissions: [u8; 64],

    pub limit_entries: [TicketLimitEntry; 8],
    // TODO(IMPLEMENT) Support for V1 tickets
}

impl Ticket {
    /// Create a new installable Wad representation.
    ///
    /// # Safety
    /// The given buffer is assumed to be from an installable WAD,
    /// the current position of the Seek pointer is taken as the start.
    pub(crate) unsafe fn from_reader<T: Read + Seek>(
        reader: &mut T,
    ) -> Result<Ticket, TicketError> {
        let signature_kind = TicketSignatureKind::from_identifier(reader.read_u32::<BigEndian>()?)?;

        let mut signature = [0; 256];
        reader.read_exact(&mut signature)?;

        // Skip padding of 60 bytes
        reader.seek(SeekFrom::Current(60))?;

        let mut signature_issuer_bytes = [0; 64];
        reader.read_exact(&mut signature_issuer_bytes)?;

        let signature_issuer = crate::string_from_null_terminated_bytes(&signature_issuer_bytes)?;

        let mut ecdh_data = [0; 60];
        reader.read_exact(&mut ecdh_data)?;

        let ticket_version = TicketVersion::from_number(reader.read_u8()?)?;

        // Skip 2 reserved bytes
        reader.seek(SeekFrom::Current(2))?;

        let mut encrypted_title_key = [0; 16];
        reader.read_exact(&mut encrypted_title_key)?;

        // Skip 1 byte whose use is still unknown
        reader.seek(SeekFrom::Current(1))?;

        let mut ticket_id = [0; 8];
        reader.read_exact(&mut ticket_id)?;

        let console_id = reader.read_u32::<BigEndian>()?;

        let mut title_id = [0; 8];
        reader.read_exact(&mut title_id)?;

        // Skip 2 byte whose use is still unknown
        reader.seek(SeekFrom::Current(2))?;

        let title_version = reader.read_u16::<BigEndian>()?;
        let permitted_titles_mask = reader.read_u32::<BigEndian>()?;
        let permit_mask = reader.read_u32::<BigEndian>()?;

        let title_export_flag_bytes = reader.read_u8()?;

        let is_title_export_allowed = match title_export_flag_bytes {
            0 => false,
            1 => true,
            flag_value => return Err(TicketError::InvalidTitleExportFlag(flag_value)),
        };

        let common_key_kind = CommonKeyKind::from_index(reader.read_u8()?)?;

        // Skip 48 byte whose use is still unknown
        reader.seek(SeekFrom::Current(48))?;

        let mut content_access_permissions = [0; 64];
        reader.read_exact(&mut content_access_permissions)?;

        // Skip padding of 2 bytes
        reader.seek(SeekFrom::Current(2))?;

        let mut limit_entries = [const { TicketLimitEntry::NoLimit }; 8];
        for limit_entry in &mut limit_entries {
            *limit_entry = TicketLimitEntry::new(
                // Kind
                reader.read_u32::<BigEndian>()?,
                // Associated value
                reader.read_u32::<BigEndian>()?,
            )?;
        }

        Ok(Ticket {
            signature_kind,
            signature,
            signature_issuer,
            ecdh_data,
            ticket_version,
            encrypted_title_key,
            ticket_id,
            console_id,
            title_id,
            title_version,
            permitted_titles_mask,
            permit_mask,
            is_title_export_allowed,
            common_key_kind,
            content_access_permissions,
            limit_entries,
        })
    }

    pub fn decrypt_title_key(&self) -> [u8; 16] {
        let id = if self.console_id != 0 {
            self.ticket_id
        } else {
            self.title_id
        };

        let iv: [u8; 16] = [id, [0; 8]]
            .concat()
            .try_into()
            .expect("Will never fail, the `id` slice has always a size of 8");

        type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
        let cipher = Aes128CbcDec::new(&self.common_key_kind.bytes().into(), &iv.into());

        let mut title_key = self.encrypted_title_key;
        cipher
            .decrypt_padded_mut::<NoPadding>(&mut title_key)
            .unwrap();

        title_key
    }
}

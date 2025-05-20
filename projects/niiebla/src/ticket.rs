use aes::cipher::{block_padding::NoPadding, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use aes::Aes128;
use byteorder::{BigEndian, ReadBytesExt};
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::string::FromUtf8Error;
use thiserror::Error;

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

#[derive(Debug)]
pub enum CommonKeyKind {
    Normal,
    Korean,
    WiiU,
}

impl CommonKeyKind {
    pub const fn common_key(&self) -> [u8; 16] {
        match self {
            CommonKeyKind::Normal => [
                0xeb, 0xe4, 0x2a, 0x22, 0x5e, 0x85, 0x93, 0xe4, 0x48, 0xd9, 0xc5, 0x45, 0x73, 0x81,
                0xaa, 0xf7,
            ],
            CommonKeyKind::Korean => [
                0x63, 0xb8, 0x2b, 0xb4, 0xf4, 0x61, 0x4e, 0x2e, 0x13, 0xf2, 0xfe, 0xfb, 0xba, 0x4c,
                0x9b, 0x7e,
            ],
            CommonKeyKind::WiiU => [
                0x30, 0xbf, 0xc7, 0x6e, 0x7c, 0x19, 0xaf, 0xbb, 0x23, 0x16, 0x33, 0x30, 0xce, 0xd7,
                0xc2, 0x8d,
            ],
        }
    }
}

#[derive(Debug)]
pub enum TicketSignatureType {
    Rsa2048,
}

#[derive(Debug)]
pub enum TicketVersion {
    Version0,
    Version1,
}

#[derive(Debug)]
pub enum TicketLimitEntry {
    NoLimit,
    TimeLimit { minutes: u32 },
    LaunchLimit { number_of_launches: u32 },
}

#[derive(Debug)]
pub struct Ticket {
    pub signature_type: TicketSignatureType,
    pub signature: [u8; 256],
    pub signature_issuer: String,
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
    pub content_access_permissions: [u8; 64],
    pub limit_entries: [TicketLimitEntry; 8],
    // TODO(IMPLEMENT) Support for V1 tickets
}

#[derive(Error, Debug)]
pub enum TicketError {
    #[error("An IO error has occurred: {0}")]
    IoError(#[from] io::Error),

    #[error("Unknown signature type: {0:#X}")]
    UnknownSignatureType(u32),

    #[error("Converting into UTF-8 failed: {0}")]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("Unknown ticket version: {0:#X}")]
    UnknownTicketVersion(u8),

    #[error("Invalid title export flag value: {0:#X}")]
    InvalidTitleExportFlag(u8),

    #[error("Unknown common key: {0:#X}")]
    UnknownCommonKey(u8),

    #[error("Unknown limit entry type: {0:#X}")]
    UnknownLimitEntryType(u32),
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
        const SIGNATURE_TYPE_RSA_2048: u32 = 0x10001;

        let signature_type = match reader.read_u32::<BigEndian>()? {
            SIGNATURE_TYPE_RSA_2048 => TicketSignatureType::Rsa2048,

            bytes => return Err(TicketError::UnknownSignatureType(bytes)),
        };

        let mut signature = [0; 256];
        reader.read_exact(&mut signature)?;

        // Skip padding of 60 bytes
        reader.seek(SeekFrom::Current(60))?;

        let mut signature_issuer_bytes = [0; 64];
        reader.read_exact(&mut signature_issuer_bytes)?;

        let signature_issuer = crate::string_from_null_terminated_bytes(&signature_issuer_bytes)?;

        let mut ecdh_data = [0; 60];
        reader.read_exact(&mut ecdh_data)?;

        let ticket_version = match reader.read_u8()? {
            0 => TicketVersion::Version0,
            1 => TicketVersion::Version1,

            version => return Err(TicketError::UnknownTicketVersion(version)),
        };

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

        let common_key_kind = match reader.read_u8()? {
            0 => CommonKeyKind::Normal,
            1 => CommonKeyKind::Korean,
            2 => CommonKeyKind::WiiU,
            common_key_value => return Err(TicketError::UnknownCommonKey(common_key_value)),
        };

        // Skip 48 byte whose use is still unknown
        reader.seek(SeekFrom::Current(48))?;

        let mut content_access_permissions = [0; 64];
        reader.read_exact(&mut content_access_permissions)?;

        // Skip padding of 2 bytes
        reader.seek(SeekFrom::Current(2))?;

        let mut limit_entries = [const { TicketLimitEntry::NoLimit }; 8];
        for limit_entry in &mut limit_entries {
            let limit_entry_value = reader.read_u32::<BigEndian>()?;

            *limit_entry = match reader.read_u32::<BigEndian>()? {
                0 | 3 => TicketLimitEntry::NoLimit,

                1 => TicketLimitEntry::TimeLimit {
                    minutes: limit_entry_value,
                },

                2 => TicketLimitEntry::LaunchLimit {
                    number_of_launches: limit_entry_value,
                },

                limit_entry_type => {
                    return Err(TicketError::UnknownLimitEntryType(limit_entry_type))
                }
            };
        }

        Ok(Ticket {
            signature_type,
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
        /*
        let mut title_id = Vec::from([0; 8]);
        title_id.extend(self.title_id);
        */

        // TODO(FIXME): Choose title or ticket IDs
        let mut title_id = self.title_id.clone().to_vec();
        for _ in 0..8 {
            title_id.push(0);
        }

        let title_id: [u8; 16] = title_id.try_into().unwrap();

        type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
        let cipher = Aes128CbcDec::new(&self.common_key_kind.common_key().into(), &title_id.into());

        let mut title_key = self.encrypted_title_key;
        cipher
            .decrypt_padded_mut::<NoPadding>(&mut title_key)
            .unwrap();

        title_key
    }
}

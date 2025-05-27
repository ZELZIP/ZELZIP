use crate::aes::Aes128CbcDec;
use crate::common_key::{CommonKeyKind, CommonKeyKindError};
use crate::signed_blob_header::{SignedBlobHeader, SignedBlobHeaderError};
use crate::title_id::TitleId;
use crate::WriteEx;
use aes::cipher::{block_padding::NoPadding, BlockDecryptMut, KeyIvInit};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io;
use std::io::Read;
use std::io::{Seek, Write};
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TicketError {
    #[error("An IO error has occurred: {0}")]
    IoError(#[from] io::Error),

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

    #[error("Unable to parse the signed blob header: {0}")]
    SignedBlobHeaderError(#[from] SignedBlobHeaderError),

    #[error("Invalid is virtual console title flag value: {0:#X}")]
    InvalidIsVirtualConsoleFlag(u8),
}

#[derive(Debug)]
pub enum TicketVersion {
    Version0,
    Version1,
}

impl TicketVersion {
    fn dump<T: Write>(&self, writer: &mut T) -> io::Result<()> {
        writer.write_u8(match self {
            TicketVersion::Version0 => 0,
            TicketVersion::Version1 => 1,
        })?;

        Ok(())
    }
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

#[derive(Debug)]
pub enum TicketLimitEntry {
    NoLimit { kind: u32 },
    TimeLimit { minutes: u32 },
    LaunchLimit { number_of_launches: u32 },
}

impl TicketLimitEntry {
    fn new(kind: u32, associated_value: u32) -> Result<TicketLimitEntry, TicketError> {
        Ok(match kind {
            0 | 3 => TicketLimitEntry::NoLimit { kind },

            1 => TicketLimitEntry::TimeLimit {
                minutes: associated_value,
            },

            2 => TicketLimitEntry::LaunchLimit {
                number_of_launches: associated_value,
            },

            kind => return Err(TicketError::UnknownLimitEntryType(kind)),
        })
    }

    fn dump<T: Write>(&self, writer: &mut T) -> io::Result<()> {
        // TODO(CLEAN UP): Is this redundant?
        match self {
            TicketLimitEntry::NoLimit { kind } => {
                writer.write_u32::<BigEndian>(*kind)?;
                writer.write_zeroed(4)?;
            }
            TicketLimitEntry::TimeLimit { minutes } => {
                writer.write_u32::<BigEndian>(1)?;
                writer.write_u32::<BigEndian>(*minutes)?;
            }
            TicketLimitEntry::LaunchLimit { number_of_launches } => {
                writer.write_u32::<BigEndian>(4)?;
                writer.write_u32::<BigEndian>(*number_of_launches)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Ticket {
    pub signed_blob_header: SignedBlobHeader,

    pub signature_issuer: String,
    pub ecdh_data: [u8; 60],
    pub ticket_version: TicketVersion,
    pub encrypted_title_key: [u8; 16],

    pub ticket_id: u64,

    pub console_id: u32,
    pub title_id: TitleId,
    pub title_version: u16,
    pub permitted_titles_mask: u32,
    pub permit_mask: u32,
    pub is_title_export_allowed: bool,
    pub common_key_kind: CommonKeyKind,
    pub is_virtual_console_title: bool,

    // TODO(IMPROVE): Represent this as a bitmask?
    pub content_access_permissions: [u8; 64],

    pub limit_entries: [TicketLimitEntry; 8],
    // TODO(IMPLEMENT): Support for V1 tickets
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
        let signed_blob_header = SignedBlobHeader::from_reader(reader)?;

        let mut signature_issuer_bytes = [0; 64];
        reader.read_exact(&mut signature_issuer_bytes)?;

        let signature_issuer = crate::string_from_null_terminated_bytes(&signature_issuer_bytes)?;

        let mut ecdh_data = [0; 60];
        reader.read_exact(&mut ecdh_data)?;

        let ticket_version = TicketVersion::from_number(reader.read_u8()?)?;

        // Skip 2 reserved bytes
        reader.seek_relative(2)?;

        let mut encrypted_title_key = [0; 16];
        reader.read_exact(&mut encrypted_title_key)?;

        // Skip 1 byte whose use is still unknown
        reader.seek_relative(1)?;

        let ticket_id = reader.read_u64::<BigEndian>()?;

        let console_id = reader.read_u32::<BigEndian>()?;

        let title_id = TitleId::new(reader.read_u64::<BigEndian>()?);

        // Skip 2 byte whose use is still unknown
        reader.seek_relative(2)?;

        let title_version = reader.read_u16::<BigEndian>()?;
        let permitted_titles_mask = reader.read_u32::<BigEndian>()?;
        let permit_mask = reader.read_u32::<BigEndian>()?;

        let is_title_export_allowed = match reader.read_u8()? {
            0 => false,
            1 => true,
            flag_value => return Err(TicketError::InvalidTitleExportFlag(flag_value)),
        };

        let common_key_kind = CommonKeyKind::from_identifier(reader.read_u8()?)?;

        // Skip 47 byte whose use is still unknown
        reader.seek_relative(47)?;

        let is_virtual_console_title = match reader.read_u8()? {
            0 => false,
            1 => true,
            flag_value => return Err(TicketError::InvalidIsVirtualConsoleFlag(flag_value)),
        };

        let mut content_access_permissions = [0; 64];
        reader.read_exact(&mut content_access_permissions)?;

        // Skip padding of 2 bytes
        reader.seek_relative(2)?;

        let mut limit_entries = [const { TicketLimitEntry::NoLimit { kind: 0 } }; 8];
        for limit_entry in &mut limit_entries {
            *limit_entry = TicketLimitEntry::new(
                // Kind
                reader.read_u32::<BigEndian>()?,
                // Associated value
                reader.read_u32::<BigEndian>()?,
            )?;
        }

        Ok(Ticket {
            signed_blob_header,
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
            is_virtual_console_title,
            content_access_permissions,
            limit_entries,
        })
    }

    pub fn decrypt_title_key(&self) -> [u8; 16] {
        let id = if self.console_id != 0 {
            self.ticket_id
        } else {
            self.title_id.get()
        };

        let iv: [u8; 16] = [id.to_be_bytes(), [0; 8]]
            .concat()
            .try_into()
            .expect("Will never fail, the `id` slice has always a size of 8");

        let cipher = Aes128CbcDec::new(&self.common_key_kind.bytes().into(), &iv.into());

        let mut title_key = self.encrypted_title_key;
        cipher
            .decrypt_padded_mut::<NoPadding>(&mut title_key)
            .unwrap();

        title_key
    }

    pub fn dump<T: Write>(&self, writer: &mut T) -> io::Result<()> {
        self.signed_blob_header.dump(writer)?;
        writer.write_as_c_string_padded(&self.signature_issuer, 64)?;
        writer.write_all(&self.ecdh_data)?;
        self.ticket_version.dump(writer)?;

        // Skip 2 reserved bytes
        writer.write_zeroed(2)?;

        writer.write_all(&self.encrypted_title_key)?;

        // Skip 1 unknown byte
        writer.write_zeroed(1)?;

        writer.write_u64::<BigEndian>(self.ticket_id)?;
        writer.write_u32::<BigEndian>(self.console_id)?;
        self.title_id.dump(writer)?;

        // Skip 2 unknown bytes
        //writer.write_zeroed(2)?;
        writer.write_all(&[0xFF, 0xFF])?;

        writer.write_u16::<BigEndian>(self.title_version)?;
        writer.write_u32::<BigEndian>(self.permitted_titles_mask)?;
        writer.write_u32::<BigEndian>(self.permit_mask)?;
        writer.write_bool(self.is_title_export_allowed)?;
        self.common_key_kind.dump_identifier(writer)?;

        // Skip 47 unknown bytes
        writer.write_zeroed(47)?;

        writer.write_bool(self.is_virtual_console_title)?;
        writer.write_all(&self.content_access_permissions)?;

        // Skip 2 bytes of padding
        writer.write_zeroed(2)?;

        for limit_entry in &self.limit_entries {
            limit_entry.dump(writer)?;
        }

        Ok(())
    }
}

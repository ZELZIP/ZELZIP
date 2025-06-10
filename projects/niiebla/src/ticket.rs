//! Implementation of the binary file format used by Nintendo to store tickets.

use crate::signed_blob_header::{SignedBlobHeader, SignedBlobHeaderError};
use crate::title_id::TitleId;
use crate::wii_common_key::{CommonKeyKindError, WiiCommonKeyKind};
use aes::cipher::{BlockDecryptMut, KeyIvInit, block_padding::NoPadding};
use bitflags::bitflags;
use byteorder::{BE, ReadBytesExt, WriteBytesExt};
use std::io;
use std::io::Read;
use std::io::{Seek, Write};
use std::string::FromUtf8Error;
use thiserror::Error;
use util::Aes128CbcDec;
use util::AesCbcStream;
use util::WriteEx;

/// Manifest data regard the ownership of a title and its permissions over the hardware.
///
/// Only compatible with versions zero (V0) and one (V1), present on the Nintendo Wii, Wii U,
/// DSi and 3DS, as version two (V2), used on the Nintendo Switch and forward,
/// has a completly different and incompatible format whose version entry
/// has been reallocated to a different offset.
#[derive(Debug)]
pub struct PreSwitchTicket {
    /// Header with data to prove the authenticity that this data
    /// has being created by an authorized entity.
    pub signed_blob_header: SignedBlobHeader,

    /// Public key emited by the "ticketing server",
    /// used for installation of the title in some platforms.
    pub ecc_public_key: [u8; 60],

    /// Version of the
    /// [Certificate revocation list](https://en.wikipedia.org/wiki/Certificate_revocation_list)
    /// used for the Certificate Authority (CA) certificate.
    pub certificate_authority_certificate_revocation_list_version: u8,

    /// Version of the
    /// [Certificate revocation list](https://en.wikipedia.org/wiki/Certificate_revocation_list)
    /// used for the signer certificate.
    pub signer_certificate_revocation_list_version: u8,

    /// Encrypted title key, this symetric key (after decryption)
    /// is used to encrypt the title content.
    pub encrypted_title_key: [u8; 16],

    /// The ID of the ticket.
    pub ticket_id: u64,

    /// The ID of the device associated with this ticket,
    /// `None` is the ticket is valid for all consoles.
    pub device_id: Option<u32>,

    /// The ID of the associated title.
    pub title_id: TitleId,

    /// The permissions of the "System App" to access the contents of the title.
    pub system_app_content_access: PreSwitchTicketSystemAppContentAccessFlags,

    /// The version of the title.
    pub title_version: u16,

    /// See [Self::permitted_generic_title_id].
    pub permitted_generic_title_id: u32,

    /// Here be dragons!, the desired behavior of the following functionality has not been fully understood yet.
    /// This value is used as inverse mask (the bits with 1 are discarted) over the title ID,
    /// the resulting partial ID is then comapared with the [Self::permitted_generic_title_id] value to determinated
    /// if the ID is valid or not.
    ///
    /// How this is useful (given that the ID is already hardcoded in the ticket) and where is has been used is still unknown.
    // TODO(IMPROVE): Truly understand this.
    pub permitted_generic_title_id_mask: u32,

    /// The license of the title.
    pub license: PreTicketLicense,

    /// The index of the common key to be used to decrypt the title content, the value of the
    /// common key is platform dependant.
    pub common_key_kind_index: u8,

    /// Audit or revision of the title. The meaning is still not clear.
    // TODO(IMPROVE): Understand this.
    pub audit: u8,

    /// Set of bitflags regard if a content can be accessed (1) or not (0).
    // TODO(IMPROVE): Understand what "limit access" this regulates.
    pub content_access_permissions: [u8; 64],

    /// A set of limits over the use of the title.
    pub limit_entries: [PreSwitchTicketLimitEntry; 8],

    /// Extra data only present on the v1 version of a ticket.
    // TODO(IMPLEMENT): Add support for v1 tickets.
    version_1_extension: Option<()>,
}

impl PreSwitchTicket {
    /// Parse a ticket.
    pub fn new<T: Read + Seek>(mut stream: T) -> Result<Self, PreSwitchTicketError> {
        let signed_blob_header = SignedBlobHeader::new(&mut stream)?;
        let ecc_public_key = util::read_exact!(stream, 60)?;

        // TODO(IMPLEMENT): This should change when V1 support is here. Also greater than v1 should
        // error.
        stream.seek_relative(1)?;
        let version_1_extension = None;

        let certificate_authority_certificate_revocation_list_version = stream.read_u8()?;
        let signer_certificate_revocation_list_version = stream.read_u8()?;

        let encrypted_title_key = util::read_exact!(stream, 16)?;

        // Skip 1 reserved byte
        stream.seek_relative(1)?;

        let ticket_id = stream.read_u64::<BE>()?;

        let device_id = match stream.read_u32::<BE>()? {
            0 => None,
            value => Some(value),
        };

        let title_id = TitleId::new(stream.read_u64::<BE>()?);

        #[allow(clippy::expect_used)]
        let system_app_content_access =
            PreSwitchTicketSystemAppContentAccessFlags::from_bits(stream.read_u16::<BE>()?)
                .expect("This will never panic as the bitflags covers all the 16bit range");

        let title_version = stream.read_u16::<BE>()?;

        let permitted_generic_title_id = stream.read_u32::<BE>()?;
        let permitted_generic_title_id_mask = stream.read_u32::<BE>()?;

        let license = PreTicketLicense::new(stream.read_u8()?)?;
        let common_key_kind_index = stream.read_u8()?;

        // Skip 47 byte whose use is still unknown
        stream.seek_relative(47)?;

        let audit = stream.read_u8()?;
        let content_access_permissions = util::read_exact!(stream, 64)?;

        // Skip padding of 2 bytes
        stream.seek_relative(2)?;

        let mut limit_entries = [const { PreSwitchTicketLimitEntry::NoLimit { kind: 0 } }; 8];
        for limit_entry in &mut limit_entries {
            *limit_entry = PreSwitchTicketLimitEntry::new(
                // Kind
                stream.read_u32::<BE>()?,
                // Associated value
                stream.read_u32::<BE>()?,
            )?;
        }

        Ok(Self {
            signed_blob_header,
            ecc_public_key,
            certificate_authority_certificate_revocation_list_version,
            signer_certificate_revocation_list_version,
            encrypted_title_key,
            ticket_id,
            device_id,
            title_id,
            system_app_content_access,
            title_version,
            permitted_generic_title_id,
            permitted_generic_title_id_mask,
            license,
            common_key_kind_index,
            audit,
            content_access_permissions,
            limit_entries,
            version_1_extension,
        })
    }

    /// Either if this ticket was generated to be used only in a specific console (the associated
    /// title was purchased) or not.
    pub fn is_device_unique(&self) -> bool {
        self.device_id.is_some()
    }

    /// Decrypt the title key using the corrent common key, only works on Nintendo Wii tickets.
    pub fn decrypt_title_key_wii_method(&self) -> [u8; 16] {
        let id = if self.is_device_unique() {
            self.ticket_id
        } else {
            self.title_id.inner()
        };

        #[allow(clippy::expect_used)]
        let iv: [u8; 16] = [id.to_be_bytes(), [0; 8]]
            .concat()
            .try_into()
            .expect("Will never fail, the `id` slice has always a size of 8");

        // TODO(IMPLEMENT): Add support all of the rest of encryption methods and remove this
        // unwrap.
        #[allow(clippy::unwrap_used)]
        let common_key_kind = WiiCommonKeyKind::new(self.common_key_kind_index).unwrap();
        let cipher = Aes128CbcDec::new((&common_key_kind.bytes()).into(), &iv.into());

        let mut title_key = self.encrypted_title_key;

        // TODO(IMPROVE): Too bad! Add proper error handling.
        #[allow(clippy::unwrap_used)]
        cipher
            .decrypt_padded_mut::<NoPadding>(&mut title_key)
            .unwrap();

        title_key
    }

    /// Dump into a stream.
    pub fn dump<T: Write + Seek>(&self, mut stream: T) -> io::Result<()> {
        self.signed_blob_header.dump(&mut stream)?;
        stream.write_all(&self.ecc_public_key)?;
        stream.write_bool(self.version_1_extension.is_some())?;
        stream.write_u8(self.certificate_authority_certificate_revocation_list_version)?;
        stream.write_u8(self.signer_certificate_revocation_list_version)?;
        stream.write_all(&self.encrypted_title_key)?;

        // Skip 1 reserved byte
        stream.write_zeroed(1)?;

        stream.write_u64::<BE>(self.ticket_id)?;
        stream.write_u32::<BE>(self.device_id.unwrap_or(0))?;
        self.title_id.dump(&mut stream)?;
        stream.write_u16::<BE>(self.system_app_content_access.bits())?;
        stream.write_u16::<BE>(self.title_version)?;
        stream.write_u32::<BE>(self.permitted_generic_title_id)?;
        stream.write_u32::<BE>(self.permitted_generic_title_id_mask)?;
        self.license.dump(&mut stream)?;
        stream.write_u8(self.common_key_kind_index)?;

        // Skip 47 assigned but unused bytes
        stream.write_zeroed(47)?;

        stream.write_u8(self.audit)?;
        stream.write_all(&self.content_access_permissions)?;

        // Skip 2 bytes of padding
        stream.write_zeroed(2)?;

        for limit_entry in &self.limit_entries {
            limit_entry.dump(&mut stream)?;
        }

        // TODO(IMPLEMENT): v1 ticket dumping.

        Ok(())
    }

    /// Get the sizes of the ticket in bytes.
    pub fn size(&self) -> u32 {
        if self.version_1_extension.is_none() {
            // Manually calculated value for v0 tickets
            return 292 + self.signed_blob_header.size();
        }

        // TODO(IMPROVE): Support for v1 ticket.
        panic!();
    }

    fn get_key_and_iv(
        &self,
        content_index: u16,
    ) -> Result<([u8; 16], [u8; 16]), PreSwitchTicketError> {
        let title_key = self.decrypt_title_key_wii_method();

        // Add 14 trailing zeroed bytes to the IV
        let mut iv = Vec::from(content_index.to_be_bytes());
        iv.append(&mut Vec::from([0; 14]));

        #[allow(clippy::expect_used)]
        let iv: [u8; 16] = iv
            .try_into()
            .expect("Will never fail, the `content_index` is always 16 bits");

        Ok((title_key, iv))
    }

    // TODO(IMPROVE): Proper multi platform support.
    /// Get a decryptor of a content for the given stream.
    pub fn cryptographic_stream_wii_method<T: Seek>(
        &self,
        stream: T,
        content_index: u16,
    ) -> Result<AesCbcStream<T>, PreSwitchTicketError> {
        let (title_key, iv) = self.get_key_and_iv(content_index)?;

        Ok(AesCbcStream::new(stream, title_key, iv)?)
    }
}

#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum PreSwitchTicketError {
    /// IO error.
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Converting into UTF-8 failed: {0}")]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("Unknown ticket version: {0:#X}")]
    UnknownTicketVersion(u8),

    #[error("Unknown common key kind index: {0:#X}")]
    UnknownCommonKeyKindIndex(u8),

    #[error("An error has occurred while handling the common key: {0}")]
    CommonKeyError(#[from] CommonKeyKindError),

    #[error("Unknown limit entry type: {0:#X}")]
    UnknownLimitEntryType(u32),

    #[error("Unable to parse the signed blob header: {0}")]
    SignedBlobHeaderError(#[from] SignedBlobHeaderError),

    #[error("Invalid license kind identifier value")]
    InvalidLicenseKindIdentifierValue(u8),
}

bitflags! {
    /// Bitflags that indicate if a content (given its content index) can be accessed by the
    /// "System App" (the meaning and consequences of this "System App" are not known yet).
    // TODO(IMPROVE): Discover what the "System App" is.
    #[derive(Debug)]
    pub struct PreSwitchTicketSystemAppContentAccessFlags: u16 {
        /// Content 0.
        const Content0 =  1 << 0;

        /// Content 1.
        const Content1 =  1 << 1;

        /// Content 2.
        const Content2 =  1 << 2;

        /// Content 3.
        const Content3 =  1 << 3;

        /// Content 4.
        const Content4 =  1 << 4;

        /// Content 5.
        const Content5 =  1 << 5;

        /// Content 6.
        const Content6 =  1 << 6;

        /// Content 7.
        const Content7 =  1 << 7;

        /// Content 8.
        const Content8 =  1 << 8;

        /// Content 9.
        const Content9 =  1 << 9;

        /// Content 10.
        const Content10 = 1 << 10;

        /// Content 11.
        const Content11 = 1 << 11;

        /// Content 12.
        const Content12 = 1 << 12;

        /// Content 13.
        const Content13 = 1 << 13;

        /// Content 14.
        const Content14 = 1 << 14;

        /// Content 15.
        const Content15 = 1 << 15;
    }
}

/// The kind of license used in a ticket.
// TODO(IMRPOVE): Maybe this can be understood as a "policy"?
#[derive(Debug)]
pub enum PreTicketLicense {
    /// The normal license of a Ticket.
    Normal,

    /// The ticket can be "exported".
    // TODO(IMPROVE): Maybe to an external device?
    CanBeExported,
}

impl PreTicketLicense {
    fn new(identifier: u8) -> Result<Self, PreSwitchTicketError> {
        Ok(match identifier {
            0 => Self::Normal,
            1 => Self::CanBeExported,

            _ => {
                return Err(PreSwitchTicketError::InvalidLicenseKindIdentifierValue(
                    identifier,
                ));
            }
        })
    }

    fn dump<T: Write>(&self, mut stream: T) -> io::Result<()> {
        stream.write_u8(match self {
            Self::Normal => 0,
            Self::CanBeExported => 1,
        })?;

        Ok(())
    }
}

#[derive(Debug)]
/// Limits over the use of a ticket.
pub enum PreSwitchTicketLimitEntry {
    /// The title doesn't have any limits.
    NoLimit {
        /// The no limit entryhave been seen with multiple values (zero and three), to preserve
        /// reproducibility it has to be stored. Probably you don't want to edit it.
        kind: u32,
    },

    /// The title can only be executed a deteminate number of minutes.
    TimeLimit {
        /// The number of minutes that can be played.
        minutes: u32,
    },

    /// The title can only be launched a determinate number of times.
    LaunchLimit {
        /// The number of times the title can be launched.
        number_of_launches: u32,
    },
}

impl PreSwitchTicketLimitEntry {
    fn new(kind: u32, associated_value: u32) -> Result<Self, PreSwitchTicketError> {
        Ok(match kind {
            0 | 3 => Self::NoLimit { kind },

            1 => Self::TimeLimit {
                minutes: associated_value,
            },

            2 => Self::LaunchLimit {
                number_of_launches: associated_value,
            },

            _ => return Err(PreSwitchTicketError::UnknownLimitEntryType(kind)),
        })
    }

    fn dump<T: Write>(&self, mut stream: T) -> io::Result<()> {
        match self {
            Self::NoLimit { kind } => {
                stream.write_u32::<BE>(*kind)?;
                stream.write_zeroed(4)?;
            }
            Self::TimeLimit { minutes } => {
                stream.write_u32::<BE>(1)?;
                stream.write_u32::<BE>(*minutes)?;
            }
            Self::LaunchLimit { number_of_launches } => {
                stream.write_u32::<BE>(4)?;
                stream.write_u32::<BE>(*number_of_launches)?;
            }
        }

        Ok(())
    }
}

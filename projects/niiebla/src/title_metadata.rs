use crate::signed_blob_header::{SignedBlobHeader, SignedBlobHeaderError};
use crate::title_id::TitleId;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::string::FromUtf8Error;
use thiserror::Error;
use util::{ReadEx, WriteEx};

/// Manifest data regard the title itself, its structure and allowed system access.
///
/// Compatible with both versions zero (V0) and one (V1), present on the Nintendo Wii, Wii U
/// DSi and 3DS
///
/// Not compatible with "PackagedContentMeta" (aka CNMT) used on the Nintendo Switch and forward.
#[derive(Debug)]
pub struct TitleMetadata {
    /// Header with data to prove the authenticity that this data
    /// has being created by an authorized entity.
    pub signed_blob_header: SignedBlobHeader,

    /// Version of the
    /// [Certificate revocation list](https://en.wikipedia.org/wiki/Certificate_revocation_list)
    /// used for the Certificate Authority (CA) certificate.
    pub certificate_authority_certificate_revocation_list_version: u8,

    /// Version of the
    /// [Certificate revocation list](https://en.wikipedia.org/wiki/Certificate_revocation_list)
    /// used for the signer certificate.
    pub signer_certificate_revocation_list_version: u8,

    /// Title ID of the title used as "System runtime", its exact meaning depends on the platform
    /// the title is for:
    ///
    /// # iQue NetCard
    /// The product was never released, so its use is unknown.
    ///
    /// # Nintendo Wii
    /// If `Some` it's the title of the IOS to be used for this title, if `None` then the title is
    /// itself an IOS.
    ///
    /// If the title is a boot2 program (title ID: `00000001-00000001`) then this entry will be its
    /// same title ID.
    ///
    /// # Nintendo Wii U
    /// If `Some` it's the title of the IOSU to be used for this title, if `None` then the title is
    /// itself an IOSU.
    ///
    // TODO(IMPROVE): Document meaning in 3DS and DSi.
    pub system_runtime_title_id: Option<TitleId>,

    /// Title ID of the title.
    pub title_id: TitleId,

    /// Group ID of the title.
    // TODO(IMPROVE): Discover more about this entry.
    pub group_id: u16,

    /// Bitflags of access right to the hardware, its meaning depends on the platform, the access
    /// to this entry is recommended to use platform aware methods like [Self::has_ppu_access] or [Self::has_dvd_access].
    // TODO(IMPLEMENT): Per platform methods.
    pub access_rights: u32,

    /// The version of the title.
    // TODO(IMPROVE): This has a hidden format in hex, would be useful to wrap around a
    // newtype.
    pub title_version: u16,

    /// The index value of the content entry where the boot data is located.
    pub boot_content_index: u16,

    /// Platform dependant data.
    pub platform_data: TitleMetadataPlatformData,

    /// Extra data only present on the v1 version of a title metadata.
    // TODO(IMPLEMENT): Add support for content info entries.
    pub version_1_extension: Option<()>,

    /// Entries to the different content chunks.
    pub content_chunk_entries: Vec<TitleMetadataContentEntry>,
}

impl TitleMetadata {
    /// Create a new installable Wad representation.
    ///
    /// # Safety
    /// The given buffer is assumed to be from an title metadata entry,
    /// the current position of the Seek pointer is taken as the start.
    pub unsafe fn new<T: Read + Seek>(stream: &mut T) -> Result<Self, TitleMetadataError> {
        let signed_blob_header = SignedBlobHeader::new(stream)?;

        // TODO(IMPLEMENT): Add support for v1.
        let _format_version = stream.read_u8()?;

        let certificate_authority_certificate_revocation_list_version = stream.read_u8()?;
        let signer_certificate_revocation_list_version = stream.read_u8()?;

        // On some platforms this byte has a meaning as a bool
        let first_reserved_byte = stream.read_bool()?;

        let system_runtime_title_id = match stream.read_u64::<BigEndian>()? {
            0 => None,
            title_id => Some(TitleId::new(title_id)),
        };

        let title_id = TitleId::new(stream.read_u64::<BigEndian>()?);

        let mut platform_data =
            TitleMetadataPlatformData::new_dummy_from_identifier(stream.read_u32::<BigEndian>()?)?;

        let group_id = stream.read_u16::<BigEndian>()?;

        match platform_data {
            TitleMetadataPlatformData::IQueNetCard => {
                stream.seek_relative(62)?;
            }

            TitleMetadataPlatformData::Wii {
                ref mut is_wii_u_vwii_only_title,
                ref mut region,
                ref mut ratings,
                ref mut ipc_mask,
            } => {
                *is_wii_u_vwii_only_title = first_reserved_byte;

                // Skip 2 zeroed bytes
                stream.seek_relative(2)?;

                *region = TitleMetadataPlatformDataWiiRegion::from_identifier(
                    stream.read_u16::<BigEndian>()?,
                )?;

                *ratings = util::read_exact!(stream, 16)?;

                // Skip 12 reserved bytes
                stream.seek_relative(12)?;

                *ipc_mask = util::read_exact!(stream, 12)?;

                // Skip 18 reserved bytes
                stream.seek_relative(18)?;
            }
        }

        let access_rights = stream.read_u32::<BigEndian>()?;
        let title_version = stream.read_u16::<BigEndian>()?;
        let number_of_content_entries = stream.read_u16::<BigEndian>()?;
        let boot_content_index = stream.read_u16::<BigEndian>()?;

        // Skip the title minor version as it was never used
        stream.seek_relative(2)?;

        let mut content_chunk_entries = Vec::new();

        for _ in 0..number_of_content_entries {
            content_chunk_entries.push(TitleMetadataContentEntry::new(stream)?);
        }

        let version_1_extension = None;

        Ok(Self {
            signed_blob_header,
            certificate_authority_certificate_revocation_list_version,
            signer_certificate_revocation_list_version,
            system_runtime_title_id,
            title_id,
            platform_data,
            group_id,
            title_version,
            boot_content_index,
            access_rights,
            version_1_extension,
            content_chunk_entries,
        })
    }

    /// Dump into a stream.
    pub fn dump<T: Write + Seek>(&self, stream: &mut T) -> io::Result<()> {
        self.signed_blob_header.dump(stream)?;
        stream.write_bool(self.version_1_extension.is_some())?;
        stream.write_u8(self.certificate_authority_certificate_revocation_list_version)?;
        stream.write_u8(self.signer_certificate_revocation_list_version)?;

        stream.write_u8(match self.platform_data {
            TitleMetadataPlatformData::IQueNetCard => 0,
            TitleMetadataPlatformData::Wii {
                is_wii_u_vwii_only_title,
                region: _,
                ratings: _,
                ipc_mask: _,
            } => {
                if is_wii_u_vwii_only_title {
                    1
                } else {
                    0
                }
            }
        })?;

        match &self.system_runtime_title_id {
            None => stream.write_u8(0)?,
            Some(title_id) => title_id.dump(stream)?,
        };

        self.title_id.dump(stream)?;
        self.platform_data.dump_identifier(stream)?;
        stream.write_u16::<BigEndian>(self.group_id)?;

        match &self.platform_data {
            TitleMetadataPlatformData::IQueNetCard => {
                stream.write_zeroed(62)?;
            }

            TitleMetadataPlatformData::Wii {
                is_wii_u_vwii_only_title: _,
                region,
                ratings,
                ipc_mask,
            } => {
                stream.write_zeroed(2)?;

                region.dump_identifier(stream)?;

                stream.write_all(ratings)?;
                stream.write_zeroed(12)?;
                stream.write_all(ipc_mask)?;
                stream.write_zeroed(18)?;
            }
        }

        stream.write_u32::<BigEndian>(self.access_rights)?;
        stream.write_u16::<BigEndian>(self.title_version)?;
        stream.write_u16::<BigEndian>(self.content_chunk_entries.len() as u16)?;
        stream.write_u16::<BigEndian>(self.boot_content_index)?;

        // Skip the title minor version as it was never used
        stream.seek_relative(2)?;

        for content_entry in &self.content_chunk_entries {
            content_entry.dump(stream)?;
        }

        Ok(())
    }

    /// If the title has access to the DVD drive. Only on Wii and Wii U platforms.
    pub fn has_dvd_access(&self) -> Result<bool, TitleMetadataError> {
        // TODO(IMPLEMENT): Add support for Wii U.
        if let TitleMetadataPlatformData::Wii {
            is_wii_u_vwii_only_title: _,
            region: _,
            ratings: _,
            ipc_mask: _,
        } = self.platform_data
        {
            return Ok((self.access_rights & 0b10) != 0);
        }

        Err(TitleMetadataError::ActionInvalid())
    }

    /// If the title has access to all hardware from its main PPC chip without using a IOS between
    /// the comunication (aka disable the `AHBPROT` protection).
    /// Only on Wii (and Wii U vWii) platforms.
    pub fn has_ppm_access(&self) -> Result<bool, TitleMetadataError> {
        if let TitleMetadataPlatformData::Wii {
            is_wii_u_vwii_only_title: _,
            region: _,
            ratings: _,
            ipc_mask: _,
        } = self.platform_data
        {
            return Ok((self.access_rights & 0b1) != 0);
        }

        Err(TitleMetadataError::ActionInvalid())
    }
}

#[derive(Error, Debug)]
pub enum TitleMetadataError {
    #[error("An IO error has occurred: {0}")]
    IoError(#[from] io::Error),

    #[error("Unable to parse the signed blob header: {0}")]
    SignedBlobHeaderError(#[from] SignedBlobHeaderError),

    #[error("Converting into UTF-8 failed: {0}")]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("Invalid value for the 'is vWii title' flag")]
    InvalidIsVWiiValue(u8),

    #[error("The given title metadata platform is not known: {0}")]
    UnknownPlatform(u32),

    #[error(
        "The given title metadata Nintendo Wii 
        region is not known: {0}"
    )]
    UnknownWiiRegion(u16),

    #[error("The given content entry kind is not known: {0}")]
    UnknownContentEntryKind(u16),

    #[error("The action is invalid for the platform of the title")]
    ActionInvalid(),
}

#[derive(Debug)]
pub enum TitleMetadataPlatformData {
    IQueNetCard,
    Wii {
        /// If the title is made to only run on Wii U vWii (The virtual Wii system inside the
        /// Nintendo Wii U).
        is_wii_u_vwii_only_title: bool,

        region: TitleMetadataPlatformDataWiiRegion,
        ratings: [u8; 16],
        ipc_mask: [u8; 12],
    },
    // TODO(IMPLEMENT): Support for DSi, 3DS and Wii U.
}

impl TitleMetadataPlatformData {
    fn new_dummy_from_identifier(identifier: u32) -> Result<Self, TitleMetadataError> {
        match identifier {
            0x00000000 => Ok(Self::IQueNetCard),
            0x00000001 => Ok(Self::Wii {
                is_wii_u_vwii_only_title: false,
                region: TitleMetadataPlatformDataWiiRegion::RegionFree,
                ratings: [0; 16],
                ipc_mask: [0; 12],
            }),
            identifier => Err(TitleMetadataError::UnknownPlatform(identifier)),
        }
    }

    fn dump_identifier<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u32::<BigEndian>(match self {
            Self::IQueNetCard => 0,

            Self::Wii {
                is_wii_u_vwii_only_title: _,
                region: _,
                ratings: _,
                ipc_mask: _,
            } => 1,
        })?;

        Ok(())
    }
}

/// The different regions a title can be on a Wii console.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum TitleMetadataPlatformDataWiiRegion {
    Japan,
    USA,
    Europe,
    RegionFree,
    Korea,
}

impl TitleMetadataPlatformDataWiiRegion {
    fn from_identifier(identifier: u16) -> Result<Self, TitleMetadataError> {
        match identifier {
            0 => Ok(Self::Japan),
            1 => Ok(Self::USA),
            2 => Ok(Self::Europe),
            3 => Ok(Self::RegionFree),
            4 => Ok(Self::Korea),

            identifier => Err(TitleMetadataError::UnknownWiiRegion(identifier)),
        }
    }

    fn dump_identifier<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u16::<BigEndian>(match &self {
            Self::Japan => 0,
            Self::USA => 1,
            Self::Europe => 2,
            Self::RegionFree => 3,
            Self::Korea => 4,
        })?;

        Ok(())
    }
}

/// An entry of a content of a title, a content is just a signed
#[derive(Debug)]
pub struct TitleMetadataContentEntry {
    /// The ID of the content. Unique per title.
    pub id: u32,

    /// The index of the content. Unique per title "bundle" (WAD file, disc image, etc).
    pub index: u16,

    /// The kind of the content.
    pub kind: TitleMetadataContentEntryKind,

    /// The size of the content.
    pub size: u64,

    /// The SHA-1 hash of the content.
    // TODO(IMPROVE): On v1 this is a SHA-256 (32 bytes on size). Too bad!
    pub hash: [u8; 20],
}

#[derive(Debug)]
/// The kind (behaviour of the content inside the system) of the content.
pub enum TitleMetadataContentEntryKind {
    /// A normal content.
    Normal,

    /// A downloadable content for a title.
    Dlc,

    /// A content that can be shared between different title, the system may store then on its
    /// internal memory for reuse.
    Shared,
}

impl TitleMetadataContentEntry {
    /// Create a new installable Wad representation.
    ///
    /// # Safety
    /// The given buffer is assumed to be from an title metadata content entry,
    /// the current position of the Seek pointer is taken as the start.
    pub fn new<T: Read + Seek>(stream: &mut T) -> Result<Self, TitleMetadataError> {
        let id = stream.read_u32::<BigEndian>()?;
        let index = stream.read_u16::<BigEndian>()?;

        let kind = match stream.read_u16::<BigEndian>()? {
            0x0001 => TitleMetadataContentEntryKind::Normal,
            0x4001 => TitleMetadataContentEntryKind::Dlc,
            0x8001 => TitleMetadataContentEntryKind::Shared,

            identifier => return Err(TitleMetadataError::UnknownContentEntryKind(identifier)),
        };

        let size = stream.read_u64::<BigEndian>()?;

        let mut hash = [0; 20];
        stream.read_exact(&mut hash)?;

        Ok(Self {
            id,
            index,
            kind,
            size,
            hash,
        })
    }

    fn dump<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u32::<BigEndian>(self.id)?;
        writer.write_u16::<BigEndian>(self.index)?;

        writer.write_u16::<BigEndian>(match &self.kind {
            TitleMetadataContentEntryKind::Normal => 0x0001,
            TitleMetadataContentEntryKind::Dlc => 0x4001,
            TitleMetadataContentEntryKind::Shared => 0x8001,
        })?;

        writer.write_u64::<BigEndian>(self.size)?;
        writer.write_all(&self.hash)?;

        Ok(())
    }
}

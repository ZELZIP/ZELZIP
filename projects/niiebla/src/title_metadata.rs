use crate::signed_blob_header::{SignedBlobHeader, SignedBlobHeaderError};
use crate::title_id::TitleId;
use crate::WriteEx;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Debug)]
pub enum TitleMetadataContentEntryKind {
    Normal,
    Dlc,
    Shared,
}

#[derive(Debug)]
pub struct TitleMetadataContentEntry {
    pub id: u32,
    pub index: u16,
    pub kind: TitleMetadataContentEntryKind,
    pub size: u64,
    pub hash: [u8; 20],
}

impl TitleMetadataContentEntry {
    /// Create a new installable Wad representation.
    ///
    /// # Safety
    /// The given buffer is assumed to be from an title metadata content entry,
    /// the current position of the Seek pointer is taken as the start.
    pub unsafe fn from_reader<T: Read + Seek>(
        reader: &mut T,
    ) -> Result<TitleMetadataContentEntry, TitleMetadataError> {
        let id = reader.read_u32::<BigEndian>()?;
        let index = reader.read_u16::<BigEndian>()?;

        let kind = match reader.read_u16::<BigEndian>()? {
            0x0001 => TitleMetadataContentEntryKind::Normal,
            0x4001 => TitleMetadataContentEntryKind::Dlc,
            0x8001 => TitleMetadataContentEntryKind::Shared,

            identifier => return Err(TitleMetadataError::UnknownContentEntryKind(identifier)),
        };

        let size = reader.read_u64::<BigEndian>()?;

        let mut hash = [0; 20];
        reader.read_exact(&mut hash)?;

        Ok(TitleMetadataContentEntry {
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

#[derive(Debug)]
pub enum TitleMetadataPlatformKind {
    IQueNetCard,
    Wii,
}

impl TitleMetadataPlatformKind {
    fn from_identifier(identifier: u32) -> Result<TitleMetadataPlatformKind, TitleMetadataError> {
        match identifier {
            0x00000000 => Ok(TitleMetadataPlatformKind::IQueNetCard),
            0x00000001 => Ok(TitleMetadataPlatformKind::Wii),
            identifier => Err(TitleMetadataError::UnknownPlatformKind(identifier)),
        }
    }

    fn dump<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u32::<BigEndian>(match self {
            TitleMetadataPlatformKind::IQueNetCard => 0,
            TitleMetadataPlatformKind::Wii => 1,
        })?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum TitleMetadataRegion {
    Japan,
    USA,
    Europe,
    RegionFree,
    Korea,
}

impl TitleMetadataRegion {
    fn from_identifier(identifier: u16) -> Result<TitleMetadataRegion, TitleMetadataError> {
        match identifier {
            0 => Ok(TitleMetadataRegion::Japan),
            1 => Ok(TitleMetadataRegion::USA),
            2 => Ok(TitleMetadataRegion::Europe),
            3 => Ok(TitleMetadataRegion::RegionFree),
            4 => Ok(TitleMetadataRegion::Korea),

            identifier => Err(TitleMetadataError::UnknownRegion(identifier)),
        }
    }

    fn dump_identifier<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u16::<BigEndian>(match &self {
            TitleMetadataRegion::Japan => 0,
            TitleMetadataRegion::USA => 1,
            TitleMetadataRegion::Europe => 2,
            TitleMetadataRegion::RegionFree => 3,
            TitleMetadataRegion::Korea => 4,
        })?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct TitleMetadata {
    pub signed_blob_header: SignedBlobHeader,

    pub signature_issuer: String,
    pub tmd_version: u8,

    pub certificate_authority_certificate_revocation_list_version: u8,
    pub signer_certificate_revocation_list_version: u8,

    pub is_vwii_only: bool,

    /// Nothing if the title is an IOS, if not here it'll stored either the title ID of the IOS
    /// needed to run the title or the title ID of the boot2 version stored
    pub ios_or_boot2_title_id: Option<TitleId>,

    pub title_id: TitleId,

    pub platform_kind: TitleMetadataPlatformKind,
    pub group_id: u16,
    pub region: TitleMetadataRegion,

    // TODO(IMPROVE): Discover the format of this
    pub ratings: [u8; 16],

    // TODO(IMPROVE): Discover the format of this
    pub ipc_mask: [u8; 12],

    pub is_dvd_access_allowed: bool,
    pub is_full_ppc_access_allowed: bool,

    pub title_version: u16,
    pub number_of_content_entries: u16,

    /// The index of the content entry where the boot file is located
    pub boot_content_index: u16,

    pub content_entries: Vec<TitleMetadataContentEntry>,
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

    #[error("The given title metadata platform kind is not known: {0}")]
    UnknownPlatformKind(u32),

    #[error("The given title metadata region is not known: {0}")]
    UnknownRegion(u16),

    #[error("The given content entry kind is not known: {0}")]
    UnknownContentEntryKind(u16),
}

impl TitleMetadata {
    /// Create a new installable Wad representation.
    ///
    /// # Safety
    /// The given buffer is assumed to be from an title metadata entry,
    /// the current position of the Seek pointer is taken as the start.
    pub unsafe fn from_reader<T: Read + Seek>(
        reader: &mut T,
    ) -> Result<TitleMetadata, TitleMetadataError> {
        let signed_blob_header = SignedBlobHeader::from_reader(reader)?;

        let mut signature_issuer_bytes = [0; 64];
        reader.read_exact(&mut signature_issuer_bytes)?;

        let signature_issuer = crate::string_from_null_terminated_bytes(&signature_issuer_bytes)?;

        let tmd_version = reader.read_u8()?;
        let certificate_authority_certificate_revocation_list_version = reader.read_u8()?;
        let signer_certificate_revocation_list_version = reader.read_u8()?;

        let is_vwii_only = match reader.read_u8()? {
            0 => false,
            1 => true,

            value => return Err(TitleMetadataError::InvalidIsVWiiValue(value)),
        };

        let ios_or_boot2_title_id_bytes = reader.read_u64::<BigEndian>()?;

        let ios_or_boot2_title_id = if ios_or_boot2_title_id_bytes != 0 {
            Some(TitleId::new(ios_or_boot2_title_id_bytes))
        } else {
            None
        };

        let title_id = TitleId::new(reader.read_u64::<BigEndian>()?);

        let platform_kind =
            TitleMetadataPlatformKind::from_identifier(reader.read_u32::<BigEndian>()?)?;
        let group_id = reader.read_u16::<BigEndian>()?;

        // Skip 2 zeroed bytes
        reader.seek_relative(2)?;

        let region = TitleMetadataRegion::from_identifier(reader.read_u16::<BigEndian>()?)?;

        let mut ratings = [0; 16];
        reader.read_exact(&mut ratings)?;

        // Skip 12 reserved bytes
        reader.seek_relative(12)?;

        let mut ipc_mask = [0; 12];
        reader.read_exact(&mut ipc_mask)?;

        // Skip 18 reserved bytes
        reader.seek_relative(18)?;

        // Skip 3 access rights bytes as they were never used
        reader.seek_relative(3)?;

        let access_rights_byte = reader.read_u8()?;

        let is_full_ppc_access_allowed = (access_rights_byte & 0b00000001) == 1;
        let is_dvd_access_allowed = (access_rights_byte & 0b00000010) >> 1 == 1;

        let title_version = reader.read_u16::<BigEndian>()?;
        let number_of_content_entries = reader.read_u16::<BigEndian>()?;
        let boot_content_index = reader.read_u16::<BigEndian>()?;

        // Skip 2 unused bytes
        reader.seek_relative(2)?;

        let mut content_entries = Vec::new();

        for _ in 0..number_of_content_entries {
            content_entries.push(TitleMetadataContentEntry::from_reader(reader)?);
        }

        Ok(TitleMetadata {
            signed_blob_header,
            signature_issuer,
            tmd_version,
            certificate_authority_certificate_revocation_list_version,
            signer_certificate_revocation_list_version,
            is_vwii_only,
            ios_or_boot2_title_id,
            title_id,
            platform_kind,
            group_id,
            region,
            ratings,
            ipc_mask,
            is_full_ppc_access_allowed,
            is_dvd_access_allowed,
            title_version,
            number_of_content_entries,
            boot_content_index,
            content_entries,
        })
    }

    pub fn dump<T: Write + Seek>(&self, writer: &mut T) -> io::Result<()> {
        self.signed_blob_header.dump(writer)?;
        writer.write_as_c_string_padded(&self.signature_issuer, 64)?;
        writer.write_u8(self.tmd_version)?;
        writer.write_u8(self.certificate_authority_certificate_revocation_list_version)?;
        writer.write_u8(self.signer_certificate_revocation_list_version)?;
        writer.write_bool(self.is_vwii_only)?;

        match &self.ios_or_boot2_title_id {
            None => writer.write_zeroed(1)?,
            Some(title_id) => title_id.dump(writer)?,
        };

        self.title_id.dump(writer)?;
        self.platform_kind.dump(writer)?;
        writer.write_u16::<BigEndian>(self.group_id)?;

        // Skip 2 always zeroed bytes
        writer.write_zeroed(2)?;

        self.region.dump_identifier(writer)?;
        writer.write_all(&self.ratings)?;

        // Skip 12 reserved bytes
        writer.seek_relative(12)?;

        writer.write_all(&self.ipc_mask)?;

        // Skip 18 reserved bytes
        writer.seek_relative(18)?;

        // The first 3 bytes of the access rights were never used
        writer.seek_relative(3)?;

        let mut fouth_access_right_byte = 0;

        if self.is_full_ppc_access_allowed {
            fouth_access_right_byte |= 0b00000001;
        }

        if self.is_dvd_access_allowed {
            fouth_access_right_byte |= 0b00000010;
        }

        writer.write_u8(fouth_access_right_byte)?;
        writer.write_u16::<BigEndian>(self.title_version)?;
        writer.write_u16::<BigEndian>(self.number_of_content_entries)?;
        writer.write_u16::<BigEndian>(self.boot_content_index)?;

        // Skip the title minor version as it was never used
        writer.seek_relative(2)?;

        for content_entry in &self.content_entries {
            content_entry.dump(writer)?;
        }

        Ok(())
    }
}

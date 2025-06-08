mod certificate_chain;
mod content;
mod ticket;
mod title_metadata;

use crate::ticket::PreSwitchTicketError;
use crate::title_metadata::TitleMetadataError;
use byteorder::{BigEndian, ReadBytesExt};
use std::io;
use std::io::Read;
use std::io::Seek;
use thiserror::Error;

#[derive(Debug)]
pub struct InstallableWad {
    pub header_size: u32,
    pub kind: InstallableWadKind,
    pub version: u16,
    pub certificate_chain_size: u32,
    pub ticket_size: u32,
    pub title_metadata_size: u32,
    pub content_size: u32,
    pub footer_size: u32,
}

impl InstallableWad {
    const HEADER_SIZE: u64 = 64;
    const SECTION_BOUNDARY: u64 = 64;
    const NUMBER_OF_CERTIFICATES_STORED: usize = 3;

    fn align(value: u32) -> u64 {
        util::align_to_boundary(value as u64, Self::SECTION_BOUNDARY)
    }

    /// Create a new installable Wad representation.
    ///
    /// # Safety
    /// The given buffer is assumed to be from an installable WAD,
    /// the current position of the Seek pointer is taken as the start.
    pub(crate) fn new<T: Read + Seek>(stream: &mut T) -> Result<Self, InstallableWadError> {
        let header_size = stream.read_u32::<BigEndian>()?;
        let kind = InstallableWadKind::new(stream)?;
        let version = stream.read_u16::<BigEndian>()?;
        let certificate_chain_size = stream.read_u32::<BigEndian>()?;

        // Skip four reserved bytes
        stream.seek_relative(4)?;

        let ticket_size = stream.read_u32::<BigEndian>()?;
        let title_metadata_size = stream.read_u32::<BigEndian>()?;
        let content_size = stream.read_u32::<BigEndian>()?;
        let footer_size = stream.read_u32::<BigEndian>()?;

        Ok(Self {
            header_size,
            kind,
            version,
            certificate_chain_size,
            ticket_size,
            title_metadata_size,
            content_size,
            footer_size,
        })
    }
}

#[derive(Error, Debug)]
pub enum InstallableWadError {
    #[error("An IO error has occurred: {0}")]
    IoError(#[from] io::Error),

    #[error("Unknown installable wad type: {0:?}")]
    UnknownInstallableWadTypeError([u8; 2]),

    #[error("Ticket error: {0}")]
    TicketError(#[from] PreSwitchTicketError),

    #[error("Title metadata error: {0}")]
    TitleMetadataError(#[from] TitleMetadataError),

    #[error("The given content entry index doesn't exist: {0}")]
    ContentEntryIndexDoesntExist(u16),
}

#[derive(Debug)]
pub enum InstallableWadKind {
    Normal,
    Boot2,
}

impl InstallableWadKind {
    fn new<T: Read>(stream: &mut T) -> Result<Self, InstallableWadError> {
        let bytes = util::read_exact!(stream, 2)?;

        Ok(match &bytes {
            b"Is" => Self::Normal,

            b"ib" => Self::Boot2,

            _ => return Err(InstallableWadError::UnknownInstallableWadTypeError(bytes)),
        })
    }
}

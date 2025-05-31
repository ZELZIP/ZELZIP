mod certificate_chain;
mod content;
mod ticket;
mod title_metadata;

use crate::ticket::TicketError;
use crate::title_metadata::TitleMetadataError;
use byteorder::{BigEndian, ReadBytesExt};
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InstallableWadError {
    #[error("An IO error has occurred: {0}")]
    IoError(#[from] io::Error),

    #[error("Unknown installable wad type: {0:?}")]
    UnknownInstallableWadTypeError([u8; 2]),

    #[error("Ticket error: {0}")]
    TicketError(#[from] TicketError),

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
    fn from_bytes(bytes: [u8; 2]) -> Result<InstallableWadKind, InstallableWadError> {
        Ok(match &bytes {
            b"Is" => InstallableWadKind::Normal,

            b"ib" => InstallableWadKind::Boot2,

            _ => return Err(InstallableWadError::UnknownInstallableWadTypeError(bytes)),
        })
    }
}

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
        crate::align_to_boundary(value as u64, InstallableWad::SECTION_BOUNDARY)
    }

    /// Create a new installable Wad representation.
    ///
    /// # Safety
    /// The given buffer is assumed to be from an installable WAD,
    /// the current position of the Seek pointer is taken as the start.
    pub(crate) unsafe fn from_reader<T: Read + Seek>(
        buffer: &mut T,
    ) -> Result<InstallableWad, InstallableWadError> {
        let header_size = buffer.read_u32::<BigEndian>()?;

        let mut kind_bytes = [0; 2];
        buffer.read_exact(&mut kind_bytes)?;
        let kind = InstallableWadKind::from_bytes(kind_bytes)?;

        let version = buffer.read_u16::<BigEndian>()?;
        let certificate_chain_size = buffer.read_u32::<BigEndian>()?;

        // Skip four reserved bytes
        buffer.seek_relative(4)?;

        let ticket_size = buffer.read_u32::<BigEndian>()?;
        let title_metadata_size = buffer.read_u32::<BigEndian>()?;
        let content_size = buffer.read_u32::<BigEndian>()?;
        let footer_size = buffer.read_u32::<BigEndian>()?;

        Ok(InstallableWad {
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

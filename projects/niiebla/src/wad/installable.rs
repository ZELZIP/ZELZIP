//! Implementation of a installable WAD file.

mod certificate_chain;
mod content;
mod ticket;
mod title_metadata;

use crate::ticket::PreSwitchTicketError;
use crate::title_metadata::TitleMetadataError;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use thiserror::Error;
use util::StreamPin;
use util::WriteEx;

#[derive(Debug)]
pub struct InstallableWad {
    pub header_size: u32,
    pub kind: InstallableWadKind,
    pub version: u16,
    pub certificate_chain_size: u32,
    pub ticket_size: u32,
    pub title_metadata_size: u32,
    pub content_size: u32,
    // TODO(IMPLEMENT): Support for footer info.
    pub footer_size: u32,
}

impl InstallableWad {
    const HEADER_SIZE: u64 = 64;
    const SECTION_BOUNDARY: u64 = 64;
    const NUMBER_OF_CERTIFICATES_STORED: usize = 3;

    fn align_u64(value: u32) -> u64 {
        util::align_to_boundary(value as u64, Self::SECTION_BOUNDARY)
    }

    /// Create a new installable Wad representation.
    ///
    /// # Safety
    /// The given buffer is assumed to be from an installable WAD.
    pub(crate) unsafe fn new<T: Read + Seek>(mut stream: T) -> Result<Self, InstallableWadError> {
        let header_size = stream.read_u32::<BigEndian>()?;
        let kind = InstallableWadKind::new(&mut stream)?;
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

    /// Dump into a stream.
    pub fn dump<T: Write + Seek>(&self, stream: T) -> io::Result<()> {
        let mut stream = StreamPin::new(stream)?;

        stream.write_u32::<BigEndian>(32)?;
        write!(stream, "Is")?;
        stream.write_u16::<BigEndian>(0)?;
        stream.write_u32::<BigEndian>(self.certificate_chain_size)?;
        stream.write_zeroed(4)?;

        stream.write_u32::<BigEndian>(self.ticket_size)?;
        stream.write_u32::<BigEndian>(self.title_metadata_size)?;
        stream.write_u32::<BigEndian>(self.content_size)?;
        stream.write_u32::<BigEndian>(self.footer_size)?;
        stream.align_zeroed(64)?;

        Ok(())
    }
}

#[derive(Error, Debug)]
#[allow(missing_docs)]
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
    fn new<T: Read>(mut stream: T) -> Result<Self, InstallableWadError> {
        let bytes = util::read_exact!(stream, 2)?;

        Ok(match &bytes {
            b"Is" => Self::Normal,

            b"ib" => Self::Boot2,

            _ => return Err(InstallableWadError::UnknownInstallableWadTypeError(bytes)),
        })
    }
}

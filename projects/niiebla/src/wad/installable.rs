//! Implementation of a installable WAD file.

mod certificate_chain;
mod content;
mod ticket;
mod title_metadata;

use crate::ContentSelector;
use crate::TitleMetadata;
use crate::certificate_chain::CertificateChainError;
use crate::ticket::PreSwitchTicketError;
use crate::title_metadata::TitleMetadataError;
use byteorder::{BE, ReadBytesExt, WriteBytesExt};
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
    pub footer_size: u32,
}

struct ContentsStore {
    contents: Vec<Vec<u8>>,
    first_content_physical_position: usize,
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
        let header_size = stream.read_u32::<BE>()?;
        let kind = InstallableWadKind::new(&mut stream)?;
        let version = stream.read_u16::<BE>()?;
        let certificate_chain_size = stream.read_u32::<BE>()?;

        // Skip four reserved bytes
        stream.seek_relative(4)?;

        let ticket_size = stream.read_u32::<BE>()?;
        let title_metadata_size = stream.read_u32::<BE>()?;
        let content_size = stream.read_u32::<BE>()?;
        let footer_size = stream.read_u32::<BE>()?;

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

        stream.write_u32::<BE>(32)?;
        write!(stream, "Is")?;
        stream.write_u16::<BE>(0)?;
        stream.write_u32::<BE>(self.certificate_chain_size)?;
        stream.write_zeroed(4)?;

        stream.write_u32::<BE>(self.ticket_size)?;
        stream.write_u32::<BE>(self.title_metadata_size)?;
        stream.write_u32::<BE>(self.content_size)?;
        stream.write_u32::<BE>(self.footer_size)?;
        stream.align_zeroed(64)?;

        Ok(())
    }

    fn store_contents<T: Read + Write + Seek>(
        &mut self,
        mut stream: T,
        title_metadata: &TitleMetadata,
        first_content_physical_position: usize,
    ) -> Result<ContentsStore, InstallableWadError> {
        let mut all_contents_bytes = vec![];

        if title_metadata.content_chunk_entries.len() == 0 {
            return Ok(ContentsStore {
                contents: all_contents_bytes,
                first_content_physical_position,
            });
        }

        for i in first_content_physical_position..title_metadata.content_chunk_entries.len() {
            let mut view = self.encrypted_content_view(
                &mut stream,
                title_metadata,
                title_metadata.select_with_physical_position(i),
            )?;

            let mut content_bytes = vec![];
            view.read_to_end(&mut content_bytes)?;
            all_contents_bytes.push(content_bytes);
        }

        Ok(ContentsStore {
            contents: all_contents_bytes,
            first_content_physical_position,
        })
    }

    fn restore_contents<T: Write + Read + Seek>(
        &mut self,
        stream: &mut StreamPin<T>,
        title_metadata: &TitleMetadata,
        contents_store: &ContentsStore,
    ) -> Result<(), InstallableWadError> {
        self.seek_content(
            &mut *stream,
            title_metadata,
            title_metadata
                .select_with_physical_position(contents_store.first_content_physical_position),
        )?;

        for bytes in &contents_store.contents {
            stream.write_all(&bytes)?;
            stream.align_zeroed(Self::SECTION_BOUNDARY)?;
        }

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

    #[error("Certificate chain error: {0}")]
    CertificateChainError(#[from] CertificateChainError),

    #[error("The given title is not for the Wii platform")]
    NotAWiiTitle,

    #[error("Missing a to modify this content: {0}")]
    ModifyContentMissingSetting(&'static str),
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

use crate::ticket::{Ticket, TicketError};
use crate::certificate_chain::{CertificateChain, CertificateChainError};
use byteorder::{BigEndian, ReadBytesExt};
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InstallableWadError {
    #[error("An IO error has occurred: {0}")]
    IoError(#[from] io::Error),

    #[error("Unknown installable wad type: {0:?}")]
    UnknownInstallableWadTypeError([u8; 2]),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum InstallableWadKind {
    Normal,
    Boot2,
}

impl InstallableWadKind {
    fn from_bytes(bytes: [u8; 2]) -> Result<InstallableWadKind, InstallableWadError> {
        Ok(match &bytes {
            b"Is" => InstallableWadKind::Normal,

            b"ib" => InstallableWadKind::Boot2,

            _ => {
                return Err(InstallableWadError::UnknownInstallableWadTypeError(
                    bytes,
                ))
            }
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
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
        buffer.seek(SeekFrom::Current(4))?;

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

    pub fn certificate_chain<T: Read + Seek>(&self, reader: &mut T) -> Result<CertificateChain, CertificateChainError> {
        reader.seek(SeekFrom::Start(InstallableWad::HEADER_SIZE))?;

        Ok(unsafe { CertificateChain::from_reader(reader)?})
    }

    pub fn ticket<T: Read + Seek>(
        &self,
        reader: &mut T,
    ) -> Result<Ticket, TicketError> {
        let ticket_offset = 
            // The header is always aligned to the boundary
            InstallableWad::HEADER_SIZE
            + crate::align_offset(self.certificate_chain_size as u64, InstallableWad::SECTION_BOUNDARY);

        reader.seek(SeekFrom::Start(ticket_offset))?;

        Ok(unsafe { Ticket::from_reader(reader)? })
    }
}

use crate::wad::installable::{InstallableWad, InstallableWadError};
use aes::cipher::KeyIvInit;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use util::{Aes128CbcDec, AesCbcStream};
use util::{StreamPin, View};

// TODO(IMPLEMENT): Content iterator.

impl InstallableWad {
    pub fn seek_content<T: Read + Seek>(
        &self,
        mut stream: T,
        physical_position: usize,
    ) -> Result<(), InstallableWadError> {
        // The header is always aligned to the boundary
        let mut content_offset = Self::HEADER_SIZE
            + Self::align_u64(self.certificate_chain_size)
            + Self::align_u64(self.ticket_size)
            + Self::align_u64(self.title_metadata_size);

        let title_metadata = self.title_metadata(&mut stream)?;

        for (index, content_entry) in title_metadata.content_chunk_entries.iter().enumerate() {
            if index == physical_position {
                stream.seek(SeekFrom::Start(content_offset))?;
                return Ok(());
            }

            content_offset += util::align_to_boundary(content_entry.size, Self::SECTION_BOUNDARY);
        }

        Err(InstallableWadError::ContentEntryPhysicalPositionDoesntExist(physical_position))
    }

    pub fn encrypted_content_view<T: Read + Seek>(
        &self,
        mut stream: T,
        physical_position: usize,
    ) -> Result<View<T>, InstallableWadError> {
        // The header is always aligned to the boundary
        let mut content_offset = Self::HEADER_SIZE
            + Self::align_u64(self.certificate_chain_size)
            + Self::align_u64(self.ticket_size)
            + Self::align_u64(self.title_metadata_size);

        let title_metadata = self.title_metadata(&mut stream)?;

        self.seek_content(&mut stream, physical_position)?;

        Ok(View::new(
            stream,
            title_metadata.content_chunk_entries[physical_position].size as usize,
        )?)
    }

    // TODO: Change name to something more meaningful and use physical position instead of index.
    /// Get a decryption stream of a content blob.
    pub fn decrypted_content_view<T: Read + Seek>(
        &self,
        mut stream: T,
        physical_position: usize,
    ) -> Result<AesCbcStream<View<T>>, InstallableWadError> {
        let ticket = self.ticket(&mut stream)?;
        let title_metadata = self.title_metadata(&mut stream)?;

        let content_view = self.encrypted_content_view(stream, physical_position)?;

        Ok(ticket.cryptographic_stream_wii_method(
            content_view,
            title_metadata.content_chunk_entries[physical_position].index,
        )?)
    }

    /// Write a content given its `physical_position` (its position inside the WAD file itself).
    /// Data after this content may be unaligned or overwritten. Using [[Self::write_content_safe]]
    /// may be preferred.
    pub fn write_content_raw<T: Read, S: Read + Write + Seek>(
        &mut self,
        mut new_data: T,
        stream: S,
        physical_position: usize,
        new_index: Option<u16>,
        new_id: Option<u32>,
    ) -> Result<(), InstallableWadError> {
        let mut stream = StreamPin::new(stream)?;

        let ticket = self.ticket(&mut stream)?;
        let mut new_data_loaded = vec![];
        new_data.read_to_end(&mut new_data_loaded)?;

        let mut title_metadata = self.title_metadata(&mut stream)?;
        let content_chunk_entry = &mut title_metadata.content_chunk_entries[physical_position];

        if let Some(index) = new_index {
            content_chunk_entry.index = index;
        }

        if let Some(id) = new_id {
            content_chunk_entry.id = id;
        }

        content_chunk_entry.size = new_data_loaded.len() as u64;
        // TODO(IMPLEMENT): Calculate hash for new content.

        self.write_title_metadata(&title_metadata, &mut stream)?;

        self.seek_content(&mut stream, physical_position)?;

        let mut stream = ticket.cryptographic_stream_wii_method(
            stream,
            title_metadata.content_chunk_entries[physical_position].index,
        )?;

        stream.write(&new_data_loaded)?;

        stream.into_inner().align_zeroed(64)?;

        Ok(())
    }

    /// Like [Self::write_content_raw] but will make a in-memory copy off all the trailing data to
    /// realign it.
    pub fn write_content_safe<T: Read, S: Read + Seek>(
        &mut self,
        mut new_data: T,
        stream: S,
        physical_position: usize,
        new_index: Option<u16>,
        new_id: Option<u32>,
    ) -> Result<(), InstallableWadError> {
        // TODO: NEXT TO IMPLEMENT.

        Ok(())
    }

    /// Like [Self::write_content_safe] but will also trim the size of the file to avoid garbage
    /// data or useless zeroes.
    pub fn write_content_safe_file<T: Read>(
        &mut self,
        mut new_data: T,
        file: File,
        physical_position: usize,
        new_index: Option<u16>,
        new_id: Option<u32>,
    ) -> Result<(), InstallableWadError> {
        // TODO: NEXT TO IMPLEMENT.

        Ok(())
    }
}

use crate::title_metadata::TitleMetadataContentEntryHashKind;
use crate::wad::installable::{InstallableWad, InstallableWadError};
use crate::{PreSwitchTicket, TitleMetadata};
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use util::AesCbcStream;
use util::{StreamPin, View, WriteEx};

impl InstallableWad {
    /// Seek the stream of the WAD to the start of the desired content.
    pub fn seek_content<T: Read + Seek>(
        &self,
        mut stream: T,
        title_metadata: &TitleMetadata,
        physical_position: usize,
    ) -> Result<(), InstallableWadError> {
        // The header is always aligned to the boundary
        let mut content_offset = Self::HEADER_SIZE
            + Self::align_u64(self.certificate_chain_size)
            + Self::align_u64(self.ticket_size)
            + Self::align_u64(self.title_metadata_size);

        for (index, content_entry) in title_metadata.content_chunk_entries.iter().enumerate() {
            if index == physical_position {
                stream.seek(SeekFrom::Start(content_offset))?;
                return Ok(());
            }

            content_offset += util::align_to_boundary(content_entry.size, Self::SECTION_BOUNDARY);
        }

        Err(InstallableWadError::ContentEntryPhysicalPositionDoesntExist(physical_position))
    }

    /// Create a [View] into the desired content stored inside the WAD stream. Be aware that the
    /// stream will be only of encrypted data, [Self::decrypted_content_view] may be prefered.
    pub fn encrypted_content_view<T: Read + Seek>(
        &self,
        mut stream: T,
        title_metadata: &TitleMetadata,
        physical_position: usize,
    ) -> Result<View<T>, InstallableWadError> {
        self.seek_content(&mut stream, title_metadata, physical_position)?;

        Ok(View::new(
            stream,
            title_metadata.content_chunk_entries[physical_position].size as usize,
        )?)
    }

    /// Create a [View] into the desired content stored inside the WAD stream. Decryption is done
    /// in place, be aware that **zero caching is implemented on the [AesCbcStream] type, wrapping
    /// the stream on a [std::io::BufReader] may be useful.
    pub fn decrypted_content_view<T: Read + Seek>(
        &self,
        stream: T,
        ticket: &PreSwitchTicket,
        title_metadata: &TitleMetadata,
        physical_position: usize,
    ) -> Result<AesCbcStream<View<T>>, InstallableWadError> {
        let content_view =
            self.encrypted_content_view(stream, title_metadata, physical_position)?;

        Ok(ticket.cryptographic_stream_wii_method(
            content_view,
            title_metadata.content_chunk_entries[physical_position].index,
        )?)
    }

    /// Write a content given its `physical_position` (its position inside the WAD file itself)
    /// The `title_metadata` entry will be updated with the correct hash and size and
    /// change the index and/or ID if requested.
    ///
    /// # Safety
    /// Data after this content may be unaligned or overwritten. Using
    /// [Self::write_content_safe_wii] or [Self::write_content_safe_file_wii]
    /// may be preferred.
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn write_content_raw_wii<T: Read, S: Read + Write + Seek>(
        &mut self,
        mut new_data: T,
        stream: S,
        ticket: &PreSwitchTicket,
        title_metadata: &mut TitleMetadata,
        physical_position: usize,
        new_index: Option<u16>,
        new_id: Option<u32>,
    ) -> Result<(), InstallableWadError> {
        let mut stream = StreamPin::new(stream)?;

        let mut new_data_vec = vec![];
        new_data.read_to_end(&mut new_data_vec)?;
        let mut new_data = new_data_vec;

        let content_chunk_entry = &mut title_metadata.content_chunk_entries[physical_position];

        if let Some(index) = new_index {
            content_chunk_entry.index = index;
        }

        if let Some(id) = new_id {
            content_chunk_entry.id = id;
        }

        content_chunk_entry.size = new_data.len() as u64;

        content_chunk_entry.hash = match content_chunk_entry.hash {
            TitleMetadataContentEntryHashKind::Version0(_) => {
                TitleMetadataContentEntryHashKind::Version0(Sha1::digest(&new_data).into())
            }
            TitleMetadataContentEntryHashKind::Version1(_) => {
                return Err(InstallableWadError::NotAWiiTitle);
            }
        };

        new_data.write_zeroed(
            (util::align_to_boundary(content_chunk_entry.size, 16) - content_chunk_entry.size)
                as usize,
        )?;

        self.write_title_metadata_raw(title_metadata, &mut stream)?;

        self.seek_content(&mut stream, title_metadata, physical_position)?;

        let mut stream = ticket.cryptographic_stream_wii_method(
            stream,
            title_metadata.content_chunk_entries[physical_position].index,
        )?;

        stream.write(&new_data)?;

        stream.into_inner().align_zeroed(64)?;

        Ok(())
    }

    /// Like [Self::write_content_raw_wii] but will make a in-memory copy off all the trailing data to
    /// realign it.
    #[allow(clippy::too_many_arguments)]
    pub fn write_content_safe_wii<T: Read, S: Read + Write + Seek>(
        &mut self,
        new_data: T,
        stream: S,
        ticket: &PreSwitchTicket,
        title_metadata: &mut TitleMetadata,
        physical_position: usize,
        new_index: Option<u16>,
        new_id: Option<u32>,
    ) -> Result<(), InstallableWadError> {
        let mut stream = StreamPin::new(stream)?;

        let mut trailing_content_bytes = vec![];

        for position in physical_position + 1..title_metadata.content_chunk_entries.len() {
            let mut content_view =
                self.encrypted_content_view(&mut stream, title_metadata, position)?;

            let mut content_bytes = vec![];
            content_view.read_to_end(&mut content_bytes)?;

            trailing_content_bytes.push(content_bytes);
        }

        unsafe {
            self.write_content_raw_wii(
                new_data,
                &mut stream,
                ticket,
                title_metadata,
                physical_position,
                new_index,
                new_id,
            )?;
        }

        for content_bytes in trailing_content_bytes {
            stream.write_all(&content_bytes)?;
            stream.align_zeroed(64)?;
        }

        Ok(())
    }

    /// Like [Self::write_content_safe_wii] but will also trim the size of the file to avoid garbage
    /// data or useless zeroes.
    #[allow(clippy::too_many_arguments)]
    pub fn write_content_safe_file_wii<T: Read>(
        &mut self,
        new_data: T,
        wad_file: &mut File,
        ticket: &PreSwitchTicket,
        title_metadata: &mut TitleMetadata,
        physical_position: usize,
        new_index: Option<u16>,
        new_id: Option<u32>,
    ) -> Result<(), InstallableWadError> {
        let mut file = StreamPin::new(wad_file)?;

        self.write_content_safe_wii(
            new_data,
            &mut file,
            ticket,
            title_metadata,
            physical_position,
            new_index,
            new_id,
        )?;

        let last_content_physical_position = title_metadata.content_chunk_entries.len() - 1;
        let last_content_entry =
            &title_metadata.content_chunk_entries[last_content_physical_position];

        self.seek_content(&mut file, title_metadata, last_content_physical_position)?;
        file.seek_relative(last_content_entry.size as i64)?;
        file.align_position(64)?;

        let len = file.stream_position()?;

        file.into_inner().set_len(len)?;

        Ok(())
    }
}

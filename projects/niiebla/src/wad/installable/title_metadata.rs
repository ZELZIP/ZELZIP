use crate::title_metadata::{TitleMetadata, TitleMetadataError};
use crate::wad::InstallableWad;
use crate::wad::InstallableWadError;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use util::{StreamPin, View};

impl InstallableWad {
    /// Seek the stream of the WAD to the start of the title metadata.
    pub fn seek_title_metadata<T: Seek>(&self, mut stream: T) -> Result<(), TitleMetadataError> {
        // The header is always aligned to the boundary
        let title_metadata_offset = Self::HEADER_SIZE
            + Self::align_u64(self.certificate_chain_size)
            + Self::align_u64(self.ticket_size);

        stream.seek(SeekFrom::Start(title_metadata_offset))?;
        Ok(())
    }

    /// Crate a [View] into the title metadata stored inside the WAD stream.
    pub fn title_metadata_view<T: Read + Seek>(
        &self,
        mut stream: T,
    ) -> Result<View<T>, TitleMetadataError> {
        self.seek_title_metadata(&mut stream)?;

        Ok(View::new(stream, self.title_metadata_size as usize)?)
    }

    /// Parse the title metadata stored inside the WAD stream.
    pub fn title_metadata<T: Read + Seek>(
        &self,
        mut stream: T,
    ) -> Result<TitleMetadata, TitleMetadataError> {
        self.seek_title_metadata(&mut stream)?;

        TitleMetadata::new(&mut stream)
    }

    /// Write a new title metadata into the stream of a WAD.
    ///
    /// # Safety
    /// Data after the title metedata (the content blobs) may be unaligned or overwritten. Using
    /// [Self::write_title_metadata_safe] or [Self::write_title_metadata_safe_file]
    /// may be preferred.
    pub unsafe fn write_title_metadata_raw<T: Write + Seek>(
        &mut self,
        new_title_metadata: &TitleMetadata,
        stream: T,
    ) -> Result<(), TitleMetadataError> {
        let mut stream = StreamPin::new(stream)?;

        self.seek_title_metadata(&mut stream)?;

        new_title_metadata.dump(&mut stream)?;
        stream.align_zeroed(64)?;

        self.title_metadata_size = new_title_metadata.size();

        stream.rewind()?;
        self.dump(stream)?;

        Ok(())
    }

    /// Like [Self::write_title_metadata_raw] but will make a in-memory copy off all the trailing data to
    /// realign it.
    ///
    /// Be aware that the given new title metadata should have cohesion with the stored content
    /// blobs.
    pub fn write_title_metadata_safe<T: Read + Write + Seek>(
        &mut self,
        mut stream: T,
        new_title_metadata: &TitleMetadata,
    ) -> Result<(), InstallableWadError> {
        let mut stream = StreamPin::new(stream)?;

        let contents = self.store_contents(&mut stream, new_title_metadata, 0)?;

        unsafe {
            self.write_title_metadata_raw(new_title_metadata, &mut stream)?;
        }

        self.restore_contents(&mut stream, new_title_metadata, &contents);

        Ok(())
    }

    /// Like [Self::write_ticket_safe] but will also trim the size of the file to avoid garbage
    /// data or useless zeroes.
    ///
    /// Be aware that the given new title metadata should have cohesion with the stored content
    /// blobs.
    pub fn write_title_metadata_safe_file(
        &mut self,
        file: &mut File,
        new_title_metadata: &TitleMetadata,
    ) -> Result<(), InstallableWadError> {
        self.write_title_metadata_safe(&mut *file, new_title_metadata)?;

        let new_file_size = file.stream_position()?;
        file.set_len(new_file_size)?;

        Ok(())
    }
}

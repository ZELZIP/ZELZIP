use crate::title_metadata::{TitleMetadata, TitleMetadataError};
use crate::wad::InstallableWad;
use std::io::{Read, Seek, SeekFrom, Write};
use util::View;

impl InstallableWad {
    pub fn seek_title_metadata<T: Seek>(&self, stream: &mut T) -> Result<(), TitleMetadataError> {
        // The header is always aligned to the boundary
        let title_metadata_offset = Self::HEADER_SIZE
            + Self::align(self.certificate_chain_size)
            + Self::align(self.ticket_size);

        stream.seek(SeekFrom::Start(title_metadata_offset))?;
        Ok(())
    }

    pub fn title_metadata_view<T: Read + Seek>(
        &self,
        mut stream: T,
    ) -> Result<View<T>, TitleMetadataError> {
        self.seek_title_metadata(&mut stream)?;

        Ok(View::new(stream, self.title_metadata_size as usize)?)
    }

    pub fn title_metadata<T: Read + Seek>(
        &self,
        stream: &mut T,
    ) -> Result<TitleMetadata, TitleMetadataError> {
        self.seek_title_metadata(stream)?;

        TitleMetadata::new(stream)
    }

    pub fn write_title_metadata<T: Write + Seek>(
        &mut self,
        new_title_metadata: &TitleMetadata,
        stream: &mut T,
    ) -> Result<(), TitleMetadataError> {
        self.seek_title_metadata(stream)?;

        // TODO(IMPROVE): The size of the TMD should change when the number of cntent entries
        // changes
        new_title_metadata.dump(stream)?;

        Ok(())
    }
}

use crate::title_metadata::{TitleMetadata, TitleMetadataError};
use crate::wad::InstallableWad;
use std::io::{Read, Seek, SeekFrom, Write};
use takes::{Ext, Takes};

impl InstallableWad {
    pub fn seek_title_metadata<T: Seek>(&self, stream: &mut T) -> Result<(), TitleMetadataError> {
        // The header is always aligned to the boundary
        let title_metadata_offset = Self::HEADER_SIZE
            + Self::align(self.certificate_chain_size)
            + Self::align(self.ticket_size);

        stream.seek(SeekFrom::Start(title_metadata_offset))?;
        Ok(())
    }

    pub fn take_title_metadata<'a, T: Read + Seek>(
        &self,
        reader: &'a mut T,
    ) -> Result<Takes<&'a mut T>, TitleMetadataError> {
        self.seek_title_metadata(reader)?;

        Ok(reader.takes(self.title_metadata_size as u64)?)
    }

    pub fn title_metadata<T: Read + Seek>(
        &self,
        reader: &mut T,
    ) -> Result<TitleMetadata, TitleMetadataError> {
        self.seek_title_metadata(reader)?;

        Ok(unsafe { TitleMetadata::from_reader(reader)? })
    }

    pub fn write_title_metadata<W: Write + Seek>(
        &mut self,
        new_title_metadata: &TitleMetadata,
        writer: &mut W,
    ) -> Result<(), TitleMetadataError> {
        self.seek_title_metadata(writer)?;

        // TODO(IMPROVE): The size of the TMD should change when the number of cntent entries
        // changes
        new_title_metadata.dump(writer)?;

        Ok(())
    }
}

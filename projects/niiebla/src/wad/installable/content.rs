use crate::ContentSelector;
use crate::CryptographicMethod;
use crate::title_metadata::{
    TitleMetadataContentEntry, TitleMetadataContentEntryHashKind, TitleMetadataContentEntryKind,
};
use crate::wad::installable::{InstallableWad, InstallableWadError};
use crate::{PreSwitchTicket, TitleMetadata};
use sha1::{Digest, Sha1};
use sha2::Sha256;
use std::any::Any;
use std::fs::File;
use std::io;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use util::AesCbcStream;
use util::{StreamPin, View, WriteEx};

impl InstallableWad {
    /// Seek the stream of the WAD to the start of the desired content.
    pub fn seek_content<T: Read + Seek>(
        &self,
        mut stream: T,
        title_metadata: &TitleMetadata,
        selector: ContentSelector,
    ) -> Result<(), InstallableWadError> {
        // The header is always aligned to the boundary
        let mut content_offset = Self::HEADER_SIZE
            + Self::align_u64(self.certificate_chain_size)
            + Self::align_u64(self.ticket_size)
            + Self::align_u64(self.title_metadata_size);

        let position = selector.physical_position(title_metadata)?;

        for (i, content_entry) in title_metadata.content_chunk_entries.iter().enumerate() {
            if i == position {
                stream.seek(SeekFrom::Start(content_offset))?;
                return Ok(());
            }

            content_offset += util::align_to_boundary(content_entry.size, Self::SECTION_BOUNDARY);
        }

        unreachable!()
    }

    /// Create a [View] into the desired content stored inside the WAD stream. Be aware that the
    /// stream will be only of the encrypted data, [Self::decrypted_content_view] may be prefered.
    pub fn encrypted_content_view<T: Read + Seek>(
        &self,
        mut stream: T,
        title_metadata: &TitleMetadata,
        selector: ContentSelector,
    ) -> Result<View<T>, InstallableWadError> {
        self.seek_content(&mut stream, title_metadata, selector)?;
        let entry = selector.content_entry(title_metadata)?;

        Ok(View::new(stream, entry.size as usize)?)
    }

    /// Create a [View] into the desired content stored inside the WAD stream. Decryption is done
    /// in place, be aware that **zero caching is implemented on the [AesCbcStream] type, wrapping
    /// the stream on a [std::io::BufReader] may be useful.
    pub fn decrypted_content_view<T: Read + Seek>(
        &self,
        stream: T,
        ticket: &PreSwitchTicket,
        title_metadata: &TitleMetadata,
        cryptographic_method: CryptographicMethod,
        selector: ContentSelector,
    ) -> Result<AesCbcStream<View<T>>, InstallableWadError> {
        let content_view = self.encrypted_content_view(stream, title_metadata, selector)?;

        Ok(ticket.cryptographic_stream(
            content_view,
            title_metadata,
            selector,
            cryptographic_method,
        )?)
    }

    pub fn modify_content<'a, 'b, 'c, T: Read + Write + Seek + Any + Sized>(
        &'a mut self,
        stream: &'b mut T,
    ) -> ModifyContentBuilder<'a, 'b, 'c, T> {
        ModifyContentBuilder {
            wad: self,
            wad_stream: stream,
            new_id: None,
            new_index: None,
            new_kind: None,
            ticket: None,
            cryptographic_method: None,
            trim_if_is_file: false,
            safe: false,
        }
    }
}

enum ModifyContentBuilderAction {
    Replace(ContentSelector),
    Add,
    Delete(ContentSelector),
}

pub struct ModifyContentBuilder<'a, 'b, 'c, T: Read + Write + Seek + Any> {
    wad: &'a mut InstallableWad,
    wad_stream: &'b mut T,
    new_id: Option<u32>,
    new_index: Option<u16>,
    new_kind: Option<TitleMetadataContentEntryKind>,
    ticket: Option<&'c PreSwitchTicket>,
    cryptographic_method: Option<CryptographicMethod>,
    trim_if_is_file: bool,
    safe: bool,
}

impl<'c, T: Read + Write + Seek + Any> ModifyContentBuilder<'_, '_, 'c, T> {
    pub fn crypotgraphy(
        &mut self,
        ticket: &'c PreSwitchTicket,
        crytographic_method: CryptographicMethod,
    ) -> &mut Self {
        self.ticket = Some(ticket);
        self.cryptographic_method = Some(crytographic_method);

        self
    }

    pub fn set_id(&mut self, id: u32) -> &mut Self {
        self.new_id = Some(id);

        self
    }

    pub fn set_index(&mut self, index: u16) -> &mut Self {
        self.new_index = Some(index);

        self
    }

    pub fn set_kind(&mut self, kind: TitleMetadataContentEntryKind) -> &mut Self {
        self.new_kind = Some(kind);

        self
    }

    pub fn trim_if_file(&mut self, flag: bool) -> &mut Self {
        self.trim_if_is_file = flag;

        self
    }

    pub fn safe(&mut self, flag: bool) -> &mut Self {
        self.safe = flag;

        self
    }

    pub fn replace<S: Read + Write + Seek>(
        &mut self,
        new_data: S,
        content_selector: ContentSelector,
        title_metadata: &mut TitleMetadata,
    ) -> Result<(), InstallableWadError> {
        Ok(())
    }

    pub fn add<S: Read + Write + Seek>(
        &mut self,
        mut new_data: S,
        title_metadata: &mut TitleMetadata,
    ) -> Result<(), InstallableWadError> {
        // TODO ADD LOGGING (INFO), ADD IS ALWAYS A SAFE OPERATION

        // TODO: Change this
        let id = self.new_id.unwrap();
        let index = self.new_index.unwrap();
        let kind = self.new_kind.unwrap();

        let ticket = self.ticket.unwrap();
        let cryptographic_method = self.cryptographic_method.unwrap();

        let mut wad_stream = StreamPin::new(&mut self.wad_stream)?;
        let content_selector = title_metadata.select_last();

        let mut new_data_vec = vec![];
        new_data.read_to_end(&mut new_data_vec);

        let hash = if title_metadata.version_1_extension.is_some() {
            TitleMetadataContentEntryHashKind::Version0(Sha1::digest(&new_data_vec).into())
        } else {
            TitleMetadataContentEntryHashKind::Version1(Sha256::digest(&new_data_vec).into())
        };

        let entry = TitleMetadataContentEntry {
            id,
            index,
            kind,
            hash,
            size: new_data_vec.len() as u64,
        };

        title_metadata.content_chunk_entries.push(entry);

        self.wad
            .write_title_metadata_safe(&mut wad_stream, title_metadata)?;

        self.wad
            .seek_content(&mut wad_stream, title_metadata, content_selector)?;

        wad_stream.seek_relative(content_selector.content_entry(title_metadata)?.size as i64);
        wad_stream.align_position(InstallableWad::SECTION_BOUNDARY)?;

        let mut wad_stream = ticket.cryptographic_stream(
            &mut wad_stream,
            title_metadata,
            content_selector,
            cryptographic_method,
        )?;

        wad_stream.write(&mut new_data_vec)?;

        Ok(())
    }

    pub fn delete(
        &mut self,
        content_selector: ContentSelector,
        title_metadata: &mut TitleMetadata,
    ) -> Result<(), InstallableWadError> {
        Ok(())
    }
}

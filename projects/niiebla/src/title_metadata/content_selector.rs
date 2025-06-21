use crate::CryptographicMethod;
use crate::title_metadata::TitleMetadataError;
use crate::title_metadata::{
    TitleMetadataContentEntry, TitleMetadataContentEntryHashKind, TitleMetadataContentEntryKind,
};
use crate::wad::installable::{InstallableWad, InstallableWadError};
use crate::{PreSwitchTicket, TitleMetadata};
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use util::AesCbcStream;
use util::{StreamPin, View, WriteEx};

#[derive(Clone, Copy)]
pub struct ContentSelector {
    pub(super) method: ContentSelectorMethod,
}

#[derive(Clone, Copy)]
pub(super) enum ContentSelectorMethod {
    WithPhysicalPosition(usize),
    WithIndex(u16),
    WithId(u32),
}

impl ContentSelector {
    pub fn content_entry(
        &self,
        title_metadata: &TitleMetadata,
    ) -> Result<TitleMetadataContentEntry, TitleMetadataError> {
        (match self.method {
            ContentSelectorMethod::WithPhysicalPosition(pos) => {
                Some(title_metadata.content_chunk_entries[pos].clone())
            }

            ContentSelectorMethod::WithId(id) => title_metadata
                .content_chunk_entries
                .iter()
                .find(|entry| entry.id == id)
                .cloned(),

            ContentSelectorMethod::WithIndex(index) => title_metadata
                .content_chunk_entries
                .iter()
                .find(|entry| entry.index == index)
                .cloned(),
        })
        .ok_or_else(TitleMetadataError::ContentNotFound)
    }

    pub fn physical_position(
        &self,
        title_metadata: &TitleMetadata,
    ) -> Result<usize, TitleMetadataError> {
        (match self.method {
            ContentSelectorMethod::WithPhysicalPosition(pos) => Some(pos),

            ContentSelectorMethod::WithId(id) => title_metadata
                .content_chunk_entries
                .iter()
                .position(|entry| entry.id == id),

            ContentSelectorMethod::WithIndex(index) => title_metadata
                .content_chunk_entries
                .iter()
                .position(|entry| entry.index == index),
        })
        .ok_or_else(TitleMetadataError::ContentNotFound)
    }

    pub fn id(&self, title_metadata: &TitleMetadata) -> Result<u32, TitleMetadataError> {
        Ok(match self.method {
            ContentSelectorMethod::WithId(id) => id,

            ContentSelectorMethod::WithPhysicalPosition(_)
            | ContentSelectorMethod::WithIndex(_) => self.content_entry(title_metadata)?.id,
        })
    }

    pub fn index(&self, title_metadata: &TitleMetadata) -> Result<u16, TitleMetadataError> {
        Ok(match self.method {
            ContentSelectorMethod::WithIndex(index) => index,

            ContentSelectorMethod::WithPhysicalPosition(_) | ContentSelectorMethod::WithId(_) => {
                self.content_entry(title_metadata)?.index
            }
        })
    }
}

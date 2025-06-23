//! Implementation of the [ContentSelector] used in [TitleMetadata].

use crate::TitleMetadata;
use crate::title_metadata::TitleMetadataContentEntry;
use crate::title_metadata::TitleMetadataError;

/// Lazy evaluated selector of a content, crate one from the `select_` methods of a
/// [TitleMetadata].
#[derive(Clone, Copy)]
pub struct ContentSelector {
    pub(super) method: ContentSelectorMethod,
}

#[derive(Clone, Copy)]
pub(super) enum ContentSelectorMethod {
    WithPhysicalPosition(usize),
    WithIndex(u16),
    WithId(u32),
    Last,
}

impl ContentSelector {
    fn get_last(title_metadata: &TitleMetadata) -> Self {
        Self {
            method: ContentSelectorMethod::WithPhysicalPosition(
                title_metadata.content_chunk_entries.len() - 1,
            ),
        }
    }

    /// Get the selected content entry.
    pub fn content_entry(
        &self,
        title_metadata: &TitleMetadata,
    ) -> Result<TitleMetadataContentEntry, TitleMetadataError> {
        if let ContentSelectorMethod::Last = self.method {
            return Self::get_last(title_metadata).content_entry(title_metadata);
        }

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

            ContentSelectorMethod::Last => unreachable!(),
        })
        .ok_or_else(TitleMetadataError::ContentNotFound)
    }

    /// Get the physical position of the selected content entry.
    pub fn physical_position(
        &self,
        title_metadata: &TitleMetadata,
    ) -> Result<usize, TitleMetadataError> {
        if let ContentSelectorMethod::Last = self.method {
            return Self::get_last(title_metadata).physical_position(title_metadata);
        }

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

            ContentSelectorMethod::Last => unreachable!(),
        })
        .ok_or_else(TitleMetadataError::ContentNotFound)
    }

    /// Get the ID of the selected content entry.
    pub fn id(&self, title_metadata: &TitleMetadata) -> Result<u32, TitleMetadataError> {
        Ok(match self.method {
            ContentSelectorMethod::WithId(id) => id,

            ContentSelectorMethod::Last => Self::get_last(title_metadata).id(title_metadata)?,

            ContentSelectorMethod::WithPhysicalPosition(_)
            | ContentSelectorMethod::WithIndex(_) => self.content_entry(title_metadata)?.id,
        })
    }

    /// Get the index of the selected content entry.
    pub fn index(&self, title_metadata: &TitleMetadata) -> Result<u16, TitleMetadataError> {
        Ok(match self.method {
            ContentSelectorMethod::WithIndex(index) => index,

            ContentSelectorMethod::Last => Self::get_last(title_metadata).index(title_metadata)?,

            ContentSelectorMethod::WithPhysicalPosition(_) | ContentSelectorMethod::WithId(_) => {
                self.content_entry(title_metadata)?.index
            }
        })
    }
}

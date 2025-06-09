use crate::wad::installable::{InstallableWad, InstallableWadError};
use aes::cipher::KeyIvInit;
use std::io::{Read, Seek, SeekFrom};
use util::View;
use util::{Aes128CbcDec, AesCbcDecryptStream};

// TODO(IMPLEMENT): List:
//   - By content ID.
//   - Iterator.

impl InstallableWad {
    pub fn encrypted_content_view<T: Read + Seek>(
        &self,
        mut stream: T,
        content_index: u16,
    ) -> Result<View<T>, InstallableWadError> {
        // The header is always aligned to the boundary
        let mut content_offset = Self::HEADER_SIZE
            + Self::align(self.certificate_chain_size)
            + Self::align(self.ticket_size)
            + Self::align(self.title_metadata_size);

        let title_metadata = self.title_metadata(&mut stream)?;

        for content_entry in title_metadata.content_chunk_entries {
            if content_entry.index == content_index {
                stream.seek(SeekFrom::Start(content_offset))?;
                return Ok(View::new(stream, content_entry.size as usize)?);
            }

            content_offset += util::align_to_boundary(content_entry.size, Self::SECTION_BOUNDARY);
        }

        Err(InstallableWadError::ContentEntryIndexDoesntExist(
            content_index,
        ))
    }

    pub fn decrypted_content_view<T: Read + Seek>(
        &self,
        mut stream: T,
        content_index: u16,
    ) -> Result<AesCbcDecryptStream<View<T>>, InstallableWadError> {
        let title_key = self.ticket(&mut stream)?.decrypt_title_key_wii_method();
        let content_view = self.encrypted_content_view(stream, content_index)?;

        // Add 14 trailing zeroed bytes to the IV
        let mut iv = Vec::from(content_index.to_be_bytes());
        iv.append(&mut Vec::from([0; 14]));

        #[allow(clippy::expect_used)]
        let iv: [u8; 16] = iv
            .try_into()
            .expect("Will never fail, the `content_index` is always 16 bits");

        let cipher = Aes128CbcDec::new(&title_key.into(), &iv.into());
        Ok(AesCbcDecryptStream::new(content_view, cipher)?)
    }

    // TODO(IMPLEMENT): Add write content
}

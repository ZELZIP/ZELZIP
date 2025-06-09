use crate::wad::installable::{InstallableWad, InstallableWadError};
use aes::cipher::KeyIvInit;
use std::io::{Read, Seek, SeekFrom, Take};
use util::{Aes128CbcDec, AesCbcDecryptStream};

// TODO: Put this like Ticket sub-impl

impl InstallableWad {
    pub fn take_encrypted_content<T: Read + Seek>(
        &self,
        mut reader: T,
        content_index: u16,
    ) -> Result<Take<T>, InstallableWadError> {
        // The header is always aligned to the boundary
        let mut content_offset = InstallableWad::HEADER_SIZE
            + util::align_to_boundary(
                self.certificate_chain_size as u64,
                InstallableWad::SECTION_BOUNDARY,
            )
            + util::align_to_boundary(self.ticket_size as u64, InstallableWad::SECTION_BOUNDARY)
            + util::align_to_boundary(
                self.title_metadata_size as u64,
                InstallableWad::SECTION_BOUNDARY,
            );

        let title_metadata = self.title_metadata(&mut reader)?;

        for content_entry in title_metadata.content_chunk_entries {
            if content_entry.index == content_index {
                reader.seek(SeekFrom::Start(content_offset))?;
                return Ok(reader.take(content_entry.size));
            }

            content_offset +=
                util::align_to_boundary(content_entry.size, InstallableWad::SECTION_BOUNDARY);
        }

        Err(InstallableWadError::ContentEntryIndexDoesntExist(
            content_index,
        ))
    }

    pub fn take_decrypted_content<T: Read + Seek>(
        &self,
        mut reader: T,
        content_index: u16,
    ) -> Result<AesCbcDecryptStream<T>, InstallableWadError> {
        let title_key = self.ticket(&mut reader)?.decrypt_title_key_wii_method();
        let content_take = self.take_encrypted_content(reader, content_index)?;

        // Add 14 trailing zeroed bytes to the IV
        let mut iv = Vec::from(content_index.to_be_bytes());
        iv.append(&mut Vec::from([0; 14]));
        let iv: [u8; 16] = iv
            .try_into()
            .expect("Will never fail, the `content_index` is always 16 bits");

        let cipher = Aes128CbcDec::new(&title_key.into(), &iv.into());
        Ok(AesCbcDecryptStream::new(content_take, cipher)?)
    }
}

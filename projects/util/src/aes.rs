use aes::cipher::{BlockDecryptMut, block_padding::NoPadding};
use std::io;
use std::io::{Read, Seek, SeekFrom};

/// Decryptor of AES-128 encrypted bytes.
pub type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

/// Readable stream of AES-128 encrypted bytes.
pub struct AesCbcDecryptStream<T: Read + Seek> {
    stream: T,
    cipher: Aes128CbcDec,
}

impl<T: Read + Seek> AesCbcDecryptStream<T> {
    /// Create a new decryption stream.
    pub fn new(stream: T, cipher: Aes128CbcDec) -> Result<Self, io::Error> {
        Ok(Self { stream, cipher })
    }

    /// Get the stored stream.
    pub fn into_inner(self) -> T {
        self.stream
    }
}

impl<T: Read + Seek> Read for AesCbcDecryptStream<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let original_position = self.stream.stream_position()?;
        let stream_len = self.stream.seek(SeekFrom::End(0))? + 1;

        // If the position is not aligned to a block then align it to the previous 16 byte boundary
        let start_position = if original_position % 16 == 0 {
            original_position
        } else {
            crate::align_to_boundary(self.stream.stream_position()?, 16) - 16
        };

        let mut buf_len = buf.len() as u64;

        if start_position + buf_len > stream_len {
            buf_len -= (start_position + buf_len) - stream_len
        };

        let start_padding = start_position - original_position;

        self.stream.seek(std::io::SeekFrom::Start(start_position))?;

        // Make the buffer big enough to store the targer buffer size, the extra start padding and
        // the padding up to the next 16 byte boundary.
        let len = crate::align_to_boundary(start_padding + buf_len, 16) as usize;

        let mut encrypted_buffer = vec![0; len].into_boxed_slice();
        let mut decrypted_buffer = vec![0; len].into_boxed_slice();

        self.stream.read(&mut encrypted_buffer)?;

        self.cipher
            .clone()
            .decrypt_padded_b2b_mut::<NoPadding>(&encrypted_buffer, &mut decrypted_buffer)
            .map_err(|err| io::Error::other(format!("Unable to decrypt the buffer: {err}")))?;

        for (i, value) in decrypted_buffer
            [start_padding as usize..(start_padding + buf_len) as usize]
            .iter()
            .enumerate()
        {
            buf[i] = *value
        }

        Ok(buf_len as usize)
    }
}

impl<T: Read + Seek> Seek for AesCbcDecryptStream<T> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.stream.seek(pos)
    }
}

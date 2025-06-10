use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::NoPadding};
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};

/// Decryptor of AES-128 encrypted bytes.
pub type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
pub type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;

/// Stream of AES-128 encrypted bytes.
pub struct AesCbcStream<T> {
    stream: T,
    decryptor: Aes128CbcDec,
    encryptor: Aes128CbcEnc,
}

impl<T> AesCbcStream<T> {
    /// Create a new decryption stream.
    pub fn new(stream: T, key: [u8; 16], iv: [u8; 16]) -> Result<Self, io::Error> {
        let decryptor = Aes128CbcDec::new(&key.into(), &iv.into());
        let encryptor = Aes128CbcEnc::new(&key.into(), &iv.into());

        Ok(Self {
            stream,
            decryptor,
            encryptor,
        })
    }

    /// Get the stored stream.
    pub fn into_inner(self) -> T {
        self.stream
    }
}

impl<T: Read + Seek> Read for AesCbcStream<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let original_position = self.stream.stream_position()?;
        let stream_len = self.stream.seek(SeekFrom::End(0))? + 1;

        // If the position is not aligned to a block then align it to the previous 16 byte boundary
        let start_position = if original_position == 0 {
            original_position
        } else if original_position % 16 == 0 {
            original_position - 16
        } else {
            crate::align_to_boundary(original_position, 16) - 16
        };

        let start_padding = original_position - start_position;

        let mut buf_len = buf.len() as u64;

        if original_position + buf_len > stream_len {
            buf_len -= (original_position + buf_len) - stream_len
        };

        self.stream.seek(std::io::SeekFrom::Start(start_position))?;

        // Make the buffer big enough to store the targer buffer size, the extra start padding and
        // the padding up to the next 16 byte boundary.
        let len = crate::align_to_boundary(start_padding + buf_len, 16) as usize;

        let mut encrypted_buffer = vec![0; len].into_boxed_slice();
        let mut decrypted_buffer = vec![0; len].into_boxed_slice();

        self.stream.read(&mut encrypted_buffer)?;

        self.decryptor
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

        Ok((buf_len) as usize)
    }
}

impl<T: Write + Seek> AesCbcStream<T> {
    /// Encrypt and write into the buffer a set of bytes, it's not available as a [std::io::Write]
    /// implementation nor can be split into smaller units because the IV vector of AES CBC changes
    /// depending on the position the data to be written.
    pub fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut encrypted_buffer = vec![0; buf.len()];

        self.encryptor
            .clone()
            .encrypt_padded_b2b_mut::<NoPadding>(buf, &mut encrypted_buffer)
            .map_err(|err| io::Error::other(format!("Unable to encrypt the buffer: {err}")))?;

        self.stream.write(&encrypted_buffer)
    }
}

impl<T: Seek> Seek for AesCbcStream<T> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.stream.seek(pos)
    }
}

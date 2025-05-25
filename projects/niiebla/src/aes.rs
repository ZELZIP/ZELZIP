use aes::cipher::{block_padding::NoPadding, BlockDecryptMut};
use std::io;
use std::io::{Read, Seek, SeekFrom, Take};

pub type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

pub struct AesCbcDecryptStream<T: Read + Seek> {
    inner: T,
    cipher: Aes128CbcDec,
    size: u64,
    start_position: u64,
    current_position: u64,
}

impl<T: Read + Seek> AesCbcDecryptStream<T> {
    pub(crate) fn new(
        take: Take<T>,
        cipher: Aes128CbcDec,
    ) -> Result<AesCbcDecryptStream<T>, io::Error> {
        let size = take.limit();
        let mut inner = take.into_inner();
        let start_position = inner.stream_position()?;

        Ok(AesCbcDecryptStream {
            inner,
            cipher,
            size,
            start_position,
            current_position: 0,
        })
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: Read + Seek> Read for AesCbcDecryptStream<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let start = if self.current_position == 0 {
            0
        } else {
            crate::align_to_boundary(self.current_position, 16) - 16
        };

        let start_offset = self.current_position - start;

        let len = crate::align_to_boundary(start_offset + buf.len() as u64, 16) as usize;

        let mut encrypted_buffer = vec![0; len].into_boxed_slice();
        let mut decrypted_buffer = vec![0; len].into_boxed_slice();

        self.inner
            .seek(std::io::SeekFrom::Start(self.start_position + start))?;

        self.inner.read(&mut encrypted_buffer)?;

        self.cipher
            .clone()
            .decrypt_padded_b2b_mut::<NoPadding>(&encrypted_buffer, &mut decrypted_buffer)
            .map_err(|err| io::Error::other(format!("Unable to decrypt the buffer: {err}")))?;

        buf.clone_from_slice(
            &decrypted_buffer[start_offset as usize..start_offset as usize + buf.len()],
        );

        self.current_position += buf.len() as u64;

        Ok(buf.len())
    }
}

impl<T: Read + Seek> Seek for AesCbcDecryptStream<T> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        const SEEK_OUT_OF_BOUNDS_TOO_SMALL_MESSAGE: &str =
            "An AES stream cannot be seeked outside its size";

        let new_current_position = match pos {
            SeekFrom::Start(value) => {
                if value >= self.size {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        SEEK_OUT_OF_BOUNDS_TOO_SMALL_MESSAGE,
                    ));
                }

                self.current_position = value;

                return Ok(self.current_position);
            }

            SeekFrom::Current(value) => self.current_position as i64 + value,

            SeekFrom::End(value) => (self.size as i64 - 1) + value,
        };

        if new_current_position < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "An AES stream cannot be seeked to negative values",
            ));
        }

        if new_current_position as u64 >= self.size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                SEEK_OUT_OF_BOUNDS_TOO_SMALL_MESSAGE,
            ));
        }

        self.current_position = new_current_position as u64;

        Ok(self.current_position)
    }
}

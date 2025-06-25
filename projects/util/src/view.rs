// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use std::cmp;
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};

/// Create a bounded limited view of a stream ([Seek] with [Read] and/or [Write]).
///
/// The **original position of the stream may be changed**, please conside [crate::RecallView] as an
/// alternative.
pub struct View<T: Seek> {
    inner: T,
    start_position: u64,

    /// The length of the viewble range inside the stream.
    pub len: usize,
}

impl<T: Seek> View<T> {
    /// Create a new [View].
    ///
    /// # Safety
    /// The length must be greater than zero.
    pub fn new(mut stream: T, len: usize) -> io::Result<Self> {
        assert!(len > 0);

        let start_position = stream.stream_position()?;

        Ok(Self {
            inner: stream,
            start_position,
            len,
        })
    }

    /// Consume the [View] and get back the wrapped stream.
    pub fn into_inner(self) -> T {
        self.inner
    }

    fn end_position(&mut self) -> u64 {
        self.start_position + self.len as u64 - 1
    }

    fn relative_position(&mut self) -> io::Result<u64> {
        Ok(self.inner.stream_position()? - self.start_position)
    }

    fn calc_position_from(&self, position: u64, value: i64) -> Result<u64, io::Error> {
        const SEEK_NEGATIVE_OFFSET_ERROR_MESAGE: &str = "Seeked into a negative offset";

        let new_position: u64 = (position as i64 + value)
            .try_into()
            .map_err(|_| io::Error::other(SEEK_NEGATIVE_OFFSET_ERROR_MESAGE))?;

        if new_position < self.start_position {
            return Err(io::Error::other(SEEK_NEGATIVE_OFFSET_ERROR_MESAGE));
        }

        Ok(new_position)
    }
}

impl<T: Read + Seek> Read for View<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let max_bytes_to_read = cmp::min(
            // Just read 0 bytes if the seek position is out of bounds
            self.len.saturating_sub(self.relative_position()? as usize),
            buf.len(),
        );

        self.inner.read(&mut buf[0..max_bytes_to_read])
    }
}

impl<T: Write + Seek> Write for View<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let max_bytes_to_write = cmp::min(
            // Just write 0 bytes if the seek position is out of bounds
            self.len.saturating_sub(self.relative_position()? as usize),
            buf.len(),
        );

        self.inner.write(&buf[0..max_bytes_to_write])
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T: Seek> Seek for View<T> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_position = match pos {
            SeekFrom::Start(value) => self.start_position + value,

            SeekFrom::Current(value) => {
                let position = self.inner.stream_position()?;
                self.calc_position_from(position, value)?
            }

            SeekFrom::End(value) => {
                let position = self.end_position();
                self.calc_position_from(position, value)?
            }
        };

        self.inner.seek(SeekFrom::Start(new_position))?;

        self.relative_position()
    }
}

#[cfg(test)]
mod tests {
    mod read;
    mod seek_end;
    mod seek_relative;
    mod seek_start;
    mod write;

    use super::*;
}

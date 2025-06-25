// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use crate::WriteEx;
use std::io::{self, Read, Seek, SeekFrom, Write};

/// Wrapper for a stream ([Seek] and [Write] and/or [Read]) that stores the position when the pin
/// was created and allow to do some operations around that value.
pub struct StreamPin<T: Seek> {
    stream: T,
    start_position: u64,
}

impl<T: Seek> StreamPin<T> {
    /// Create a new [StreamPin].
    pub fn new(mut stream: T) -> io::Result<Self> {
        let start_position = stream.stream_position()?;

        Ok(Self {
            stream,
            start_position,
        })
    }

    /// Get the inner stream stored inside the pin.
    pub fn into_inner(self) -> T {
        self.stream
    }

    /// Go to the position when the pin was created.
    pub fn go_to_pin(&mut self) -> io::Result<()> {
        self.seek(SeekFrom::Start(self.start_position))?;

        Ok(())
    }

    /// Get the position of the stream relative to the pinned position.
    pub fn relative_position(&mut self) -> io::Result<i64> {
        Ok(self.stream_position()? as i64 - self.start_position as i64)
    }

    /// Seek to a position starting from the pinned position.
    pub fn seek_from_pin(&mut self, step: i64) -> io::Result<u64> {
        self.seek(SeekFrom::Start((self.start_position as i64 + step) as u64))
    }

    /// Align the position of the stream relative to the pinned position.
    pub fn align_position(&mut self, boundary: u64) -> io::Result<()> {
        let relative_position = self.stream_position()? - self.start_position;

        self.seek(SeekFrom::Start(
            self.start_position + crate::align_to_boundary(relative_position, boundary),
        ))?;

        Ok(())
    }
}

impl<T: Write + Seek> StreamPin<T> {
    /// Align the position of the stream relative to the pinned position and fill the intermediate
    /// bytes with zeroes.
    pub fn align_zeroed(&mut self, boundary: u64) -> io::Result<()> {
        let relative_position = self.stream_position()?.abs_diff(self.start_position);

        let aligned_position = crate::align_to_boundary(relative_position, boundary);

        self.write_zeroed((aligned_position - relative_position) as usize)?;

        Ok(())
    }
}

impl<T: Seek> Seek for StreamPin<T> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.stream.seek(pos)
    }
}

impl<T: Seek + Read> Read for StreamPin<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl<T: Seek + Write> Write for StreamPin<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::ReadBytesExt;
    use std::io::Cursor;

    #[test]
    fn go_to_pin() {
        let mut stream = Cursor::new([0, 1, 2, 3, 4]);
        stream.seek_relative(1).unwrap();

        let mut pin = StreamPin::new(stream).unwrap();
        pin.seek_relative(3).unwrap();

        assert_eq!(pin.read_u8().unwrap(), 4);

        pin.go_to_pin().unwrap();

        assert_eq!(pin.read_u8().unwrap(), 1);
    }

    #[test]
    fn align_position_zero() {
        let mut stream = Cursor::new([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        stream.seek_relative(1).unwrap();

        let mut pin = StreamPin::new(stream).unwrap();

        pin.align_position(2).unwrap();

        assert_eq!(pin.stream_position().unwrap(), 1);
        assert_eq!(pin.read_u8().unwrap(), 1);
    }

    #[test]
    fn align_position_aligned_and_non_zero() {
        let mut stream = Cursor::new([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        stream.seek_relative(1).unwrap();

        let mut pin = StreamPin::new(stream).unwrap();

        pin.seek_relative(2).unwrap();
        pin.align_position(2).unwrap();

        assert_eq!(pin.stream_position().unwrap(), 3);
        assert_eq!(pin.read_u8().unwrap(), 3);
    }

    #[test]
    fn align_position_unaligned() {
        let mut stream = Cursor::new([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        stream.seek_relative(1).unwrap();

        let mut pin = StreamPin::new(stream).unwrap();

        pin.seek_relative(3).unwrap();
        pin.align_position(2).unwrap();

        assert_eq!(pin.stream_position().unwrap(), 5);
        assert_eq!(pin.read_u8().unwrap(), 5);
    }

    #[test]
    fn align_zeroed() {
        let mut stream = Cursor::new([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        stream.seek_relative(1).unwrap();

        let mut pin = StreamPin::new(stream).unwrap();

        pin.seek_relative(1).unwrap();
        pin.align_zeroed(4).unwrap();

        assert_eq!(pin.stream_position().unwrap(), 5);
        assert_eq!(pin.read_u8().unwrap(), 5);

        let data = pin.into_inner().into_inner();
        assert_eq!(data[3], 0);
        assert_eq!(data[4], 0);
    }

    #[test]
    fn go_relative() {
        let mut stream = Cursor::new([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        stream.seek_relative(5).unwrap();

        let mut pin = StreamPin::new(stream).unwrap();

        pin.seek_from_pin(2).unwrap();
        assert_eq!(pin.read_u8().unwrap(), 7);

        pin.seek_from_pin(-3).unwrap();
        assert_eq!(pin.read_u8().unwrap(), 2);
    }
}

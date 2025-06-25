// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use crate::View;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::mem;
use std::ptr;

/// Like [View] but can remember the original position of the stream and automatically put it back
/// to its correct place.
#[cfg(feature = "std")]
pub struct RecallView<T: Seek> {
    view: View<T>,
    original_position: u64,
}

impl<T: Seek> RecallView<T> {
    /// Create a new [View].
    ///
    /// # Safety
    /// The length must be greater than zero.
    pub fn new(stream: T, len: usize) -> io::Result<Self> {
        assert!(len > 0);

        let mut view = View::new(stream, len)?;
        let original_position = view.stream_position()?;

        Ok(Self {
            view,
            original_position,
        })
    }

    /// Reset the position to its original one.
    pub fn reset_position(&mut self) -> io::Result<()> {
        self.view
            .seek(io::SeekFrom::Start(self.original_position))?;

        Ok(())
    }

    /// Consume the [RecallView] and get back the wrapped [View].
    pub fn into_view(mut self) -> io::Result<View<T>> {
        self.reset_position()?;

        Ok(self.into_view_no_reset())
    }

    /// Consume the [RecallView] and the [View] and get back the wrapped stream.
    pub fn into_inner(self) -> io::Result<T> {
        Ok(self.into_view()?.into_inner())
    }

    /// Consume the [RecallView] and get back the wrapped [View] **without resetting to the
    /// original position**.
    pub fn into_view_no_reset(self) -> View<T> {
        let value = mem::ManuallyDrop::new(self);

        unsafe { ptr::read(&value.view) }
    }

    /// Consume the [RecallView] and the [View] and get back the wrapped [View] **without resetting to the
    /// original position**.
    pub fn into_inner_no_reset(self) -> T {
        self.into_view_no_reset().into_inner()
    }
}

impl<T: Read + Seek> Read for RecallView<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.view.read(buf)
    }
}

impl<T: Write + Seek> Write for RecallView<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.view.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.view.flush()
    }
}

impl<T: Seek> Seek for RecallView<T> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.view.seek(pos)
    }
}

impl<T: Seek> Drop for RecallView<T> {
    fn drop(&mut self) {
        #[allow(clippy::expect_used)]
        self.reset_position()
            .expect("Unable to reset the RecallView to its original position");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::ReadBytesExt;
    use std::io::Cursor;
    use std::io::Read;

    #[test]
    fn read() {
        let mut buffer = Cursor::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        {
            let mut recall_view = RecallView::new(&mut buffer, 5).unwrap();

            let mut data = [0; 5];
            recall_view.read_exact(&mut data).unwrap();
        }

        assert_eq!(buffer.read_u8().unwrap(), 1);
    }

    #[test]
    fn into_inner() {
        let mut buffer = Cursor::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let recall_view = RecallView::new(&mut buffer, 5).unwrap();
        let _buffer = recall_view.into_inner().unwrap();
    }
}

use byteorder::WriteBytesExt;
use std::io;
use std::io::Write;

/// Extension trait of [Write] with useful miscellaneous operations.
pub trait WriteEx: Write {
    /// Write zero the given number of times
    fn write_zeroed(&mut self, number_of_zeroes: usize) -> io::Result<()> {
        self.write_all(&vec![0; number_of_zeroes])?;

        Ok(())
    }

    /// Write a buffer of bytes and the pad with zeroes up to the `padding`.
    fn write_bytes_padded(&mut self, buffer: &[u8], padding: usize) -> io::Result<()> {
        assert!(buffer.len() <= padding);

        self.write_all(buffer)?;
        self.write_zeroed(padding - buffer.len())?;

        Ok(())
    }

    /// Write a bool.
    fn write_bool(&mut self, value: bool) -> io::Result<()> {
        self.write_u8(value as u8)?;

        Ok(())
    }
}

impl<T: Write> WriteEx for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_zeroed_three_times() {
        let mut buffer = vec![1, 2, 3];
        buffer.write_zeroed(3).unwrap();

        assert_eq!(buffer, [1, 2, 3, 0, 0, 0]);
    }

    #[test]
    fn write_zeroed_zero_times() {
        let mut buffer = vec![1, 2, 3];
        buffer.write_zeroed(0).unwrap();

        assert_eq!(buffer, [1, 2, 3]);
    }

    #[test]
    fn write_bytes_padded_same_size() {
        let mut buffer = vec![1, 2, 3];
        buffer.write_bytes_padded(&[4, 5, 6], 3).unwrap();

        assert_eq!(buffer, [1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn write_bytes_padded_different_size() {
        let mut buffer = vec![1, 2];
        buffer.write_bytes_padded(&[3, 4], 6).unwrap();

        assert_eq!(buffer, [1, 2, 3, 4, 0, 0, 0, 0]);
    }

    #[test]
    fn write_bool_true() {
        let mut buffer = vec![1, 2];
        buffer.write_bool(true).unwrap();

        assert_eq!(buffer, [1, 2, 1]);
    }

    #[test]
    fn write_bool_false() {
        let mut buffer = vec![1, 2];
        buffer.write_bool(false).unwrap();

        assert_eq!(buffer, [1, 2, 0]);
    }
}

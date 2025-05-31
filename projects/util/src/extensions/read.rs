use byteorder::ReadBytesExt;
use std::io;
use std::io::Read;

/// Extension trait of [Write] with useful miscellaneous operations.
pub trait ReadEx: Read {
    /// Read a bool.
    fn read_bool(&mut self) -> io::Result<bool> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),

            value => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("The given value cannot be converted into a bool: {value}"),
            )),
        }
    }
}

impl<T: ?Sized + Read> ReadEx for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_bool_true() {
        let mut buffer = Cursor::new([1, 255]);
        assert!(buffer.read_bool().unwrap())
    }

    #[test]
    fn test_read_bool_false() {
        let mut buffer = Cursor::new([0, 255]);
        assert!(!buffer.read_bool().unwrap())
    }

    #[test]
    fn test_read_bool_invalid() {
        let mut buffer = Cursor::new([77, 255]);
        assert!(buffer.read_bool().is_err())
    }
}

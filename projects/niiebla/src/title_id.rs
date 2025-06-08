use byteorder::{BigEndian, WriteBytesExt};
use std::fmt::{self, Display};
use std::io;
use std::io::Write;

// TODO(IMPLEMENT): Add multiple display modes with newtype wrappers.

#[derive(Debug)]
/// 64 bit value used to uniquely identify titles on Nintendo consoles.
pub struct TitleId(u64);

impl TitleId {
    /// Create a new [TitleId].
    pub fn new(title_id_value: u64) -> Self {
        Self(title_id_value)
    }

    /// Get the stored value inside the title ID.
    pub fn inner(&self) -> u64 {
        self.0
    }

    /// Dump a title ID into a writer ([Write]).
    pub fn dump<T: Write>(&self, writer: &mut T) -> io::Result<()> {
        writer.write_u64::<BigEndian>(self.0)?;

        Ok(())
    }
}

impl Display for TitleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let upper_half = (self.0 & 0xFFFFFFFF00000000) >> 32;
        let lower_half = self.0 & 0xFFFFFFFF;

        if upper_half != 0x00000001 {
            let lower_half = lower_half.to_be_bytes().to_vec();

            #[allow(clippy::expect_used)]
            let lower_half = String::from_utf8(lower_half)
                .expect("Unable to convert the lower half of the title ID to UTF-8");

            return write!(f, "{upper_half:08X}-{lower_half}");
        }

        // Essential system titles
        let text = match lower_half {
            0x00000001 => String::from("BOOT2 (Wii)"),
            0x00000002 => String::from("System Menu (Wii)"),
            0x00000100 => String::from("BC (Wii)"),
            0x00000101 => String::from("MIOS (Wii)"),

            lower_half => {
                format!("IOSv{lower_half} (Wii)")
            }
        };

        write!(f, "{text}")
    }
}

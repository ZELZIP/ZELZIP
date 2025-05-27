use byteorder::{BigEndian, WriteBytesExt};
use std::fmt::{self, Display};
use std::io;
use std::io::Write;

#[derive(Debug)]
pub struct TitleId(u64);

impl TitleId {
    pub fn new(title_id_value: u64) -> TitleId {
        TitleId(title_id_value)
    }

    pub fn get(&self) -> u64 {
        self.0
    }

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
            let lower_half = String::from_utf8(lower_half)
                .expect("Unable to convert the lower half of the title ID to UTF-8");

            return write!(f, "{:08X}-{}", upper_half, lower_half);
        }

        // Essential system titles
        let text = match lower_half {
            0x00000001 => String::from("BOOT2"),
            0x00000002 => String::from("System Menu"),
            0x00000100 => String::from("BC"),
            0x00000101 => String::from("MIOS"),

            lower_half => {
                format!("IOSv{}", lower_half)
            }
        };

        write!(f, "{text}")
    }
}

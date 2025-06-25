// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Implementation of a newtype wrapper around the title ID of a title.

use byteorder::{BE, WriteBytesExt};
use std::fmt::{self, Display};
use std::io;
use std::io::Write;

#[derive(Debug)]
/// 64 bit value used to uniquely identify titles on Nintendo consoles.
///
/// On all formatters (if applicable) the alternative flag (`#`) can be used to put the hex values
/// with uppercase letters.
pub struct TitleId(u64);

impl TitleId {
    /// Create a new [TitleId].
    pub fn new(title_id_value: u64) -> Self {
        Self(title_id_value)
    }

    /// Create a new [TitleId] given a lower and a higher halfs.
    pub fn new_with_halfs(higher_half: u32, lower_half: u32) -> Self {
        Self(((higher_half as u64) << 32) | lower_half as u64)
    }

    /// Get the stored value inside the title ID.
    pub fn inner(&self) -> u64 {
        self.0
    }

    /// Dump a title ID into a stream.
    pub fn dump<T: Write>(&self, mut stream: T) -> io::Result<()> {
        stream.write_u64::<BE>(self.0)?;

        Ok(())
    }

    /// Get the lower half of the ID.
    pub fn lower_half(&self) -> u32 {
        (self.0 & 0xFFFFFFFF) as u32
    }

    /// Get the higher half of the ID.
    pub fn higher_half(&self) -> u32 {
        ((self.0 & 0xFFFFFFFF00000000) >> 32) as u32
    }

    /// Set a new lower half.
    pub fn set_lower_half(&mut self, lower_half: u32) {
        *self = Self::new_with_halfs(self.higher_half(), lower_half);
    }

    /// Set a new higher half.
    pub fn set_higher_half(&mut self, higher_half: u32) {
        *self = Self::new_with_halfs(higher_half, self.lower_half());
    }

    /// Get a wrapper that can display the title ID with ASCII characters in its lower half, if the
    /// character is not visible a fallback to the normal display will be made.
    pub fn display_ascii(&self) -> TitleIdAsciiDisplay<'_> {
        TitleIdAsciiDisplay(self)
    }

    /// Wrapper that can display the title ID with custom display values for well-known IDs (IOS,
    /// BOOT2, vWii ancast images, etc).
    pub fn display_wii_platform(&self) -> TitleIdWiiPlatformDisplay<'_> {
        TitleIdWiiPlatformDisplay(self)
    }
}

impl Display for TitleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let higher_half = self.higher_half();
        let lower_half = self.lower_half();

        if !f.alternate() {
            write!(f, "{higher_half:08x}-{lower_half:08x}")
        } else {
            write!(f, "{higher_half:08X}-{lower_half:08X}")
        }
    }
}

/// Wrapper that can display the title ID with ASCII characters in its lower half, if the
/// character is not visible a fallback to the normal display will be made.
pub struct TitleIdAsciiDisplay<'a>(&'a TitleId);

impl Display for TitleIdAsciiDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let higher_half = self.0.higher_half();
        let lower_half = self.0.lower_half();

        let lower_half = match String::from_utf8(lower_half.to_be_bytes().to_vec()) {
            Ok(lower_half) => lower_half,
            Err(_err) => return self.0.fmt(f),
        };

        if !lower_half.chars().all(char::is_alphanumeric) {
            return self.0.fmt(f);
        }

        if !f.alternate() {
            write!(f, "{higher_half:08x}-{lower_half}")
        } else {
            write!(f, "{higher_half:08X}-{lower_half}")
        }
    }
}

/// Wrapper that can display the title ID with custom display values for well-known IDs (IOS,
/// BOOT2, vWii ancast images, etc).
pub struct TitleIdWiiPlatformDisplay<'a>(&'a TitleId);

impl Display for TitleIdWiiPlatformDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let higher_half = self.0.higher_half();
        let lower_half = self.0.lower_half();

        if higher_half != 0x00000001 {
            return self.0.fmt(f);
        };

        let text = match lower_half {
            0x00000001 => String::from("BOOT2"),
            0x00000002 => String::from("System Menu"),

            0x00000100 => String::from("BC"),
            0x00000101 => String::from("MIOS"),

            0x00000200 => String::from("BC-NAND"),
            0x00000201 => String::from("BC-WFS"),

            lower_half => {
                format!("IOS{lower_half} (Wii)")
            }
        };

        write!(f, "{text}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Fake ID "4A4132BC-HAGA"
    const TEST_ID_NUMBER: u64 = 5350613616540337985;

    const TEST_ID_NUMBER_NOT_VALID_ASCII: u64 = 5350613615614431505;

    #[test]
    fn default_display() {
        let title_id = TitleId::new(TEST_ID_NUMBER);

        assert_eq!("4a4132bc-48414741", format!("{title_id}"))
    }

    #[test]
    fn default_display_alternative_mode() {
        let title_id = TitleId::new(TEST_ID_NUMBER);

        assert_eq!("4A4132BC-48414741", format!("{title_id:#}"))
    }

    #[test]
    fn ascii_display() {
        let title_id = TitleId::new(TEST_ID_NUMBER);

        assert_eq!("4a4132bc-HAGA", format!("{}", title_id.display_ascii()))
    }

    #[test]
    fn ascii_display_alternative_mode() {
        let title_id = TitleId::new(TEST_ID_NUMBER);

        assert_eq!("4A4132BC-HAGA", format!("{:#}", title_id.display_ascii()))
    }

    #[test]
    fn ascii_display_invalid_ascii() {
        let title_id = TitleId::new(TEST_ID_NUMBER_NOT_VALID_ASCII);

        assert_eq!("4a4132bc-11111111", format!("{}", title_id.display_ascii()))
    }

    #[test]
    fn ascii_display_invalid_ascii_alternative_mode() {
        let title_id = TitleId::new(TEST_ID_NUMBER_NOT_VALID_ASCII);

        assert_eq!(
            "4A4132BC-11111111",
            format!("{:#}", title_id.display_ascii())
        )
    }

    #[test]
    fn set_lower_half() {
        let mut title_id = TitleId::new_with_halfs(500, 500);
        title_id.set_lower_half(100);

        assert_eq!(title_id.lower_half(), 100);
        assert_eq!(title_id.higher_half(), 500);
    }

    #[test]
    fn set_higher_half() {
        let mut title_id = TitleId::new_with_halfs(500, 500);
        title_id.set_higher_half(100);

        assert_eq!(title_id.lower_half(), 500);
        assert_eq!(title_id.higher_half(), 100);
    }
}

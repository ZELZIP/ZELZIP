// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

/// Macro that creates a local buffer slice variable to store some arbitraty data from
/// a [std::io::Read].
///
/// # Examples
/// ```
/// use std::io::Cursor;
/// let mut stream = Cursor::new([0, 1, 2, 3]);
///
/// let buf = gotre_util::read_exact!(stream, 2).unwrap();
///
/// assert_eq!(buf, [0, 1]);
/// ```
#[cfg(feature = "std")]
#[macro_export]
macro_rules! read_exact {
    ($stream: ident, $num_of_bytes: expr) => {
        'scope: {
            use std::io::Read;

            let mut buffer = [0; $num_of_bytes];

            if let Err(err) = $stream.read_exact(&mut buffer) {
                break 'scope Err(err);
            }

            Ok(buffer)
        }
    };
}

#[cfg(feature = "std")]
#[macro_export]
/// Macro that creates an string (null-terminated) of a fixed sized from a stream.
/// a [std::io::Read].
///
/// # Examples
/// ```
/// use std::io::Cursor;
/// let mut stream = Cursor::new([72, 105, 33]);
///
/// let string = gotre_util::read_string!(stream, 3).unwrap();
///
/// assert_eq!(string, "Hi!");
/// ```
///
/// # Safety
/// This macro panics if the string is not valid UTF-8.
macro_rules! read_string {
    ($stream: ident, $num_of_bytes: expr) => {
        'scope: {
            use $crate::StringEx;

            let buf = match $crate::read_exact!($stream, $num_of_bytes) {
                Ok(buf) => buf,
                Err(err) => break 'scope Err(err),
            };

            let string = String::from_null_terminated_bytes(&buf)
                .expect("The given buffer is not an UTF-8 stream");

            Ok(string)
        }
    };
}

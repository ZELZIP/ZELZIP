//! Miscellaneous code shared for all the crates inside the GOTRE project.
//!
//! Has partial support for `no_std` mode by disabling the default `std` feature flag. Extra suport
//! for "alloc-compatible" `no_std` environments is available by enabling the `alloc` feature flag.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(doc, feature(doc_auto_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

mod extensions;
mod macros;

#[allow(unused_imports)]
pub use extensions::*;

#[cfg(feature = "std")]
mod recall_view;

#[cfg(feature = "std")]
mod view;

#[cfg(feature = "std")]
mod stream_pin;

#[cfg(feature = "std")]
mod aes;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "std")] {
        pub use view::View;
        pub use recall_view::RecallView;
        pub use stream_pin::StreamPin;
        pub use aes::{Aes128CbcDec, AesCbcDecryptStream};
    }
}

/// Align a value to the next multiple of the given boundary.
pub fn align_to_boundary(value: u64, boundary: u64) -> u64 {
    if value == 0 {
        return 0;
    }

    value + (boundary - (value % boundary)) % boundary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_to_boundary_unaligned_value() {
        assert_eq!(align_to_boundary(117, 64), 128);
    }

    #[test]
    fn test_align_to_boundary_aligned_value() {
        assert_eq!(align_to_boundary(100, 50), 100);
    }

    #[test]
    fn test_align_to_boundary_same_value() {
        assert_eq!(align_to_boundary(73, 73), 73);
    }

    #[test]
    fn test_align_to_boundary_zero() {
        assert_eq!(align_to_boundary(0, 0), 0);
    }
}

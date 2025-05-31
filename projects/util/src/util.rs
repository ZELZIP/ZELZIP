//! Miscellaneous code shared for all the crates inside the GOTRE project.
//!
//! Has partial support for `no_std` mode by disabling the default `std` feature flag.

pub(crate) mod extensions;
pub(crate) mod recall_view;
pub(crate) mod view;

pub use extensions::read::ReadEx;
pub use extensions::string::StringEx;
pub use extensions::write::WriteEx;
pub use recall_view::RecallView;
pub use view::View;

/// Align a value to the next multiple of the given boundary
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

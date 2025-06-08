#[cfg(feature = "std")]
mod read;

#[cfg(feature = "std")]
mod write;

#[cfg(feature = "alloc")]
mod string;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "std")] {
        pub use read::ReadEx;
        pub use write::WriteEx;
    }
}

#[cfg(feature = "alloc")]
pub use string::StringEx;

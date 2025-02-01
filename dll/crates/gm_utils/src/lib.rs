//! A crate containing utilities for interfacing with Gamemaker's Extension FFI

pub mod buffer;
pub mod func;

#[cfg(feature = "nom")]
pub mod parsing;

/// Private API, do not use.
#[doc(hidden)]
pub mod __private;

/// A prelude with most common types
pub mod prelude {
    pub use crate::buffer::GmBuffer;
    pub use crate::func::{GmArg, GmReturn};
    pub use gm_utils_macro::gm_func;
}

pub use gm_utils_macro::gm_func;

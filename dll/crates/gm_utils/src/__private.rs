use std::os::raw::c_char;

/// Used as return values in case of a `panic!()` from Rust code.
#[doc(hidden)]
pub trait GmDefault {
    fn default() -> Self;
}

impl GmDefault for f64 {
    #[inline]
    fn default() -> Self {
        0.0
    }
}

impl GmDefault for *const c_char {
    #[inline]
    fn default() -> Self {
        b"\0".as_ptr().cast()
    }
}

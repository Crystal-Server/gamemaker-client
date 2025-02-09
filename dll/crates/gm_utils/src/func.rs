//! Contains traits for Gamemaker function call FFI.

use std::{
    borrow::Cow,
    ffi::{CStr, CString},
    fmt::Debug,
    os::raw::c_char,
};

use bstr::BStr;
use private::{GmPossibleArg, GmPossibleReturn};

mod private {
    use std::os::raw::c_char;

    pub trait GmPossibleReturn {}
    pub trait GmPossibleArg {}

    impl GmPossibleReturn for f64 {}
    impl GmPossibleReturn for *const c_char {}
    impl GmPossibleArg for f64 {}
    impl GmPossibleArg for *mut c_char {}
}

/// Implemented by all types which may be returned to Gamemaker as a return type.
///
/// The return type in the Gamemaker extension must be a `Double` when `Self::Return == f64`
/// and a `String` when `Self::Return == *const c_char`.
///
/// View the specific type's implementation of this trait for additional information.
pub trait GmReturn {
    type Return: GmPossibleReturn;

    /// Converts from `Self::Return` to `Self`.
    /// # Safety
    /// Must be called from the same thread as Gamemaker.
    unsafe fn to_return(self) -> Self::Return;
}

/// Implemented by all types which may be received as an argument from a Gamemaker function call.
///
/// The argument type in the Gamemaker extension must be a `Double` when `Self::Arg == f64`
/// and a `String` when `Self::Arg == *mut c_char`.
///
/// View the specific type's implementation of this trait for additional information.
pub trait GmArg {
    type Arg: GmPossibleArg;

    /// Converts from `Self::Arg` to `Self`.
    /// # Safety
    /// Must be called from the same thread as Gamemaker.
    unsafe fn to_arg(arg: Self::Arg) -> Self;
}

/// Helper function for returning data to Gamemaker, note you must add a null-terminator yourself.
/// # Safety
/// Must be called from the same thread as Gamemaker.
#[inline]
#[allow(static_mut_refs)]
pub unsafe fn return_with_buffer<F: FnOnce(&mut Vec<u8>)>(f: F) -> *const c_char {
    static mut RETURN_DATA: Vec<u8> = Vec::new();

    RETURN_DATA.resize(0, 0);
    // SAFETY: This function must be called from the same thread as Gamemaker
    //         therefore `RETURN_DATA` cannot be accessed from other threads at the same time.
    f(&mut RETURN_DATA);
    // SAFETY: Cast is allowed because u8 and i8 have the same size.
    RETURN_DATA.as_ptr().cast()
}

macro_rules! impl_with_into {
    (ret $source:ty, $target:ty) => {
        impl GmReturn for $source {
            type Return = $target;

            #[inline]
            unsafe fn to_return(self) -> $target {
                self.into()
            }
        }
    };
    (arg $source:ty, $target:ty) => {
        impl GmArg for $source {
            type Arg = $target;

            #[inline]
            unsafe fn to_arg(arg: $target) -> Self {
                arg.into()
            }
        }
    };
}

macro_rules! impl_with_cast {
    (ret $source:ty, $target:ty) => {
        /// Casts the return value, therefore some data loss might occur.
        ///
        /// See [type cast expressions] for further detail.
        ///
        /// [type cast expressions]: https://doc.rust-lang.org/reference/expressions/operator-expr.html#semantics
        impl GmReturn for $source {
            type Return = $target;

            #[inline]
            unsafe fn to_return(self) -> $target {
                self as $target
            }
        }
    };
    (arg $source:ty, $target:ty) => {
        /// Casts the argument, therefore some data loss might occur.
        ///
        /// See [type cast expressions] for further detail.
        ///
        /// [type cast expressions]: https://doc.rust-lang.org/reference/expressions/operator-expr.html#semantics
        impl GmArg for $source {
            type Arg = $target;

            #[inline]
            unsafe fn to_arg(arg: $target) -> Self {
                arg as $source
            }
        }
    };
}

impl_with_into!(ret f64, f64);
impl_with_into!(ret f32, f64);
impl_with_into!(ret u8, f64);
impl_with_into!(ret i8, f64);
impl_with_into!(ret u16, f64);
impl_with_into!(ret i16, f64);
impl_with_into!(ret u32, f64);
impl_with_into!(ret i32, f64);

impl GmReturn for bool {
    type Return = f64;

    #[inline]
    unsafe fn to_return(self) -> f64 {
        if self {
            1.0
        } else {
            0.0
        }
    }
}

impl GmReturn for () {
    type Return = f64;

    #[inline]
    unsafe fn to_return(self) -> f64 {
        0.0
    }
}

impl GmReturn for CString {
    type Return = *const c_char;

    #[inline]
    unsafe fn to_return(self) -> Self::Return {
        return_with_buffer(|data| {
            data.extend_from_slice(self.as_bytes_with_nul());
        })
    }
}

impl GmReturn for &CStr {
    type Return = *const c_char;

    #[inline]
    unsafe fn to_return(self) -> Self::Return {
        self.as_ptr()
    }
}

/// # Panics
/// Panics when trying to return a string with an internal 0 byte.
impl GmReturn for String {
    type Return = *const c_char;

    #[inline]
    unsafe fn to_return(self) -> Self::Return {
        return_with_buffer(|data| {
            data.extend_from_slice(
                CString::new(self)
                    .expect("internal NUL byte found")
                    .as_bytes_with_nul(),
            );
        })
    }
}

impl<'a> GmReturn for &'a str {
    type Return = *const c_char;

    #[inline]
    unsafe fn to_return(self) -> Self::Return {
        return_with_buffer(|data| {
            data.extend_from_slice(
                CString::new(self)
                    .expect("internal NUL byte found")
                    .as_bytes_with_nul(),
            );
        })
    }
}

/// # Panics
/// Panics when the error variant is returned.
impl<T: GmReturn, E: Debug> GmReturn for Result<T, E> {
    type Return = T::Return;

    #[inline]
    unsafe fn to_return(self) -> T::Return {
        self.expect("Attempted to return `Err` value").to_return()
    }
}

impl_with_into!(arg f64, f64);
impl_with_into!(arg *mut c_char, *mut c_char);
impl_with_cast!(arg f32, f64);
impl_with_cast!(arg i8, f64);
impl_with_cast!(arg u8, f64);
impl_with_cast!(arg u16, f64);
impl_with_cast!(arg i16, f64);
impl_with_cast!(arg u32, f64);
impl_with_cast!(arg i32, f64);
impl_with_cast!(arg u64, f64);
impl_with_cast!(arg i64, f64);
impl_with_cast!(arg usize, f64);
impl_with_cast!(arg isize, f64);

/// # Safety
/// The argument must be a valid pointer to a null-terminated C String.
impl GmArg for &CStr {
    type Arg = *mut c_char;

    #[inline]
    unsafe fn to_arg(arg: *mut c_char) -> Self {
        CStr::from_ptr(arg)
    }
}

/// # Safety
/// The argument must be a valid pointer to a null-terminated C String.
impl GmArg for &[u8] {
    type Arg = *mut c_char;

    #[inline]
    unsafe fn to_arg(arg: *mut c_char) -> Self {
        let len = libc::strlen(arg);
        std::slice::from_raw_parts(arg.cast::<u8>(), len)
    }
}

/// # Safety
/// The argument must be a valid pointer to a null-terminated C String.
impl GmArg for &mut [u8] {
    type Arg = *mut c_char;

    #[inline]
    unsafe fn to_arg(arg: *mut c_char) -> Self {
        let len = libc::strlen(arg);
        std::slice::from_raw_parts_mut(arg.cast::<u8>(), len)
    }
}

/// # Safety
/// The argument must be a valid pointer to a null-terminated C String.
impl GmArg for &BStr {
    type Arg = *mut c_char;

    #[inline]
    unsafe fn to_arg(arg: *mut c_char) -> Self {
        let len = libc::strlen(arg);
        std::slice::from_raw_parts(arg.cast::<u8>(), len).into()
    }
}

/// # Safety
/// The argument must be a valid pointer to a null-terminated C String.
/// # Note
/// This checks for valid UTF-8.
/// If you'd like to skip the check, you may want to use a `&[u8]` argument and [`std::str::from_utf8_unchecked`].
impl GmArg for &str {
    type Arg = *mut c_char;

    #[inline]
    unsafe fn to_arg(arg: *mut c_char) -> Self {
        CStr::from_ptr(arg).to_str().expect("Expected UTF-8 string")
    }
}

impl<'a, B> GmReturn for Cow<'a, B>
where
    B: ?Sized + 'a + ToOwned,
    &'a B: GmReturn,
    B::Owned: GmReturn<Return = <&'a B as GmReturn>::Return>,
{
    type Return = <B::Owned as GmReturn>::Return;

    #[inline]
    unsafe fn to_return(self) -> Self::Return {
        match self {
            Cow::Borrowed(b) => b.to_return(),
            Cow::Owned(o) => o.to_return(),
        }
    }
}

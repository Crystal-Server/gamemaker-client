use std::{ffi::CStr, ops::RangeFrom};

use half::f16;
use nom::{error::ParseError, IResult, InputIter, InputLength, InputTake, Slice};

pub use nom::number::complete::{
    le_f32, le_f64, le_i16, le_i32, le_i64, le_i8, le_u16, le_u32, le_u64, le_u8,
};

/// Recognizes a little endian 2 bytes floating point number.
///
/// *Complete version*: Returns an error if there is not enough input data.
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed};
/// # use half::f16;
/// use gm_utils::parsing::complete::le_f16;
///
/// let parser = |s| {
///   le_f16::<_, (_, ErrorKind)>(s)
/// };
///
/// assert_eq!(parser(&[0x00, 0x41][..]), Ok((&b""[..], f16::from_f32(2.5))));
/// assert_eq!(parser(&[0x01][..]), Err(Err::Error((&[0x01][..], ErrorKind::Eof))));
/// ```
#[inline]
pub fn le_f16<I, E: ParseError<I>>(input: I) -> IResult<I, f16, E>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    match le_u16(input) {
        Ok((i, o)) => Ok((i, f16::from_bits(o))),
        Err(e) => Err(e),
    }
}

/// Recognizes a 1 byte boolean.
///
/// *Complete version*: Returns an error if there is not enough input data.
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed};
/// use gm_utils::parsing::complete::bool;
///
/// let parser = bool::<_, (_, ErrorKind)>;
///
/// assert_eq!(parser(&[0x00, 0x05][..]), Ok((&[5][..], false)));
/// assert_eq!(parser(&b""[..]), Err(Err::Error((&b""[..], ErrorKind::Eof))));
/// ```
#[inline]
pub fn bool<I, E: ParseError<I>>(input: I) -> IResult<I, bool, E>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    match le_u8(input) {
        Ok((i, o)) => Ok((i, o != 0)),
        Err(e) => Err(e),
    }
}

/// Recognizes a null-terminated string.
///
/// *Complete version*: Returns an error if there is not enough input data.
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed};
/// # use std::ffi::CStr;
/// use gm_utils::parsing::complete::string;
///
/// let parser = string::<(_, ErrorKind)>;
///
/// assert_eq!(
///     parser(&b"foo\0bar"[..]),
///     Ok((
///        &b"bar"[..],
///        CStr::from_bytes_with_nul(&b"foo\0"[..]).unwrap()
///     ))
/// );
/// assert_eq!(parser(&b""[..]), Err(Err::Error((&b""[..], ErrorKind::Eof))));
/// ```
#[inline]
pub fn string<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], &'a CStr, E> {
    use nom::error::{make_error, ErrorKind};
    use nom::Err;
    match input.iter().position(|&c| c == 0) {
        Some(i) => {
            let (i, o) = input.take_split(i + 1);
            // SAFETY: our output will contain exactly one null byte, the last.
            Ok((i, unsafe { CStr::from_bytes_with_nul_unchecked(o) }))
        }
        None => Err(Err::Error(make_error(input, ErrorKind::Eof))),
    }
}

//! Utilities to work with Gamemaker buffers.

use std::{marker::PhantomData, os::raw::c_char};

use crate::prelude::*;

/// Helper type for accessing Gamemaker's buffers.
pub struct GmBuffer<'a>(*mut u8, PhantomData<&'a ()>);

/// # Safety
/// the pointer must be pointing to a buffer.
/// Most likely this is a pointer obtained through Gamemaker's `buffer_get_address` and passed as a `String`.
impl GmArg for GmBuffer<'_> {
    type Arg = *mut c_char;

    #[inline]
    unsafe fn to_arg(arg: *mut c_char) -> Self {
        Self(arg.cast(), PhantomData)
    }
}

impl<'buf> GmBuffer<'buf> {
    #[inline]
    pub fn as_ptr(&self) -> *const c_char {
        self.0.cast()
    }

    #[inline]
    pub fn as_mut_ptr(&self) -> *mut c_char {
        self.0.cast()
    }

    /// Return a `&[u8]` given a length.
    /// # Safety
    /// The buffer pointed to must be at least size `len`.
    #[inline]
    pub unsafe fn as_slice<'slice>(&self, len: usize) -> &'slice [u8]
    where
        'buf: 'slice,
    {
        std::slice::from_raw_parts(self.0.cast(), len)
    }

    /// Return a `&mut [u8]` slice given a length.
    /// # Safety
    /// The buffer pointed to must be at least size `len`.
    #[inline]
    pub unsafe fn as_mut_slice<'slice>(&mut self, len: usize) -> &'slice mut [u8]
    where
        'buf: 'slice,
    {
        std::slice::from_raw_parts_mut(self.0.cast(), len)
    }

    /// Return a `&[u8]` assuming the first 8 bytes correspond to the buffer's length.
    /// The slice returned does not include the bytes specifying the size of the buffer.
    /// # Safety
    /// The buffer pointed to must be at least the size of it's first 8 bytes interpreted as a u64.
    #[inline]
    pub unsafe fn as_slice_sized<'slice>(&self) -> &'slice [u8]
    where
        'buf: 'slice,
    {
        let len = u64::from_le(std::ptr::read(self.0.cast()));
        std::slice::from_raw_parts(self.0.offset(8), len as usize - 8)
    }

    /// Return a mutable `&mut [u8]` assuming the first 8 bytes correspond to the buffer's length.
    /// The slice returned does not include the bytes specifying the size of the buffer.
    /// # Safety
    /// The buffer pointed to must be at least the size of it's first 8 bytes interpreted as a u64.
    #[inline]
    pub unsafe fn as_mut_slice_sized<'slice>(&self) -> &'slice [u8]
    where
        'buf: 'slice,
    {
        let len = u64::from_le(std::ptr::read(self.0.cast()));
        std::slice::from_raw_parts_mut(self.0.offset(8), len as usize - 8)
    }
}

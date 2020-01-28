use core::{
    mem::{self, MaybeUninit},
    slice, str,
};

use super::{BASE64_LEN, LEN};
use crate::enc::base64;

/// The raw parts of an [`OcidV0`](struct.OcidV0.html).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct RawOcidV0 {
    /// The ID version.
    ///
    /// This must always be zero.
    pub version: u8,
    /// The content size.
    pub size: [u8; 6],
    /// The [BLAKE3] hash output.
    ///
    /// [BLAKE3]: https://en.wikipedia.org/wiki/BLAKE_(hash_function)#BLAKE3
    pub hash: [u8; 32],
}

impl From<super::OcidV0> for RawOcidV0 {
    #[inline]
    fn from(id: super::OcidV0) -> Self {
        id.into_raw()
    }
}

impl RawOcidV0 {
    /// Creates an ID from the raw bytes.
    #[inline]
    pub fn from_bytes(bytes: [u8; LEN]) -> RawOcidV0 {
        unsafe { mem::transmute(bytes) }
    }

    /// Returns a slice of bytes for all of `ids`.
    #[inline]
    pub fn slice_as_bytes(ids: &[Self]) -> &[u8] {
        let ptr = ids.as_ptr() as *const u8;
        let len = ids.len() * LEN;
        unsafe { slice::from_raw_parts(ptr, len) }
    }

    /// Returns the ID as its bytes.
    #[inline]
    pub fn into_bytes(self) -> [u8; LEN] {
        unsafe { mem::transmute(self) }
    }

    /// Returns a shared reference to the bytes of the ID.
    #[inline]
    pub fn as_bytes(&self) -> &[u8; LEN] {
        unsafe { &*(self as *const Self as *const [u8; LEN]) }
    }

    /// Returns a mutable reference to the bytes of the ID.
    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8; LEN] {
        unsafe { &mut *(self as *mut Self as *mut [u8; LEN]) }
    }

    /// Returns the result of calling `f` on the [Base64] encoding of the ID.
    ///
    /// The string passed into `f` is temporarily stack-allocated.
    ///
    /// [Base64]: https://en.wikipedia.org/wiki/Base64
    #[inline]
    pub fn with_base64<F, T>(&self, f: F) -> T
    where
        F: for<'b> FnOnce(&'b mut str) -> T,
    {
        f(self.encode_base64_uninit(&mut [MaybeUninit::uninit(); BASE64_LEN]))
    }

    /// Writes the [Base64] encoding of the ID to `buf`, returning it as a
    /// mutable UTF-8 string slice.
    ///
    /// [Base64]: https://en.wikipedia.org/wiki/Base64
    #[inline]
    pub fn encode_base64<'b>(
        &self,
        buf: &'b mut [u8; BASE64_LEN],
    ) -> &'b mut str {
        base64::encode_base8_39(self.as_bytes(), buf)
    }

    /// Writes the [Base64] encoding of the ID to `buf`, returning it as a
    /// mutable UTF-8 string slice.
    ///
    /// [Base64]: https://en.wikipedia.org/wiki/Base64
    #[inline]
    pub fn encode_base64_uninit<'b>(
        &self,
        buf: &'b mut [MaybeUninit<u8>; BASE64_LEN],
    ) -> &'b mut str {
        base64::encode_base8_39_uninit(self.as_bytes(), buf)
    }
}

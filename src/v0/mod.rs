//! Version 0.

use core::{
    cmp, fmt, hash,
    mem::{self, MaybeUninit},
    slice,
    convert::TryFrom,
};

mod raw;
pub use raw::RawOcidV0;

const LEN: usize = 39;
const BASE64_LEN: usize = 52;

#[inline]
fn size_from_u64(size: u64) -> Option<[u8; 6]> {
    #[repr(C)]
    struct SizeComposition {
        invalid: [u8; 2],
        valid: [u8; 6],
    }

    let size = size.to_be_bytes();
    let comp: SizeComposition = unsafe { mem::transmute(size) };

    if comp.invalid == [0, 0] {
        Some(comp.valid)
    } else {
        None
    }
}

/// Ocean Content ID, Version 0.
///
/// # Memory Representation
///
/// | Component | Offset | Size | Description
/// | :-------- | :----- | :--- | :----------
/// | Version   | 0      |  1   | ID version number; always zero
/// | Size      | 1      |  6   | [Big-endian] content size
/// | Hash      | 7      | 32   | [BLAKE3] content hash
///
/// This representation has some notable properties:
///
/// - Because the file size is known, retrieving data of the wrong size is an
///   immediate indication that something went wrong.
///
/// - In the high unlikelihood that files of different sizes have the same
///   [BLAKE3] hash, their IDs won't collide despite their hashes colliding.
///
/// - IDs have a [lexicographical order] based on file size. Sorting them
///   results in the smallest file being first and the largest file being last.
///
/// # Base64 Encoding
///
/// The [`Display`] implementation encodes an `OcidV0` as [Base64] using the
/// alphabet described in the homepage for this documentation. The methods
/// [`with_base64`] and [`encode_base64`] enable getting a [`&mut str`] without
/// any heap allocations.
///
/// [`with_base64`]:  #method.with_base64
/// [`encode_base64`]: #method.encode_base64
///
/// [`&mut str`]: https://doc.rust-lang.org/std/primitive.str.html
/// [`Display`]:  https://doc.rust-lang.org/std/fmt/trait.Display.html
///
/// [Base64]:                https://en.wikipedia.org/wiki/Base64
/// [Big-endian]:            https://en.wikipedia.org/wiki/Endianness#Big-endian
/// [BLAKE3]:                https://en.wikipedia.org/wiki/BLAKE_(hash_function)#BLAKE3
/// [lexicographical order]: https://en.wikipedia.org/wiki/Lexicographical_order
#[derive(Clone, Copy)]
pub struct OcidV0(RawOcidV0);

impl PartialEq for OcidV0 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        // Compare fields instead of bodies since this doesn't generate any call
        // to an external libc function.
        self.0.size == other.0.size && self.0.hash == other.0.hash
    }
}

impl Eq for OcidV0 {}

impl PartialOrd for OcidV0 {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OcidV0 {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        // Compare bodies since comparing fields seems to generate a call to
        // `memcmp` anyway.
        self.body().cmp(other.body())
    }
}

impl hash::Hash for OcidV0 {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        state.write(self.as_bytes());
    }

    #[inline]
    fn hash_slice<H: hash::Hasher>(data: &[Self], state: &mut H) {
        state.write(Self::as_bytes_slice(data))
    }
}

impl fmt::Debug for OcidV0 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Manually implement this to remove one layer from the raw information.
        f.debug_struct("OcidV0")
            .field("version", &self.version())
            .field("size", &self.size())
            .field("hash", &self.0.hash)
            .finish()
    }
}

impl fmt::Display for OcidV0 {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.with_base64(|b64| b64.fmt(f))
    }
}

impl OcidV0 {
    /// Generates an ID by hashing `content` using [BLAKE3].
    ///
    /// [BLAKE3]: https://en.wikipedia.org/wiki/BLAKE_(hash_function)#BLAKE3
    #[cfg(any(test, docsrs, feature = "blake3"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "blake3")))]
    #[inline]
    pub fn new(content: &[u8]) -> Option<OcidV0> {
        let size = u64::try_from(content.len()).ok()?;
        let size = size_from_u64(size)?;

        let hash = blake3::hash(content);

        Some(Self::from_parts(size, hash.into()))
    }

    /// Generates a random ID from `rng`.
    ///
    /// If the generated ID has a size of zero, this will attempt once to
    /// generate a non-zero size.
    #[cfg(any(test, docsrs, feature = "rand_core"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand_core")))]
    #[inline]
    pub fn rand<R>(mut rng: R) -> OcidV0
    where
        R: rand_core::RngCore,
    {
        let mut id = Self::from_parts([0; 6], [0; 32]);
        rng.fill_bytes(id.body_mut());

        // Don't loop since `rng` could just emit zeros indefinitely. Calling
        // once more has a high probability of emitting a non-zero value.
        if id.is_empty() {
            rng.fill_bytes(&mut id.0.size);
        }

        id
    }

    /// Attempts to generate a random ID from `rng`, returning an error upon
    /// failure.
    ///
    /// If the generated ID has a size of zero, this will attempt once to
    /// generate a non-zero size.
    #[cfg(any(test, docsrs, feature = "rand_core"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand_core")))]
    #[inline]
    pub fn try_rand<R>(mut rng: R) -> Result<OcidV0, rand_core::Error>
    where
        R: rand_core::RngCore,
    {
        let mut id = Self::from_parts([0; 6], [0; 32]);
        rng.try_fill_bytes(id.body_mut())?;

        // Don't loop since `rng` could just emit zeros indefinitely. Calling
        // once more has a high probability of emitting a non-zero value.
        if id.is_empty() {
            rng.try_fill_bytes(&mut id.0.size)?;
        }

        Ok(id)
    }

    /// Creates an ID from `size` and `hash`.
    #[inline]
    pub const fn from_parts(size: [u8; 6], hash: [u8; 32]) -> OcidV0 {
        Self(RawOcidV0 {
            version: 0,
            size,
            hash,
        })
    }

    /// Creates an ID from the raw internals.
    #[inline]
    pub fn from_raw(raw: RawOcidV0) -> Option<OcidV0> {
        match raw.version {
            0 => Some(Self(raw)),
            _ => None,
        }
    }

    /// Creates an ID from the raw internals.
    #[inline]
    pub const unsafe fn from_raw_unchecked(raw: RawOcidV0) -> OcidV0 {
        Self(raw)
    }

    /// Creates an ID from the raw bytes.
    #[inline]
    pub fn from_bytes(bytes: [u8; LEN]) -> Option<OcidV0> {
        match bytes[0] {
            0 => Some(unsafe { Self::from_bytes_unchecked(bytes) }),
            _ => None,
        }
    }

    /// Creates an ID from the raw bytes.
    #[inline]
    pub fn from_bytes_ref(bytes: &[u8; LEN]) -> Option<&OcidV0> {
        match bytes[0] {
            0 => Some(unsafe { Self::from_bytes_ref_unchecked(bytes) }),
            _ => None,
        }
    }

    /// Creates an ID from the raw bytes.
    #[inline]
    pub fn from_bytes_mut(bytes: &mut [u8; LEN]) -> Option<&mut OcidV0> {
        match bytes[0] {
            0 => Some(unsafe { Self::from_bytes_mut_unchecked(bytes) }),
            _ => None,
        }
    }

    /// Creates an ID from the raw bytes.
    #[inline]
    pub unsafe fn from_bytes_unchecked(bytes: [u8; LEN]) -> OcidV0 {
        mem::transmute(bytes)
    }

    /// Creates an ID from the raw bytes.
    #[inline]
    pub unsafe fn from_bytes_ref_unchecked(bytes: &[u8; LEN]) -> &OcidV0 {
        &*(bytes.as_ptr() as *const Self)
    }

    /// Creates an ID from the raw bytes.
    #[inline]
    pub unsafe fn from_bytes_mut_unchecked(
        bytes: &mut [u8; LEN],
    ) -> &mut OcidV0 {
        &mut *(bytes.as_mut_ptr() as *mut Self)
    }

    /// Creates an ID from the raw bytes.
    #[inline]
    pub fn from_slice(bytes: &[u8]) -> Option<(&OcidV0, &[u8])> {
        if bytes.len() >= LEN {
            let head = unsafe { &*(bytes.as_ptr() as *const [u8; LEN]) };
            let tail = &bytes[LEN..];
            let id = Self::from_bytes_ref(head)?;
            Some((id, tail))
        } else {
            None
        }
    }

    /// Creates an ID from the raw bytes.
    #[inline]
    pub fn from_slice_mut(
        bytes: &mut [u8],
    ) -> Option<(&mut OcidV0, &mut [u8])> {
        if bytes.len() >= LEN {
            let head = unsafe { &mut *(bytes.as_mut_ptr() as *mut [u8; LEN]) };
            let tail = &mut bytes[LEN..];
            let id = Self::from_bytes_mut(head)?;
            Some((id, tail))
        } else {
            None
        }
    }

    /// Creates an ID that represents an empty file.
    #[inline]
    pub fn empty() -> OcidV0 {
        Self::from_parts([0; 6], [0; 32])
    }

    /// Returns a slice of raw IDs for all of `ids`.
    #[inline]
    pub fn as_raw_slice(ids: &[Self]) -> &[RawOcidV0] {
        let ptr = ids.as_ptr() as *const RawOcidV0;
        unsafe { slice::from_raw_parts(ptr, ids.len()) }
    }

    /// Returns a slice of bytes for all of `ids`.
    #[inline]
    pub fn as_bytes_slice(ids: &[Self]) -> &[u8] {
        let ptr = ids.as_ptr() as *const u8;
        let len = ids.len() * LEN;
        unsafe { slice::from_raw_parts(ptr, len) }
    }

    /// Returns the ID version.
    ///
    /// In correct code, this always returns 0.
    #[inline]
    pub fn version(&self) -> u8 {
        let version = self.0.version;
        debug_assert_eq!(version, 0, "{} is not version 0", self);
        version
    }

    /// Returns the size of the source content as big-endian integer bytes.
    #[inline]
    pub fn size(&self) -> &[u8; 6] {
        &self.0.size
    }

    /// Returns the size of the source content as a native integer.
    #[inline]
    pub fn size_u64(&self) -> u64 {
        // SAFETY: The bytes after `size` belong to `hash`. The top 2 bytes are
        // read but then discarded by the shift.
        let size = unsafe {
            u64::from_be_bytes(*self.0.size.as_ptr().cast::<[u8; 8]>())
        };
        size >> 16
    }

    /// Returns whether the content has a size of 0.
    ///
    /// While it is valid for an ID to have a size of 0, it is generally
    /// indicative of a programming error. Ocean itself does not accept IDs with
    /// a size of 0.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.size == [0; 6]
    }

    /// Returns the [BLAKE3] hash of the content.
    ///
    /// [BLAKE3]: https://en.wikipedia.org/wiki/BLAKE_(hash_function)#BLAKE3
    #[inline]
    pub fn hash(&self) -> &[u8; 32] {
        &self.0.hash
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
        self.0.with_base64(f)
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
        self.0.encode_base64(buf)
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
        self.0.encode_base64_uninit(buf)
    }

    /// Returns a shared reference to the body of the ID, i.e. everything after
    /// the version number.
    #[inline]
    pub fn body(&self) -> &[u8; LEN - 1] {
        unsafe { &*(self.0.size.as_ptr() as *const _) }
    }

    /// Returns a mutable reference to the body of the ID, i.e. everything after
    /// the version number.
    #[inline]
    pub fn body_mut(&mut self) -> &mut [u8; LEN - 1] {
        unsafe { &mut *(self.0.size.as_mut_ptr() as *mut _) }
    }

    /// Converts `self` into a raw ID.
    #[inline]
    pub fn into_raw(self) -> RawOcidV0 {
        self.0
    }

    /// Returns a shared reference to the raw ID.
    #[inline]
    pub fn as_raw(&self) -> &RawOcidV0 {
        &self.0
    }

    /// Returns a mutable reference to the raw ID.
    #[inline]
    pub unsafe fn as_raw_mut(&mut self) -> &mut RawOcidV0 {
        &mut self.0
    }

    /// Returns the ID as its bytes.
    #[inline]
    pub fn into_bytes(self) -> [u8; LEN] {
        self.into_raw().into_bytes()
    }

    /// Returns a shared reference to the bytes of the ID.
    #[inline]
    pub fn as_bytes(&self) -> &[u8; LEN] {
        self.as_raw().as_bytes()
    }

    /// Returns a mutable reference to the bytes of the ID.
    #[inline]
    pub unsafe fn as_bytes_mut(&mut self) -> &mut [u8; LEN] {
        self.as_raw_mut().as_bytes_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::RngCore;

    #[test]
    fn size_u64() {
        let mut rng = rand_core::OsRng;

        let mut gen_size = || -> (u64, [u8; 6]) {
            let size_u64 = rng.next_u64() >> 16;
            let size = size_from_u64(size_u64).unwrap();
            (size_u64, size)
        };

        for _ in 0..1024 {
            let (size_u64, size) = gen_size();

            let id = OcidV0::from_parts(size, [0; 32]);
            assert_eq!(id.size_u64(), size_u64);
        }
    }
}

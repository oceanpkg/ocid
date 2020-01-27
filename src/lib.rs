//! Ocean Content ID (OCID).
//!
//! These IDs serve as a way to address content by their hash within the Ocean
//! package manager.
//!
//! # Memory Representation
//!
//! Every ID follows this basic memory representation:
//!
//! | Component | Offset | Size | Description
//! | :-------- | :----- | :--- | :----------
//! | Version   | 0      | 1    | ID version number; currently only 0
//! | Body      | 1      | _n_  | ID value
//!
//! _Body_ is defined entirely by the ID version. Check out the
//! ["Memory Representation"](struct.OcidV0.html#memory-representation) section
//! for [`OcidV0`] to see how it defines this.
//!
//! # Base64 Encoding
//!
//! Ocean's content IDs are represented in [UTF-8] using a [Base64] encoding. It
//! uses the character set described in [RFC 4648 §5] and can thus be safely
//! used in URLs and file paths.
//!
//! | Values | Characters
//! | :----- | :---------
//! | 0      | `-`
//! | 1-10   | `0123456789`
//! | 11-36  | `ABCDEFGHIJKLMNOPQRSTUVWXYZ`
//! | 37     | `_`
//! | 38-63  | `abcdefghijklmnopqrstuvwxyz`
//!
//! As a result of this alphabet, an [`OcidV0`] can be encoded as:
//!
//! ```txt
//! ------IsAAc5y5h0P2AEb3mPtfrloZ2IVxrdMhEfUeAeo6iwUjr-
//! ```
//!
//! Note that characters are ordered by their [ASCII] value. This allows IDs to
//! have the same [lexicographical order] regardless if they're represented as
//! raw bytes, [Base64], or even [hexadecimal].
//!
#![cfg_attr(not(feature = "rand_core"), doc = "```rust,ignore")]
#![cfg_attr(feature = "rand_core", doc = "```")]
//! # use ocid::OcidV0;
//! # use rand_core::OsRng;
//! let mut rng = OsRng;
//! let a = OcidV0::rand(&mut rng);
//! let b = OcidV0::rand(&mut rng);
//!
//! let direct_ord = a.cmp(&b);
//! let base64_ord = a.to_string().cmp(&b.to_string());
//!
//! assert_eq!(direct_ord, base64_ord);
//! ```
//!
//! # Security Considerations
//!
//! OCIDs use cryptographic hashes for their collision reistance. This is
//! important because two files having the same content ID would result in
//! unwanted behavior.
//!
//! Note that OCIDs _do not_ use constant-time comparison functions, despite the
//! cryptographic hash. **This is intentional.** OCIDs are not used as security
//! codes, such as [message authentication codes (MACs)][mac]. They are strictly
//! content addresses. As a result, comparisons should be fast.
//!
//! [`OcidV0`]: struct.OcidV0.html
//!
//! [ASCII]:                 https://en.wikipedia.org/wiki/ASCII
//! [Base64]:                https://en.wikipedia.org/wiki/Base64
//! [hexadecimal]:           https://en.wikipedia.org/wiki/Hexadecimal
//! [lexicographical order]: https://en.wikipedia.org/wiki/Lexicographical_order
//! [mac]:                   https://en.wikipedia.org/wiki/Message_authentication_code
//! [RFC 4648 §5]:           https://tools.ietf.org/html/rfc4648#section-5
//! [UTF-8]:                 https://en.wikipedia.org/wiki/UTF-8

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(test), no_std)]

use core::fmt;

mod base64;

pub mod v0;

#[doc(inline)]
pub use v0::OcidV0;

/// Ocean Content ID.
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum Ocid {
    /// Version 0; analogous to an [`OcidV0`].
    ///
    /// [`OcidV0`]: struct.OcidV0.html
    V0 {
        /// The content size.
        size: [u8; 6],
        /// The [BLAKE3] hash output.
        ///
        /// [BLAKE3]: https://en.wikipedia.org/wiki/BLAKE_(hash_function)#BLAKE3
        hash: [u8; 32],
    },
}

impl From<v0::RawOcidV0> for Ocid {
    #[inline]
    fn from(v0: v0::RawOcidV0) -> Self {
        Ocid::V0 {
            size: v0.size,
            hash: v0.hash,
        }
    }
}

impl From<OcidV0> for Ocid {
    #[inline]
    fn from(v0: OcidV0) -> Self {
        v0.into_raw().into()
    }
}

impl fmt::Debug for Ocid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Ocid::V0 { size, hash } => {
                let size = u64::from_be_bytes([
                    0, 0, size[0], size[1], size[2], size[3], size[4], size[5],
                ]);

                f.debug_struct("V0")
                    .field("size", &size)
                    .field("hash", hash)
                    .finish()
            }
        }
    }
}

impl fmt::Display for Ocid {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.with_base64(|b64| b64.fmt(f))
    }
}

impl Ocid {
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
        match *self {
            Ocid::V0 { size, hash } => {
                OcidV0::from_parts(size, hash).with_base64(f)
            }
        }
    }
}

//! [Base64] encoding/decoding.
//!
//! # Alphabet
//!
//! | Values | Characters
//! | :----- | :---------
//! | 0      | `-`
//! | 1-10   | `0123456789`
//! | 11-36  | `ABCDEFGHIJKLMNOPQRSTUVWXYZ`
//! | 37     | `_`
//! | 38-63  | `abcdefghijklmnopqrstuvwxyz`
//!
//! [Base64]: https://en.wikipedia.org/wiki/Base64

use core::{mem::MaybeUninit, str};

#[cfg(test)]
mod tests;

const LEN_39: usize = 39 / 3 * 4;

// URL-safe character set with lexicographical ordering.
const ALPHABET: [u8; 64] = *b"-\
                              0123456789\
                              ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                              _\
                              abcdefghijklmnopqrstuvwxyz";

/// Encodes the 39 base-8 `bytes` into `buf` as base-64, returning the encoded
/// UTF-8 string.
pub fn encode_base8_39<'a>(
    bytes: &[u8; 39],
    buf: &'a mut [MaybeUninit<u8>; LEN_39],
) -> &'a mut str {
    #![allow(clippy::many_single_char_names)]

    // This uses the same strategy as version 0.11 of the `base64` crate,
    // however it handles all of `bytes` at once.

    macro_rules! read_u64 {
        ($($offset:expr),+) => {
            ($({
                let ptr = bytes.as_ptr().add($offset * 6) as *const [u8; 8];
                u64::from_be_bytes(*ptr)
            }),+)
        }
    }

    let (a, b, c, d, e, f) = unsafe { read_u64!(0, 1, 2, 3, 4, 5) };

    #[rustfmt::skip]
    let g = u32::from_be_bytes([
        bytes[36], bytes[37],
        bytes[38], 0,
    ]);

    const LOW_SIX_BITS_64: u64 = 0x3F;
    const LOW_SIX_BITS_32: u32 = LOW_SIX_BITS_64 as u32;

    macro_rules! write_u64 {
        ($($i:expr),+) => {{
            let offset = 0;
            $(
                buf[offset] = MaybeUninit::new(
                    ALPHABET[(($i >> 58) & LOW_SIX_BITS_64) as usize]
                );
                buf[offset + 1] = MaybeUninit::new(
                    ALPHABET[(($i >> 52) & LOW_SIX_BITS_64) as usize]
                );
                buf[offset + 2] = MaybeUninit::new(
                    ALPHABET[(($i >> 46) & LOW_SIX_BITS_64) as usize]
                );
                buf[offset + 3] = MaybeUninit::new(
                    ALPHABET[(($i >> 40) & LOW_SIX_BITS_64) as usize]
                );
                buf[offset + 4] = MaybeUninit::new(
                    ALPHABET[(($i >> 34) & LOW_SIX_BITS_64) as usize]
                );
                buf[offset + 5] = MaybeUninit::new(
                    ALPHABET[(($i >> 28) & LOW_SIX_BITS_64) as usize]
                );
                buf[offset + 6] = MaybeUninit::new(
                    ALPHABET[(($i >> 22) & LOW_SIX_BITS_64) as usize]
                );
                buf[offset + 7] = MaybeUninit::new(
                    ALPHABET[(($i >> 16) & LOW_SIX_BITS_64) as usize]
                );

                #[allow(unused_variables)]
                let offset = offset + 8;
            )+
        }}
    }

    write_u64!(a, b, c, d, e, f);

    let offset = LEN_39 - 4;
    buf[offset] =
        MaybeUninit::new(ALPHABET[((g >> 26) & LOW_SIX_BITS_32) as usize]);
    buf[offset + 1] =
        MaybeUninit::new(ALPHABET[((g >> 20) & LOW_SIX_BITS_32) as usize]);
    buf[offset + 2] =
        MaybeUninit::new(ALPHABET[((g >> 14) & LOW_SIX_BITS_32) as usize]);
    buf[offset + 3] =
        MaybeUninit::new(ALPHABET[((g >> 8) & LOW_SIX_BITS_32) as usize]);

    unsafe {
        let buf = &mut *(buf as *mut _ as *mut [u8; LEN_39]);
        str::from_utf8_unchecked_mut(buf)
    }
}

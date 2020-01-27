use core::{mem::MaybeUninit, str};

#[cfg(test)]
mod tests;

const LEN_42: usize = 42 / 3 * 4;

// URL-safe character set with lexicographical ordering.
const ALPHABET: [u8; 64] = *b"-\
                              0123456789\
                              ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                              _\
                              abcdefghijklmnopqrstuvwxyz";

pub fn encode_42<'a>(
    bytes: &[u8; 42],
    buf: &'a mut [MaybeUninit<u8>; LEN_42],
) -> &'a mut str {
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
    let g = u64::from_be_bytes([
        bytes[36], bytes[37],
        bytes[38], bytes[39],
        bytes[40], bytes[41],
        0, 0,
    ]);

    const LOW_SIX_BITS: u64 = 0x3F;

    macro_rules! write_u64 {
        ($($i:expr),+) => {{
            let offset = 0;
            $(
                buf[offset + 0] = MaybeUninit::new(
                    ALPHABET[(($i >> 58) & LOW_SIX_BITS) as usize]
                );
                buf[offset + 1] = MaybeUninit::new(
                    ALPHABET[(($i >> 52) & LOW_SIX_BITS) as usize]
                );
                buf[offset + 2] = MaybeUninit::new(
                    ALPHABET[(($i >> 46) & LOW_SIX_BITS) as usize]
                );
                buf[offset + 3] = MaybeUninit::new(
                    ALPHABET[(($i >> 40) & LOW_SIX_BITS) as usize]
                );
                buf[offset + 4] = MaybeUninit::new(
                    ALPHABET[(($i >> 34) & LOW_SIX_BITS) as usize]
                );
                buf[offset + 5] = MaybeUninit::new(
                    ALPHABET[(($i >> 28) & LOW_SIX_BITS) as usize]
                );
                buf[offset + 6] = MaybeUninit::new(
                    ALPHABET[(($i >> 22) & LOW_SIX_BITS) as usize]
                );
                buf[offset + 7] = MaybeUninit::new(
                    ALPHABET[(($i >> 16) & LOW_SIX_BITS) as usize]
                );

                #[allow(unused_variables)]
                let offset = offset + 8;
            )+
        }}
    }

    write_u64!(a, b, c, d, e, f, g);

    unsafe {
        let buf = &mut *(buf as *mut _ as *mut [u8; LEN_42]);
        str::from_utf8_unchecked_mut(buf)
    }
}

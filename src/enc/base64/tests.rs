use core::str;
use rand_core::RngCore;

use super::*;

// Tests that our implementation is correct, using the implementation from the
// `base64` crate as a reference. Having this test enables us to update the
// implementation of `encode_base8_39` to something faster while ensuring that
// it stays correct.
#[test]
fn encode_base8_39() {
    let mut rng = rand_core::OsRng;
    let mut base64_buf = [0u8; LEN_39 * 2];
    let mut crate_buf = [0u8; LEN_39];

    for _ in 0..2048 {
        let mut bytes = [0u8; 39];
        rng.fill_bytes(&mut bytes);

        let base64_len =
            base64::encode_to_slice(&bytes, &mut base64_buf, &ALPHABET);
        assert_eq!(base64_len, LEN_39);
        let base64 = str::from_utf8(&base64_buf[..base64_len]).unwrap();

        let encoded = super::encode_base8_39(&bytes, &mut crate_buf);

        assert_eq!(encoded, base64);
    }
}

// Sanity check that `ALPHABET` is indeed sorted.
#[test]
fn sorted_alphabet() {
    for i in 0..(ALPHABET.len() - 1) {
        let j = i + 1;

        let a = ALPHABET[i] as char;
        let b = ALPHABET[j] as char;

        assert!(
            a < b,
            "alphabet not sorted; {a} at {i} is not less than {b} at {j}",
            i = i,
            j = j,
            a = a,
            b = b,
        );
    }
}

mod base64 {
    use core::convert::TryInto;

    fn read_u64(s: &[u8]) -> u64 {
        u64::from_be_bytes(s[..8].try_into().unwrap())
    }

    // Taken verbatim from the `base64` crate. This code is vendored directly
    // since the `base64` crate does not provide a way to encode with a provided
    // alphabet.
    //
    // Copyright (c) Alice Maz, Marshall Pierce
    #[rustfmt::skip]
    pub fn encode_to_slice(input: &[u8], output: &mut [u8], encode_table: &[u8; 64]) -> usize {
        let mut input_index: usize = 0;

        const BLOCKS_PER_FAST_LOOP: usize = 4;
        const LOW_SIX_BITS: u64 = 0x3F;

        // we read 8 bytes at a time (u64) but only actually consume 6 of those bytes. Thus, we need
        // 2 trailing bytes to be available to read..
        let last_fast_index = input.len().saturating_sub(BLOCKS_PER_FAST_LOOP * 6 + 2);
        let mut output_index = 0;

        if last_fast_index > 0 {
            while input_index <= last_fast_index {
                // Major performance wins from letting the optimizer do the bounds check once, mostly
                // on the output side
                let input_chunk = &input[input_index..(input_index + (BLOCKS_PER_FAST_LOOP * 6 + 2))];
                let output_chunk = &mut output[output_index..(output_index + BLOCKS_PER_FAST_LOOP * 8)];

                // Hand-unrolling for 32 vs 16 or 8 bytes produces yields performance about equivalent
                // to unsafe pointer code on a Xeon E5-1650v3. 64 byte unrolling was slightly better for
                // large inputs but significantly worse for 50-byte input, unsurprisingly. I suspect
                // that it's a not uncommon use case to encode smallish chunks of data (e.g. a 64-byte
                // SHA-512 digest), so it would be nice if that fit in the unrolled loop at least once.
                // Plus, single-digit percentage performance differences might well be quite different
                // on different hardware.

                let input_u64 = read_u64(&input_chunk[0..]);

                output_chunk[0] = encode_table[((input_u64 >> 58) & LOW_SIX_BITS) as usize];
                output_chunk[1] = encode_table[((input_u64 >> 52) & LOW_SIX_BITS) as usize];
                output_chunk[2] = encode_table[((input_u64 >> 46) & LOW_SIX_BITS) as usize];
                output_chunk[3] = encode_table[((input_u64 >> 40) & LOW_SIX_BITS) as usize];
                output_chunk[4] = encode_table[((input_u64 >> 34) & LOW_SIX_BITS) as usize];
                output_chunk[5] = encode_table[((input_u64 >> 28) & LOW_SIX_BITS) as usize];
                output_chunk[6] = encode_table[((input_u64 >> 22) & LOW_SIX_BITS) as usize];
                output_chunk[7] = encode_table[((input_u64 >> 16) & LOW_SIX_BITS) as usize];

                let input_u64 = read_u64(&input_chunk[6..]);

                output_chunk[8] = encode_table[((input_u64 >> 58) & LOW_SIX_BITS) as usize];
                output_chunk[9] = encode_table[((input_u64 >> 52) & LOW_SIX_BITS) as usize];
                output_chunk[10] = encode_table[((input_u64 >> 46) & LOW_SIX_BITS) as usize];
                output_chunk[11] = encode_table[((input_u64 >> 40) & LOW_SIX_BITS) as usize];
                output_chunk[12] = encode_table[((input_u64 >> 34) & LOW_SIX_BITS) as usize];
                output_chunk[13] = encode_table[((input_u64 >> 28) & LOW_SIX_BITS) as usize];
                output_chunk[14] = encode_table[((input_u64 >> 22) & LOW_SIX_BITS) as usize];
                output_chunk[15] = encode_table[((input_u64 >> 16) & LOW_SIX_BITS) as usize];

                let input_u64 = read_u64(&input_chunk[12..]);

                output_chunk[16] = encode_table[((input_u64 >> 58) & LOW_SIX_BITS) as usize];
                output_chunk[17] = encode_table[((input_u64 >> 52) & LOW_SIX_BITS) as usize];
                output_chunk[18] = encode_table[((input_u64 >> 46) & LOW_SIX_BITS) as usize];
                output_chunk[19] = encode_table[((input_u64 >> 40) & LOW_SIX_BITS) as usize];
                output_chunk[20] = encode_table[((input_u64 >> 34) & LOW_SIX_BITS) as usize];
                output_chunk[21] = encode_table[((input_u64 >> 28) & LOW_SIX_BITS) as usize];
                output_chunk[22] = encode_table[((input_u64 >> 22) & LOW_SIX_BITS) as usize];
                output_chunk[23] = encode_table[((input_u64 >> 16) & LOW_SIX_BITS) as usize];

                let input_u64 = read_u64(&input_chunk[18..]);

                output_chunk[24] = encode_table[((input_u64 >> 58) & LOW_SIX_BITS) as usize];
                output_chunk[25] = encode_table[((input_u64 >> 52) & LOW_SIX_BITS) as usize];
                output_chunk[26] = encode_table[((input_u64 >> 46) & LOW_SIX_BITS) as usize];
                output_chunk[27] = encode_table[((input_u64 >> 40) & LOW_SIX_BITS) as usize];
                output_chunk[28] = encode_table[((input_u64 >> 34) & LOW_SIX_BITS) as usize];
                output_chunk[29] = encode_table[((input_u64 >> 28) & LOW_SIX_BITS) as usize];
                output_chunk[30] = encode_table[((input_u64 >> 22) & LOW_SIX_BITS) as usize];
                output_chunk[31] = encode_table[((input_u64 >> 16) & LOW_SIX_BITS) as usize];

                output_index += BLOCKS_PER_FAST_LOOP * 8;
                input_index += BLOCKS_PER_FAST_LOOP * 6;
            }
        }

        // Encode what's left after the fast loop.

        const LOW_SIX_BITS_U8: u8 = 0x3F;

        let rem = input.len() % 3;
        let start_of_rem = input.len() - rem;

        // start at the first index not handled by fast loop, which may be 0.

        while input_index < start_of_rem {
            let input_chunk = &input[input_index..(input_index + 3)];
            let output_chunk = &mut output[output_index..(output_index + 4)];

            output_chunk[0] = encode_table[(input_chunk[0] >> 2) as usize];
            output_chunk[1] =
                encode_table[((input_chunk[0] << 4 | input_chunk[1] >> 4) & LOW_SIX_BITS_U8) as usize];
            output_chunk[2] =
                encode_table[((input_chunk[1] << 2 | input_chunk[2] >> 6) & LOW_SIX_BITS_U8) as usize];
            output_chunk[3] = encode_table[(input_chunk[2] & LOW_SIX_BITS_U8) as usize];

            input_index += 3;
            output_index += 4;
        }

        if rem == 2 {
            output[output_index] = encode_table[(input[start_of_rem] >> 2) as usize];
            output[output_index + 1] = encode_table[((input[start_of_rem] << 4
                | input[start_of_rem + 1] >> 4)
                & LOW_SIX_BITS_U8) as usize];
            output[output_index + 2] =
                encode_table[((input[start_of_rem + 1] << 2) & LOW_SIX_BITS_U8) as usize];
            output_index += 3;
        } else if rem == 1 {
            output[output_index] = encode_table[(input[start_of_rem] >> 2) as usize];
            output[output_index + 1] =
                encode_table[((input[start_of_rem] << 4) & LOW_SIX_BITS_U8) as usize];
            output_index += 2;
        }

        output_index
    }
}

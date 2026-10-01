use core::blake::{blake2s_compress, blake2s_finalize};
use core::box::BoxTrait;

/// 2^31, used for encoding small felt252 values.
const MSB_U32: u32 = 0x80000000;

pub const BLAKE2S_256_INITIAL_STATE: [u32; 8] = [
    0x6B08E647, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];

pub fn hash_u32s(mut values: Span<u32>) -> Box<[u32; 8]> {
    let mut state = BoxTrait::new(BLAKE2S_256_INITIAL_STATE);
    let mut byte_count = 0;
    if let Some(mut msg) = values.multi_pop_front::<16>() {
        byte_count += 64;
        while let Some(head) = values.multi_pop_front::<16>() {
            // Compress and re-fill msg.
            state = blake2s_compress(state, byte_count, *msg);
            msg = head;
            byte_count += 64;
        }

        // Here `msg` is the last full 16-element block, if there are no remaining values, we can
        // finalize the hash and return the result.
        if values.is_empty() {
            return blake2s_finalize(state, byte_count, *msg);
        }

        // Otherwise, update the state and handle the remaining values.
        state = blake2s_compress(state, byte_count, *msg);
    }

    /// pad the remaining values to a full 16-element block and hash them as a final block.
    let mut msg = array![];
    let i = values.len();
    msg.append_span(values);
    for _ in i..16 {
        msg.append(0);
    }
    byte_count += i * 4;
    blake2s_finalize(state, byte_count, *msg.span().try_into().unwrap())
}

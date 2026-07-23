use itertools::zip_eq;
use stwo::core::utils::SliceExt;

use crate::context::{Context, Var};
use crate::eval;
use crate::ivalue::IValue;
use crate::ops::sub;

/// Concatenates the little-endian byte encodings of each u32 in `words`.
///
/// The output length `M` must equal `4 * N`.
pub fn bytes_from_le_u32s<const N: usize, const M: usize>(words: [u32; N]) -> [u8; M] {
    let mut bytes = [0u8; M];
    for (dst, word) in zip_eq(bytes.checked_as_chunks_mut::<4>(), words) {
        *dst = word.to_le_bytes();
    }
    bytes
}

/// Decodes each little-endian 4-byte chunk of `bytes` into a u32 word.
///
/// The input length `M` must equal `4 * N`.
pub fn le_u32s_from_bytes<const N: usize, const M: usize>(bytes: [u8; M]) -> [u32; N] {
    let mut words = [0u32; N];
    for (word, src) in zip_eq(&mut words, bytes.checked_as_chunks::<4>()) {
        *word = u32::from_le_bytes(*src);
    }
    words
}

/// Implements a multiplexer.
/// Given a vector `values` and an index (represented in its bit decomposition `index_bits`)
/// returns a new variable equal to `values[index]`.
///
/// A multiplexer costs `3 * (n - 1) + log_2(n)` gates, where `n` is `values.len()`.
pub fn select_by_index<Value: IValue>(
    context: &mut Context<Value>,
    values: &[Var],
    index_bits: &[Var],
) -> Var {
    assert!(values.len().is_power_of_two());
    assert_eq!(values.len(), 1 << index_bits.len());

    let one = context.one();
    let mut layer = values.to_vec();
    let mut curr_layer_len = layer.len();

    for &bit in index_bits {
        let one_minus_bit = sub(context, one, bit);
        for i in (0..curr_layer_len).step_by(2) {
            let (left, right) = (layer[i], layer[i + 1]);
            layer[i >> 1] = eval!(context, ((one_minus_bit) * (left)) + ((bit) * (right)));
        }
        curr_layer_len >>= 1;
    }
    layer[0]
}

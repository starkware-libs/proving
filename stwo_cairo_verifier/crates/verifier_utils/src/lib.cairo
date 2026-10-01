#[cfg(test)]
mod test;
pub mod zip_eq;
#[cfg(test)]
mod zip_eq_test;
use bounded_int::impls::*;
use bounded_int::{NZ_U32_SHIFT, div_rem, upcast};

/// Equals `2^31`.
pub const M31_SHIFT: felt252 = 0x80000000; // 2**31.

/// Equals `(2^31)^4`.
pub const M31_SHIFT_POW_4: felt252 = M31_SHIFT * M31_SHIFT * M31_SHIFT * M31_SHIFT;

// Equals `(2^31)^8`
pub const M31_SHIFT_POW_8: felt252 = M31_SHIFT_POW_4 * M31_SHIFT_POW_4;

pub mod blake2s;
pub mod poseidon252;
// TODO(Gil): Remove this global use and use the explicit module imports instead everywhere it is
// currently used.
#[cfg(not(feature: "poseidon252_verifier"))]
use stwo_verifier_utils::blake2s::*;
#[cfg(feature: "poseidon252_verifier")]
use stwo_verifier_utils::poseidon252::*;


/// Deconstructs a `felt252` to 8 u32 little-endian limbs.
pub fn deconstruct_f252(x: felt252) -> Box<[u32; 8]> {
    let u256 { low, high } = x.into();

    // Deconstruct the low 128 bits.
    let (q, r0) = div_rem(low, NZ_U32_SHIFT);
    let (q, r1) = div_rem(q, NZ_U32_SHIFT);
    let (r3, r2) = div_rem(q, NZ_U32_SHIFT);

    // Deconstruct the high 128 bits.
    let (q, r4) = div_rem(high, NZ_U32_SHIFT);
    let (q, r5) = div_rem(q, NZ_U32_SHIFT);
    let (r7, r6) = div_rem(q, NZ_U32_SHIFT);

    BoxTrait::new(
        [
            upcast(r0), upcast(r1), upcast(r2), upcast(r3), upcast(r4), upcast(r5), upcast(r6),
            upcast(r7),
        ],
    )
}

/// A utility function used to modify the most significant bits of a felt252.
/// Provided that `n_packed_elements` < 8 and `word` < 2^248, the functions injects
/// `n_packed_elements` into the bits at indices [248, 251) of `word`.
///
/// Typically, `word` is a packing of u32s or M31s, `n_packed_elements` is the number
/// of packed elements, and the resulting felt252 is fed into a hash.
/// The purpose of this function in this case is to avoid hash collisions between different-length
/// lists of u32s or M31s that would lead to the same packing.
#[inline(always)]
pub fn add_length_padding(word: felt252, n_packed_elements: usize) -> felt252 {
    word + n_packed_elements.into() * M31_SHIFT_POW_8
}

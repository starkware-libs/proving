use stwo::core::fields::m31::M31;

use crate::context::{Context, Var};
use crate::eval;
use crate::ivalue::IValue;
use crate::ops::CircuitOps;
use crate::simd::Simd;
use crate::wrappers::M31Wrapper;

#[cfg(test)]
#[path = "extract_bits_test.rs"]
pub mod test;

/// Helper trait for `extract_bits` and `combine_bits` to allow using the same code for both
/// [M31Wrapper<Var>] and [Simd].
pub trait BitPackingOps: CircuitOps + Clone {
    /// Multiplies `self` by the base-field scalar `scalar` (applied to every element of `self` for
    /// vector types such as [Simd]).
    fn scalar_mul(&self, context: &mut Context<impl IValue>, scalar: M31Wrapper<Var>) -> Self;

    /// Returns an *unconstrained* bit value initialized (hint) with the LSB of `self` (applied to
    /// every element of `self` for vector types such as [Simd]).
    ///
    /// NOTE: The result is guaranteed to be either 0 or 1, but the caller MUST verify that it is
    /// indeed the LSB of `self`.
    fn guess_lsb(&self, context: &mut Context<impl IValue>) -> Self;

    /// Asserts that the value is either 0 or 1.
    fn assert_bit(&self, context: &mut Context<impl IValue>);

    /// Adds gates to the circuit that assert that `a` and `b` are equal.
    fn eq(context: &mut Context<impl IValue>, a: &Self, b: &Self);
}

impl BitPackingOps for Simd {
    fn scalar_mul(&self, context: &mut Context<impl IValue>, scalar: M31Wrapper<Var>) -> Self {
        Simd::scalar_mul(context, self, &scalar)
    }

    fn guess_lsb(&self, context: &mut Context<impl IValue>) -> Self {
        self.guess_lsb(context)
    }

    fn assert_bit(&self, context: &mut Context<impl IValue>) {
        self.assert_bits(context);
    }

    fn eq(context: &mut Context<impl IValue>, a: &Self, b: &Self) {
        Simd::eq(context, a, b);
    }
}

/// Returns the `n_bits`-bit binary decomposition of `input` (per lane for vector types such as
/// [Simd]).
/// If any input exceeds `2^n_bits - 1`, a circuit constraints is going to be violated and
/// `context.is_circuit_valid()` will return `false`.
///
/// `n_bits` must be at most 30, so that the decomposition is unique (a 31-bit decomposition
/// would allow `0` to be represented both as `0b0000...0000` and as `0b1111...1111`).
pub fn extract_bits<T: BitPackingOps>(
    context: &mut Context<impl IValue>,
    input: &T,
    n_bits: u32,
) -> Vec<T> {
    assert!(n_bits <= 30, "extract_bits supports at most 30 bits, got {n_bits}.");
    let inv_two = M31Wrapper::const_m31(context, M31::from(2).inverse());

    let mut value = input.clone();
    let mut bits = Vec::new();
    for _ in 0..(n_bits - 1) {
        let lsb = value.guess_lsb(context);
        bits.push(lsb.clone());
        value = eval!(context, (value) - (lsb));
        value = value.scalar_mul(context, inv_two);
    }

    // `value` is now the MSB. Check that it is 0 or 1.
    value.assert_bit(context);
    bits.push(value.clone());

    bits
}

/// Packs little-endian bits into an integer (per lane for vector types such as [Simd]).
/// This is the inverse of `extract_bits`.
///
/// Assumes each `bits[i]` contains only 0/1 values;
/// returns `Σ bits[i] * (1 << i)`.
///
/// `bits` must be non-empty and have at most 30 elements (so the result fits in M31 without
/// wrapping).
pub fn combine_bits<T: BitPackingOps>(context: &mut Context<impl IValue>, bits: &[T]) -> T {
    assert!(bits.len() <= 30, "combine_bits supports at most 30 bits, got {}.", bits.len());
    let mut iter = bits.iter().rev();
    let mut res = iter.next().unwrap().clone();
    let two = M31Wrapper::const_m31(context, 2.into());
    for bit in iter {
        res = res.scalar_mul(context, two);
        res = eval!(context, (res) + (*bit));
    }
    res
}

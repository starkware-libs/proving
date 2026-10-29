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

    /// Returns an *unconstrained* value initialized (hint) with `1/x` if `x != 0` and `0` if
    /// `x = 0` (applied to every element of `self` for vector types such as [Simd]).
    ///
    /// NOTE: The caller MUST verify that the result is correct.
    fn guess_inv_or_zero(&self, context: &mut Context<impl IValue>) -> Self;

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

    fn guess_inv_or_zero(&self, context: &mut Context<impl IValue>) -> Self {
        self.guess_inv_or_zero(context)
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
/// If `n_bits == 31`, additionally enforces that `0` is encoded canonically as
/// `0b0000...0000` (and not `0b1111...1111`).
/// If any input exceeds `2^n_bits - 1`, a circuit constraints is going to be violated and
/// `context.is_circuit_valid()` will return `false`.
pub fn extract_bits<T: BitPackingOps>(
    context: &mut Context<impl IValue>,
    input: &T,
    n_bits: u32,
) -> Vec<T> {
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

    if n_bits >= 31 {
        assert_eq!(n_bits, 31);
        // Check that `0` is represented as `0b0000...0000`, rather than `0b1111...1111`.
        validate_extract_bits(context, input, &bits[0]);
    }

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

/// Forbids the case `0b1111...1111`, as this number should be represented as `0`.
fn validate_extract_bits<T: BitPackingOps>(context: &mut Context<impl IValue>, input: &T, lsb: &T) {
    // Add a new variable `aux` and check the constraint:
    //   (there exists `aux` such that:) `(input * aux - 1) * lsb = 0`,
    // which guarantees that if the input is 0, then the LSB must be 0.
    //
    // If `input != 0`, we simply choose `aux = 1/input` which satisfies the constraint.
    let zero = T::zero(context, input);
    let one = T::one(context, input);

    let aux = input.guess_inv_or_zero(context);
    let constraint_val = eval!(context, (((*input) * (aux)) - (one)) * (*lsb));
    T::eq(context, &constraint_val, &zero);
}

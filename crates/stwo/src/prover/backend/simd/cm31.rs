use std::array;
use std::ops::{Add, Mul, MulAssign, Neg, Sub};

use bytemuck::{Pod, Zeroable};
use num_traits::{One, Zero};

use super::batch_inverse::{
    BatchInverseScratch, PACKED_CM31_BATCH_INVERSE_CHUNK_SIZE, batch_inverse_via_base_norms,
};
use super::m31::{N_LANES, PackedM31};
use crate::core::fields::cm31::CM31;
use crate::core::fields::{FieldExpOps, batch_inverse_in_place};
use crate::core::utils;

/// SIMD implementation of [`CM31`].
#[derive(Copy, Clone, Debug)]
pub struct PackedCM31(pub [PackedM31; 2]);

unsafe impl Send for PackedCM31 {}
unsafe impl Sync for PackedCM31 {}

impl PackedCM31 {
    /// Constructs a new instance with all vector elements set to `value`.
    pub const fn broadcast(value: CM31) -> Self {
        Self([PackedM31::broadcast(value.0), PackedM31::broadcast(value.1)])
    }

    /// Returns all `a` values such that each vector element is represented as `a + bi`.
    pub const fn a(&self) -> PackedM31 {
        self.0[0]
    }

    /// Returns all `b` values such that each vector element is represented as `a + bi`.
    pub const fn b(&self) -> PackedM31 {
        self.0[1]
    }

    pub fn to_array(&self) -> [CM31; N_LANES] {
        let a = self.a().to_array();
        let b = self.b().to_array();
        array::from_fn(|i| CM31(a[i], b[i]))
    }

    pub fn from_array(values: [CM31; N_LANES]) -> Self {
        Self([
            PackedM31::from_array(values.map(|v| v.0)),
            PackedM31::from_array(values.map(|v| v.1)),
        ])
    }

    /// Interleaves two vectors.
    pub fn interleave(self, other: Self) -> (Self, Self) {
        let Self([a_evens, b_evens]) = self;
        let Self([a_odds, b_odds]) = other;
        let (a_lhs, a_rhs) = a_evens.interleave(a_odds);
        let (b_lhs, b_rhs) = b_evens.interleave(b_odds);
        (Self([a_lhs, b_lhs]), Self([a_rhs, b_rhs]))
    }

    /// Deinterleaves two vectors.
    pub fn deinterleave(self, other: Self) -> (Self, Self) {
        let Self([a_self, b_self]) = self;
        let Self([a_other, b_other]) = other;
        let (a_evens, a_odds) = a_self.deinterleave(a_other);
        let (b_evens, b_odds) = b_self.deinterleave(b_other);
        (Self([a_evens, b_evens]), Self([a_odds, b_odds]))
    }

    /// Doubles each element in the vector.
    pub fn double(self) -> Self {
        let Self([a, b]) = self;
        Self([a.double(), b.double()])
    }
}

impl Add for PackedCM31 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self([self.a() + rhs.a(), self.b() + rhs.b()])
    }
}

impl Sub for PackedCM31 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self([self.a() - rhs.a(), self.b() - rhs.b()])
    }
}

impl Mul for PackedCM31 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        // Compute using Karatsuba.
        let ac = self.a() * rhs.a();
        let bd = self.b() * rhs.b();
        // Computes (a + b) * (c + d).
        let ab_t_cd = (self.a() + self.b()) * (rhs.a() + rhs.b());
        // (ac - bd) + (ad + bc)i.
        Self([ac - bd, ab_t_cd - ac - bd])
    }
}

impl Zero for PackedCM31 {
    fn zero() -> Self {
        Self([PackedM31::zero(), PackedM31::zero()])
    }

    fn is_zero(&self) -> bool {
        self.a().is_zero() && self.b().is_zero()
    }
}

unsafe impl Pod for PackedCM31 {}

unsafe impl Zeroable for PackedCM31 {
    fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}

impl One for PackedCM31 {
    fn one() -> Self {
        Self([PackedM31::one(), PackedM31::zero()])
    }
}

impl MulAssign for PackedCM31 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl FieldExpOps for PackedCM31 {
    fn square(&self) -> Self {
        // (a + bi)^2 = (a + b)(a - b) + 2ab*i. Two base field multiplications instead of the
        // three that the Karatsuba `Mul` takes.
        let Self([a, b]) = *self;
        Self([(a + b) * (a - b), (a * b).double()])
    }

    fn inverse(&self) -> Self {
        assert!(!self.is_zero(), "0 has no inverse");
        // 1 / (a + bi) = (a - bi) / (a^2 + b^2).
        Self([self.a(), -self.b()]) * norm(*self).inverse()
    }

    fn batch_inverse(column: &[Self]) -> Vec<Self> {
        let mut result = unsafe { utils::uninit_vec(column.len()) };
        batch_inverse_via_base_norms(
            column,
            &mut result,
            PACKED_CM31_BATCH_INVERSE_CHUNK_SIZE,
            batch_inverse_chunk,
        );
        result
    }
}

/// The lane-wise norm of `x` over `M31`: `N(a + bi) = (a + bi)(a - bi) = a^2 + b^2`.
#[inline(always)]
pub(super) fn norm(x: PackedCM31) -> PackedM31 {
    x.a().square() + x.b().square()
}

/// Inverts a single chunk.
///
/// For a `CM31` element `x`, the inverse is
///
/// ```text
/// x^-1 = conj(x) * N(x)^-1.
/// ```
///
/// Only the norms `N(x)`, which are base field elements, are batch inverted. That keeps the
/// serial dependency chain on [`PackedM31`] and leaves every other operation pointwise, at
/// ~7 base field multiplications per element against ~9 for Montgomery's batch inverse over
/// `CM31`.
///
/// As with any batch inversion, a zero in any lane makes the whole batch's output for that
/// lane meaningless.
fn batch_inverse_chunk(
    column: &[PackedCM31],
    dst: &mut [PackedCM31],
    scratch: &mut BatchInverseScratch,
) {
    let (base_norms, base_norm_invs) = scratch.buffers(column.len());

    for (&x, base_norm) in column.iter().zip(&mut *base_norms) {
        *base_norm = norm(x);
    }

    batch_inverse_in_place(base_norms, base_norm_invs);

    for ((&x, base_norm_inv), dst) in column.iter().zip(base_norm_invs).zip(dst) {
        *dst = PackedCM31([x.a() * *base_norm_inv, -(x.b() * *base_norm_inv)]);
    }
}

impl Add<PackedM31> for PackedCM31 {
    type Output = Self;

    fn add(self, rhs: PackedM31) -> Self::Output {
        Self([self.a() + rhs, self.b()])
    }
}

impl Sub<PackedM31> for PackedCM31 {
    type Output = Self;

    fn sub(self, rhs: PackedM31) -> Self::Output {
        let Self([a, b]) = self;
        Self([a - rhs, b])
    }
}

impl Mul<PackedM31> for PackedCM31 {
    type Output = Self;

    fn mul(self, rhs: PackedM31) -> Self::Output {
        let Self([a, b]) = self;
        Self([a * rhs, b * rhs])
    }
}

impl Neg for PackedCM31 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let Self([a, b]) = self;
        Self([-a, -b])
    }
}

#[cfg(test)]
mod tests {
    use std::array;

    use num_traits::One;
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    use super::PACKED_CM31_BATCH_INVERSE_CHUNK_SIZE;
    use crate::core::fields::FieldExpOps;
    use crate::core::fields::cm31::CM31;
    use crate::prover::backend::simd::cm31::PackedCM31;
    use crate::prover::backend::simd::m31::N_LANES;

    #[test]
    fn addition_works() {
        let mut rng = SmallRng::seed_from_u64(0);
        let lhs = rng.random();
        let rhs = rng.random();
        let packed_lhs = PackedCM31::from_array(lhs);
        let packed_rhs = PackedCM31::from_array(rhs);

        let res = packed_lhs + packed_rhs;

        assert_eq!(res.to_array(), array::from_fn(|i| lhs[i] + rhs[i]));
    }

    #[test]
    fn subtraction_works() {
        let mut rng = SmallRng::seed_from_u64(0);
        let lhs = rng.random();
        let rhs = rng.random();
        let packed_lhs = PackedCM31::from_array(lhs);
        let packed_rhs = PackedCM31::from_array(rhs);

        let res = packed_lhs - packed_rhs;

        assert_eq!(res.to_array(), array::from_fn(|i| lhs[i] - rhs[i]));
    }

    #[test]
    fn multiplication_works() {
        let mut rng = SmallRng::seed_from_u64(0);
        let lhs = rng.random();
        let rhs = rng.random();
        let packed_lhs = PackedCM31::from_array(lhs);
        let packed_rhs = PackedCM31::from_array(rhs);

        let res = packed_lhs * packed_rhs;

        assert_eq!(res.to_array(), array::from_fn(|i| lhs[i] * rhs[i]));
    }

    #[test]
    fn negation_works() {
        let mut rng = SmallRng::seed_from_u64(0);
        let values = rng.random();
        let packed_values = PackedCM31::from_array(values);

        let res = -packed_values;

        assert_eq!(res.to_array(), values.map(|v| -v));
    }

    #[test]
    fn square_works() {
        let mut rng = SmallRng::seed_from_u64(0);
        let values: [CM31; N_LANES] = rng.random();

        let res = PackedCM31::from_array(values).square();

        assert_eq!(res.to_array(), values.map(|v| v * v));
    }

    #[test]
    fn inverse_works() {
        let mut rng = SmallRng::seed_from_u64(0);
        let values: [CM31; N_LANES] = rng.random();

        let res = PackedCM31::from_array(values).inverse();

        assert_eq!(res.to_array(), values.map(|v| v.inverse()));
    }

    #[test]
    fn batch_inverse_works() {
        let mut rng = SmallRng::seed_from_u64(0);
        // Cover a partial chunk, an exact chunk, several chunks with a partial tail, and the
        // lengths `batch_inverse_in_place` falls back to the classic algorithm for.
        for len in [
            0,
            1,
            2,
            3,
            4,
            5,
            7,
            8,
            PACKED_CM31_BATCH_INVERSE_CHUNK_SIZE - 1,
            PACKED_CM31_BATCH_INVERSE_CHUNK_SIZE,
            2 * PACKED_CM31_BATCH_INVERSE_CHUNK_SIZE + 3,
        ] {
            let column: Vec<PackedCM31> =
                (0..len).map(|_| PackedCM31::from_array(rng.random())).collect();

            let res = PackedCM31::batch_inverse(&column);

            assert_eq!(res.len(), len, "len = {len}");
            for (i, (x, x_inv)) in column.iter().zip(&res).enumerate() {
                assert_eq!(
                    (*x * *x_inv).to_array(),
                    [CM31::one(); N_LANES],
                    "len = {len}, index = {i}"
                );
            }
        }
    }
}

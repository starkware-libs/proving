use core::array;
use core::fmt::{Debug, Display};
use core::iter::{Product, Sum};
use core::ops::{Mul, MulAssign, Neg};

use num_traits::{NumAssign, NumAssignOps, NumOps, One};
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use std_shims::Vec;

use super::utils;

pub mod cm31;
pub mod m31;
pub mod qm31;

pub trait FieldExpOps: Mul<Output = Self> + MulAssign + Sized + One + Clone {
    fn square(&self) -> Self {
        self.clone() * self.clone()
    }

    fn pow(&self, exp: u128) -> Self {
        let mut res = Self::one();
        let mut base = self.clone();
        let mut exp = exp;
        while exp > 0 {
            if exp & 1 == 1 {
                res *= base.clone();
            }
            base = base.square();
            exp >>= 1;
        }
        res
    }

    fn inverse(&self) -> Self;

    /// Inverts `column` into `dst`. Panics if `dst` is shorter.
    fn batch_inverse(column: &[Self], dst: &mut [Self]) {
        batch_inverse_interleaved(column, dst);
    }
}

/// Inverts `column` into `dst` with Montgomery's batch inverse.
///
/// Trades `n` inversions for one inversion and ~`3n` multiplications: build the cumulative
/// products of `column`, invert the total, then unwind that back into the individual inverses.
///
/// Prefer [`batch_inverse_interleaved`]: it is faster, and it accepts any length by falling
/// back to this single-chain form.
fn montgomery_batch_inverse<T: FieldExpOps>(column: &[T], dst: &mut [T]) {
    let n = column.len();
    debug_assert!(dst.len() >= n);

    if let Some(first) = column.first() {
        dst[0] = first.clone();
    } else {
        return;
    }

    // First pass.
    for i in 1..n {
        dst[i] = dst[i - 1].clone() * column[i].clone();
    }

    // Inverse cumulative product.
    let mut curr_inverse = dst[n - 1].inverse();

    // Second pass.
    for i in (1..n).rev() {
        dst[i] = dst[i - 1].clone() * curr_inverse.clone();
        curr_inverse *= column[i].clone();
    }
    dst[0] = curr_inverse;
}

const INTERLEAVED_BATCH_INVERSE_WIDTH: usize = 4;

pub(crate) fn batch_inverse_interleaved<F: FieldExpOps>(column: &[F], dst: &mut [F]) {
    const WIDTH: usize = INTERLEAVED_BATCH_INVERSE_WIDTH;
    let n = column.len();
    debug_assert!(dst.len() >= n);

    if n <= WIDTH || !n.is_multiple_of(WIDTH) {
        montgomery_batch_inverse(column, dst);
        return;
    }

    let mut cum_prod: [F; WIDTH] = array::from_fn(|_| F::one());
    for i in 0..n {
        cum_prod[i % WIDTH] *= column[i].clone();
        dst[i] = cum_prod[i % WIDTH].clone();
    }

    // Invert each chain's total product.
    let mut tail_inverses: [F; WIDTH] = array::from_fn(|_| F::one());
    montgomery_batch_inverse(&dst[n - WIDTH..n], &mut tail_inverses);

    // Second pass.
    for i in (WIDTH..n).rev() {
        dst[i] = dst[i - WIDTH].clone() * tail_inverses[i % WIDTH].clone();
        tail_inverses[i % WIDTH] *= column[i].clone();
    }
    dst[0..WIDTH].clone_from_slice(&tail_inverses);
}

/// Inverts every element of `column` into a fresh [`Vec`].
pub fn batch_inverse<F: FieldExpOps>(column: &[F]) -> Vec<F> {
    let mut dst = unsafe { utils::uninit_vec(column.len()) };
    F::batch_inverse(column, &mut dst);
    dst
}

/// Inverts `column` into `dst` in independent chunks of `chunk_size`, in parallel when the
/// `parallel` feature is on.
///
/// Chunking bounds the working set to cache and is what makes the otherwise serial batch
/// parallelizable. Callers pick `chunk_size` per type.
#[cfg(feature = "prover")]
pub(crate) fn batch_inverse_chunked<T: FieldExpOps + Send + Sync>(
    column: &[T],
    dst: &mut [T],
    chunk_size: usize,
) {
    assert!(column.len() <= dst.len());

    #[cfg(not(feature = "parallel"))]
    let iter = dst.chunks_mut(chunk_size).zip(column.chunks(chunk_size));

    #[cfg(feature = "parallel")]
    let iter = dst.par_chunks_mut(chunk_size).zip(column.par_chunks(chunk_size));

    iter.for_each(|(dst, column)| {
        batch_inverse_interleaved(column, dst);
    });
}

pub trait Field:
    NumAssign
    + Neg<Output = Self>
    + ComplexConjugate
    + Copy
    + Default
    + Debug
    + Display
    + PartialOrd
    + Ord
    + Send
    + Sync
    + Sized
    + FieldExpOps
    + Product
    + for<'a> Product<&'a Self>
    + Sum
    + for<'a> Sum<&'a Self>
{
    fn double(&self) -> Self {
        *self + *self
    }
}

pub trait ComplexConjugate {
    /// # Example
    ///
    /// ```
    /// use stwo::core::fields::ComplexConjugate;
    /// use stwo::core::fields::m31::P;
    /// use stwo::core::fields::qm31::QM31;
    ///
    /// let x = QM31::from_u32_unchecked(1, 2, 3, 4);
    /// assert_eq!(x.complex_conjugate(), QM31::from_u32_unchecked(1, 2, P - 3, P - 4));
    /// ```
    fn complex_conjugate(&self) -> Self;
}

pub trait ExtensionOf<F: Field>: Field + From<F> + NumOps<F> + NumAssignOps<F> {
    const EXTENSION_DEGREE: usize;
}

impl<F: Field> ExtensionOf<F> for F {
    const EXTENSION_DEGREE: usize = 1;
}

#[macro_export]
macro_rules! impl_field {
    ($field_name:ty, $field_size:ident) => {
        use core::iter::{Product, Sum};

        use num_traits::{Num, One, Zero};
        use $crate::core::fields::Field;

        impl Num for $field_name {
            type FromStrRadixErr = std_shims::Box<dyn core::error::Error>;

            fn from_str_radix(_str: &str, _radix: u32) -> Result<Self, Self::FromStrRadixErr> {
                unimplemented!(
                    "Num::from_str_radix is not implemented for {}",
                    stringify!($field_name)
                );
            }
        }

        impl Field for $field_name {}

        impl AddAssign for $field_name {
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs;
            }
        }

        impl SubAssign for $field_name {
            fn sub_assign(&mut self, rhs: Self) {
                *self = *self - rhs;
            }
        }

        impl MulAssign for $field_name {
            fn mul_assign(&mut self, rhs: Self) {
                *self = *self * rhs;
            }
        }

        impl Div for $field_name {
            type Output = Self;

            #[allow(clippy::suspicious_arithmetic_impl)]
            fn div(self, rhs: Self) -> Self::Output {
                self * rhs.inverse()
            }
        }

        impl DivAssign for $field_name {
            fn div_assign(&mut self, rhs: Self) {
                *self = *self / rhs;
            }
        }

        impl Rem for $field_name {
            type Output = Self;

            fn rem(self, _rhs: Self) -> Self::Output {
                unimplemented!("Rem is not implemented for {}", stringify!($field_name));
            }
        }

        impl RemAssign for $field_name {
            fn rem_assign(&mut self, _rhs: Self) {
                unimplemented!("RemAssign is not implemented for {}", stringify!($field_name));
            }
        }

        impl Product for $field_name {
            fn product<I>(mut iter: I) -> Self
            where
                I: Iterator<Item = Self>,
            {
                let first = iter.next().unwrap_or_else(Self::one);
                iter.fold(first, |a, b| a * b)
            }
        }

        impl<'a> Product<&'a Self> for $field_name {
            fn product<I>(iter: I) -> Self
            where
                I: Iterator<Item = &'a Self>,
            {
                iter.map(|&v| v).product()
            }
        }

        impl Sum for $field_name {
            fn sum<I>(mut iter: I) -> Self
            where
                I: Iterator<Item = Self>,
            {
                let first = iter.next().unwrap_or_else(Self::zero);
                iter.fold(first, |a, b| a + b)
            }
        }

        impl<'a> Sum<&'a Self> for $field_name {
            fn sum<I>(iter: I) -> Self
            where
                I: Iterator<Item = &'a Self>,
            {
                iter.map(|&v| v).sum()
            }
        }
    };
}

/// Used to extend a field (with characteristic M31) by 2.
#[macro_export]
macro_rules! impl_extension_field {
    ($field_name:ident, $extended_field_name:ty) => {
        use rand::distr::{Distribution, StandardUniform};
        use $crate::core::fields::ExtensionOf;

        impl ExtensionOf<M31> for $field_name {
            const EXTENSION_DEGREE: usize =
                <$extended_field_name as ExtensionOf<M31>>::EXTENSION_DEGREE * 2;
        }

        impl Add for $field_name {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0, self.1 + rhs.1)
            }
        }

        impl Neg for $field_name {
            type Output = Self;

            fn neg(self) -> Self::Output {
                Self(-self.0, -self.1)
            }
        }

        impl Sub for $field_name {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0, self.1 - rhs.1)
            }
        }

        impl One for $field_name {
            fn one() -> Self {
                Self(<$extended_field_name>::one(), <$extended_field_name>::zero())
            }
        }

        impl Zero for $field_name {
            fn zero() -> Self {
                Self(<$extended_field_name>::zero(), <$extended_field_name>::zero())
            }

            fn is_zero(&self) -> bool {
                *self == Self::zero()
            }
        }

        impl Add<M31> for $field_name {
            type Output = Self;

            fn add(self, rhs: M31) -> Self::Output {
                Self(self.0 + rhs, self.1)
            }
        }

        impl Add<$field_name> for M31 {
            type Output = $field_name;

            fn add(self, rhs: $field_name) -> Self::Output {
                rhs + self
            }
        }

        impl Sub<M31> for $field_name {
            type Output = Self;

            fn sub(self, rhs: M31) -> Self::Output {
                Self(self.0 - rhs, self.1)
            }
        }

        impl Sub<$field_name> for M31 {
            type Output = $field_name;

            fn sub(self, rhs: $field_name) -> Self::Output {
                -rhs + self
            }
        }

        impl Mul<M31> for $field_name {
            type Output = Self;

            fn mul(self, rhs: M31) -> Self::Output {
                Self(self.0 * rhs, self.1 * rhs)
            }
        }

        impl Mul<$field_name> for M31 {
            type Output = $field_name;

            fn mul(self, rhs: $field_name) -> Self::Output {
                rhs * self
            }
        }

        impl Div<M31> for $field_name {
            type Output = Self;

            fn div(self, rhs: M31) -> Self::Output {
                Self(self.0 / rhs, self.1 / rhs)
            }
        }

        impl Div<$field_name> for M31 {
            type Output = $field_name;

            #[allow(clippy::suspicious_arithmetic_impl)]
            fn div(self, rhs: $field_name) -> Self::Output {
                rhs.inverse() * self
            }
        }

        impl ComplexConjugate for $field_name {
            fn complex_conjugate(&self) -> Self {
                Self(self.0, -self.1)
            }
        }

        impl From<M31> for $field_name {
            fn from(x: M31) -> Self {
                Self(x.into(), <$extended_field_name>::zero())
            }
        }

        impl AddAssign<M31> for $field_name {
            fn add_assign(&mut self, rhs: M31) {
                *self = *self + rhs;
            }
        }

        impl SubAssign<M31> for $field_name {
            fn sub_assign(&mut self, rhs: M31) {
                *self = *self - rhs;
            }
        }

        impl MulAssign<M31> for $field_name {
            fn mul_assign(&mut self, rhs: M31) {
                *self = *self * rhs;
            }
        }

        impl DivAssign<M31> for $field_name {
            fn div_assign(&mut self, rhs: M31) {
                *self = *self / rhs;
            }
        }

        impl Rem<M31> for $field_name {
            type Output = Self;

            fn rem(self, _rhs: M31) -> Self::Output {
                unimplemented!("Rem is not implemented for {}", stringify!($field_name));
            }
        }

        impl RemAssign<M31> for $field_name {
            fn rem_assign(&mut self, _rhs: M31) {
                unimplemented!("RemAssign is not implemented for {}", stringify!($field_name));
            }
        }

        impl Distribution<$field_name> for StandardUniform {
            // Not intended for cryptographic use. Should only be used in tests and benchmarks.
            fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> $field_name {
                $field_name(rng.random(), rng.random())
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use num_traits::Zero;
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};
    use std_shims::Vec;

    use super::{INTERLEAVED_BATCH_INVERSE_WIDTH, batch_inverse_interleaved};
    #[cfg(feature = "prover")]
    use crate::core::fields::batch_inverse_chunked;
    use crate::core::fields::m31::M31;
    use crate::core::fields::{FieldExpOps, batch_inverse};
    #[cfg(feature = "prover")]
    use crate::core::utils;

    #[test]
    fn test_batch_inverse() {
        let mut rng = SmallRng::seed_from_u64(0);
        let elements: [M31; 16] = rng.random();
        let expected = elements.iter().map(|e| e.inverse()).collect::<Vec<_>>();

        let actual = batch_inverse(&elements);

        assert_eq!(expected, actual);
    }

    #[test]
    #[should_panic]
    fn test_slice_batch_inverse_wrong_dst_size() {
        let mut rng = SmallRng::seed_from_u64(0);
        let elements: [M31; 16] = rng.random();
        let mut dst = [M31::zero(); 15];

        batch_inverse_interleaved(&elements, &mut dst);
    }

    #[test]
    fn test_batch_inverse_into_longer_dst() {
        let mut rng = SmallRng::seed_from_u64(0);
        const LENGTH: usize = 2 * INTERLEAVED_BATCH_INVERSE_WIDTH;
        const TAIL: usize = 2;
        let elements: [M31; LENGTH] = rng.random();
        let mut dst = [M31::zero(); LENGTH + TAIL];

        M31::batch_inverse(&elements, &mut dst);

        for (x, x_inv) in elements.iter().zip(&dst) {
            assert_eq!(*x * *x_inv, M31::from(1));
        }
        assert_eq!(dst[LENGTH..], [M31::zero(); TAIL], "the tail of dst must be untouched");
    }

    #[test]
    #[cfg(feature = "prover")]
    fn test_batch_inverse_chunked() {
        let mut rng = SmallRng::seed_from_u64(0);
        let elements: [M31; 16] = rng.random();
        let chunk_size = 4;
        let expected = batch_inverse(&elements);

        let mut result = unsafe { utils::uninit_vec(elements.len()) };
        batch_inverse_chunked(&elements, &mut result, chunk_size);

        assert_eq!(expected, result);
    }
}

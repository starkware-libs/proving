use core::fmt::{Debug, Display};
use core::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

use serde::{Deserialize, Serialize};

use super::{ComplexConjugate, FieldExpOps};
use crate::core::fields::m31::M31;
use crate::{impl_extension_field, impl_field};
pub const P2: u64 = 4611686014132420609; // (2 ** 31 - 1) ** 2

/// Complex extension field of M31.
/// Equivalent to M31\[x\] over (x^2 + 1) as the irreducible polynomial.
/// Represented as (a, b) of a + bi.
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct CM31(pub M31, pub M31);

impl_field!(CM31, P2);
impl_extension_field!(CM31, M31);

impl CM31 {
    pub const fn from_u32_unchecked(a: u32, b: u32) -> CM31 {
        Self(M31::from_u32_unchecked(a), M31::from_u32_unchecked(b))
    }

    pub const fn from_m31(a: M31, b: M31) -> CM31 {
        Self(a, b)
    }
}

impl Display for CM31 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} + {}i", self.0, self.1)
    }
}

impl Debug for CM31 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} + {}i", self.0, self.1)
    }
}

impl Mul for CM31 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        // Compute (a + bi) * (c + di) = (ac - bd) + ((a + b)(c + d) - ac - bd)i using 3 M31 muls.
        let ac = self.0 * rhs.0;
        let bd = self.1 * rhs.1;
        let ab_t_cd = (self.0 + self.1) * (rhs.0 + rhs.1);
        Self(ac - bd, ab_t_cd - ac - bd)
    }
}

impl TryInto<M31> for CM31 {
    type Error = ();

    fn try_into(self) -> Result<M31, Self::Error> {
        if self.1 != M31::zero() {
            return Err(());
        }
        Ok(self.0)
    }
}

impl FieldExpOps for CM31 {
    #[inline]
    fn square(&self) -> Self {
        // (a + bi)^2 = (a + b)(a - b) + 2ab*i (2 M31 muls instead of 4).
        let ab = self.0 * self.1;
        Self((self.0 + self.1) * (self.0 - self.1), ab + ab)
    }

    fn inverse(&self) -> Self {
        assert!(!self.is_zero(), "0 has no inverse");
        // 1 / (a + bi) = (a - bi) / (a^2 + b^2).
        Self(self.0, -self.1) * (self.0.square() + self.1.square()).inverse()
    }
}

#[cfg(test)]
#[macro_export]
macro_rules! cm31 {
    ($m0:expr, $m1:expr) => {
        CM31::from_u32_unchecked($m0, $m1)
    };
}

#[cfg(test)]
mod tests {
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    use super::CM31;
    use crate::core::fields::FieldExpOps;
    use crate::core::fields::m31::P;
    use crate::m31;

    fn naive_cm31_mul(lhs: CM31, rhs: CM31) -> CM31 {
        CM31(lhs.0 * rhs.0 - lhs.1 * rhs.1, lhs.0 * rhs.1 + lhs.1 * rhs.0)
    }

    #[test]
    fn test_inverse() {
        let cm = cm31!(1, 2);
        let cm_inv = cm.inverse();
        assert_eq!(cm * cm_inv, cm31!(1, 0));
    }

    #[test]
    fn test_ops() {
        let cm0 = cm31!(1, 2);
        let cm1 = cm31!(4, 5);
        let m = m31!(8);
        let cm = CM31::from(m);
        let cm0_x_cm1 = cm31!(P - 6, 13);

        assert_eq!(cm0 + cm1, cm31!(5, 7));
        assert_eq!(cm1 + m, cm1 + cm);
        assert_eq!(cm0 * cm1, cm0_x_cm1);
        assert_eq!(cm1 * m, cm1 * cm);
        assert_eq!(-cm0, cm31!(P - 1, P - 2));
        assert_eq!(cm0 - cm1, cm31!(P - 3, P - 3));
        assert_eq!(cm1 - m, cm1 - cm);
        assert_eq!(cm0_x_cm1 / cm1, cm31!(1, 2));
        assert_eq!(cm1 / m, cm1 / cm);
    }

    #[test]
    fn test_mul_and_square_exhaustive_edges_and_random() {
        let edges = [0, 1, 2, P - 2, P - 1];
        for &a0 in &edges {
            for &a1 in &edges {
                let x = cm31!(a0, a1);
                assert_eq!(x.square(), naive_cm31_mul(x, x));
                for &b0 in &edges {
                    for &b1 in &edges {
                        let y = cm31!(b0, b1);
                        assert_eq!(x * y, naive_cm31_mul(x, y));
                    }
                }
            }
        }

        let mut rng = SmallRng::seed_from_u64(0xc310_5eed);
        for _ in 0..10_000 {
            let x: CM31 = rng.random();
            let y: CM31 = rng.random();
            assert_eq!(x * y, naive_cm31_mul(x, y));
            assert_eq!(x.square(), naive_cm31_mul(x, x));
        }
    }
}

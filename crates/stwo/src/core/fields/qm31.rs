use core::fmt::{Debug, Display};
use core::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

use serde::{Deserialize, Serialize};

use super::{ComplexConjugate, FieldExpOps};
use crate::core::fields::cm31::CM31;
use crate::core::fields::m31::{M31, P};
use crate::{impl_extension_field, impl_field};

pub const P4: u128 = 21267647892944572736998860269687930881; // (2 ** 31 - 1) ** 4
pub const R: CM31 = CM31::from_u32_unchecked(2, 1);

const P64: u64 = P as u64;
const P2_U64: u64 = P64 * P64;
const INV_24_MOD_P: u64 = 1_521_134_250;
const INV_120_MOD_P: u64 = 304_226_850;

#[inline(always)]
const fn fold_u64(val: u64) -> u64 {
    (val >> 31) + (val & P64)
}

/// Exact Mersenne-31 reduction for any `val` in `0..=u64::MAX`.
#[inline(always)]
const fn reduce_u64(val: u64) -> M31 {
    let f = fold_u64(val);
    let t = ((f >> 31) + (f & P64)) as u32;
    M31((((t + 1) >> 31) + t) & P)
}

/// Multiplies a [`CM31`] element by [`R`] (`2 + i`) using only additions and shifts.
#[inline]
pub const fn mul_by_r(x: CM31) -> CM31 {
    let x0 = x.0.0 as u64;
    let x1 = x.1.0 as u64;
    CM31(M31::reduce((x0 << 1) + P64 - x1), M31::reduce(x0 + (x1 << 1)))
}

/// Extension field of CM31.
/// Equivalent to CM31\[x\] over (x^2 - 2 - i) as the irreducible polynomial.
/// Represented as ((a, b), (c, d)) of (a + bi) + (c + di)u.
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct QM31(pub CM31, pub CM31);
pub type SecureField = QM31;

pub const SECURE_EXTENSION_DEGREE: usize =
    <SecureField as ExtensionOf<super::m31::BaseField>>::EXTENSION_DEGREE;

impl_field!(QM31, P4);
impl_extension_field!(QM31, CM31);

impl QM31 {
    pub const fn from_u32_unchecked(a: u32, b: u32, c: u32, d: u32) -> Self {
        Self(CM31::from_u32_unchecked(a, b), CM31::from_u32_unchecked(c, d))
    }

    pub const fn from_m31(a: M31, b: M31, c: M31, d: M31) -> Self {
        Self(CM31::from_m31(a, b), CM31::from_m31(c, d))
    }

    pub const fn from_m31_array(array: [M31; SECURE_EXTENSION_DEGREE]) -> Self {
        Self::from_m31(array[0], array[1], array[2], array[3])
    }

    pub const fn to_m31_array(self) -> [M31; SECURE_EXTENSION_DEGREE] {
        [self.0.0, self.0.1, self.1.0, self.1.1]
    }

    /// Returns the combined value, given the values of its composing base field polynomials at that
    /// point.
    pub fn from_partial_evals(evals: [Self; SECURE_EXTENSION_DEGREE]) -> Self {
        let mut res = evals[0];
        res += evals[1] * Self::from_u32_unchecked(0, 1, 0, 0);
        res += evals[2] * Self::from_u32_unchecked(0, 0, 1, 0);
        res += evals[3] * Self::from_u32_unchecked(0, 0, 0, 1);
        res
    }

    // Note: Adding this as a Mul impl drives rust insane, and it tries to infer Qm31*Qm31 as
    // QM31*CM31.
    pub fn mul_cm31(self, rhs: CM31) -> Self {
        Self(self.0 * rhs, self.1 * rhs)
    }

    /// Multiplies two [`QM31`] elements using the certified minimum-rank (`Rank-7`) bilinear
    /// decomposition over `M31` (`7` variable `M31` multiplications), via the unimodular
    /// isomorphism `(a0 + a1*i) + (b0 + b1*i)*u <-> (a0 - 2*a1) + (b0 - 2*b1)*u + a1*u^2 + b1*u^3`
    /// modulo `u^4 - 4*u^2 + 5 = 0` evaluated at `{0, 1, -1, 2, -2, 3, inf}`.
    #[inline(always)]
    pub const fn mul_rank7(self, rhs: Self) -> Self {
        let a0 = self.0.0.0 as u64;
        let a1 = self.0.1.0 as u64;
        let b0 = self.1.0.0 as u64;
        let b1 = self.1.1.0 as u64;
        let c0 = rhs.0.0.0 as u64;
        let c1 = rhs.0.1.0 as u64;
        let d0 = rhs.1.0.0 as u64;
        let d1 = rhs.1.1.0 as u64;

        let x_sa = a0 + P64 - a1;
        let x_sb = b0 + P64 - b1;
        let x_ta = a0 + (a1 << 1);
        let x_tb = (b0 << 1) + (b1 << 2);
        let f0 = fold_u64(a0 + 2 * P64 - (a1 << 1));
        let f1 = fold_u64(x_sa + x_sb);
        let fm1 = fold_u64(x_sa + 2 * P64 - x_sb);
        let f2 = fold_u64(x_ta + x_tb);
        let fm2 = fold_u64(x_ta + 6 * P64 - x_tb);
        let f3 = fold_u64(a0 + 7 * a1 + 3 * b0 + 21 * b1);

        let y_sa = c0 + P64 - c1;
        let y_sb = d0 + P64 - d1;
        let y_ta = c0 + (c1 << 1);
        let y_tb = (d0 << 1) + (d1 << 2);
        let g0 = fold_u64(c0 + 2 * P64 - (c1 << 1));
        let g1 = fold_u64(y_sa + y_sb);
        let gm1 = fold_u64(y_sa + 2 * P64 - y_sb);
        let g2 = fold_u64(y_ta + y_tb);
        let gm2 = fold_u64(y_ta + 6 * P64 - y_tb);
        let g3 = fold_u64(c0 + 7 * c1 + 3 * d0 + 21 * d1);

        let v0 = fold_u64(f0 * g0);
        let v1 = fold_u64(f1 * g1);
        let vm1 = fold_u64(fm1 * gm1);
        let v2 = fold_u64(f2 * g2);
        let vm2 = fold_u64(fm2 * gm2);
        let v3 = fold_u64(f3 * g3);
        let vinf = fold_u64(b1 * d1);

        let s1 = v1 + vm1;
        let s2 = v2 + vm2;
        let vinf5 = 5 * vinf;

        let n0 = fold_u64(s2 + 20 * s1 + 72 * P64 - 18 * v0);
        let out0 = reduce_u64(n0 * INV_24_MOD_P + 20 * P64 - vinf5);

        let n1 = s2 + 8 * P64 - (v0 << 1);
        let out1 = reduce_u64((n1 << 28) + 20 * P64 - vinf5);

        let n2 = fold_u64(30 * v0 + 10 * v1 + 25 * v2 + 260 * P64 - 55 * vm1 - 7 * vm2 - 3 * v3);
        let out2 = reduce_u64(n2 * INV_120_MOD_P + 9 * vinf);

        let n3 = fold_u64(10 * v0 + 15 * (vm1 + v2) + 160 * P64 - 30 * v1 - 9 * vm2 - v3);
        let out3 = reduce_u64(n3 * INV_120_MOD_P + 3 * vinf);

        Self(CM31(out0, out1), CM31(out2, out3))
    }
}

impl Display for QM31 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "({}) + ({})u", self.0, self.1)
    }
}

impl Debug for QM31 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "({}) + ({})u", self.0, self.1)
    }
}

impl Mul for QM31 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        // Outer Karatsuba over CM31 with unreduced u64 Mersenne accumulation and a single
        // reduction per output coordinate:
        //   (a + b*u) * (c + d*u) = (a*c + (2 + i)*b*d) + ((a + b)*(c + d) - a*c - b*d)*u.
        let a0 = self.0.0.0 as u64;
        let a1 = self.0.1.0 as u64;
        let b0 = self.1.0.0 as u64;
        let b1 = self.1.1.0 as u64;
        let c0 = rhs.0.0.0 as u64;
        let c1 = rhs.0.1.0 as u64;
        let d0 = rhs.1.0.0 as u64;
        let d1 = rhs.1.1.0 as u64;

        let ac0 = a0 * c0;
        let ac1 = a1 * c1;
        let ac_im = a0 * c1 + a1 * c0;

        let bd0 = b0 * d0;
        let bd1 = b1 * d1;
        let bd_im = b0 * d1 + b1 * d0;

        let ab0 = a0 + b0;
        let ab1 = a1 + b1;
        let cd0 = fold_u64(c0 + d0);
        let cd1 = fold_u64(c1 + d1);
        let s0 = ab0 * cd0;
        let s1 = ab1 * cd1;
        let s_im = ab0 * cd1 + ab1 * cd0;

        let ac_re_f = fold_u64(ac0 + P2_U64 - ac1);
        let ac_im_f = fold_u64(ac_im);
        let bd_re_f = fold_u64(bd0 + P2_U64 - bd1);
        let bd_im_f = fold_u64(bd_im);

        let out0 = reduce_u64(ac_re_f + (bd_re_f << 1) + 4 * P64 - bd_im_f);
        let out1 = reduce_u64(ac_im_f + bd_re_f + (bd_im_f << 1));
        let out2 = reduce_u64(fold_u64(s0 + ac1 + bd1) + 6 * P64 - fold_u64(s1 + ac0 + bd0));
        let out3 = reduce_u64(fold_u64(s_im) + 6 * P64 - fold_u64(ac_im + bd_im));
        Self(CM31(out0, out1), CM31(out2, out3))
    }
}

impl From<usize> for QM31 {
    fn from(value: usize) -> Self {
        M31::from(value).into()
    }
}

impl From<u32> for QM31 {
    fn from(value: u32) -> Self {
        M31::from(value).into()
    }
}

impl From<i32> for QM31 {
    fn from(value: i32) -> Self {
        M31::from(value).into()
    }
}

impl TryInto<M31> for QM31 {
    type Error = ();

    fn try_into(self) -> Result<M31, Self::Error> {
        if self.1 != CM31::zero() {
            return Err(());
        }
        self.0.try_into()
    }
}

impl FieldExpOps for QM31 {
    #[inline(always)]
    fn square(&self) -> Self {
        // (a + b*u)^2 = (a^2 + (2 + i)*b^2) + 2*a*b*u using 7 unreduced u64 multiplications
        // (2 for a^2, 2 for b^2, 3 for Karatsuba a*b) and 4 final reductions.
        let a0 = self.0.0.0 as u64;
        let a1 = self.0.1.0 as u64;
        let b0 = self.1.0.0 as u64;
        let b1 = self.1.1.0 as u64;

        let a_re = fold_u64((a0 + a1) * (a0 + P64 - a1));
        let a_im = fold_u64((a0 << 1) * a1);
        let b_re = fold_u64((b0 + b1) * (b0 + P64 - b1));
        let b_im = fold_u64((b0 << 1) * b1);

        let q0 = a0 * b0;
        let q1 = a1 * b1;
        let q_im = (a0 + a1) * (b0 + b1) - q0 - q1;

        let out0 = reduce_u64(a_re + (b_re << 1) + 4 * P64 - b_im);
        let out1 = reduce_u64(a_im + b_re + (b_im << 1));
        let out2 = reduce_u64(fold_u64(q0 + P2_U64 - q1) << 1);
        let out3 = reduce_u64(fold_u64(q_im) << 1);
        Self(CM31(out0, out1), CM31(out2, out3))
    }

    fn inverse(&self) -> Self {
        assert!(!self.is_zero(), "0 has no inverse");
        // (a + bu)^-1 = (a - bu) / (a^2 - (2+i)b^2).
        let b2 = self.1.square();
        let denom = self.0.square() - mul_by_r(b2);
        let denom_inverse = denom.inverse();
        Self(self.0 * denom_inverse, -(self.1 * denom_inverse))
    }
}

#[cfg(test)]
#[macro_export]
macro_rules! qm31 {
    ($m0:expr, $m1:expr, $m2:expr, $m3:expr) => {{
        use $crate::core::fields::qm31::QM31;
        QM31::from_u32_unchecked($m0, $m1, $m2, $m3)
    }};
}

#[cfg(test)]
mod tests {
    use num_traits::One;
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    use super::{QM31, R, mul_by_r};
    use crate::core::fields::FieldExpOps;
    use crate::core::fields::cm31::CM31;
    use crate::core::fields::m31::P;
    use crate::m31;

    fn naive_cm31_mul(lhs: CM31, rhs: CM31) -> CM31 {
        CM31(lhs.0 * rhs.0 - lhs.1 * rhs.1, lhs.0 * rhs.1 + lhs.1 * rhs.0)
    }

    fn naive_qm31_mul(lhs: QM31, rhs: QM31) -> QM31 {
        let ac = naive_cm31_mul(lhs.0, rhs.0);
        let bd = naive_cm31_mul(lhs.1, rhs.1);
        let r_bd = naive_cm31_mul(R, bd);
        let ad = naive_cm31_mul(lhs.0, rhs.1);
        let bc = naive_cm31_mul(lhs.1, rhs.0);
        QM31(ac + r_bd, ad + bc)
    }

    #[test]
    fn test_inverse() {
        let qm = qm31!(1, 2, 3, 4);
        let qm_inv = qm.inverse();
        assert_eq!(qm * qm_inv, QM31::one());
    }

    #[test]
    fn test_ops() {
        let qm0 = qm31!(1, 2, 3, 4);
        let qm1 = qm31!(4, 5, 6, 7);
        let m = m31!(8);
        let qm = QM31::from(m);
        let qm0_x_qm1 = qm31!(P - 71, 93, P - 16, 50);

        assert_eq!(qm0 + qm1, qm31!(5, 7, 9, 11));
        assert_eq!(qm1 + m, qm1 + qm);
        assert_eq!(qm0 * qm1, qm0_x_qm1);
        assert_eq!(qm1 * m, qm1 * qm);
        assert_eq!(-qm0, qm31!(P - 1, P - 2, P - 3, P - 4));
        assert_eq!(qm0 - qm1, qm31!(P - 3, P - 3, P - 3, P - 3));
        assert_eq!(qm1 - m, qm1 - qm);
        assert_eq!(qm0_x_qm1 / qm1, qm31!(1, 2, 3, 4));
        assert_eq!(qm1 / m, qm1 / qm);
    }

    #[test]
    fn test_mul_rank7_basis_edges_and_random() {
        let basis = [qm31!(1, 0, 0, 0), qm31!(0, 1, 0, 0), qm31!(0, 0, 1, 0), qm31!(0, 0, 0, 1)];
        // All 16 basis pairs.
        for &e_i in &basis {
            for &e_j in &basis {
                let expected = naive_qm31_mul(e_i, e_j);
                assert_eq!(e_i * e_j, expected);
                assert_eq!(e_i.mul_rank7(e_j), expected);
            }
        }

        // All 3^4 = 81 combinations of {0, 1, P - 1} against each other (6,561 edge pairs).
        let edges = [0, 1, P - 1];
        let mut edge_elems = [qm31!(0, 0, 0, 0); 81];
        let mut idx = 0;
        for &a in &edges {
            for &b in &edges {
                for &c in &edges {
                    for &d in &edges {
                        edge_elems[idx] = qm31!(a, b, c, d);
                        idx += 1;
                    }
                }
            }
        }
        for &x in &edge_elems {
            assert_eq!(mul_by_r(x.0), naive_cm31_mul(R, x.0));
            assert_eq!(x.square(), naive_qm31_mul(x, x));
            if x != qm31!(0, 0, 0, 0) {
                assert_eq!(x * x.inverse(), QM31::one());
            }
            for &y in &edge_elems {
                let expected = naive_qm31_mul(x, y);
                assert_eq!(x * y, expected);
                assert_eq!(x.mul_rank7(y), expected);
            }
        }

        // 10,000 random pairs.
        let mut rng = SmallRng::seed_from_u64(0x7a17_5eed);
        for _ in 0..10_000 {
            let x: QM31 = rng.random();
            let y: QM31 = rng.random();
            let expected = naive_qm31_mul(x, y);
            assert_eq!(x * y, expected);
            assert_eq!(x.mul_rank7(y), expected);
            assert_eq!(x.square(), naive_qm31_mul(x, x));
            assert_eq!(mul_by_r(x.0), naive_cm31_mul(R, x.0));
        }
    }
}

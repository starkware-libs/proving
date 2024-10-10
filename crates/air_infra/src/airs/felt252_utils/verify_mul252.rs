use std::array::from_fn;
use std::cmp::{max, min};
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use inst_def::InstDef;

use crate::airs::casm::const_tables::range_check::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint32_expr::*;
use crate::core::prover_types::*;

// Macros
use crate::const_expr;
use crate::const_u32_expr;

/// Verifying that two 252-bit felts multiply to a third.
/// The function assumes all inputs have range-checked limbs.
/// None of the inputs are constrained to be fully reduced, but (a * b - c)/P must be in [0, 2**252).
#[derive(Clone, Debug, InstDef)]
pub struct VerifyMul252 {}

impl AirFn for VerifyMul252 {
    type In = [Felt252Expr; 3];
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, [a, b, c]: Self::In) -> Self::Out {
        let shift = const_expr!(1 << FELT252_BITS_PER_WORD);
        let shift_inverse = const_expr!(1) / shift.clone();

        const CONV_LEN: usize = 2 * FELT252_N_WORDS - 1;
        const MAX_WORD: i32 = (1 << FELT252_BITS_PER_WORD) - 1;
        const MUL_RANGE_CHECKS: u16 = 19;

        // Compute the limbs of a * b - c in long-form: limb i holds the i-th coefficient of the
        // convolution of a and b, minus the i-th coefficient of c (where i < FELT252_N_WORDS).
        // TODO: Optimize the convolution: e.g. using Karatsuba, Toom-Cook, or even NTT.
        // TODO: Optimize the arithmetic in the convolution: M31 muls and adds are slower than u32s,
        // because of the modulo operations, which are not necessary since the limbs are small.
        let mut conv_tmps: [BoundedFeltExpr; CONV_LEN] = from_fn(|_| BoundedFeltExpr::default());
        #[allow(clippy::needless_range_loop)]
        for i in 0..CONV_LEN {
            let mut conv = BoundedFeltExpr::default();
            if i < FELT252_N_WORDS {
                conv -= (c.get_felt(i), MAX_WORD, 0).into();
            }
            let convolution_start = max(i, FELT252_N_WORDS - 1) - (FELT252_N_WORDS - 1);
            let convolution_end = min(i, FELT252_N_WORDS - 1);
            for j in convolution_start..=convolution_end {
                conv += (a.get_felt(j) * b.get_felt(i - j), MAX_WORD * MAX_WORD, 0).into()
            }
            conv.expr = air_builder.let_(conv.expr);
            conv_tmps[i] = conv;
        }

        // Compute the limbs of -4 * 2**(-21 * 9) * (a * b - c), partially reduced modulo P.
        // The partial reduction is performed by reducing the coefficients of the limbs, but not
        // the limbs themselves (which can be much larger than the radix).
        // The coefficient -4 ** 2**(-21 * 9) is specifically chosen to yield small coefficients
        // (i.e., with few non-zero limbs, which are themselves small) when multiplied by all powers
        // 2**(0 * 9), 2**(1 * 9), ..., 2**(54 * 9) and reduced modulo P.
        let mut conv_mod_tmps: [BoundedFeltExpr; FELT252_N_WORDS] =
            from_fn(|_| BoundedFeltExpr::default());
        // The lowest 21 words of the convolution are multiplied by
        // -4 * 2**(-21 * 9) % P = 32 * 2**0 + 1 * 2**9 + 2 * 2**(7 * 9).
        #[allow(clippy::needless_range_loop)]
        for i in 0..21 {
            for (coef, word_shift) in [(32, 0), (1, 1), (2, 7)] {
                let j = i + word_shift;
                conv_mod_tmps[j] += conv_tmps[i].clone() * coef;
            }
        }
        // The middle 28 words of the convolution are multiplied simply by -4.
        #[allow(clippy::needless_range_loop)]
        for i in 21..49 {
            let j = i - 21;
            conv_mod_tmps[j] -= conv_tmps[i].clone() * 4u32;
        }
        // The highest 6 words of the convolution are multiplied by
        // -4 * 2**(28 * 9) % P = 8 * 2**0 + 64 * 2**(21 * 9) + 2 * 2**(22 * 9).
        #[allow(clippy::needless_range_loop)]
        for i in 49..CONV_LEN {
            for (coef, word_shift) in [(8, 0), (64, 21), (2, 22)] {
                let j = i + word_shift - 49;
                conv_mod_tmps[j] += conv_tmps[i].clone() * coef;
            }
        }
        // Save the reduced convolution elements as temp variables.
        for conv_mod in conv_mod_tmps.iter_mut() {
            conv_mod.expr = air_builder.let_(conv_mod.expr.clone());
        }

        // Compute and deduce k: the coefficient of P in the equation
        //   PR := PartialReduction(-4 * 2**(-21 * 9) * (a * b - c)) = k * P.
        // The possible values of k (determined by the bounds on the highest limbs of the partial
        // reduction) lie in the range (-29*2**10, 45*2**10), or more loosely (-2**16, 2**16).
        // Since P % (2**18) == 1, it follows that we can extract k by reducing PR modulo 2**18,
        // for which it suffices to consider its lowest two limbs in the radix 2**9.
        //
        // To work modulo 2**18, we convert the limbs to Uint32s; the limbs may both be negative,
        // but bounded in (-2**25, 2**25), so we add 2**27 to make them non-negative before
        // converting to a Uint, without changing their residues modulo 2**18.
        // Similarly, to convert k from Uint modulo 2**18 to a signed felt in the range
        // (-2**16, 2**16), we add 2**16 to the Uint before the modulo, and subtract it again after
        // the conversion to felt.
        let k_high_mod_2_9 =
            UInt32Expr::from(conv_mod_tmps[1].expr.clone() + const_expr!(1u32 << 27))
                & const_u32_expr!((1u32 << 9) - 1);
        let k_low = UInt32Expr::from(conv_mod_tmps[0].expr.clone() + const_expr!(1u32 << 27));
        let mut k_mod_2_18_biased =
            (k_low + (k_high_mod_2_9 << const_u32_expr!(9u32)) + const_u32_expr!(1u32 << 16))
                & const_u32_expr!((1u32 << 18) - 1);
        k_mod_2_18_biased = air_builder.let_for_deduction(k_mod_2_18_biased);
        let k_expr = air_builder.deduce(
            &mut (k_mod_2_18_biased.low().as_felt()
                + (k_mod_2_18_biased.high().as_felt() - const_expr!(1)) * const_expr!(1u32 << 16)),
        );
        // The range of k fits inside a range check of 2**17, but the smallest commonly used size
        // is 19, the size of the largest range checks needed for the the carries.
        air_builder.lookup_call(
            &RangeCheck {
                bits: [MUL_RANGE_CHECKS],
            },
            [k_expr.clone() + const_expr!(1u32 << 18)],
        );
        // Bounds on k based on the range check constraint.
        let k = BoundedFeltExpr {
            expr: k_expr,
            max_bound: (1i32 << MUL_RANGE_CHECKS) - (1i32 << 18) - 1,
            min_bound: -(1i32 << 18),
        };

        // Subtract k*P from the reduced convolution. P has only three non-zero limbs.
        for (coef, i) in [(1, 0), (136, 21), (256, 27)] {
            conv_mod_tmps[i] -= k.clone() * coef;
        }

        // Verify that PR - k*P = 0 by evaluating and range-checking the carries between the limbs.
        let mut carry = BoundedFeltExpr::default();
        for conv_mod in conv_mod_tmps.iter().take(FELT252_N_WORDS - 1) {
            let shifted_carry = conv_mod.clone() + carry;
            carry = BoundedFeltExpr {
                expr: air_builder.deduce(&mut (shifted_carry.expr.clone() * shift_inverse.clone())),
                max_bound: shifted_carry.max_bound >> FELT252_BITS_PER_WORD,
                min_bound: shifted_carry.min_bound >> FELT252_BITS_PER_WORD,
            };
            air_builder.constrain(carry.expr.clone() * shift.clone() - shifted_carry.expr);

            // All carries fit inside the range (-2**17, 2**19 - 2**17), and are range-checked
            // correspondigly. This range is nearly sharp for the largest carries, and in particular
            // a range of size 2**18 is insufficient.
            assert!(carry.max_bound < (1i32 << MUL_RANGE_CHECKS) - (1i32 << 17));
            assert!(carry.min_bound >= -(1i32 << 17));

            air_builder.lookup_call(
                &RangeCheck {
                    bits: [MUL_RANGE_CHECKS],
                },
                [carry.expr.clone() + const_expr!(1u32 << 17)],
            );
            // Bounds on the carry based on the range-check constraint.
            carry.max_bound = (1i32 << MUL_RANGE_CHECKS) - (1i32 << 17) - 1;
            carry.min_bound = -(1i32 << 17);
        }
        // For the final limb, the computation must yield zero with no further carry.
        air_builder.constrain(conv_mod_tmps[FELT252_N_WORDS - 1].expr.clone() + carry.expr);
    }
}

#[derive(Clone, Debug, Default)]
struct BoundedFeltExpr {
    pub expr: FeltExpr,
    pub max_bound: i32,
    pub min_bound: i32,
}

impl Add for BoundedFeltExpr {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            expr: self.expr + other.expr,
            max_bound: self.max_bound + other.max_bound,
            min_bound: self.min_bound + other.min_bound,
        }
    }
}

impl AddAssign for BoundedFeltExpr {
    fn add_assign(&mut self, other: Self) {
        *self = self.clone() + other;
    }
}

impl Sub for BoundedFeltExpr {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            expr: self.expr - other.expr,
            max_bound: self.max_bound - other.min_bound,
            min_bound: self.min_bound - other.max_bound,
        }
    }
}

impl SubAssign for BoundedFeltExpr {
    fn sub_assign(&mut self, other: Self) {
        *self = self.clone() - other;
    }
}

impl Mul<u32> for BoundedFeltExpr {
    type Output = Self;
    fn mul(self, other: u32) -> Self {
        Self {
            expr: const_expr!(other) * self.expr,
            max_bound: (other as i32) * self.max_bound,
            min_bound: (other as i32) * self.min_bound,
        }
    }
}

impl From<(FeltExpr, i32, i32)> for BoundedFeltExpr {
    fn from((expr, max_bound, min_bound): (FeltExpr, i32, i32)) -> Self {
        Self {
            expr,
            max_bound,
            min_bound,
        }
    }
}

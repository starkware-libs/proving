use std::array::from_fn;

use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::bounded_felt::BoundedFeltExpr;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::core::variables::AirVar;
use air_infra::range_check::range_check_variant;
use air_infra::{const_expr, const_u32_expr};
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::{FELT252_BITS_PER_WORD, FELT252_N_WORDS};

use crate::convolution_utils::karatsuba::*;

/// Verifies that two 252-bit felts multiply to a third.
/// The function assumes the following bounds on the limbs of the inputs:
///  - Multiplicands a, b both have limbs in the range (-2**9, 2**9), i.e. they are either
///    range-checked to 9 bits or the difference between two range-checked limbs of other felt252s
///  - The product c has limbs in the range (-2**9, 3*2**9). This range could be greatly extended if
///    necessary. The bounds are chosen to support possibilities of limbs which are a difference of
///    two range-checked limbs or the sum of up to three, which occur in the EcAdd air.
///
/// None of the inputs are constrained to be fully reduced.
#[derive(Clone, Debug, Serialize)]
pub struct VerifyMul252 {}

impl AirFn for VerifyMul252 {
    type ExtIn = ();
    type In = [Felt252Expr; 3];
    type Out = ();

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![Some("a".to_string()), Some("b".to_string()), Some("c".to_string())])
    }

    fn call(&self, air_builder: &mut AirBuilder, _: (), [a, b, c]: Self::In) -> Self::Out {
        let shift = const_expr!(1 << FELT252_BITS_PER_WORD);
        let shift_inverse = shift.clone().inverse();

        const CONV_LEN: usize = 2 * FELT252_N_WORDS - 1;
        const MAX_WORD: i32 = (1 << FELT252_BITS_PER_WORD) - 1;
        const MUL_RANGE_CHECKS: u16 = 20;

        // Compute the limbs of a * b - c in long-form: limb i holds the i-th coefficient of the
        // convolution of a and b, minus the i-th coefficient of c (where i < FELT252_N_WORDS).

        // Compute the convolution a * b using Karatsuba.
        let karatsuba = DoubleKaratsuba::<{ FELT252_N_WORDS / 4 }>::new(
            MAX_WORD * MAX_WORD,
            -MAX_WORD * MAX_WORD,
        );
        let error_message = &format!("felt252 should have {FELT252_N_WORDS} limbs");
        let mut conv_tmps = air_builder.call(
            &karatsuba,
            [
                a.as_felts().try_into().expect(error_message),
                b.as_felts().try_into().expect(error_message),
            ],
        );

        #[allow(clippy::needless_range_loop)]
        for i in 0..FELT252_N_WORDS {
            conv_tmps[i] -= (c.get_felt(i), 3 * MAX_WORD + 1, -MAX_WORD).into();
        }
        conv_tmps = air_builder.let_(conv_tmps, "conv");

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
        conv_mod_tmps = air_builder.let_(conv_mod_tmps, "conv_mod");

        // Compute and deduce k: the coefficient of P in the equation
        //   PR := PartialReduction(-4 * 2**(-21 * 9) * (a * b - c)) = k * P.
        // The possible values of k (determined by the bounds on the highest limbs of the partial
        // reduction) lie in the range (-74*2**10, 74*2**10), or more loosely (-2**17, 2**17).
        // Since P % (2**18) == 1, it follows that we can extract k by reducing PR modulo 2**18,
        // for which it suffices to consider its lowest two limbs in the radix 2**9.
        //
        // To work modulo 2**18, we convert the limbs to Uint32s; the limbs may both be negative,
        // but bounded in (-2**26, 2**26), so we add 2**27 to make them non-negative before
        // converting to a Uint, without changing their residues modulo 2**18.
        // Similarly, to convert k from Uint modulo 2**18 to a signed felt in the range
        // (-2**17, 2**17), we add 2**17 to the Uint before the modulo, and subtract it again after
        // the conversion to felt.
        let k_high_mod_2_9 =
            UInt32Expr::from(conv_mod_tmps[1].var.clone() + const_expr!(1u32 << 27))
                & const_u32_expr!((1u32 << 9) - 1);
        let k_low = UInt32Expr::from(conv_mod_tmps[0].var.clone() + const_expr!(1u32 << 27));
        let mut k_mod_2_18_biased =
            (k_low + (k_high_mod_2_9 << const_u32_expr!(9u32)) + const_u32_expr!(1u32 << 17))
                & const_u32_expr!((1u32 << 18) - 1);
        k_mod_2_18_biased = air_builder.let_for_deduction(k_mod_2_18_biased, "k_mod_2_18_biased");

        let k_expr = air_builder.deduce(
            &mut (k_mod_2_18_biased.low().as_felt()
                + (k_mod_2_18_biased.high().as_felt() - const_expr!(2)) * const_expr!(1u32 << 16)),
            "k",
        );
        // The range of k fits inside a range check of 2**18, but the smallest commonly used size
        // is 20, the size of the largest range checks needed for the the carries.
        range_check_variant(
            air_builder,
            &[MUL_RANGE_CHECKS],
            &[k_expr.clone() + const_expr!(1u32 << 19)],
            0,
        );

        // Bounds on k based on the range check constraint.
        let k = BoundedFeltExpr::new(
            k_expr,
            (1i32 << MUL_RANGE_CHECKS) - (1i32 << 19) - 1,
            -(1i32 << 19),
        );

        // Subtract k*P from the reduced convolution. P has only three non-zero limbs.
        for (coef, i) in [(1, 0), (136, 21), (256, 27)] {
            conv_mod_tmps[i] -= k.clone() * coef;
        }

        // Verify that PR - k*P = 0 by evaluating and range-checking the carries between the limbs.
        let mut carry = BoundedFeltExpr::default();
        for (i, conv_mod) in conv_mod_tmps.iter().take(FELT252_N_WORDS - 1).enumerate() {
            let shifted_carry = conv_mod.clone() + carry;
            carry = BoundedFeltExpr::new(
                air_builder.deduce(
                    &mut (shifted_carry.var.clone() * shift_inverse.clone()),
                    &format!("carry_{i}"),
                ),
                shifted_carry.max_bound() >> FELT252_BITS_PER_WORD,
                shifted_carry.min_bound() >> FELT252_BITS_PER_WORD,
            );
            air_builder.constrain(carry.var.clone() * shift.clone() - shifted_carry.var, "");

            // All carries fit inside the range (-2**19, 2**19), and are range-checked
            // accordingly. This range is nearly sharp for the largest carries, and in particular
            // a range of size 2**19 is insufficient.
            assert!(carry.max_bound() < (1i32 << 19));
            assert!(carry.min_bound() >= -(1i32 << 19));

            range_check_variant(
                air_builder,
                &[MUL_RANGE_CHECKS],
                &[carry.var.clone() + const_expr!(1u32 << 19)],
                (i + 1) % 8,
            );

            // Bounds on the carry based on the range-check constraint.
            carry.set_max_bound((1i32 << MUL_RANGE_CHECKS) - (1i32 << 19) - 1);
            carry.set_min_bound(-(1i32 << 19));
        }

        // For the final limb, the computation must yield zero with no further carry.
        air_builder.constrain(conv_mod_tmps[FELT252_N_WORDS - 1].var.clone() + carry.var, "");
    }
}

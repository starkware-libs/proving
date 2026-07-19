use std::cmp::{max, min};
use std::slice::from_ref;

use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::range_check::range_check;
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::FELT252_BITS_PER_WORD;

// The number of FELT252_BITS_PER_WORD bit limbs in a multiplication factor.
const NUM_LIMBS: usize = 4;

// Verifying that two NUM_LIMBS*FELT252_BITS_PER_WORD-bit felts multiply to a third
// 2*NUM_LIMBS*FELT252_BITS_PER_WORD bit felt.
// The function assumes all inputs have range-checked limbs.
// It is assumed that 2*NUM_LIMBS*FELT252_BITS_PER_WORD < 252.
#[derive(Clone, Debug, Serialize)]
pub struct VerifyMulSmall {}

impl AirFn for VerifyMulSmall {
    type ExtIn = ();
    type In = [Felt252Expr; 3];
    type Out = ();

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![Some("a".to_string()), Some("b".to_string()), Some("c".to_string())])
    }

    fn call(&self, air_builder: &mut AirBuilder, _: (), [a, b, c]: Self::In) -> Self::Out {
        let shift = const_expr!(1 << FELT252_BITS_PER_WORD);
        let double_shift = shift.clone() * shift.clone();
        let double_shift_inverse = double_shift.clone().inverse();

        let mut limb_accumulator = const_expr!(0u32);

        for i in 0..(2 * NUM_LIMBS - 2) {
            let conditional_shift = if i % 2 == 1 { shift.clone() } else { const_expr!(1) };
            let convolution_start = max(i, NUM_LIMBS - 1) - (NUM_LIMBS - 1);
            let convolution_end = min(i, NUM_LIMBS - 1);
            let mut convolution = const_expr!(0u32);
            for j in convolution_start..=convolution_end {
                convolution = convolution + a.get_felt(j) * b.get_felt(i - j);
            }

            limb_accumulator =
                limb_accumulator + (convolution - c.get_felt(i)) * conditional_shift.clone();
            if i % 2 == 1 {
                let carry = air_builder.deduce(
                    &mut (limb_accumulator.clone() * double_shift_inverse.clone()),
                    &format!("carry_{i}"),
                );
                // Each convolution has at most 4 addends, each addend has at most 2**9-1 overflow.
                range_check(air_builder, &[11], from_ref(&carry));
                air_builder.constrain(
                    carry.clone() * double_shift.clone() - limb_accumulator,
                    &format!("carry {i} definition"),
                );
                limb_accumulator = carry;
            }
        }

        limb_accumulator = limb_accumulator + a.get_felt(NUM_LIMBS - 1) * b.get_felt(NUM_LIMBS - 1);
        air_builder.constrain(
            limb_accumulator
                - c.get_felt(2 * NUM_LIMBS - 1) * shift
                - c.get_felt(2 * NUM_LIMBS - 2),
            "final limb constraint",
        );
    }
}

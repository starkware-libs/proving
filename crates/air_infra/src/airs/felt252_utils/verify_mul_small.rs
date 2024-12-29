use std::cmp::{max, min};

use inst_def::InstDef;
use prover_types::cpu::FELT252_BITS_PER_WORD;

use crate::airs::casm::const_tables::range_check::*;
// Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;

// The number of FELT252_BITS_PER_WORD bit limbs in a multiplication factor.
const NUM_LIMBS: usize = 4;

// Verifying that two NUM_LIMBS*FELT252_BITS_PER_WORD-bit felts multiply to a third
// 2*NUM_LIMBS*FELT252_BITS_PER_WORD bit felt.
// The function assumes all inputs have range-checked limbs.
// It is assumed that 2*NUM_LIMBS*FELT252_BITS_PER_WORD < 252.
#[derive(Clone, Debug, InstDef)]
pub struct VerifyMulSmall {}

impl AirFn for VerifyMulSmall {
    type In = [Felt252Expr; 3];
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, [a, b, c]: Self::In) -> Self::Out {
        let shift = const_expr!(1 << FELT252_BITS_PER_WORD);
        let double_shift = shift.clone() * shift.clone();
        let double_shift_inverse = const_expr!(1) / double_shift.clone();

        let mut limb_accumulator = const_expr!(0u32);

        for i in 0..(2 * NUM_LIMBS - 2) {
            let conditional_shift = if i % 2 == 1 {
                shift.clone()
            } else {
                const_expr!(1)
            };
            let convolution_start = max(i, NUM_LIMBS - 1) - (NUM_LIMBS - 1);
            let convolution_end = min(i, NUM_LIMBS - 1);
            for j in convolution_start..=convolution_end {
                limb_accumulator = limb_accumulator
                    + a.get_felt(j) * b.get_felt(i - j) * conditional_shift.clone();
            }

            limb_accumulator = limb_accumulator - c.get_felt(i) * conditional_shift.clone();
            if i % 2 == 1 {
                let carry = air_builder.deduce(
                    &mut (limb_accumulator.clone() * double_shift_inverse.clone()),
                    &format!("carry_{}", i),
                );
                // Each convolution has at most 4 addends, each addend has at most 2**9-1 overflow.
                air_builder.lookup_call(&RangeCheck { bits: [11] }, [carry.clone()]);
                air_builder.constrain(
                    carry.clone() * double_shift.clone() - limb_accumulator,
                    &format!("carry {} definition", i),
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

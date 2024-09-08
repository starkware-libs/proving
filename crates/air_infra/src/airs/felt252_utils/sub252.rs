use inst_def::InstDef;

use super::verify_add252::*;
use crate::airs::casm::const_tables::range_check::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::prover_types::*;
use crate::core::variables::*;

/// Subtraction of two 252-bit felts.
/// The function assumes the inputs have range-checked limbs, and range-checks the result.
/// The result is not constrained to be fully reduced, i.e. an assignement with b = ((c-a) % P) + P
/// could satisfy the constraints.
#[derive(Clone, Debug, InstDef)]
pub struct Sub252 {}

impl AirFn for Sub252 {
    type In = [Felt252Expr; 2];
    type Out = Felt252Expr;

    fn call(&self, air_builder: &mut AirBuilder, [c, a]: Self::In) -> Self::Out {
        let mut b = air_builder.let_for_deduction(c.clone() - a.clone());
        for b_limb in b.as_felts_mut() {
            air_builder.deduce(b_limb);
            // TODO(DanC): Consider batching these into vector range checks.
            air_builder.lookup_call(
                &RangeCheck {
                    bits: [FELT252_BITS_PER_WORD as u16],
                },
                [b_limb.clone()],
            );
        }

        air_builder.call(&VerifyAdd252 {}, [a, b.clone(), c]);

        b
    }
}

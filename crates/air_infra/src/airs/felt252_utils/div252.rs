use inst_def::InstDef;

use super::verify_mul252::*;
use crate::airs::casm::const_tables::range_check::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::prover_types::*;
use crate::core::variables::*;

/// Division of two 252-bit felts.
/// The function assumes the inputs have range-checked limbs, and range-checks the result.
/// The result is not constrained to be fully reduced, i.e. an assignement with c = ((a / b) % P) + P
/// could satisfy the constraints.
/// The function will panic if the denominator is 0 modulo P.
#[derive(Clone, Debug, InstDef)]
pub struct Div252 {}

impl AirFn for Div252 {
    type In = [Felt252Expr; 2];
    type Out = Felt252Expr;

    fn call(&self, air_builder: &mut AirBuilder, [c, a]: Self::In) -> Self::Out {
        let mut b = air_builder.let_for_deduction(c.clone() / a.clone());
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

        air_builder.call(&VerifyMul252 {}, [a, b.clone(), c]);

        b
    }
}

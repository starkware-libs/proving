use inst_def::InstDef;

use super::verify_mul252::*;
use crate::airs::casm::const_tables::range_check::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::prover_types::*;
use crate::core::variables::*;

/// Multiplication of two 252-bit felts.
/// The function assumes the inputs have range-checked limbs, and range-checks the result.
/// The result is not constrained to be fully reduced, i.e. an assignement with c = ((a*b) % P) + P
/// could satisfy the constraints.
#[derive(Clone, Debug, InstDef)]
pub struct Mul252 {}

impl AirFn for Mul252 {
    type In = [Felt252Expr; 2];
    type Out = Felt252Expr;

    fn call(&self, air_builder: &mut AirBuilder, [a, b]: Self::In) -> Self::Out {
        let mut c = air_builder.let_for_deduction(a.clone() * b.clone());
        for c_limb in c.as_felts_mut() {
            air_builder.deduce(c_limb);
            // TODO(DanC): Consider batching these into vector range checks.
            air_builder.lookup_call(
                &RangeCheck {
                    bits: [FELT252_BITS_PER_WORD as u16],
                },
                [c_limb.clone()],
            );
        }

        air_builder.call(&VerifyMul252 {}, [a, b, c.clone()]);

        c
    }
}

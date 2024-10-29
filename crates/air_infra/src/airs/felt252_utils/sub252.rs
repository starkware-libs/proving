use inst_def::InstDef;

use super::verify_add252::*;
use crate::airs::felt252_id_memory::id_to_big::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
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
        for (i, b_limb) in b.as_felts_mut().into_iter().enumerate() {
            air_builder.deduce(b_limb, &format!("sub_res_limb_{}", i));
        }
        air_builder.call(&RangeCheckBigValue {}, b.clone());

        air_builder.call(&VerifyAdd252 {}, [a, b.clone(), c]);

        b
    }
}

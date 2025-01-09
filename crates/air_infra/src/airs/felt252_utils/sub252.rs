use inst_def::InstDef;

use super::verify_add252::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::felt252_id_memory::id_to_big::*;

/// Subtraction of two 252-bit felts.
/// The function assumes the inputs have range-checked limbs, and range-checks the result.
/// The result is not constrained to be fully reduced, i.e. an assignement with b = ((c-a) % P) + P
/// could satisfy the constraints.
#[derive(Clone, Debug, InstDef)]
pub struct Sub252 {}

impl AirFn for Sub252 {
    type ExtIn = ();
    type In = [Felt252Expr; 2];
    type Out = Felt252Expr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), [c, a]: Self::In) -> Self::Out {
        let b = air_builder.deduce_air_var(c.clone() - a.clone(), "sub_res");

        air_builder.call(&RangeCheckBigValue {}, b.clone());

        air_builder.call(&VerifyAdd252 {}, [a, b.clone(), c]);

        b
    }
}

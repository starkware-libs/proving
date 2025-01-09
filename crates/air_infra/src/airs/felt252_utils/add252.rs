use inst_def::InstDef;

use super::verify_add252::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::felt252_id_memory::id_to_big::*;

/// Addition of two 252-bit felts.
/// The function assumes the inputs have range-checked limbs, and range-checks the result.
/// The result is not constrained to be fully reduced,
/// i.e. an assignement with c = ((a+b) % P) + P could satisfy the constraints.
#[derive(Clone, Debug, InstDef)]
pub struct Add252 {}

impl AirFn for Add252 {
    type ExtIn = ();
    type In = [Felt252Expr; 2];
    type Out = Felt252Expr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), [a, b]: Self::In) -> Self::Out {
        let c = air_builder.deduce_air_var(a.clone() + b.clone(), "add_res");

        air_builder.call(&RangeCheckBigValue {}, c.clone());

        air_builder.call(&VerifyAdd252 {}, [a, b, c.clone()]);

        c
    }
}

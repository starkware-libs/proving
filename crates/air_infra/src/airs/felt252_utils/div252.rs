use inst_def::InstDef;

use super::verify_mul252::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::felt252_id_memory::id_to_big::*;

/// Division of two 252-bit felts.
/// The function assumes the inputs have range-checked limbs, and range-checks the result.
/// The result is not constrained to be fully reduced,
/// i.e. an assignement with c = ((a / b) % P) + P could satisfy the constraints.
/// The function will panic if the denominator is 0 modulo P.
#[derive(Clone, Debug, InstDef)]
pub struct Div252 {}

impl AirFn for Div252 {
    type ExtIn = ();
    type In = [Felt252Expr; 2];
    type Out = Felt252Expr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), [c, a]: Self::In) -> Self::Out {
        let b = air_builder.deduce_air_var(c.clone() / a.clone(), "div_res");

        air_builder.call(&RangeCheckBigValue {}, b.clone());

        air_builder.call(&VerifyMul252 {}, [a, b.clone(), c]);

        b
    }
}

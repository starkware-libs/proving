use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::FELT252_N_WORDS;

use super::verify_add252::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::felt252_id_memory::id_to_big::*;
use crate::core::variables::*;

/// Addition of two 252-bit felts.
/// The function assumes the inputs have range-checked limbs, and range-checks the result.
/// The result is not constrained to be fully reduced,
/// i.e. an assignement with c = ((a+b) % P) + P could satisfy the constraints.
#[derive(Clone, Debug, Serialize)]
pub struct Add252 {}

impl AirFn for Add252 {
    type ExtIn = ();
    type In = [Felt252Expr; 2];
    type Out = Felt252Expr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), [a, b]: Self::In) -> Self::Out {
        let c = air_builder.deduce_air_var(a.clone() + b.clone(), "add_res");

        air_builder.call(
            &RangeCheckMemValue::<FELT252_N_WORDS>::new(),
            c.as_felts()
                .try_into()
                .expect("Expected 'FELT252_N_WORDS' limbs in felt252"),
        );

        air_builder.call(&VerifyAdd252 {}, [a, b, c.clone()]);

        c
    }
}

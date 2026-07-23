use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::variables::AirVar;
use air_infra::felt252_id_memory::id_to_big::RangeCheckMemValue;
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::FELT252_N_WORDS;

use super::verify_add252::*;

/// Subtraction of two 252-bit felts.
/// The function assumes the inputs have range-checked limbs, and range-checks the result.
/// The result is not constrained to be fully reduced, i.e. an assignment with b = ((c-a) % P) + P
/// could satisfy the constraints.
#[derive(Clone, Debug, Serialize)]
pub struct Sub252 {}

impl AirFn for Sub252 {
    type ExtIn = ();
    type In = [Felt252Expr; 2];
    type Out = Felt252Expr;

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![Some("c".to_string()), Some("a".to_string())])
    }

    fn output_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![Some("sub_res".to_string())])
    }

    fn call(&self, air_builder: &mut AirBuilder, _: (), [c, a]: Self::In) -> Self::Out {
        let b = air_builder.deduce_air_var(c.clone() - a.clone(), "sub_res");

        air_builder.call(
            &RangeCheckMemValue::<FELT252_N_WORDS>::new(),
            b.as_felts().try_into().expect("Expected 'FELT252_N_WORDS' limbs in felt252"),
        );

        air_builder.call(&VerifyAdd252 {}, [a, b.clone(), c]);

        b
    }
}

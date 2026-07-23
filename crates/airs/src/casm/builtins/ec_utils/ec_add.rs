use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::variables::AirVar;
use air_infra::felt252_id_memory::id_to_big::RangeCheckMemValue;
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::FELT252_N_WORDS;

use crate::felt252_utils::verify_mul252::*;

#[derive(Debug, Serialize)]
pub struct ECAdd {}

// Elliptic curve point addition.
// Assumes that the input felt252-s have range-checked limbs and range-checks
// the limbs of the result.
impl AirFn for ECAdd {
    type ExtIn = ();
    type In = [Felt252Expr; 4];
    type Out = [Felt252Expr; 2];

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![
            Some("x1".to_string()),
            Some("y1".to_string()),
            Some("x2".to_string()),
            Some("y2".to_string()),
        ])
    }

    fn call(&self, air_builder: &mut AirBuilder, _: (), [x1, y1, x2, y2]: Self::In) -> Self::Out {
        // Deduce, range-check and constrain slope = (y2 - y1) / (x2 - x1).
        let slope = air_builder
            .deduce_air_var((y2.clone() - y1.clone()) / (x2.clone() - x1.clone()), "slope");
        air_builder.call(
            &RangeCheckMemValue::<FELT252_N_WORDS>::new(),
            slope.as_felts().try_into().expect("Expected 'FELT252_N_WORDS' limbs in felt252"),
        );
        let x_diff: Felt252Expr = (0..FELT252_N_WORDS)
            .map(|i| air_builder.let_(x2.get_felt(i) - x1.get_felt(i), &format!("x_diff_{i}")))
            .collect::<Vec<_>>()
            .into();
        let y_diff: Felt252Expr = (0..FELT252_N_WORDS)
            .map(|i| air_builder.let_(y2.get_felt(i) - y1.get_felt(i), &format!("y_diff_{i}")))
            .collect::<Vec<_>>()
            .into();
        air_builder.call(&VerifyMul252 {}, [slope.clone(), x_diff, y_diff]);

        // Deduce, range-check and constrain result_x = slope * slope - x1 - x2.
        let result_x = air_builder
            .deduce_air_var((slope.clone() * slope.clone()) - x1.clone() - x2.clone(), "result_x");
        air_builder.call(
            &RangeCheckMemValue::<FELT252_N_WORDS>::new(),
            result_x.as_felts().try_into().expect("Expected 'FELT252_N_WORDS' limbs in felt252"),
        );
        let x_sum: Felt252Expr = (0..FELT252_N_WORDS)
            .map(|i| {
                air_builder.let_(
                    x1.get_felt(i) + x2.get_felt(i) + result_x.get_felt(i),
                    &format!("x_sum_{i}"),
                )
            })
            .collect::<Vec<_>>()
            .into();
        air_builder.call(&VerifyMul252 {}, [slope.clone(), slope.clone(), x_sum]);

        // Deduce, range-check and constrain result_y = slope * (x1 - result_x) - y1.
        let result_y = air_builder.deduce_air_var(
            slope.clone() * (x1.clone() - result_x.clone()) - y1.clone(),
            "result_y",
        );
        air_builder.call(
            &RangeCheckMemValue::<FELT252_N_WORDS>::new(),
            result_y.as_felts().try_into().expect("Expected 'FELT252_N_WORDS' limbs in felt252"),
        );
        let x_diff_2: Felt252Expr = (0..FELT252_N_WORDS)
            .map(|i| {
                air_builder.let_(x1.get_felt(i) - result_x.get_felt(i), &format!("x_diff2_{i}"))
            })
            .collect::<Vec<_>>()
            .into();
        let y_sum: Felt252Expr = (0..FELT252_N_WORDS)
            .map(|i| air_builder.let_(y1.get_felt(i) + result_y.get_felt(i), &format!("y_sum_{i}")))
            .collect::<Vec<_>>()
            .into();
        air_builder.call(&VerifyMul252 {}, [slope, x_diff_2, y_sum]);

        [result_x, result_y]
    }
}

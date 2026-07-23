use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::variables::AirVar;
use air_infra::felt252_id_memory::id_to_big::RangeCheckMemValue;
use air_infra::{const_expr, const_felt252_expr};
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::FELT252_N_WORDS;

use crate::felt252_utils::add252::*;
use crate::felt252_utils::mul252::*;
use crate::felt252_utils::verify_mul252::*;

#[derive(Debug, Serialize)]
pub struct ECDouble {}

// Elliptic curve point doubling.
// Assumes that the input felt252-s have range-checked limbs and range-checks
// the limbs of the result. Also assumes the input's y-coordinate is non-zero.
impl AirFn for ECDouble {
    type ExtIn = ();
    type In = [Felt252Expr; 2];
    type Out = [Felt252Expr; 2];

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![Some("x".to_string()), Some("y".to_string())])
    }

    fn call(&self, air_builder: &mut AirBuilder, _: (), [x, y]: Self::In) -> Self::Out {
        // Deduce, range-check and constrain slope = (3*x*x + 1) / (2*y).
        let x_squared = air_builder.call(&Mul252 {}, [x.clone(), x.clone()]);
        // TODO(Dan): Consider adjusting VerifyMul to support using (2*y) directly.
        let y_doubled = air_builder.call(&Add252 {}, [y.clone(), y.clone()]);
        let slope = air_builder.deduce_air_var(
            (const_felt252_expr!(3) * x_squared.clone() + const_felt252_expr!(1))
                / (y_doubled.clone()),
            "slope",
        );
        air_builder.call(
            &RangeCheckMemValue::<FELT252_N_WORDS>::new(),
            slope.as_felts().try_into().expect("Expected 'FELT252_N_WORDS' limbs in felt252"),
        );
        let numerator: Felt252Expr = (0..FELT252_N_WORDS)
            .map(|i| {
                air_builder.let_(
                    if i == 0 {
                        const_expr!(3) * x_squared.get_felt(i) + const_expr!(1)
                    } else {
                        const_expr!(3) * x_squared.get_felt(i)
                    },
                    &format!("numerator_{i}"),
                )
            })
            .collect::<Vec<_>>()
            .into();
        air_builder.call(&VerifyMul252 {}, [slope.clone(), y_doubled, numerator]);

        // Deduce, range-check and constrain result_x = slope * slope - 2*x.
        let result_x = air_builder
            .deduce_air_var((slope.clone() * slope.clone()) - x.clone() - x.clone(), "result_x");
        air_builder.call(
            &RangeCheckMemValue::<FELT252_N_WORDS>::new(),
            result_x.as_felts().try_into().expect("Expected 'FELT252_N_WORDS' limbs in felt252"),
        );
        let x_sum: Felt252Expr = (0..FELT252_N_WORDS)
            .map(|i| {
                air_builder.let_(
                    x.get_felt(i) + x.get_felt(i) + result_x.get_felt(i),
                    &format!("x_sum_{i}"),
                )
            })
            .collect::<Vec<_>>()
            .into();
        air_builder.call(&VerifyMul252 {}, [slope.clone(), slope.clone(), x_sum]);

        // Deduce, range-check and constrain result_y = slope * (x - result_x) - y.
        let result_y = air_builder
            .deduce_air_var(slope.clone() * (x.clone() - result_x.clone()) - y.clone(), "result_y");
        air_builder.call(
            &RangeCheckMemValue::<FELT252_N_WORDS>::new(),
            result_y.as_felts().try_into().expect("Expected 'FELT252_N_WORDS' limbs in felt252"),
        );
        let x_diff: Felt252Expr = (0..FELT252_N_WORDS)
            .map(|i| air_builder.let_(x.get_felt(i) - result_x.get_felt(i), &format!("x_diff_{i}")))
            .collect::<Vec<_>>()
            .into();
        let y_sum: Felt252Expr = (0..FELT252_N_WORDS)
            .map(|i| air_builder.let_(y.get_felt(i) + result_y.get_felt(i), &format!("y_sum_{i}")))
            .collect::<Vec<_>>()
            .into();
        air_builder.call(&VerifyMul252 {}, [slope, x_diff, y_sum]);

        [result_x, result_y]
    }
}

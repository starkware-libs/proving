use std::fmt::Debug;

use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use serde::Serialize;

/// Calculates the sum of the squares of the two input numbers.

#[derive(Clone, Debug, Serialize)]
pub struct FibStep {}

impl AirFn for FibStep {
    type ExtIn = ();
    type In = [FeltExpr; 2];
    type Out = FeltExpr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), [x, y]: Self::In) -> Self::Out {
        air_builder.assign(&mut ((x.clone() * x) + (y.clone() * y)), "")
    }
}

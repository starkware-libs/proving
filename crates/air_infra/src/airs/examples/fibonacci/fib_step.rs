use std::fmt::Debug;

// Macros
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;

/// Calculates the sum of the squares of the two input numbers.

#[derive(Clone, Debug)]
pub struct FibStep {}

impl AirFn for FibStep {
    type In = [FeltExpr; 2];
    type Out = FeltExpr;

    fn call(&self, air_builder: &mut AirBuilder, [x, y]: Self::In) -> Self::Out {
        air_builder.assign(&mut (&(&x * &x) + &(&y * &y)))
    }

    fn input_in_trace(&self) -> bool {
        true
    }
}

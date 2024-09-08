use std::fmt::Debug;

use inst_def::InstDef;

// Macros
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;

/// Calculates the sum of the squares of the two input numbers.

#[derive(Clone, Debug, InstDef)]
pub struct FibStep {}

impl AirFn for FibStep {
    type In = [FeltExpr; 2];
    type Out = FeltExpr;

    fn call(&self, air_builder: &mut AirBuilder, [x, y]: Self::In) -> Self::Out {
        air_builder.assign(&mut ((x.clone() * x) + (y.clone() * y)))
    }
}

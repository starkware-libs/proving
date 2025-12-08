use std::fmt::Debug;

use compiled_casm_air::compiled_structs::TraceType;
use serde::Serialize;

use super::fib_step::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;

/// Returns the Fibonacci number at the given index.
#[derive(Clone, Debug, Serialize)]
pub struct NarrowFib {
    pub num_steps: usize,
}

impl AirFn for NarrowFib {
    type ExtIn = ();
    type In = [FeltExpr; 2];
    type Out = [FeltExpr; 2];

    fn call(&self, air_builder: &mut AirBuilder, _: (), mut input: Self::In) -> Self::Out {
        let air_fn = FibStep {};

        for _ in 0..self.num_steps {
            let out = air_builder.call(&air_fn, input.clone());
            input = [input[1].clone(), out];
        }

        input.clone()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}

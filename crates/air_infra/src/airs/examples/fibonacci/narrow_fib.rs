use std::collections::BTreeMap;
use std::fmt::Debug;

use super::fib_step::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;

/// Returns the Fibonacci number at the given index.

#[derive(Clone, Debug)]
pub struct NarrowFib {
    pub num_steps: usize,
}

impl LookupAirFn for NarrowFib {
    type InL = [FeltExpr; 2];
    type OutL = [FeltExpr; 2];

    fn call(&self, air_builder: &mut AirBuilder, initial_state: Self::In) -> Self::Out {
        let mut input = initial_state;
        let air_fn = FibStep {};

        for _ in 0..self.num_steps {
            let out = air_builder.call(&air_fn, input.clone());
            input = [input[1].clone(), out];
        }

        input.clone()
    }

    fn inst_def(&self) -> BTreeMap<String, String> {
        [("num_steps".to_string(), self.num_steps.to_string())].into()
    }
}

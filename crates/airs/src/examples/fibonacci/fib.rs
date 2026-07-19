use std::fmt::Debug;

use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use serde::Serialize;

use super::fib_step::*;

/// Returns the Fibonacci number at the given index.
#[derive(Clone, Debug, Serialize)]
pub struct Fib {
    pub claim_index: usize,
}

impl AirFn for Fib {
    type ExtIn = ();
    type In = FeltExpr;
    type Out = FeltExpr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), mut secret: Self::In) -> Self::Out {
        let mut input = [const_expr!(1), air_builder.deduce(&mut secret, "")];
        let air_fn = FibStep {};

        for _ in 0..(self.claim_index - 2) {
            let out = air_builder.call(&air_fn, input.clone());
            input = [input[1].clone(), out];
        }

        input[1].clone()
    }
}

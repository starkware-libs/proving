use std::collections::BTreeMap;
use std::fmt::Debug;

use super::fib_step::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
// Macros
use crate::const_expr;

/// Returns the Fibonacci number at the given index.

#[derive(Clone, Debug)]
pub struct Fib {
    pub claim_index: usize,
}

impl AirFn for Fib {
    type In = FeltExpr;
    type Out = FeltExpr;

    fn call(&self, air_builder: &mut AirBuilder, mut secret: Self::In) -> Self::Out {
        let mut input = [const_expr!(1), air_builder.deduce(&mut secret)];
        let air_fn = FibStep {};

        for _ in 0..(self.claim_index - 2) {
            let out = air_builder.call(&air_fn, input.clone());
            input = [input[1].clone(), out];
        }

        input[1].clone()
    }

    fn inst_def(&self) -> BTreeMap<String, String> {
        [("claim_index".to_string(), self.claim_index.to_string())].into()
    }
}

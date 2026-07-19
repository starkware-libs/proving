use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use serde::Serialize;

use super::narrow_fib::*;

#[derive(Debug, Serialize)]
pub struct WideFib {
    pub num_narrow: usize,
    pub narrow_size: usize,
}

impl AirFn for WideFib {
    type ExtIn = ();
    type In = FeltExpr;
    type Out = FeltExpr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), mut input: Self::In) -> Self::Out {
        let narrow_fn = NarrowFib { num_steps: self.narrow_size };

        air_builder.deduce(&mut input, "");
        let mut narrow_input = [const_expr!(1), input];

        for _ in 0..self.num_narrow {
            let narrow_output = air_builder.lookup_call(&narrow_fn, (), narrow_input);
            narrow_input = narrow_output;
        }

        narrow_input[1].clone()
    }
}

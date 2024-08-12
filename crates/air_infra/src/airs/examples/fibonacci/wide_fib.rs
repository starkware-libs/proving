use indexmap::IndexMap;

use super::narrow_fib::NarrowFib;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;

// Macros
use crate::const_expr;

#[derive(Debug)]
pub struct WideFib {
    pub num_narrow: usize,
    pub narrow_size: usize,
}

impl AirFn for WideFib {
    type In = FeltExpr;
    type Out = FeltExpr;

    fn call(
        &self,
        air_builder: &mut crate::core::air_fn::AirBuilder,
        mut input: Self::In,
    ) -> Self::Out {
        let narrow_fn = NarrowFib {
            num_steps: self.narrow_size,
        };

        air_builder.deduce(&mut input);
        let mut narrow_output = [const_expr!(1), input];

        for _ in 0..self.num_narrow {
            narrow_output = air_builder.lookup_call(&narrow_fn, narrow_output);
        }

        narrow_output[1].clone()
    }

    fn inst_def(&self) -> IndexMap<String, String> {
        [
            ("num_narrow".to_string(), self.num_narrow.to_string()),
            ("narrow_size".to_string(), self.narrow_size.to_string()),
        ]
        .into()
    }
}

use inst_def::InstDef;

use super::narrow_fib::*;
// Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;

#[derive(Debug, InstDef)]
pub struct WideFib {
    pub num_narrow: usize,
    pub narrow_size: usize,
}

impl AirFn for WideFib {
    type ExtIn = ();
    type In = FeltExpr;
    type Out = FeltExpr;

    fn call(
        &self,
        air_builder: &mut crate::core::air_fn::AirBuilder,
        _: (),
        mut input: Self::In,
    ) -> Self::Out {
        let narrow_fn = NarrowFib {
            num_steps: self.narrow_size,
        };

        air_builder.deduce(&mut input, "");
        let mut narrow_input = [const_expr!(1), input];

        for _ in 0..self.num_narrow {
            let narrow_output = air_builder.lookup_call(&narrow_fn, (), narrow_input);
            narrow_input = narrow_output;
        }

        narrow_input[1].clone()
    }
}

use super::narrow_fib::NarrowFib;
use crate::core::air_fn::AirFn;
use crate::core::expressions::felt_expr::FeltExpr;
use crate::core::prover_types::Felt;
use crate::expr;

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
        input: Self::In,
    ) -> Self::Out {
        let narrow_fn = NarrowFib {
            num_steps: self.narrow_size,
        };

        let x = air_builder.deduce(&mut expr!("1", 1));
        let y = air_builder.deduce(&mut (input.clone()));

        let mut narrow_output = [x, y];

        for _ in 0..self.num_narrow {
            narrow_output = air_builder.lookup_call(&narrow_fn, narrow_output);
        }

        narrow_output[1].clone()
    }
}

use std::collections::BTreeMap;
use std::fmt::Debug;

use super::super::core::air_fn::*;
use super::super::core::expressions::bool_expr::*;
use super::super::core::expressions::uint16_expr::*;
use super::div2::Div2;

/// Unpacks a 16-bit unsigned integer into a vector of bits.

#[derive(Clone, Debug)]
pub struct BitUnpack {
    pub n_bits: usize,
}

impl AirFn for BitUnpack {
    type In = UInt16Expr;
    type Out = Vec<BoolExpr>;

    fn call(&self, air_builder: &mut AirBuilder, mut x: Self::In) -> Self::Out {
        air_builder.deduce(x.as_felt());
        let mut input = x;
        let mut output = vec![];
        let air_fn = Div2 {};

        for _ in 0..self.n_bits {
            let (bit, next_input) = air_builder.call(&air_fn, input);
            input = next_input;
            output.push(bit);
        }

        air_builder.constrain(input.as_felt().clone());
        output
    }

    fn inst_def(&self) -> BTreeMap<String, String> {
        [("n_bits".to_string(), self.n_bits.to_string())].into()
    }

    fn input_in_trace(&self) -> bool {
        false
    }
}

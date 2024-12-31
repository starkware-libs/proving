use std::fmt::Debug;

use inst_def::InstDef;

use super::div2::*;
use crate::core::air_fn::*;
use crate::core::expressions::bool_expr::*;
use crate::core::expressions::uint16_expr::*;

/// Unpacks a 16-bit unsigned integer into a vector of bits.

#[derive(Clone, Debug, InstDef)]
pub struct BitUnpack<const N: usize> {}

impl<const N: usize> AirFn for BitUnpack<N> {
    type ExtIn = ();
    type In = UInt16Expr;
    type Out = [BoolExpr; N];

    fn call(&self, air_builder: &mut AirBuilder, _: (), mut x: Self::In) -> Self::Out {
        air_builder.deduce(x.as_felt_mut(), "");
        let mut input = x;
        let mut output = vec![];
        let air_fn = Div2 {};

        for _ in 0..N {
            let (bit, next_input) = air_builder.call(&air_fn, input);
            input = next_input;
            output.push(bit);
        }

        air_builder.constrain(input.as_felt(), "");
        output.try_into().expect("Invalid number of bits")
    }
}

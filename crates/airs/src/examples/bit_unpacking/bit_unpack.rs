use std::fmt::Debug;

use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::bool_expr::BoolExpr;
use air_infra::core::expressions::uint16_expr::UInt16Expr;
use serde::Serialize;

use super::div2::*;

/// Unpacks a 16-bit unsigned integer into a vector of bits.

#[derive(Clone, Debug, Serialize)]
pub struct BitUnpack<const N: usize> {
    n: usize,
}

impl<const N: usize> BitUnpack<N> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { n: N }
    }
}

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

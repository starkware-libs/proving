use std::fmt::Debug;

use super::super::core::air_fn::*;
use super::super::core::expressions::bool_expr::*;
use super::super::core::expressions::felt_expr::*;
use super::super::core::expressions::uint16_expr::*;
use super::super::core::prover_types::*;
// Macros
use crate::const_expr;
use crate::const_u16_expr;

/// Divides a 16-bit unsigned integer by 2. Returns the quotient and the remainder.

#[derive(Clone, Debug)]
pub struct Div2 {}

impl AirFn for Div2 {
    type In = UInt16Expr;
    type Out = (BoolExpr, UInt16Expr);

    fn call(&self, air_builder: &mut AirBuilder, mut x0: Self::In) -> Self::Out {
        let mut x1 = air_builder.create_intermediate_var_for_deduction(&x0 >> &const_u16_expr!(1));
        let x1_felt = air_builder.deduce(x1.as_felt());
        // Calculate the least significant bit of the input = x0 - 2 * x1
        let lsb = &*(x0.as_felt()) - &(&x1_felt * &const_expr!(2));
        // Constrain the least significant bit to be 0 or 1, i.e. (x0 - 2x1) * (x0 - 2x1 - 1) = 0
        air_builder.constrain(&lsb * &(&lsb - &const_expr!(1)));

        (lsb.eq(const_expr!(1)), x1)
    }

    fn input_in_trace(&self) -> bool {
        true
    }
}

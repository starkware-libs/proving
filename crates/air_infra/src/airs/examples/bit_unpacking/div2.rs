use inst_def::InstDef;
use std::fmt::Debug;

use crate::core::air_fn::*;
use crate::core::expressions::bool_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;

// Macros
use crate::const_expr;
use crate::const_u16_expr;

/// Divides a 16-bit unsigned integer by 2. Returns the quotient and the remainder.

#[derive(Clone, Debug, InstDef)]
pub struct Div2 {}

impl AirFn for Div2 {
    type In = UInt16Expr;
    type Out = (BoolExpr, UInt16Expr);

    fn call(&self, air_builder: &mut AirBuilder, x0: Self::In) -> Self::Out {
        let mut x1 = air_builder.let_for_deduction(x0.clone() >> const_u16_expr!(1));
        let x1_felt = air_builder.deduce(x1.as_felt_mut(), "");
        // Calculate the least significant bit of the input = x0 - 2 * x1
        let lsb = air_builder.let_for_constraint(x0.as_felt() - (x1_felt * const_expr!(2)));
        // Constrain the least significant bit to be 0 or 1, i.e. (x0 - 2x1) * (x0 - 2x1 - 1) = 0
        air_builder.constrain(lsb.clone() * (lsb.clone() - const_expr!(1)));

        (lsb.into(), x1)
    }
}

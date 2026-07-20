use std::fmt::Debug;

use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::bool_expr::BoolExpr;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint16_expr::UInt16Expr;
use air_infra::{const_expr, const_u16_expr};
use serde::Serialize;

/// Divides a 16-bit unsigned integer by 2. Returns the quotient and the remainder.

#[derive(Clone, Debug, Serialize)]
pub struct Div2 {}

impl AirFn for Div2 {
    type ExtIn = ();
    type In = UInt16Expr;
    type Out = (BoolExpr, UInt16Expr);

    fn call(&self, air_builder: &mut AirBuilder, _: (), x0: Self::In) -> Self::Out {
        let x1 = air_builder.deduce_air_var(x0.clone() >> const_u16_expr!(1), "");
        // Calculate the least significant bit of the input = x0 - 2 * x1
        let lsb = air_builder.let_(x0.as_felt() - (x1.as_felt() * const_expr!(2)), "");
        // Constrain the least significant bit to be 0 or 1, i.e. (x0 - 2x1) * (x0 - 2x1 - 1) = 0
        air_builder.constrain(lsb.clone() * (lsb.clone() - const_expr!(1)), "");

        (lsb.into(), x1)
    }
}

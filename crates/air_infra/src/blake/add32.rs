use super::super::core::air_fn::*;
use super::super::core::expressions::felt_expr::*;
use super::super::core::expressions::uint32_expr::*;
use super::super::core::prover_types::*;
// Macros
use crate::const_expr;

/// Wrapping addition of two 32-bit unsigned integers.

#[derive(Clone, Debug)]
pub struct Add32 {}

impl AirFn for Add32 {
    type In = [UInt32Expr; 2];
    type Out = UInt32Expr;

    fn call(&self, air_builder: &mut AirBuilder, [mut a, mut b]: Self::In) -> Self::Out {
        let mut c = air_builder.create_intermediate_var(&a + &b);
        let cl = air_builder.deduce(c.low().as_felt());
        let ch = air_builder.deduce(c.high().as_felt());
        // TODO: Add range check 16 for cl and ch.

        // TODO: Use constraint intermediate variable when possible.
        // let carry = air_builder.create_intermediate_var(&(&*a.low().as_felt() + &*b.low().as_felt()) - &cl);
        air_builder.constrain(
            &(&(&*a.low().as_felt() + &*b.low().as_felt()) - &cl)
                * &(&(&(&*a.low().as_felt() + &*b.low().as_felt()) - &cl) - &const_expr!(1 << 16)),
        );
        air_builder.constrain(
            &(&(&(&*a.high().as_felt() + &*b.high().as_felt()) - &ch) * &const_expr!(1 << 16))
                + &(&(&*a.low().as_felt() + &*b.low().as_felt()) - &cl),
        );

        c
    }

    fn input_in_trace(&self) -> bool {
        true
    }
}

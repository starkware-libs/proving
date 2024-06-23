use crate::airs::range_check::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint32_expr::*;
use crate::core::prover_types::*;
// Macros
use crate::const_expr;

/// Wrapping addition of two 32-bit unsigned integers.

#[derive(Clone, Debug)]
pub struct Add32 {}

impl AirFn for Add32 {
    type In = [UInt32Expr; 2];
    type Out = UInt32Expr;

    fn call(&self, air_builder: &mut AirBuilder, [mut a, mut b]: Self::In) -> Self::Out {
        let mut c = air_builder.let_for_deduction(a.clone() + b.clone());
        let cl = air_builder.deduce(c.low().as_felt());
        let ch = air_builder.deduce(c.high().as_felt());
        air_builder.lookup_call(&RangeCheck { bits: 16 }, cl.clone());
        air_builder.lookup_call(&RangeCheck { bits: 16 }, ch.clone());

        let carry = air_builder
            .let_for_constraint((a.low().as_felt().clone() + b.low().as_felt().clone()) - cl);
        air_builder.constrain(carry.clone() * (carry.clone() - const_expr!(1 << 16)));
        air_builder.constrain(
            (((a.high().as_felt().clone() + b.high().as_felt().clone()) - ch)
                * const_expr!(1 << 16))
                + carry,
        );

        c
    }
}

use inst_def::InstDef;

use crate::airs::casm::const_tables::range_check::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint32_expr::*;

// Macros
use crate::const_expr;

/// Wrapping addition of two 32-bit unsigned integers.
#[derive(Clone, Debug, InstDef)]
pub struct Add32 {}

impl AirFn for Add32 {
    type In = [UInt32Expr; 2];
    type Out = UInt32Expr;

    fn call(&self, air_builder: &mut AirBuilder, [a, b]: Self::In) -> Self::Out {
        let mut c = air_builder.let_for_deduction(a.clone() + b.clone());
        let cl = air_builder.deduce(c.low_mut().as_felt_mut(), "add_res_limb_0");
        let ch = air_builder.deduce(c.high_mut().as_felt_mut(), "add_res_limb_1");
        air_builder.lookup_call(&RangeCheck { bits: [16] }, [cl.clone()]);
        air_builder.lookup_call(&RangeCheck { bits: [16] }, [ch.clone()]);

        // Verify addition of the low halves
        let carry = air_builder.let_for_constraint((a.low().as_felt() + b.low().as_felt()) - cl);
        air_builder.constrain(carry.clone() * (carry.clone() - const_expr!(1 << 16)));

        // Verify addition of the high halves
        let carry_hi = air_builder.let_for_constraint(
            ((a.high().as_felt() + b.high().as_felt()) - ch)
                + carry * (const_expr!(1) / const_expr!(1 << 16)),
        );
        air_builder.constrain(carry_hi.clone() * (carry_hi - const_expr!(1 << 16)));

        c
    }
}

use inst_def::InstDef;

// Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint32_expr::*;

/// Receives a, b and c each as UInt32 which is a pair of felts - low and high.
/// Returns their sum mod 2^32.
/// The caller is responsible to range check the output.
#[derive(Clone, Debug, InstDef)]
pub struct TripleSum32 {}

impl AirFn for TripleSum32 {
    type ExtIn = ();
    type In = [UInt32Expr; 3];
    type Out = UInt32Expr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), [a, b, c]: Self::In) -> Self::Out {
        let mut s =
            air_builder.let_for_deduction(a.clone() + b.clone() + c.clone(), "triple_sum32_res");
        let sl = air_builder.deduce(s.low_mut().as_felt_mut(), "triple_sum32_res_low");
        let sh = air_builder.deduce(s.high_mut().as_felt_mut(), "triple_sum32_res_high");

        // Verify addition of the low halves
        let carry_low = air_builder.let_for_constraint(
            ((a.low().as_felt() + b.low().as_felt() + c.low().as_felt()) - sl.clone())
                * (const_expr!(1) / const_expr!(1 << 16)),
            "carry_low",
        );
        air_builder.constrain(
            carry_low.clone()
                * (carry_low.clone() - const_expr!(1))
                * (carry_low.clone() - const_expr!(2)),
            "carry low is 0 or 1 or 2",
        );

        // Verify addition of the high halves
        let carry_high = air_builder.let_for_constraint(
            ((a.high().as_felt() + b.high().as_felt() + c.high().as_felt() + carry_low)
                - sh.clone())
                * (const_expr!(1) / const_expr!(1 << 16)),
            "carry_high",
        );
        air_builder.constrain(
            carry_high.clone()
                * (carry_high.clone() - const_expr!(1))
                * (carry_high.clone() - const_expr!(2)),
            "carry high is 0 or 1 or 2",
        );

        air_builder.let_vec(vec![sl, sh], "triple_sum32_res")
    }
}

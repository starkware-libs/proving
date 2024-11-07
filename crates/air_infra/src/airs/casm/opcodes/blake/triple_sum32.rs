use inst_def::InstDef;

// Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint32_expr::*;

/// Receives a = (al, ah), b = (bl, bh), and c = (cl, ch) as three pairs of range-checked felts such
/// that each pair represents UInt32.
/// Returns their sum mod 2^32.
/// The caller is responsible to range check the output.
#[derive(Clone, Debug, InstDef)]
pub struct TripleSum32 {}

impl AirFn for TripleSum32 {
    type In = [FeltExpr; 6];
    type Out = [FeltExpr; 2];

    fn call(&self, air_builder: &mut AirBuilder, [al, ah, bl, bh, cl, ch]: Self::In) -> Self::Out {
        let mut s = air_builder.let_for_deduction(
            UInt32Expr::from(vec![al.clone(), ah.clone()])
                + UInt32Expr::from(vec![bl.clone(), bh.clone()])
                + UInt32Expr::from(vec![cl.clone(), ch.clone()]),
            "triple_sum32",
        );
        let sl = air_builder.deduce(s.low_mut().as_felt_mut(), "add_res_limb_0");
        let sh = air_builder.deduce(s.high_mut().as_felt_mut(), "add_res_limb_1");

        // Verify addition of the low halves
        let carry_low = air_builder.let_for_constraint(
            ((al + bl + cl) - sl.clone()) * (const_expr!(1) / const_expr!(1 << 16)),
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
            ((ah + bh + ch + carry_low) - sh.clone()) * (const_expr!(1) / const_expr!(1 << 16)),
            "carry_high",
        );
        air_builder.constrain(
            carry_high.clone()
                * (carry_high.clone() - const_expr!(1))
                * (carry_high.clone() - const_expr!(2)),
            "carry high is 0 or 1 or 2",
        );

        [sl, sh]
    }
}

use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use serde::Serialize;

/// Receives a, b and c each as UInt32 which is a pair of felts - low and high.
/// Returns their sum mod 2^32.
/// The caller is responsible to range check the output.
#[derive(Clone, Debug, Serialize)]
pub struct TripleSum32 {}

impl AirFn for TripleSum32 {
    type ExtIn = ();
    type In = [UInt32Expr; 3];
    type Out = UInt32Expr;

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![Some("a".to_string()), Some("b".to_string()), Some("c".to_string())])
    }

    fn call(&self, air_builder: &mut AirBuilder, _: (), [a, b, c]: Self::In) -> Self::Out {
        let s = air_builder.deduce_air_var(a.clone() + b.clone() + c.clone(), "triple_sum32_res");
        air_builder.call(&VerifyTripleSum32 {}, [a, b, c, s.clone()]);
        s
    }
}

/// Verifies that `s = a + b + c mod 2^32.
#[derive(Clone, Debug, Serialize)]
pub struct VerifyTripleSum32 {}

impl AirFn for VerifyTripleSum32 {
    type ExtIn = ();
    type In = [UInt32Expr; 4]; // [a, b, c, s]
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, _: (), [a, b, c, s]: Self::In) -> Self::Out {
        // Verify addition of the low halves
        let carry_low = air_builder.let_for_constraint(
            ((a.low().as_felt() + b.low().as_felt() + c.low().as_felt()) - s.low().as_felt())
                * (const_expr!(1 << 16).inverse()),
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
                - s.high().as_felt())
                * (const_expr!(1 << 16).inverse()),
            "carry_high",
        );
        air_builder.constrain(
            carry_high.clone()
                * (carry_high.clone() - const_expr!(1))
                * (carry_high.clone() - const_expr!(2)),
            "carry high is 0 or 1 or 2",
        );
    }
}

use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::range_check::range_check;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct VerifyReduced252 {}

/// Verify that a given Felt252 is < P252. Assumes that the input has range-checked limbs.
///
/// Numbers that are < P252 fall into three categories:
///    Limb no. ->    27  26  25  24  23  22  21  20  19       1   0
///           (A)  0x100 000 000 000 000 000 088 000 000 ... 000 000
///           (B)  0x100 000 000 000 000 000   L XXX XXX ... XXX XXX  with L < 0x88
///           (C)  0x  M XXX XXX XXX XXX XXX XXX XXX XXX ... XXX XXX  with M < 0x100
///
/// We deduce two bits to check for these categories:
/// 1. ms_max - 1 in cases A,B
/// 2. both_max - 1 in case A
impl AirFn for VerifyReduced252 {
    type ExtIn = ();
    type In = Felt252Expr;
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, _: (), input: Self::In) -> Self::Out {
        // Compute and deduce the ms_max and both_max bits
        let ms_limb = input.get_felt(27);
        let ms_max = ms_limb.clone().eq(const_expr!(256));
        let both_max = ms_max.clone() & input.get_felt(21).eq(const_expr!(17 << 3));

        let ms_max = air_builder.deduce_air_var(ms_max, "ms_limb_is_max");
        let both_max = air_builder.deduce_air_var(both_max, "ms_and_mid_limbs_are_max");

        let ms_max = ms_max.as_felt();
        let both_max = both_max.as_felt();

        air_builder.constrain(ms_max.clone() * (const_expr!(1) - ms_max.clone()), "ms_max is bit");
        air_builder
            .constrain(both_max.clone() * (const_expr!(1) - both_max.clone()), "both_max is bit");

        // TODO(DanC): Change the two range_check([8]) to range_check([8,8]).

        // Range check ms_limb - ms_max. This verifies that
        // 1. ms_limb < 256, or
        // 2. ms_limb == 256 and ms_max == 1
        range_check(air_builder, &[8], &[ms_limb - ms_max.clone()]);

        // If ms_max == 1, check that the high limbs are zero
        let high_limbs_sum = (22..27).map(|i| input.get_felt(i)).sum();

        air_builder.constrain(
            ms_max.clone() * high_limbs_sum,
            "If the MS limb is max, high limbs should be 0",
        );

        // Range check ms_max * (120 + mid_limb - both_max) < 256. This verifies that either
        // 1. ms_max == 0, or
        // 2. ms_max == 1 and mid_limb < 136, or
        // 3. ms_max == 1 and mid_limb == 136 and both_max == 1
        let mut rc_input =
            ms_max.clone() * (const_expr!(120) + input.get_felt(21) - both_max.clone());
        air_builder.assign(&mut rc_input, "rc_input");
        range_check(air_builder, &[8], &[rc_input]);

        // If both_max == 1, check that the low limbs are zero
        let low_limbs_sum = (0..21).map(|i| input.get_felt(i)).sum();
        air_builder.constrain(
            both_max.clone() * low_limbs_sum,
            "If the MS and mid limbs are max, low limbs should be 0",
        );
    }
}

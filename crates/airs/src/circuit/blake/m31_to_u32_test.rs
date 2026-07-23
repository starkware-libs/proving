use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::{const_expr, const_u32_expr};
use expect_test::expect;
use stwo_cairo_common::prover_types::cpu::PRIME;

use super::m31_to_u32::M31ToU32;

#[test]
fn test_m31_to_u32() {
    let value = 1567342098;

    let m31_to_u32 = M31ToU32 {};
    let (registry, _) = AirFnRegistry::new(&m31_to_u32);
    let (state, _) =
        registry.run_air(&m31_to_u32, (), (const_expr!(value), const_u32_expr!(value)));

    expect![[r#"
        (1567342098, "input_m31"),
        (48658, "input_u32_limb_0"),
        (23915, "input_u32_limb_1"),
        (747464213, "inv_or_one"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evaluate to 0)")]
fn test_m31_to_u32_fails_on_p() {
    let m31_to_u32 = M31ToU32 {};
    let (registry, _) = AirFnRegistry::new(&m31_to_u32);
    registry.run_air(&m31_to_u32, (), (const_expr!(PRIME), const_u32_expr!(PRIME)));
}

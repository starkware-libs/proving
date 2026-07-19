use air_infra::const_u32_expr;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::core::variables::AsProverType;
use expect_test::expect;

use super::triple_sum32::*;

#[test]
fn test_verify_triple_sum() {
    let verify = VerifyTripleSum32 {};
    let (registry, _) = AirFnRegistry::new(&verify);
    registry.run_air(
        &verify,
        (),
        [
            const_u32_expr!(2724151460),
            const_u32_expr!(102499851),
            const_u32_expr!(1441071338),
            const_u32_expr!(4267722649),
        ],
    );
}

#[test]
fn test_triple_sum1() {
    let triple_sum = TripleSum32 {};
    let (registry, _) = AirFnRegistry::new(&triple_sum);
    let (state, output) = registry.run_air(
        &triple_sum,
        (),
        [const_u32_expr!(2724151460), const_u32_expr!(102499851), const_u32_expr!(1441071338)],
    );
    assert_eq!(output.calc(), "4267722649");

    // Check state
    expect![[r#"
        (18329, "triple_sum32_res_limb_0"),
        (65120, "triple_sum32_res_limb_1"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_triple_sum2() {
    let triple_sum = TripleSum32 {};
    let (registry, _) = AirFnRegistry::new(&triple_sum);
    let (state, output) = registry.run_air(
        &triple_sum,
        (),
        [const_u32_expr!(2147516416), const_u32_expr!(2147516416), const_u32_expr!(2147516416)],
    );
    assert_eq!(output.calc(), "2147581952");

    // Check state
    expect![[r#"
        (32768, "triple_sum32_res_limb_0"),
        (32769, "triple_sum32_res_limb_1"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_triple_sum3() {
    let triple_sum = TripleSum32 {};
    let (registry, _) = AirFnRegistry::new(&triple_sum);
    let (state, output) = registry.run_air(
        &triple_sum,
        (),
        [const_u32_expr!(4294967295), const_u32_expr!(4294967295), const_u32_expr!(4294967295)],
    );
    assert_eq!(output.calc(), "4294967293");

    // Check state
    expect![[r#"
        (65533, "triple_sum32_res_limb_0"),
        (65535, "triple_sum32_res_limb_1"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_pair_sum() {
    let triple_sum = TripleSum32 {};
    let (registry, _) = AirFnRegistry::new(&triple_sum);
    let (state, output) = registry.run_air(
        &triple_sum,
        (),
        [const_u32_expr!(7945302), const_u32_expr!(151796478), const_u32_expr!(0)],
    );
    assert_eq!(output.calc(), "159741780");

    // Check state
    expect![[r#"
        (30548, "triple_sum32_res_limb_0"),
        (2437, "triple_sum32_res_limb_1"),
    "#]]
    .assert_eq(&state.to_string());
}

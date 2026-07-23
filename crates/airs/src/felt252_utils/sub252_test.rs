use air_infra::const_felt252_expr;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::variables::AsProverType;
use expect_test::expect;

use super::sub252::*;

#[test]
fn test_sub252_no_underflow() {
    let air_fn = Sub252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (state, output) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(0x3000040002u128, 0u128),
            const_felt252_expr!(0x1008020001u128, 0u128),
        ],
    );
    assert_eq!(output.calc(), const_felt252_expr!(0x1ff8020001u128, 0u128).calc());
    expect![[r#"
        (1, "sub_res_limb_0"),
        (256, "sub_res_limb_1"),
        (0, "sub_res_limb_2"),
        (511, "sub_res_limb_3"),
        (1, "sub_res_limb_4"),
        (0, "sub_res_limb_5"),
        (0, "sub_res_limb_6"),
        (0, "sub_res_limb_7"),
        (0, "sub_res_limb_8"),
        (0, "sub_res_limb_9"),
        (0, "sub_res_limb_10"),
        (0, "sub_res_limb_11"),
        (0, "sub_res_limb_12"),
        (0, "sub_res_limb_13"),
        (0, "sub_res_limb_14"),
        (0, "sub_res_limb_15"),
        (0, "sub_res_limb_16"),
        (0, "sub_res_limb_17"),
        (0, "sub_res_limb_18"),
        (0, "sub_res_limb_19"),
        (0, "sub_res_limb_20"),
        (0, "sub_res_limb_21"),
        (0, "sub_res_limb_22"),
        (0, "sub_res_limb_23"),
        (0, "sub_res_limb_24"),
        (0, "sub_res_limb_25"),
        (0, "sub_res_limb_26"),
        (0, "sub_res_limb_27"),
        (0, "sub_p_bit"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_sub252_with_underflow() {
    let air_fn = Sub252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (state, output) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(
                0xffffffffffffffffffffffffffffffffu128,
                0x7ffffffffffffeeffffffffffffffffu128
            ),
            const_felt252_expr!(0, 1u128 << (251 - 128)),
        ],
    );
    assert_eq!(output.calc(), const_felt252_expr!(0, 1u128 << (251 - 128)).calc());
    expect![[r#"
        (0, "sub_res_limb_0"),
        (0, "sub_res_limb_1"),
        (0, "sub_res_limb_2"),
        (0, "sub_res_limb_3"),
        (0, "sub_res_limb_4"),
        (0, "sub_res_limb_5"),
        (0, "sub_res_limb_6"),
        (0, "sub_res_limb_7"),
        (0, "sub_res_limb_8"),
        (0, "sub_res_limb_9"),
        (0, "sub_res_limb_10"),
        (0, "sub_res_limb_11"),
        (0, "sub_res_limb_12"),
        (0, "sub_res_limb_13"),
        (0, "sub_res_limb_14"),
        (0, "sub_res_limb_15"),
        (0, "sub_res_limb_16"),
        (0, "sub_res_limb_17"),
        (0, "sub_res_limb_18"),
        (0, "sub_res_limb_19"),
        (0, "sub_res_limb_20"),
        (0, "sub_res_limb_21"),
        (0, "sub_res_limb_22"),
        (0, "sub_res_limb_23"),
        (0, "sub_res_limb_24"),
        (0, "sub_res_limb_25"),
        (0, "sub_res_limb_26"),
        (256, "sub_res_limb_27"),
        (1, "sub_p_bit"),
    "#]]
    .assert_eq(&state.to_string());
}

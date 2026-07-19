use air_infra::const_felt252_expr;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::variables::AsProverType;
use expect_test::expect;

use super::add252::*;

#[test]
fn test_add252_no_overflow() {
    let air_fn = Add252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(0x1008020001u128, 0u128),
            const_felt252_expr!(0x1ff8020001u128, 0u128),
        ],
    );
    assert_eq!(output.calc(), const_felt252_expr!(0x3000040002u128, 0u128).calc());

    expect![[r#"
        (2, "add_res_limb_0"),
        (0, "add_res_limb_1"),
        (1, "add_res_limb_2"),
        (0, "add_res_limb_3"),
        (3, "add_res_limb_4"),
        (0, "add_res_limb_5"),
        (0, "add_res_limb_6"),
        (0, "add_res_limb_7"),
        (0, "add_res_limb_8"),
        (0, "add_res_limb_9"),
        (0, "add_res_limb_10"),
        (0, "add_res_limb_11"),
        (0, "add_res_limb_12"),
        (0, "add_res_limb_13"),
        (0, "add_res_limb_14"),
        (0, "add_res_limb_15"),
        (0, "add_res_limb_16"),
        (0, "add_res_limb_17"),
        (0, "add_res_limb_18"),
        (0, "add_res_limb_19"),
        (0, "add_res_limb_20"),
        (0, "add_res_limb_21"),
        (0, "add_res_limb_22"),
        (0, "add_res_limb_23"),
        (0, "add_res_limb_24"),
        (0, "add_res_limb_25"),
        (0, "add_res_limb_26"),
        (0, "add_res_limb_27"),
        (0, "sub_p_bit"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_add252_with_overflow() {
    let air_fn = Add252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (state, output) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(0, 1u128 << (251 - 128)),
            const_felt252_expr!(0, 1u128 << (251 - 128)),
        ],
    );
    assert_eq!(
        output.calc(),
        const_felt252_expr!(
            0xffffffffffffffffffffffffffffffffu128,
            0x7ffffffffffffeeffffffffffffffffu128
        )
        .calc()
    );

    expect![[r#"
        (511, "add_res_limb_0"),
        (511, "add_res_limb_1"),
        (511, "add_res_limb_2"),
        (511, "add_res_limb_3"),
        (511, "add_res_limb_4"),
        (511, "add_res_limb_5"),
        (511, "add_res_limb_6"),
        (511, "add_res_limb_7"),
        (511, "add_res_limb_8"),
        (511, "add_res_limb_9"),
        (511, "add_res_limb_10"),
        (511, "add_res_limb_11"),
        (511, "add_res_limb_12"),
        (511, "add_res_limb_13"),
        (511, "add_res_limb_14"),
        (511, "add_res_limb_15"),
        (511, "add_res_limb_16"),
        (511, "add_res_limb_17"),
        (511, "add_res_limb_18"),
        (511, "add_res_limb_19"),
        (511, "add_res_limb_20"),
        (375, "add_res_limb_21"),
        (511, "add_res_limb_22"),
        (511, "add_res_limb_23"),
        (511, "add_res_limb_24"),
        (511, "add_res_limb_25"),
        (511, "add_res_limb_26"),
        (255, "add_res_limb_27"),
        (1, "sub_p_bit"),
    "#]]
    .assert_eq(&state.to_string());
}

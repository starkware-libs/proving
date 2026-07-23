use air_infra::const_felt252_expr;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::variables::AsProverType;
use expect_test::expect;

use super::div252::*;

#[test]
fn test_div252_no_overflow() {
    let air_fn = Div252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(0x2008020003400040001u128, 0u128),
            const_felt252_expr!(0x1ff8020001u128, 0u128),
        ],
    );
    assert_eq!(output.calc(), const_felt252_expr!(0x1008020001u128, 0u128).calc());
    expect![[r#"
        (1, "div_res_limb_0"),
        (256, "div_res_limb_1"),
        (0, "div_res_limb_2"),
        (1, "div_res_limb_3"),
        (1, "div_res_limb_4"),
        (0, "div_res_limb_5"),
        (0, "div_res_limb_6"),
        (0, "div_res_limb_7"),
        (0, "div_res_limb_8"),
        (0, "div_res_limb_9"),
        (0, "div_res_limb_10"),
        (0, "div_res_limb_11"),
        (0, "div_res_limb_12"),
        (0, "div_res_limb_13"),
        (0, "div_res_limb_14"),
        (0, "div_res_limb_15"),
        (0, "div_res_limb_16"),
        (0, "div_res_limb_17"),
        (0, "div_res_limb_18"),
        (0, "div_res_limb_19"),
        (0, "div_res_limb_20"),
        (0, "div_res_limb_21"),
        (0, "div_res_limb_22"),
        (0, "div_res_limb_23"),
        (0, "div_res_limb_24"),
        (0, "div_res_limb_25"),
        (0, "div_res_limb_26"),
        (0, "div_res_limb_27"),
        (0, "k"),
        (0, "carry_0"),
        (32, "carry_1"),
        (4097, "carry_2"),
        (160, "carry_3"),
        (8193, "carry_4"),
        (288, "carry_5"),
        (33, "carry_6"),
        (33, "carry_7"),
        (3, "carry_8"),
        (256, "carry_9"),
        (2, "carry_10"),
        (512, "carry_11"),
        (2, "carry_12"),
        (2, "carry_13"),
        (2, "carry_14"),
        (0, "carry_15"),
        (0, "carry_16"),
        (0, "carry_17"),
        (0, "carry_18"),
        (0, "carry_19"),
        (0, "carry_20"),
        (0, "carry_21"),
        (0, "carry_22"),
        (0, "carry_23"),
        (0, "carry_24"),
        (0, "carry_25"),
        (0, "carry_26"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_div252_with_overflow() {
    let air_fn = Div252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (_, output) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(
                0x4cc3ffffffffff5cdf8002u128,
                0x7fffff52ad78032ffffffffffffdbe0u128
            ),
            const_felt252_expr!(0, 1u128 << (251 - 128)),
        ],
    );
    assert_eq!(output.calc(), const_felt252_expr!(0, 1u128 << (251 - 128)).calc());
    let (_, output) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(
                0x4cc3ffffffffff5cdf8005u128,
                0x7fffff52ad78054ffffffffffffdbe0u128
            ),
            const_felt252_expr!(
                0xffffffff_ffffffff_ffffffff_ffffffffu128,
                0x07ffffff_ffffffff_ffffffff_ffffffffu128
            ),
        ],
    );
    assert_eq!(
        output.calc(),
        const_felt252_expr!(
            0xffffffff_ffffffff_ffffffff_ffffffffu128,
            0x07ffffff_ffffffff_ffffffff_ffffffffu128
        )
        .calc()
    );
    let (_, output) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(
                0x4d5e6f8091adf6392ea61d94f496c460u128,
                0x0369d0350642c1926d3a06d3a06d34bau128
            ),
            const_felt252_expr!(
                0xffffffff_ffffffff_ffffffff_ffffffffu128,
                0x07ffffff_ffffffff_ffffffff_ffffffffu128
            ),
        ],
    );
    assert_eq!(
        output.calc(),
        const_felt252_expr!(
            0x01234567_89abcdef_fedcba98_76543210u128,
            0x01234567_89abcdef_fedcba98_76543210u128
        )
        .calc()
    );
}

#[test]
#[should_panic(expected = "Division by zero")]
fn test_div252_by_zero() {
    let air_fn = Div252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (..) = registry.run_air(
        &air_fn,
        (),
        [const_felt252_expr!(0x2008020003400040001u128, 0u128), const_felt252_expr!(0u128, 0u128)],
    );
}

#[test]
#[should_panic(expected = "Division by zero")]
fn test_div252_by_p() {
    let air_fn = Div252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (..) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(0x2008020003400040001u128, 0u128),
            const_felt252_expr!(1u128, 0x08000000_00000011_00000000_00000000u128),
        ],
    );
}

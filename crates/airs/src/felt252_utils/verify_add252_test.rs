use air_infra::const_felt252_expr;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use expect_test::expect;

use super::verify_add252::*;

#[test]
fn test_verify_add252_no_overflow() {
    let air_fn = VerifyAdd252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (state, _) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(0x1008020001u128, 0u128),
            const_felt252_expr!(0x1ff8020001u128, 0u128),
            const_felt252_expr!(0x3000040002u128, 0u128),
        ],
    );
    expect![[r#"
        (0, "sub_p_bit"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_verify_add252_with_overflow() {
    let air_fn = VerifyAdd252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (state, _) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(0, 1u128 << (251 - 128)),
            const_felt252_expr!(0, 1u128 << (251 - 128)),
            const_felt252_expr!(
                0xffffffffffffffffffffffffffffffffu128,
                0x7ffffffffffffeeffffffffffffffffu128
            ),
        ],
    );
    expect![[r#"
        (1, "sub_p_bit"),
    "#]]
    .assert_eq(&state.to_string());
}

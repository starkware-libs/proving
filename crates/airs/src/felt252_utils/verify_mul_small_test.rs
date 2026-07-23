use air_infra::const_felt252_expr;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use expect_test::expect;

use super::verify_mul_small::*;

#[test]
fn test_verify_mul_small_simple() {
    let air_fn = VerifyMulSmall {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (state, _) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(0x100802001u128, 0u128),
            const_felt252_expr!(0x1ff802001u128, 0u128),
            const_felt252_expr!(0x20080200304004001u128, 0u128),
        ],
    );
    expect![[r#"
        (0, "carry_1"),
        (16, "carry_3"),
        (34, "carry_5"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_verify_mul_small_edge() {
    let air_fn = VerifyMulSmall {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (state, _) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(0xfffffffffu128, 0u128),
            const_felt252_expr!(0xfffffffffu128, 0u128),
            const_felt252_expr!(0xffffffffe000000001u128, 0u128),
        ],
    );
    expect![[r#"
        (1021, "carry_1"),
        (2043, "carry_3"),
        (1022, "carry_5"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evaluate to 0)")]
fn test_verify_mul_small_not_equal() {
    let air_fn = VerifyMulSmall {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (..) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(0xfff123fffu128, 0u128),
            const_felt252_expr!(0x456fff789u128, 0u128),
            const_felt252_expr!(0x466bf7ac5385844877u128, 0u128),
        ],
    );
}

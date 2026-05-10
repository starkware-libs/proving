use air_infra::const_felt252_expr;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::variables::AsProverType;
use expect_test::expect;

use super::linear_combination::*;

#[test]
fn test_linear_combination_for_x() {
    let air_fn = LinearCombination::new([-3, 1, 1, 1]);
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(
                0xffffffffffffffffffffffffffffffffu128,
                0x6ffffffffffffffffffffffffffffffu128
            )
            .into(),
            const_felt252_expr!(1u128, 0u128).into(),
            const_felt252_expr!(2u128, 0u128).into(),
            const_felt252_expr!(3u128, 0u128).into(),
        ],
    );
    assert_eq!(
        output.calc(),
        const_felt252_expr!(0xcu128, 0x3000000000000330000000000000000u128).calc()
    );
    expect![[r#"
        (12, "combination_limb_0"),
        (0, "combination_limb_1"),
        (0, "combination_limb_2"),
        (0, "combination_limb_3"),
        (0, "combination_limb_4"),
        (0, "combination_limb_5"),
        (0, "combination_limb_6"),
        (408, "combination_limb_7"),
        (0, "combination_limb_8"),
        (96, "combination_limb_9"),
        (2147483644, "p_coef"),
    "#]]
    .assert_eq(&state.to_string());

    let (state, output) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(1u128, 0u128).into(),
            const_felt252_expr!(
                0xffffffffffffffffffffffffffffffffu128,
                0x6ffffffffffffffffffffffffffffffu128
            )
            .into(),
            const_felt252_expr!(
                0xffffffffffffffffffffffffffffffffu128,
                0x6ffffffffffffffffffffffffffffffu128
            )
            .into(),
            const_felt252_expr!(
                0xffffffffffffffffffffffffffffffffu128,
                0x6ffffffffffffffffffffffffffffffu128
            )
            .into(),
        ],
    );
    assert_eq!(
        output.calc(),
        const_felt252_expr!(
            0xfffffffffffffffffffffffffffffff8u128,
            0x4ffffffffffffddffffffffffffffffu128
        )
        .calc()
    );
    expect![[r#"
        (134217720, "combination_limb_0"),
        (134217727, "combination_limb_1"),
        (134217727, "combination_limb_2"),
        (134217727, "combination_limb_3"),
        (134217727, "combination_limb_4"),
        (134217727, "combination_limb_5"),
        (134217727, "combination_limb_6"),
        (134217455, "combination_limb_7"),
        (134217727, "combination_limb_8"),
        (159, "combination_limb_9"),
        (2, "p_coef"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_linear_combination_for_part_round() {
    let air_fn = LinearCombination::new([2, 1, 4, 3, -1]);
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(2u128, 0u128).into(),
            const_felt252_expr!(1u128, 0u128).into(),
            const_felt252_expr!(4u128, 0u128).into(),
            const_felt252_expr!(3u128, 0u128).into(),
            const_felt252_expr!(
                0xffffffffffffffffffffffffffffffffu128,
                0x6ffffffffffffffffffffffffffffffu128
            )
            .into(),
        ],
    );
    assert_eq!(
        output.calc(),
        const_felt252_expr!(0x20u128, 0x1000000000000110000000000000000u128).calc()
    );
    expect![[r#"
        (32, "combination_limb_0"),
        (0, "combination_limb_1"),
        (0, "combination_limb_2"),
        (0, "combination_limb_3"),
        (0, "combination_limb_4"),
        (0, "combination_limb_5"),
        (0, "combination_limb_6"),
        (136, "combination_limb_7"),
        (0, "combination_limb_8"),
        (32, "combination_limb_9"),
        (2147483646, "p_coef"),
    "#]]
    .assert_eq(&state.to_string());

    let (state, output) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(
                0xffffffffffffffffffffffffffffffffu128,
                0x6ffffffffffffffffffffffffffffffu128
            )
            .into(),
            const_felt252_expr!(
                0xffffffffffffffffffffffffffffffffu128,
                0x6ffffffffffffffffffffffffffffffu128
            )
            .into(),
            const_felt252_expr!(
                0xffffffffffffffffffffffffffffffffu128,
                0x6ffffffffffffffffffffffffffffffu128
            )
            .into(),
            const_felt252_expr!(
                0xffffffffffffffffffffffffffffffffu128,
                0x6ffffffffffffffffffffffffffffffu128
            )
            .into(),
            const_felt252_expr!(1u128, 0u128).into(),
        ],
    );
    assert_eq!(
        output.calc(),
        const_felt252_expr!(
            0xffffffffffffffffffffffffffffffedu128,
            0x5ffffffffffff77ffffffffffffffffu128
        )
        .calc()
    );
    expect![[r#"
        (134217709, "combination_limb_0"),
        (134217727, "combination_limb_1"),
        (134217727, "combination_limb_2"),
        (134217727, "combination_limb_3"),
        (134217727, "combination_limb_4"),
        (134217727, "combination_limb_5"),
        (134217727, "combination_limb_6"),
        (134216639, "combination_limb_7"),
        (134217727, "combination_limb_8"),
        (191, "combination_limb_9"),
        (8, "p_coef"),
    "#]]
    .assert_eq(&state.to_string());
}

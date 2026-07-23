use air_infra::const_felt252_expr;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::variables::AsProverType;
use expect_test::expect;

use super::cube252::*;

#[test]
fn test_cube252_no_overflow() {
    let air_fn = Cube252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (state, output) =
        registry.run_air(&air_fn, (), const_felt252_expr!(0x1008020001u128, 0u128).into());
    assert_eq!(
        output.calc(),
        const_felt252_expr!(0x1018120805436188603c18060001u128, 0u128).calc()
    );
    expect![[r#"
        (1, "enabler"),
        (131073, "input_limb_0"),
        (513, "input_limb_1"),
        (0, "input_limb_2"),
        (0, "input_limb_3"),
        (0, "input_limb_4"),
        (0, "input_limb_5"),
        (0, "input_limb_6"),
        (0, "input_limb_7"),
        (0, "input_limb_8"),
        (0, "input_limb_9"),
        (1, "unpacked_limb_0"),
        (256, "unpacked_limb_1"),
        (1, "unpacked_limb_3"),
        (1, "unpacked_limb_4"),
        (0, "unpacked_limb_6"),
        (0, "unpacked_limb_7"),
        (0, "unpacked_limb_9"),
        (0, "unpacked_limb_10"),
        (0, "unpacked_limb_12"),
        (0, "unpacked_limb_13"),
        (0, "unpacked_limb_15"),
        (0, "unpacked_limb_16"),
        (0, "unpacked_limb_18"),
        (0, "unpacked_limb_19"),
        (0, "unpacked_limb_21"),
        (0, "unpacked_limb_22"),
        (0, "unpacked_limb_24"),
        (0, "unpacked_limb_25"),
        (1, "mul_res_limb_0"),
        (0, "mul_res_limb_1"),
        (1, "mul_res_limb_2"),
        (130, "mul_res_limb_3"),
        (2, "mul_res_limb_4"),
        (1, "mul_res_limb_5"),
        (2, "mul_res_limb_6"),
        (2, "mul_res_limb_7"),
        (1, "mul_res_limb_8"),
        (0, "mul_res_limb_9"),
        (0, "mul_res_limb_10"),
        (0, "mul_res_limb_11"),
        (0, "mul_res_limb_12"),
        (0, "mul_res_limb_13"),
        (0, "mul_res_limb_14"),
        (0, "mul_res_limb_15"),
        (0, "mul_res_limb_16"),
        (0, "mul_res_limb_17"),
        (0, "mul_res_limb_18"),
        (0, "mul_res_limb_19"),
        (0, "mul_res_limb_20"),
        (0, "mul_res_limb_21"),
        (0, "mul_res_limb_22"),
        (0, "mul_res_limb_23"),
        (0, "mul_res_limb_24"),
        (0, "mul_res_limb_25"),
        (0, "mul_res_limb_26"),
        (0, "mul_res_limb_27"),
        (0, "k"),
        (0, "carry_0"),
        (32, "carry_1"),
        (4097, "carry_2"),
        (128, "carry_3"),
        (32, "carry_4"),
        (33, "carry_5"),
        (1, "carry_6"),
        (0, "carry_7"),
        (2, "carry_8"),
        (256, "carry_9"),
        (0, "carry_10"),
        (2, "carry_11"),
        (2, "carry_12"),
        (0, "carry_13"),
        (0, "carry_14"),
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
        (1, "mul_res_limb_0"),
        (256, "mul_res_limb_1"),
        (1, "mul_res_limb_2"),
        (387, "mul_res_limb_3"),
        (3, "mul_res_limb_4"),
        (67, "mul_res_limb_5"),
        (390, "mul_res_limb_6"),
        (134, "mul_res_limb_7"),
        (5, "mul_res_limb_8"),
        (260, "mul_res_limb_9"),
        (4, "mul_res_limb_10"),
        (3, "mul_res_limb_11"),
        (1, "mul_res_limb_12"),
        (0, "mul_res_limb_13"),
        (0, "mul_res_limb_14"),
        (0, "mul_res_limb_15"),
        (0, "mul_res_limb_16"),
        (0, "mul_res_limb_17"),
        (0, "mul_res_limb_18"),
        (0, "mul_res_limb_19"),
        (0, "mul_res_limb_20"),
        (0, "mul_res_limb_21"),
        (0, "mul_res_limb_22"),
        (0, "mul_res_limb_23"),
        (0, "mul_res_limb_24"),
        (0, "mul_res_limb_25"),
        (0, "mul_res_limb_26"),
        (0, "mul_res_limb_27"),
        (0, "k"),
        (0, "carry_0"),
        (0, "carry_1"),
        (0, "carry_2"),
        (0, "carry_3"),
        (2080, "carry_4"),
        (97, "carry_5"),
        (1, "carry_6"),
        (32, "carry_7"),
        (33, "carry_8"),
        (1, "carry_9"),
        (0, "carry_10"),
        (130, "carry_11"),
        (2, "carry_12"),
        (0, "carry_13"),
        (2, "carry_14"),
        (2, "carry_15"),
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
fn test_cube252_with_overflows() {
    let air_fn = Cube252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (_, output) =
        registry.run_air(&air_fn, (), const_felt252_expr!(0, 1u128 << (251 - 128)).into());
    assert_eq!(
        output.calc(),
        const_felt252_expr!(
            0x30ea8ae0ccefff980d98e2efa280dcu128,
            0xe7271cbef30e79ffffe8fb09f06c60u128
        )
        .calc()
    );

    let (_, output) = registry.run_air(
        &air_fn,
        (),
        const_felt252_expr!(
            0xffffffff_ffffffff_ffffffff_ffffffffu128,
            0x07ffffff_ffffffff_ffffffff_ffffffffu128
        )
        .into(),
    );
    assert_eq!(
        output.calc(),
        const_felt252_expr!(
            0x30ea8adfe6a3ff980d98e4d90400d5u128,
            0xe7273d3e6c8de0ffffe8fb09f0d8c0u128
        )
        .calc()
    );

    let (_, output) = registry.run_air(
        &air_fn,
        (),
        const_felt252_expr!(
            0x01234567_89abcdef_fedcba98_76543210u128,
            0x01234567_89abcdef_fedcba98_76543210u128
        )
        .into(),
    );
    assert_eq!(
        output.calc(),
        const_felt252_expr!(
            0x7f0d24086b967fd21d60203b176451d4u128,
            0x552b02c038f04d62a878b4b879c017du128
        )
        .calc()
    );

    let (_, output) = registry.run_air(
        &air_fn,
        (),
        const_felt252_expr!(0xffffffffffffffffffffffffffffffu128, 0u128).into(),
    );
    assert_eq!(
        output.calc(),
        const_felt252_expr!(
            0x2ffe000000000000043ffffffffffffu128,
            0x4810000000000000000000000000000u128
        )
        .calc()
    );
}

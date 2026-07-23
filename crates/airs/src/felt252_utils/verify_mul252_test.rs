use air_infra::const_felt252_expr;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use expect_test::expect;

use super::verify_mul252::*;

#[test]
fn test_verify_mul252_no_overflow() {
    let air_fn = VerifyMul252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (state, _) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(0x1008020001u128, 0u128),
            const_felt252_expr!(0x1ff8020001u128, 0u128),
            const_felt252_expr!(0x2008020003400040001u128, 0u128),
        ],
    );
    expect![[r#"
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
fn test_verify_mul252_with_overflow() {
    let air_fn = VerifyMul252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (state, _) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(0, 1u128 << (251 - 128)),
            const_felt252_expr!(0, 1u128 << (251 - 128)),
            const_felt252_expr!(
                0x4cc3ffffffffff5cdf8002u128,
                0x7fffff52ad78032ffffffffffffdbe0u128
            ),
        ],
    );

    expect![[r#"
        (540, "k"),
        (2, "carry_0"),
        (2147483619, "carry_1"),
        (2147483630, "carry_2"),
        (2147483618, "carry_3"),
        (2147483618, "carry_4"),
        (995, "carry_5"),
        (2147483618, "carry_6"),
        (2147483614, "carry_7"),
        (2147483632, "carry_8"),
        (2147483643, "carry_9"),
        (2147483645, "carry_10"),
        (2147483645, "carry_11"),
        (2147483645, "carry_12"),
        (2147483645, "carry_13"),
        (2147483621, "carry_14"),
        (2147483618, "carry_15"),
        (2147483614, "carry_16"),
        (2147483614, "carry_17"),
        (2147483614, "carry_18"),
        (2147483614, "carry_19"),
        (2147483614, "carry_20"),
        (2147483501, "carry_21"),
        (2147483645, "carry_22"),
        (2147483645, "carry_23"),
        (2147483645, "carry_24"),
        (2147483645, "carry_25"),
        (8190, "carry_26"),
    "#]]
    .assert_eq(&state.to_string());

    let (state, _) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(
                0xffffffff_ffffffff_ffffffff_ffffffffu128,
                0x07ffffff_ffffffff_ffffffff_ffffffffu128
            ),
            const_felt252_expr!(
                0xffffffff_ffffffff_ffffffff_ffffffffu128,
                0x07ffffff_ffffffff_ffffffff_ffffffffu128
            ),
            const_felt252_expr!(
                0x4cc3ffffffffff5cdf8005u128,
                0x7fffff52ad78054ffffffffffffdbe0u128
            ),
        ],
    );

    expect![[r#"
        (18932, "k"),
        (2147475443, "carry_0"),
        (2498, "carry_1"),
        (13240, "carry_2"),
        (23959, "carry_3"),
        (34690, "carry_4"),
        (46445, "carry_5"),
        (62284, "carry_6"),
        (82203, "carry_7"),
        (102150, "carry_8"),
        (122090, "carry_9"),
        (142021, "carry_10"),
        (161950, "carry_11"),
        (181879, "carry_12"),
        (201808, "carry_13"),
        (221713, "carry_14"),
        (241639, "carry_15"),
        (261564, "carry_16"),
        (281493, "carry_17"),
        (301422, "carry_18"),
        (321351, "carry_19"),
        (341280, "carry_20"),
        (160306, "carry_21"),
        (129790, "carry_22"),
        (99130, "carry_23"),
        (68470, "carry_24"),
        (37810, "carry_25"),
        (15342, "carry_26"),
    "#]]
    .assert_eq(&state.to_string());

    let (state, _) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(
                0x01234567_89abcdef_fedcba98_76543210u128,
                0x01234567_89abcdef_fedcba98_76543210u128
            ),
            const_felt252_expr!(
                0xffffffff_ffffffff_ffffffff_ffffffffu128,
                0x07ffffff_ffffffff_ffffffff_ffffffffu128
            ),
            const_felt252_expr!(
                0x4d5e6f8091adf6392ea61d94f496c460u128,
                0x0369d0350642c1926d3a06d3a06d34bau128
            ),
        ],
    );

    expect![[r#"
        (7240, "k"),
        (2147472679, "carry_0"),
        (2147469618, "carry_1"),
        (2147478266, "carry_2"),
        (2147483469, "carry_3"),
        (11390, "carry_4"),
        (17350, "carry_5"),
        (33535, "carry_6"),
        (49488, "carry_7"),
        (65605, "carry_8"),
        (75072, "carry_9"),
        (92636, "carry_10"),
        (100768, "carry_11"),
        (104544, "carry_12"),
        (107895, "carry_13"),
        (112780, "carry_14"),
        (118318, "carry_15"),
        (122926, "carry_16"),
        (127148, "carry_17"),
        (133306, "carry_18"),
        (146715, "carry_19"),
        (163065, "carry_20"),
        (82686, "carry_21"),
        (61648, "carry_22"),
        (37898, "carry_23"),
        (19428, "carry_24"),
        (7904, "carry_25"),
        (2290, "carry_26"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_verify_mul252_with_overflow_negative_k() {
    let air_fn = VerifyMul252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (state, _) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(0, 1u128 << 88),
            const_felt252_expr!(0, 1u128 << 88),
            const_felt252_expr!(
                0x43ffffffffffff6f8000000000013310u128,
                0x14640fffe0000000000000u128
            ),
        ],
    );

    expect![[r#"
        (2147483643, "k"),
        (2147483631, "carry_0"),
        (2147483640, "carry_1"),
        (0, "carry_2"),
        (0, "carry_3"),
        (0, "carry_4"),
        (0, "carry_5"),
        (0, "carry_6"),
        (2147483632, "carry_7"),
        (2147483614, "carry_8"),
        (2147483614, "carry_9"),
        (2147483614, "carry_10"),
        (2147483614, "carry_11"),
        (2147483614, "carry_12"),
        (2147483644, "carry_13"),
        (2147483646, "carry_14"),
        (2147483645, "carry_15"),
        (2147483645, "carry_16"),
        (2147483645, "carry_17"),
        (2147483645, "carry_18"),
        (2147483645, "carry_19"),
        (2147483615, "carry_20"),
        (0, "carry_21"),
        (0, "carry_22"),
        (0, "carry_23"),
        (0, "carry_24"),
        (0, "carry_25"),
        (0, "carry_26"),
    "#]]
    .assert_eq(&state.to_string());

    let (state, _) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(0, 0x1ffu128 << 88),
            const_felt252_expr!(0, 0x1ffu128 << 88),
            const_felt252_expr!(
                0x43fffffffdc0416f80000004c774f310u128,
                0x513ec423907fe0000000010ef0u128
            ),
        ],
    );

    expect![[r#"
        (2147479563, "k"),
        (2147483639, "carry_0"),
        (2147483640, "carry_1"),
        (2147483620, "carry_2"),
        (2147483639, "carry_3"),
        (0, "carry_4"),
        (0, "carry_5"),
        (0, "carry_6"),
        (2147483632, "carry_7"),
        (2147483642, "carry_8"),
        (2147483631, "carry_9"),
        (2147483614, "carry_10"),
        (2147483614, "carry_11"),
        (2147483614, "carry_12"),
        (2147483644, "carry_13"),
        (2147483618, "carry_14"),
        (2147483644, "carry_15"),
        (2147483646, "carry_16"),
        (2147483645, "carry_17"),
        (2147483645, "carry_18"),
        (2147483645, "carry_19"),
        (2147483615, "carry_20"),
        (1082, "carry_21"),
        (2, "carry_22"),
        (0, "carry_23"),
        (0, "carry_24"),
        (0, "carry_25"),
        (0, "carry_26"),
    "#]]
    .assert_eq(&state.to_string());

    let (state, _) = registry.run_air(
        &air_fn,
        (),
        [
            const_felt252_expr!(0, 0x7fffffffffffffffu128 << 61),
            const_felt252_expr!(0, 0x7fffffffffffffffu128 << 61),
            const_felt252_expr!(
                0x800000000135530ffffffffd6eaf7e05u128,
                0x7ffffd459a75e997fffffffffff6e6fu128
            ),
        ],
    );

    expect![[r#"
        (2147457211, "k"),
        (24533, "carry_0"),
        (20423, "carry_1"),
        (16335, "carry_2"),
        (12245, "carry_3"),
        (8155, "carry_4"),
        (4067, "carry_5"),
        (2147483624, "carry_6"),
        (2147483644, "carry_7"),
        (2147483624, "carry_8"),
        (2147483635, "carry_9"),
        (2147483645, "carry_10"),
        (2147483645, "carry_11"),
        (2147483645, "carry_12"),
        (2147483645, "carry_13"),
        (2147483619, "carry_14"),
        (2147483631, "carry_15"),
        (2147483614, "carry_16"),
        (2147483614, "carry_17"),
        (2147483614, "carry_18"),
        (2147483614, "carry_19"),
        (2147483614, "carry_20"),
        (200820, "carry_21"),
        (165632, "carry_22"),
        (129862, "carry_23"),
        (94092, "carry_24"),
        (58322, "carry_25"),
        (22552, "carry_26"),
    "#]]
    .assert_eq(&state.to_string());
}

use super::felt252_packing27::*;
// Macros
use crate::const_felt252_expr;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

#[test]
fn test_entry_json_unpack() {
    let (_, entry) = AirFnRegistry::new(&Felt252UnpackFrom27 {});
    compare_json(
        &entry,
        &format!("{}{}.json", TEST_JSONS_FELT252_DIR, entry.name),
    );
}

// TODO(DanC): Remove component json tests after the full Poseidon air is added to the registry.
#[test]
fn test_entry_json_rangecheck() {
    let (_, entry) = AirFnRegistry::new(&RangeCheckFelt252Packed27 {});
    compare_json(
        &entry,
        &format!("{}{}.json", TEST_JSONS_FELT252_DIR, entry.name),
    );
}

#[test]
fn test_verify_felt252pack_into27() {
    let unpacked = const_felt252_expr!(
        0x01234567_89abcdef_fedcba98_76543210u128,
        0x01234567_89abcdef_fedcba98_76543210u128
    );
    let output = felt252_pack_into27(unpacked);
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
fn test_verify_felt252unpack_from27() {
    let air_fn = Felt252UnpackFrom27 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (state, output) = registry.run_air(
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
            0x01234567_89abcdef_fedcba98_76543210u128,
            0x01234567_89abcdef_fedcba98_76543210u128
        )
        .calc()
    );
    let expected_state = vec![
        (16, "unpacked_limb_0"),
        (25, "unpacked_limb_1"),
        (270, "unpacked_limb_3"),
        (425, "unpacked_limb_4"),
        (507, "unpacked_limb_6"),
        (479, "unpacked_limb_7"),
        (213, "unpacked_limb_9"),
        (482, "unpacked_limb_10"),
        (52, "unpacked_limb_12"),
        (9, "unpacked_limb_13"),
        (100, "unpacked_limb_15"),
        (84, "unpacked_limb_16"),
        (166, "unpacked_limb_18"),
        (407, "unpacked_limb_19"),
        (383, "unpacked_limb_21"),
        (311, "unpacked_limb_22"),
        (393, "unpacked_limb_24"),
        (179, "unpacked_limb_25"),
    ]
    .into();
    assert_expected_state(&state, &expected_state);
}

#[test]
fn test_verify_felt252packed27_rangecheck() {
    let air_fn = RangeCheckFelt252Packed27 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (state, _) = registry.run_air(
        &air_fn,
        (),
        const_felt252_expr!(
            0x01234567_89abcdef_fedcba98_76543210u128,
            0x01234567_89abcdef_fedcba98_76543210u128
        )
        .into(),
    );
    let expected_state = vec![
        (106181136, "input_limb_0"),
        (60248846, "input_limb_1"),
        (121094139, "input_limb_2"),
        (45335765, "input_limb_3"),
        (16781876, "input_limb_4"),
        (15509604, "input_limb_5"),
        (129445542, "input_limb_6"),
        (90075007, "input_limb_7"),
        (54880137, "input_limb_8"),
        (36, "input_limb_9"),
        (405, "limb_0_high_part"),
        (270, "limb_1_low_part"),
        (461, "limb_2_high_part"),
        (213, "limb_3_low_part"),
        (64, "limb_4_high_part"),
        (100, "limb_5_low_part"),
        (493, "limb_6_high_part"),
        (383, "limb_7_low_part"),
        (209, "limb_8_high_part"),
    ]
    .into();
    assert_expected_state(&state, &expected_state);
}

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
        (405, "unpacked_limb_2"),
        (270, "unpacked_limb_3"),
        (425, "unpacked_limb_4"),
        (229, "unpacked_limb_5"),
        (507, "unpacked_limb_6"),
        (479, "unpacked_limb_7"),
        (461, "unpacked_limb_8"),
        (213, "unpacked_limb_9"),
        (482, "unpacked_limb_10"),
        (172, "unpacked_limb_11"),
        (52, "unpacked_limb_12"),
        (9, "unpacked_limb_13"),
        (64, "unpacked_limb_14"),
        (100, "unpacked_limb_15"),
        (84, "unpacked_limb_16"),
        (59, "unpacked_limb_17"),
        (166, "unpacked_limb_18"),
        (407, "unpacked_limb_19"),
        (493, "unpacked_limb_20"),
        (383, "unpacked_limb_21"),
        (311, "unpacked_limb_22"),
        (343, "unpacked_limb_23"),
        (393, "unpacked_limb_24"),
        (179, "unpacked_limb_25"),
        (209, "unpacked_limb_26"),
    ]
    .into();
    assert_expected_state(&state, &expected_state);
}

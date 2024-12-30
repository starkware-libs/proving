use super::verify_add252::*;
// Macros
use crate::const_felt252_expr;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::utils::test_utils::*;

#[test]
fn test_entry_json() {
    let (_, entry) = AirFnRegistry::new(&VerifyAdd252 {});
    compare_json(
        &entry,
        &format!("{}{}.json", TEST_JSONS_FELT252_DIR, entry.name),
    );
}

#[test]
fn test_verify_add252_no_overflow() {
    let air_fn = VerifyAdd252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (state, _) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(0x1008020001u128, 0u128),
            const_felt252_expr!(0x1ff8020001u128, 0u128),
            const_felt252_expr!(0x3000040002u128, 0u128),
        ],
    );
    let expected_state = vec![(0, "sub_p_bit")].into();
    assert_expected_state(&state, &expected_state);
}

#[test]
fn test_verify_add252_with_overflow() {
    let air_fn = VerifyAdd252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (state, _) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(0, 1u128 << (251 - 128)),
            const_felt252_expr!(0, 1u128 << (251 - 128)),
            const_felt252_expr!(
                0xffffffffffffffffffffffffffffffffu128,
                0x7ffffffffffffeeffffffffffffffffu128
            ),
        ],
    );
    let expected_state = vec![(1, "sub_p_bit")].into();
    assert_expected_state(&state, &expected_state);
}

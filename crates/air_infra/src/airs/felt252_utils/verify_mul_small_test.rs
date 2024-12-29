use super::verify_mul_small::*;
// Macros
use crate::const_felt252_expr;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::utils::test_utils::*;

#[test]
fn test_entry_json() {
    let (_, entry) = AirFnRegistry::new(&VerifyMulSmall {});
    compare_json(
        &entry,
        &format!("{}{}.json", TEST_JSONS_FELT252_DIR, entry.name),
    );
}

#[test]
fn test_verify_mul_small_simple() {
    let air_fn = VerifyMulSmall {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (state, _) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(0x100802001u128, 0u128),
            const_felt252_expr!(0x1ff802001u128, 0u128),
            const_felt252_expr!(0x20080200304004001u128, 0u128),
        ],
    );
    let expected_state = vec![(0, "carry_1"), (16, "carry_3"), (34, "carry_5")].into();
    assert!(
        state == expected_state,
        "State {} does not match {}",
        state,
        expected_state
    );
}

#[test]
fn test_verify_mul_small_edge() {
    let air_fn = VerifyMulSmall {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (state, _) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(0xfffffffffu128, 0u128),
            const_felt252_expr!(0xfffffffffu128, 0u128),
            const_felt252_expr!(0xffffffffe000000001u128, 0u128),
        ],
    );
    let expected_state = vec![(1021, "carry_1"), (2043, "carry_3"), (1022, "carry_5")].into();
    assert!(
        state == expected_state,
        "State {} does not match {}",
        state,
        expected_state
    );
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evalutate to 0)")]
fn test_verify_mul_small_not_equal() {
    let air_fn = VerifyMulSmall {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (state, _) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(0xfff123fffu128, 0u128),
            const_felt252_expr!(0x456fff789u128, 0u128),
            const_felt252_expr!(0x466bf7ac5385844877u128, 0u128),
        ],
    );
    let expected_state = vec![(727, "carry"), (1230, "carry"), (569, "carry")].into();
    assert!(
        state == expected_state,
        "State {} does not match {}",
        state,
        expected_state
    );
}

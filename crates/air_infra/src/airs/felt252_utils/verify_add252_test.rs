use super::verify_add252::*;

use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::utils::test_utils::*;

// Macros
use crate::const_felt252_expr;

#[test]
fn test_verify_add252_air_body() {
    let air_fn = VerifyAdd252 {};
    let registry = AirFnRegistry::new(&air_fn);

    // Check entry
    compare_json(
        &registry.get_air_fn_entry(&air_fn.name()),
        &(TEST_JSONS_FELT252_DIR.to_owned() + "verify_add252.json"),
    );
}

#[test]
fn test_verify_add252_no_overflow() {
    let air_fn = VerifyAdd252 {};
    let registry = AirFnRegistry::new(&air_fn);
    let (state, _) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(0x1008020001u128, 0u128),
            const_felt252_expr!(0x1ff8020001u128, 0u128),
            const_felt252_expr!(0x3000040002u128, 0u128),
        ],
    );
    let expected_state = ["0"];
    assert_eq!(state.calc(), expected_state);
}

#[test]
fn test_verify_add252_with_overflow() {
    let air_fn = VerifyAdd252 {};
    let registry = AirFnRegistry::new(&air_fn);
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
    let expected_state = ["1"];
    assert_eq!(state.calc(), expected_state);
}

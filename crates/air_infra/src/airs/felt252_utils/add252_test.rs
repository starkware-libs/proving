use super::add252::*;

use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

// Macros
use crate::const_felt252_expr;

#[test]
fn test_add252_air_body() {
    let air_fn = Add252 {};
    let registry = AirFnRegistry::new(&air_fn);

    // Check entry
    compare_test_json(
        &registry,
        &air_fn.name(),
        &(TEST_JSONS_FELT252_DIR.to_owned() + "add252.json"),
    );
}

#[test]
fn test_add252_no_overflow() {
    let air_fn = Add252 {};
    let registry = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(0x1008020001u128, 0u128),
            const_felt252_expr!(0x1ff8020001u128, 0u128),
        ],
    );
    assert_eq!(
        output.calc(),
        const_felt252_expr!(0x3000040002u128, 0u128).calc()
    );
    let expected_state = [
        "2", "0", "1", "0", "3", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
        "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
    ];
    assert_eq!(state.calc(), expected_state);
}

#[test]
fn test_add252_with_overflow() {
    let air_fn = Add252 {};
    let registry = AirFnRegistry::new(&air_fn);
    let (state, output) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(0, 1u128 << (251 - 128)),
            const_felt252_expr!(0, 1u128 << (251 - 128)),
        ],
    );
    assert_eq!(
        output.calc(),
        const_felt252_expr!(
            0xffffffffffffffffffffffffffffffffu128,
            0x7ffffffffffffeeffffffffffffffffu128
        )
        .calc()
    );
    let expected_state = [
        "511", "511", "511", "511", "511", "511", "511", "511", "511", "511", "511", "511", "511",
        "511", "511", "511", "511", "511", "511", "511", "511", "375", "511", "511", "511", "511",
        "511", "255", "1",
    ];
    assert_eq!(state.calc(), expected_state);
}

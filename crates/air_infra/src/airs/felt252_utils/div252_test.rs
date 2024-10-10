use super::div252::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

// Macros
use crate::const_felt252_expr;

#[test]
fn test_entry_json() {
    let (_, entry) = AirFnRegistry::new(&Div252 {});
    compare_json(
        &entry,
        &format!(
            "{}{}.json",
            TEST_JSONS_FELT252_DIR,
            entry.name.to_lowercase()
        ),
    );
}

#[test]
fn test_div252_no_overflow() {
    let air_fn = Div252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(0x2008020003400040001u128, 0u128),
            const_felt252_expr!(0x1ff8020001u128, 0u128),
        ],
    );
    assert_eq!(
        output.calc(),
        const_felt252_expr!(0x1008020001u128, 0u128).calc()
    );
    let expected_state = [
        "1", "256", "0", "1", "1", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
        "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "32", "4097", "160", "8193",
        "288", "33", "33", "3", "256", "2", "512", "2", "2", "2", "0", "0", "0", "0", "0", "0",
        "0", "0", "0", "0", "0", "0",
    ];
    assert_eq!(state.calc(), expected_state);
}

#[test]
fn test_div252_with_overflow() {
    let air_fn = Div252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (_, output) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(
                0x4cc3ffffffffff5cdf8002u128,
                0x7fffff52ad78032ffffffffffffdbe0u128
            ),
            const_felt252_expr!(0, 1u128 << (251 - 128)),
        ],
    );
    assert_eq!(
        output.calc(),
        const_felt252_expr!(0, 1u128 << (251 - 128)).calc()
    );
    let (_, output) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(
                0x4cc3ffffffffff5cdf8005u128,
                0x7fffff52ad78054ffffffffffffdbe0u128
            ),
            const_felt252_expr!(
                0xffffffff_ffffffff_ffffffff_ffffffffu128,
                0x07ffffff_ffffffff_ffffffff_ffffffffu128
            ),
        ],
    );
    assert_eq!(
        output.calc(),
        const_felt252_expr!(
            0xffffffff_ffffffff_ffffffff_ffffffffu128,
            0x07ffffff_ffffffff_ffffffff_ffffffffu128
        )
        .calc()
    );
    let (_, output) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(
                0x4d5e6f8091adf6392ea61d94f496c460u128,
                0x0369d0350642c1926d3a06d3a06d34bau128
            ),
            const_felt252_expr!(
                0xffffffff_ffffffff_ffffffff_ffffffffu128,
                0x07ffffff_ffffffff_ffffffff_ffffffffu128
            ),
        ],
    );
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
#[should_panic(expected = "Division by zero")]
fn test_div252_by_zero() {
    let air_fn = Div252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (_, _) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(0x2008020003400040001u128, 0u128),
            const_felt252_expr!(0u128, 0u128),
        ],
    );
}

#[test]
#[should_panic(expected = "Division by zero")]
fn test_div252_by_p() {
    let air_fn = Div252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (_, _) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(0x2008020003400040001u128, 0u128),
            const_felt252_expr!(1u128, 0x08000000_00000011_00000000_00000000u128),
        ],
    );
}

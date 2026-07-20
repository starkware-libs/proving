use air_infra::const_felt252_expr;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt252_expr::Felt252Expr;

use super::verify_reduced252::*;

#[test]
fn test_verify_reduced252_valid_values() {
    let air_fn = VerifyReduced252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);

    // P - 1
    registry.run_air(&air_fn, (), const_felt252_expr!(0, 0x8000000000000110000000000000000));

    // P - 2
    registry.run_air(
        &air_fn,
        (),
        const_felt252_expr!(0xffffffffffffffffffffffffffffffff, 0x800000000000010ffffffffffffffff),
    );

    // 2 ** 251 - 1
    registry.run_air(
        &air_fn,
        (),
        const_felt252_expr!(0xffffffffffffffffffffffffffffffff, 0x7ffffffffffffffffffffffffffffff),
    );

    // Zero
    registry.run_air(&air_fn, (), const_felt252_expr!(0, 0));
}

#[test]
#[should_panic(expected = "Added incorrect constraint")]
fn test_verify_reduced252_p() {
    let air_fn = VerifyReduced252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    registry.run_air(&air_fn, (), const_felt252_expr!(1, 0x8000000000000110000000000000000));
}

#[test]
#[should_panic(expected = "Added incorrect constraint")]
fn test_verify_reduced252_high_limbs() {
    let air_fn = VerifyReduced252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    registry.run_air(&air_fn, (), const_felt252_expr!(0, 0x8000001000000110000000000000000));
}

#[test]
#[should_panic(expected = "RangeCheck8 on input 511")]
fn test_verify_reduced252_max() {
    let air_fn = VerifyReduced252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);

    // 2 ** 252 - 1
    registry.run_air(
        &air_fn,
        (),
        const_felt252_expr!(0xffffffffffffffffffffffffffffffff, 0xfffffffffffffffffffffffffffffff),
    );
}

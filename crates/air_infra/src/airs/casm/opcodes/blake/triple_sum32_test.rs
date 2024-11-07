use super::triple_sum32::*;
// Macros
use crate::const_expr;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;

#[test]
fn test_triple_sum1() {
    let air_fn = TripleSum32 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(
        &air_fn,
        [
            const_expr!(16548),
            const_expr!(41567),
            const_expr!(1547),
            const_expr!(1564),
            const_expr!(234),
            const_expr!(87525),
        ],
    );
    assert_eq!(output[0].calc(), "18329");
    assert_eq!(output[1].calc(), "65120");

    // Check state
    let expected_state = vec![(18329, "add_res_limb_0"), (65120, "add_res_limb_1")].into();
    assert!(
        state == expected_state,
        "State {} does not match {}",
        state,
        expected_state
    );
}

#[test]
fn test_triple_sum2() {
    let triple_sum = TripleSum32 {};
    let air_fn = triple_sum;
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(
        &air_fn,
        [
            const_expr!(1 << 15),
            const_expr!(1 << 15),
            const_expr!(1 << 15),
            const_expr!(1 << 15),
            const_expr!(1 << 15),
            const_expr!(1 << 15),
        ],
    );
    assert_eq!(output[0].calc(), "32768");
    assert_eq!(output[1].calc(), "32769");

    // Check state
    let expected_state = vec![(32768, "add_res_limb_0"), (32769, "add_res_limb_1")].into();
    assert!(
        state == expected_state,
        "State {} does not match {}",
        state,
        expected_state
    );
}

#[test]
fn test_triple_sum3() {
    let triple_sum = TripleSum32 {};
    let air_fn = triple_sum;
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(
        &air_fn,
        [
            const_expr!((1 << 16) - 1),
            const_expr!((1 << 16) - 1),
            const_expr!((1 << 16) - 1),
            const_expr!((1 << 16) - 1),
            const_expr!((1 << 16) - 1),
            const_expr!((1 << 16) - 1),
        ],
    );
    assert_eq!(output[0].calc(), "65533");
    assert_eq!(output[1].calc(), "65535");

    // Check state
    let expected_state = vec![(65533, "add_res_limb_0"), (65535, "add_res_limb_1")].into();
    assert!(
        state == expected_state,
        "State {} does not match {}",
        state,
        expected_state
    );
}

#[test]
fn test_pair_sum() {
    let triple_sum = TripleSum32 {};
    let air_fn = triple_sum;
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(
        &air_fn,
        [
            const_expr!(15446),
            const_expr!(121),
            const_expr!(15102),
            const_expr!(2316),
            const_expr!(0),
            const_expr!(0),
        ],
    );
    assert_eq!(output[0].calc(), "30548");
    assert_eq!(output[1].calc(), "2437");

    // Check state
    let expected_state = vec![(30548, "add_res_limb_0"), (2437, "add_res_limb_1")].into();
    assert!(
        state == expected_state,
        "State {} does not match {}",
        state,
        expected_state
    );
}

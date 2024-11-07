use super::triple_sum32::*;
// Macros
use crate::const_u32_expr;
use crate::core::air_fn_registry::*;
use crate::core::expressions::uint32_expr::*;
use crate::core::variables::*;

#[test]
fn test_triple_sum1() {
    let air_fn = TripleSum32 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(
        &air_fn,
        [
            const_u32_expr!(2724151460),
            const_u32_expr!(102499851),
            const_u32_expr!(1441071338),
        ],
    );
    assert_eq!(output.calc(), "4267722649");

    // Check state
    let expected_state = vec![
        (18329, "triple_sum32_res_low"),
        (65120, "triple_sum32_res_high"),
    ]
    .into();
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
            const_u32_expr!(2147516416),
            const_u32_expr!(2147516416),
            const_u32_expr!(2147516416),
        ],
    );
    assert_eq!(output.calc(), "2147581952");

    // Check state
    let expected_state = vec![
        (32768, "triple_sum32_res_low"),
        (32769, "triple_sum32_res_high"),
    ]
    .into();
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
            const_u32_expr!(4294967295),
            const_u32_expr!(4294967295),
            const_u32_expr!(4294967295),
        ],
    );
    assert_eq!(output.calc(), "4294967293");

    // Check state
    let expected_state = vec![
        (65533, "triple_sum32_res_low"),
        (65535, "triple_sum32_res_high"),
    ]
    .into();
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
            const_u32_expr!(7945302),
            const_u32_expr!(151796478),
            const_u32_expr!(0),
        ],
    );
    assert_eq!(output.calc(), "159741780");

    // Check state
    let expected_state = vec![
        (30548, "triple_sum32_res_low"),
        (2437, "triple_sum32_res_high"),
    ]
    .into();
    assert!(
        state == expected_state,
        "State {} does not match {}",
        state,
        expected_state
    );
}

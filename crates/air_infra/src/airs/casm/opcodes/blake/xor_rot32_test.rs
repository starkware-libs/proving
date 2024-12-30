use super::xor_rot32::*;
// Macros
use crate::const_u32_expr;
use crate::core::air_fn_registry::*;
use crate::core::expressions::uint32_expr::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;
#[test]
fn test_xor_rot7() {
    let xor_rot = XorRot32 { r: 7 };
    let (registry, _) = AirFnRegistry::new(&xor_rot);
    let (state, new_state) = registry.run_air(
        &xor_rot,
        [const_u32_expr!(95330889), const_u32_expr!(1741830375)],
    );
    assert_eq!(new_state.calc(), "1556412725");

    // Check state
    let expected_state = vec![
        (324, "a_low_9_ms_bits"),
        (11, "a_high_9_ms_bits"),
        (113, "b_low_9_ms_bits"),
        (207, "b_high_9_ms_bits"),
        (46, "xor"),
        (309, "xor"),
        (124, "xor"),
        (196, "xor"),
    ]
    .into();
    assert_expected_state(&state, &expected_state);
}

#[test]
fn test_xor_rot12() {
    let xor_rot = XorRot32 { r: 12 };
    let (registry, _) = AirFnRegistry::new(&xor_rot);
    let (state, new_state) = registry.run_air(
        &xor_rot,
        [const_u32_expr!(9510598), const_u32_expr!(8063093)],
    );
    assert_eq!(new_state.calc(), "1798311585");

    // Check state
    let expected_state = vec![
        (1, "a_low_4_ms_bits"),
        (0, "a_high_4_ms_bits"),
        (0, "b_low_4_ms_bits"),
        (0, "b_high_4_ms_bits"),
        (1715, "xor"),
        (1, "xor"),
        (234, "xor"),
        (0, "xor"),
    ]
    .into();
    assert_expected_state(&state, &expected_state);
}

#[test]
fn test_xor_rot8() {
    let xor_rot = XorRot32 { r: 8 };
    let (registry, _) = AirFnRegistry::new(&xor_rot);
    let (state, new_state) = registry.run_air(
        &xor_rot,
        [const_u32_expr!(2147516416), const_u32_expr!(33558528)],
    );
    assert_eq!(new_state.calc(), "8519824");

    // Check state
    let expected_state = vec![
        (128, "a_low_8_ms_bits"),
        (128, "a_high_8_ms_bits"),
        (16, "b_low_8_ms_bits"),
        (2, "b_high_8_ms_bits"),
        (0, "xor"),
        (144, "xor"),
        (0, "xor"),
        (130, "xor"),
    ]
    .into();
    assert_expected_state(&state, &expected_state);
}

#[test]
fn test_xor_rot16() {
    let xor_rot = XorRot32 { r: 16 };
    let (registry, _) = AirFnRegistry::new(&xor_rot);
    let (state, new_state) = registry.run_air(
        &xor_rot,
        [const_u32_expr!(3198041206), const_u32_expr!(423952538)],
    );
    assert_eq!(new_state.calc(), "1022142427");

    // Check state
    let expected_state = vec![
        (60, "a_low_8_ms_bits"),
        (190, "a_high_8_ms_bits"),
        (0, "b_low_8_ms_bits"),
        (25, "b_high_8_ms_bits"),
        (236, "xor"),
        (60, "xor"),
        (219, "xor"),
        (167, "xor"),
    ]
    .into();
    assert_expected_state(&state, &expected_state);
}

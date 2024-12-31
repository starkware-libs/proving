use super::g::*;
// Macros
use crate::const_u32_expr;
use crate::core::air_fn_registry::*;
use crate::core::expressions::uint32_expr::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

#[test]
fn test_g1() {
    let air_fn = G {};
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(
        &air_fn,
        (),
        [
            const_u32_expr!(305419896),
            const_u32_expr!(4294967295),
            const_u32_expr!(2147483647),
            const_u32_expr!(123456789),
            const_u32_expr!(987654321),
            const_u32_expr!(468798),
        ],
    );
    assert_eq!(output[0].calc(), "2827666065");
    assert_eq!(output[1].calc(), "4146123195");
    assert_eq!(output[2].calc(), "3407348176");
    assert_eq!(output[3].calc(), "3638212488");

    // Check state
    let expected_state = vec![
        (22136, "input"),
        (4660, "input"),
        (65535, "input"),
        (65535, "input"),
        (65535, "input"),
        (32767, "input"),
        (52501, "input"),
        (1883, "input"),
        (26801, "input"),
        (15070, "input"),
        (10046, "input"),
        (7, "input"),
        (48936, "triple_sum32_res_low"),
        (19730, "triple_sum32_res_high"),
        (191, "a_low_8_ms_bits"),
        (77, "a_high_8_ms_bits"),
        (205, "b_low_8_ms_bits"),
        (7, "b_high_8_ms_bits"),
        (61, "xor"),
        (114, "xor"),
        (73, "xor"),
        (74, "xor"),
        (19016, "triple_sum32_res_low"),
        (62013, "triple_sum32_res_high"),
        (15, "a_low_4_ms_bits"),
        (15, "a_high_4_ms_bits"),
        (4, "b_low_4_ms_bits"),
        (15, "b_high_4_ms_bits"),
        (1463, "xor"),
        (11, "xor"),
        (3522, "xor"),
        (0, "xor"),
        (49809, "triple_sum32_res_low"),
        (43146, "triple_sum32_res_high"),
        (194, "a_low_8_ms_bits"),
        (168, "a_high_8_ms_bits"),
        (74, "b_low_8_ms_bits"),
        (114, "b_high_8_ms_bits"),
        (216, "xor"),
        (136, "xor"),
        (183, "xor"),
        (218, "xor"),
        (464, "triple_sum32_res_low"),
        (51992, "triple_sum32_res_high"),
        (440, "a_low_9_ms_bits"),
        (182, "a_high_9_ms_bits"),
        (3, "b_low_9_ms_bits"),
        (406, "b_high_9_ms_bits"),
        (123, "xor"),
        (443, "xor"),
        (104, "xor"),
        (288, "xor"),
    ]
    .into();
    assert_expected_state(&state, &expected_state);
}

#[test]
fn test_g2() {
    let air_fn = G {};
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(
        &air_fn,
        (),
        [
            const_u32_expr!(3694142613),
            const_u32_expr!(170668591),
            const_u32_expr!(2859583592),
            const_u32_expr!(2750542364),
            const_u32_expr!(101488500),
            const_u32_expr!(3940201164),
        ],
    );
    assert_eq!(output[0].calc(), "2049993894");
    assert_eq!(output[1].calc(), "223224271");
    assert_eq!(output[2].calc(), "100412452");
    assert_eq!(output[3].calc(), "1063654435");

    // Check state
    let expected_state = vec![
        (9365, "input"),
        (56368, "input"),
        (12847, "input"),
        (2604, "input"),
        (51304, "input"),
        (43633, "input"),
        (61980, "input"),
        (41969, "input"),
        (38772, "input"),
        (1548, "input"),
        (45772, "input"),
        (60122, "input"),
        (60984, "triple_sum32_res_low"),
        (60520, "triple_sum32_res_high"),
        (238, "a_low_8_ms_bits"),
        (236, "a_high_8_ms_bits"),
        (242, "b_low_8_ms_bits"),
        (163, "b_high_8_ms_bits"),
        (36, "xor"),
        (28, "xor"),
        (153, "xor"),
        (79, "xor"),
        (6145, "triple_sum32_res_low"),
        (50838, "triple_sum32_res_high"),
        (3, "a_low_4_ms_bits"),
        (0, "a_high_4_ms_bits"),
        (1, "b_low_4_ms_bits"),
        (12, "b_high_4_ms_bits"),
        (2606, "xor"),
        (2, "xor"),
        (3258, "xor"),
        (12, "xor"),
        (27814, "triple_sum32_res_low"),
        (31280, "triple_sum32_res_high"),
        (108, "a_low_8_ms_bits"),
        (122, "a_high_8_ms_bits"),
        (79, "b_low_8_ms_bits"),
        (28, "b_high_8_ms_bits"),
        (63, "xor"),
        (35, "xor"),
        (20, "xor"),
        (102, "xor"),
        (11300, "triple_sum32_res_low"),
        (1532, "triple_sum32_res_high"),
        (407, "a_low_9_ms_bits"),
        (325, "a_high_9_ms_bits"),
        (88, "b_low_9_ms_bits"),
        (11, "b_high_9_ms_bits"),
        (6, "xor"),
        (463, "xor"),
        (16, "xor"),
        (334, "xor"),
    ]
    .into();
    assert_expected_state(&state, &expected_state);
}

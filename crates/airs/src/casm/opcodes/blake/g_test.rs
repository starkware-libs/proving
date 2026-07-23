use air_infra::const_u32_expr;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::core::variables::AsProverType;
use expect_test::expect;

use super::g::*;

#[test]
fn test_g1() {
    let air_fn = BlakeG {};
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
    expect![[r#"
        (1, "enabler"),
        (22136, "input_limb_0"),
        (4660, "input_limb_1"),
        (65535, "input_limb_2"),
        (65535, "input_limb_3"),
        (65535, "input_limb_4"),
        (32767, "input_limb_5"),
        (52501, "input_limb_6"),
        (1883, "input_limb_7"),
        (26801, "input_limb_8"),
        (15070, "input_limb_9"),
        (10046, "input_limb_10"),
        (7, "input_limb_11"),
        (48936, "triple_sum32_res_limb_0"),
        (19730, "triple_sum32_res_limb_1"),
        (191, "ms_8_bits"),
        (77, "ms_8_bits"),
        (205, "ms_8_bits"),
        (7, "ms_8_bits"),
        (61, "xor"),
        (114, "xor"),
        (73, "xor"),
        (74, "xor"),
        (19016, "triple_sum32_res_limb_0"),
        (62013, "triple_sum32_res_limb_1"),
        (15, "ms_4_bits"),
        (15, "ms_4_bits"),
        (4, "ms_4_bits"),
        (15, "ms_4_bits"),
        (1463, "xor"),
        (11, "xor"),
        (3522, "xor"),
        (0, "xor"),
        (49809, "triple_sum32_res_limb_0"),
        (43146, "triple_sum32_res_limb_1"),
        (194, "ms_8_bits"),
        (168, "ms_8_bits"),
        (74, "ms_8_bits"),
        (114, "ms_8_bits"),
        (216, "xor"),
        (136, "xor"),
        (183, "xor"),
        (218, "xor"),
        (464, "triple_sum32_res_limb_0"),
        (51992, "triple_sum32_res_limb_1"),
        (440, "ms_9_bits"),
        (182, "ms_9_bits"),
        (3, "ms_9_bits"),
        (406, "ms_9_bits"),
        (123, "xor"),
        (443, "xor"),
        (104, "xor"),
        (288, "xor"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_g2() {
    let air_fn = BlakeG {};
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
    expect![[r#"
        (1, "enabler"),
        (9365, "input_limb_0"),
        (56368, "input_limb_1"),
        (12847, "input_limb_2"),
        (2604, "input_limb_3"),
        (51304, "input_limb_4"),
        (43633, "input_limb_5"),
        (61980, "input_limb_6"),
        (41969, "input_limb_7"),
        (38772, "input_limb_8"),
        (1548, "input_limb_9"),
        (45772, "input_limb_10"),
        (60122, "input_limb_11"),
        (60984, "triple_sum32_res_limb_0"),
        (60520, "triple_sum32_res_limb_1"),
        (238, "ms_8_bits"),
        (236, "ms_8_bits"),
        (242, "ms_8_bits"),
        (163, "ms_8_bits"),
        (36, "xor"),
        (28, "xor"),
        (153, "xor"),
        (79, "xor"),
        (6145, "triple_sum32_res_limb_0"),
        (50838, "triple_sum32_res_limb_1"),
        (3, "ms_4_bits"),
        (0, "ms_4_bits"),
        (1, "ms_4_bits"),
        (12, "ms_4_bits"),
        (2606, "xor"),
        (2, "xor"),
        (3258, "xor"),
        (12, "xor"),
        (27814, "triple_sum32_res_limb_0"),
        (31280, "triple_sum32_res_limb_1"),
        (108, "ms_8_bits"),
        (122, "ms_8_bits"),
        (79, "ms_8_bits"),
        (28, "ms_8_bits"),
        (63, "xor"),
        (35, "xor"),
        (20, "xor"),
        (102, "xor"),
        (11300, "triple_sum32_res_limb_0"),
        (1532, "triple_sum32_res_limb_1"),
        (407, "ms_9_bits"),
        (325, "ms_9_bits"),
        (88, "ms_9_bits"),
        (11, "ms_9_bits"),
        (6, "xor"),
        (463, "xor"),
        (16, "xor"),
        (334, "xor"),
    "#]]
    .assert_eq(&state.to_string());
}

use air_infra::const_u32_expr;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use expect_test::expect;

use super::blake_g_gate::BlakeGGate;

#[test]
fn test_blake_g_gate() {
    let air_fn = BlakeGGate {};
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (state, _) = registry.run_air(
        &air_fn,
        (),
        [
            const_u32_expr!(305419896),
            const_u32_expr!(4294967295),
            const_u32_expr!(2147483647),
            const_u32_expr!(123456789),
            const_u32_expr!(987654321),
            const_u32_expr!(468798),
            const_u32_expr!(2827666065),
            const_u32_expr!(4146123195),
            const_u32_expr!(3407348176),
            const_u32_expr!(3638212488),
        ],
    );

    expect![[r#"
        (22136, "input_a_limb_0"),
        (4660, "input_a_limb_1"),
        (65535, "input_b_limb_0"),
        (65535, "input_b_limb_1"),
        (65535, "input_c_limb_0"),
        (32767, "input_c_limb_1"),
        (52501, "input_d_limb_0"),
        (1883, "input_d_limb_1"),
        (26801, "input_f0_limb_0"),
        (15070, "input_f0_limb_1"),
        (10046, "input_f1_limb_0"),
        (7, "input_f1_limb_1"),
        (49809, "input_a_out_limb_0"),
        (43146, "input_a_out_limb_1"),
        (53691, "input_b_out_limb_0"),
        (63264, "input_b_out_limb_1"),
        (464, "input_c_out_limb_0"),
        (51992, "input_c_out_limb_1"),
        (46984, "input_d_out_limb_0"),
        (55514, "input_d_out_limb_1"),
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
        (194, "ms_8_bits"),
        (168, "ms_8_bits"),
        (74, "ms_8_bits"),
        (114, "ms_8_bits"),
        (183, "ms_8_bits"),
        (216, "ms_8_bits"),
        (440, "ms_9_bits"),
        (182, "ms_9_bits"),
        (3, "ms_9_bits"),
        (406, "ms_9_bits"),
        (104, "ms_7_bits"),
        (123, "ms_7_bits"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_blake_g_gate2() {
    let air_fn = BlakeGGate {};
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (state, _) = registry.run_air(
        &air_fn,
        (),
        [
            const_u32_expr!(3694142613),
            const_u32_expr!(170668591),
            const_u32_expr!(2859583592),
            const_u32_expr!(2750542364),
            const_u32_expr!(101488500),
            const_u32_expr!(3940201164),
            const_u32_expr!(2049993894),
            const_u32_expr!(223224271),
            const_u32_expr!(100412452),
            const_u32_expr!(1063654435),
        ],
    );

    expect![[r#"
        (9365, "input_a_limb_0"),
        (56368, "input_a_limb_1"),
        (12847, "input_b_limb_0"),
        (2604, "input_b_limb_1"),
        (51304, "input_c_limb_0"),
        (43633, "input_c_limb_1"),
        (61980, "input_d_limb_0"),
        (41969, "input_d_limb_1"),
        (38772, "input_f0_limb_0"),
        (1548, "input_f0_limb_1"),
        (45772, "input_f1_limb_0"),
        (60122, "input_f1_limb_1"),
        (27814, "input_a_out_limb_0"),
        (31280, "input_a_out_limb_1"),
        (8655, "input_b_out_limb_0"),
        (3406, "input_b_out_limb_1"),
        (11300, "input_c_out_limb_0"),
        (1532, "input_c_out_limb_1"),
        (5155, "input_d_out_limb_0"),
        (16230, "input_d_out_limb_1"),
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
        (108, "ms_8_bits"),
        (122, "ms_8_bits"),
        (79, "ms_8_bits"),
        (28, "ms_8_bits"),
        (20, "ms_8_bits"),
        (63, "ms_8_bits"),
        (407, "ms_9_bits"),
        (325, "ms_9_bits"),
        (88, "ms_9_bits"),
        (11, "ms_9_bits"),
        (16, "ms_7_bits"),
        (6, "ms_7_bits"),
    "#]]
    .assert_eq(&state.to_string());
}

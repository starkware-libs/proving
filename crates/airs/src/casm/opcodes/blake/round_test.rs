use std::array::from_fn;

use air_infra::casm_state::CasmAddress;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::core::variables::AsProverType;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_felt252_expr, const_u32_expr};
use expect_test::expect;

use super::round::*;

#[test]
fn test_blake_round() {
    let mut air_fn = BlakeRound::default();

    // Create input.
    let state = [
        1589929985, 669959787, 3341104026, 828450965, 1955226293, 542713244, 3587648250,
        2032424797, 3147641385, 3967920621, 2006879305, 2745232376, 2456599919, 130066657,
        1468412498, 325435090,
    ];
    let blake_state: [UInt32Expr; 16] = from_fn(|i| const_u32_expr!(state[i]));
    let message_pointer = 7687346;

    // Fill memory
    let message: [i64; 16] = [
        1190313840, 586871615, 3326317950, 2157490798, 2171729911, 4006315130, 3006051123,
        3934250148, 745259603, 1963379556, 3874654107, 2051567115, 2102274589, 1991875188,
        1621381226, 1307057221,
    ];
    let memory_values: Vec<_> = (0..=15)
        .map(|i| (const_expr!(message_pointer + i), const_felt252_expr!(message[i as usize])))
        .collect();
    air_fn.memory = Felt252IdMemory::new_with_data(memory_values);
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(
        &air_fn,
        (),
        (
            const_expr!(0),
            const_expr!(5),
            (blake_state, CasmAddress::new(const_expr!(message_pointer), "blake_message_pointer")),
        ),
    );

    // Check output.
    let expected_output = [
        1367297060, 2926508617, 107924025, 162546146, 2936957071, 4075222909, 2433518464,
        3059149654, 2047295278, 3825369540, 4231837284, 4024902448, 1952998180, 1326427959,
        3193012524, 1657286418,
    ];

    for (output, expected_output) in output.2.0.into_iter().zip(expected_output) {
        assert_eq!(output.calc(), const_u32_expr!(expected_output).calc());
    }

    // Check state.
    expect![[r#"
        (1, "enabler"),
        (0, "input_limb_0"),
        (5, "input_limb_1"),
        (26625, "input_limb_2"),
        (24260, "input_limb_3"),
        (50795, "input_limb_4"),
        (10222, "input_limb_5"),
        (13210, "input_limb_6"),
        (50981, "input_limb_7"),
        (10389, "input_limb_8"),
        (12641, "input_limb_9"),
        (25269, "input_limb_10"),
        (29834, "input_limb_11"),
        (9628, "input_limb_12"),
        (8281, "input_limb_13"),
        (11002, "input_limb_14"),
        (54743, "input_limb_15"),
        (22365, "input_limb_16"),
        (31012, "input_limb_17"),
        (12841, "input_limb_18"),
        (48029, "input_limb_19"),
        (43501, "input_limb_20"),
        (60545, "input_limb_21"),
        (35913, "input_limb_22"),
        (30622, "input_limb_23"),
        (60408, "input_limb_24"),
        (41888, "input_limb_25"),
        (48495, "input_limb_26"),
        (37484, "input_limb_27"),
        (43233, "input_limb_28"),
        (1984, "input_limb_29"),
        (12882, "input_limb_30"),
        (22406, "input_limb_31"),
        (48850, "input_limb_32"),
        (4965, "input_limb_33"),
        (7687346, "input_limb_34"),
        (2, "blake_round_sigma_output_limb_0"),
        (12, "blake_round_sigma_output_limb_1"),
        (6, "blake_round_sigma_output_limb_2"),
        (10, "blake_round_sigma_output_limb_3"),
        (0, "blake_round_sigma_output_limb_4"),
        (11, "blake_round_sigma_output_limb_5"),
        (8, "blake_round_sigma_output_limb_6"),
        (3, "blake_round_sigma_output_limb_7"),
        (4, "blake_round_sigma_output_limb_8"),
        (13, "blake_round_sigma_output_limb_9"),
        (7, "blake_round_sigma_output_limb_10"),
        (5, "blake_round_sigma_output_limb_11"),
        (15, "blake_round_sigma_output_limb_12"),
        (14, "blake_round_sigma_output_limb_13"),
        (1, "blake_round_sigma_output_limb_14"),
        (9, "blake_round_sigma_output_limb_15"),
        (38270, "low_16_bits"),
        (50755, "high_16_bits"),
        (74, "low_7_ms_bits"),
        (12688, "high_14_ms_bits"),
        (24, "high_5_ms_bits"),
        (2, "message_word_0_id"),
        (10781, "low_16_bits"),
        (32078, "high_16_bits"),
        (21, "low_7_ms_bits"),
        (8019, "high_14_ms_bits"),
        (15, "high_5_ms_bits"),
        (12, "message_word_1_id"),
        (45875, "low_16_bits"),
        (45868, "high_16_bits"),
        (89, "low_7_ms_bits"),
        (11467, "high_14_ms_bits"),
        (22, "high_5_ms_bits"),
        (6, "message_word_2_id"),
        (34715, "low_16_bits"),
        (59122, "high_16_bits"),
        (67, "low_7_ms_bits"),
        (14780, "high_14_ms_bits"),
        (28, "high_5_ms_bits"),
        (10, "message_word_3_id"),
        (49008, "low_16_bits"),
        (18162, "high_16_bits"),
        (95, "low_7_ms_bits"),
        (4540, "high_14_ms_bits"),
        (8, "high_5_ms_bits"),
        (0, "message_word_4_id"),
        (28171, "low_16_bits"),
        (31304, "high_16_bits"),
        (55, "low_7_ms_bits"),
        (7826, "high_14_ms_bits"),
        (15, "high_5_ms_bits"),
        (11, "message_word_5_id"),
        (49747, "low_16_bits"),
        (11371, "high_16_bits"),
        (97, "low_7_ms_bits"),
        (2842, "high_14_ms_bits"),
        (5, "high_5_ms_bits"),
        (8, "message_word_6_id"),
        (45678, "low_16_bits"),
        (32920, "high_16_bits"),
        (89, "low_7_ms_bits"),
        (8230, "high_14_ms_bits"),
        (16, "high_5_ms_bits"),
        (3, "message_word_7_id"),
        (63479, "low_16_bits"),
        (33137, "high_16_bits"),
        (123, "low_7_ms_bits"),
        (8284, "high_14_ms_bits"),
        (16, "high_5_ms_bits"),
        (4, "message_word_8_id"),
        (39540, "low_16_bits"),
        (30393, "high_16_bits"),
        (77, "low_7_ms_bits"),
        (7598, "high_14_ms_bits"),
        (14, "high_5_ms_bits"),
        (13, "message_word_9_id"),
        (58532, "low_16_bits"),
        (60031, "high_16_bits"),
        (114, "low_7_ms_bits"),
        (15007, "high_14_ms_bits"),
        (29, "high_5_ms_bits"),
        (7, "message_word_10_id"),
        (33914, "low_16_bits"),
        (61131, "high_16_bits"),
        (66, "low_7_ms_bits"),
        (15282, "high_14_ms_bits"),
        (29, "high_5_ms_bits"),
        (5, "message_word_11_id"),
        (7237, "low_16_bits"),
        (19944, "high_16_bits"),
        (14, "low_7_ms_bits"),
        (4986, "high_14_ms_bits"),
        (9, "high_5_ms_bits"),
        (15, "message_word_12_id"),
        (20586, "low_16_bits"),
        (24740, "high_16_bits"),
        (40, "low_7_ms_bits"),
        (6185, "high_14_ms_bits"),
        (12, "high_5_ms_bits"),
        (14, "message_word_13_id"),
        (62271, "low_16_bits"),
        (8954, "high_16_bits"),
        (121, "low_7_ms_bits"),
        (2238, "high_14_ms_bits"),
        (4, "high_5_ms_bits"),
        (1, "message_word_14_id"),
        (52068, "low_16_bits"),
        (29958, "high_16_bits"),
        (101, "low_7_ms_bits"),
        (7489, "high_14_ms_bits"),
        (14, "high_5_ms_bits"),
        (9, "message_word_15_id"),
        (20854, "blake_g_output_limb_0"),
        (57359, "blake_g_output_limb_1"),
        (13995, "blake_g_output_limb_2"),
        (18896, "blake_g_output_limb_3"),
        (37505, "blake_g_output_limb_4"),
        (8501, "blake_g_output_limb_5"),
        (21594, "blake_g_output_limb_6"),
        (34877, "blake_g_output_limb_7"),
        (26397, "blake_g_output_limb_0"),
        (5687, "blake_g_output_limb_1"),
        (24486, "blake_g_output_limb_2"),
        (59901, "blake_g_output_limb_3"),
        (37692, "blake_g_output_limb_4"),
        (52607, "blake_g_output_limb_5"),
        (60571, "blake_g_output_limb_6"),
        (43297, "blake_g_output_limb_7"),
        (60981, "blake_g_output_limb_0"),
        (702, "blake_g_output_limb_1"),
        (18070, "blake_g_output_limb_2"),
        (21833, "blake_g_output_limb_3"),
        (10508, "blake_g_output_limb_4"),
        (36, "blake_g_output_limb_5"),
        (59482, "blake_g_output_limb_6"),
        (23598, "blake_g_output_limb_7"),
        (34433, "blake_g_output_limb_0"),
        (50343, "blake_g_output_limb_1"),
        (56032, "blake_g_output_limb_2"),
        (944, "blake_g_output_limb_3"),
        (57807, "blake_g_output_limb_4"),
        (46448, "blake_g_output_limb_5"),
        (12355, "blake_g_output_limb_6"),
        (5432, "blake_g_output_limb_7"),
        (19492, "blake_g_output_limb_0"),
        (20863, "blake_g_output_limb_1"),
        (63357, "blake_g_output_limb_2"),
        (62182, "blake_g_output_limb_3"),
        (46692, "blake_g_output_limb_4"),
        (64572, "blake_g_output_limb_5"),
        (12050, "blake_g_output_limb_6"),
        (25288, "blake_g_output_limb_7"),
        (64073, "blake_g_output_limb_0"),
        (44654, "blake_g_output_limb_1"),
        (35712, "blake_g_output_limb_2"),
        (37132, "blake_g_output_limb_3"),
        (9008, "blake_g_output_limb_4"),
        (61415, "blake_g_output_limb_5"),
        (25380, "blake_g_output_limb_6"),
        (29800, "blake_g_output_limb_7"),
        (51769, "blake_g_output_limb_0"),
        (1646, "blake_g_output_limb_1"),
        (60246, "blake_g_output_limb_2"),
        (46678, "blake_g_output_limb_3"),
        (16174, "blake_g_output_limb_4"),
        (31239, "blake_g_output_limb_5"),
        (44855, "blake_g_output_limb_6"),
        (20239, "blake_g_output_limb_7"),
        (16866, "blake_g_output_limb_0"),
        (2480, "blake_g_output_limb_1"),
        (26767, "blake_g_output_limb_2"),
        (44814, "blake_g_output_limb_3"),
        (33220, "blake_g_output_limb_4"),
        (58370, "blake_g_output_limb_5"),
        (33068, "blake_g_output_limb_6"),
        (48721, "blake_g_output_limb_7"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
fn test_blake_round2() {
    let mut air_fn = BlakeRound::default();

    // Create input.
    let state = [
        2981648577, 2100013035, 663841651, 2464560971, 3804981465, 2521887078, 1263129662,
        3279679818, 1291748021, 2308065230, 3957504572, 113619231, 622788508, 1137821987,
        2149537027, 2989138246,
    ];
    let blake_state: [UInt32Expr; 16] = from_fn(|i| const_u32_expr!(state[i]));
    let message_pointer = 8676;

    // Fill memory
    let message: [i64; 16] = [
        1883221824, 4159262814, 3806732234, 552650188, 2549022015, 3000021069, 2298537828,
        915357142, 1657285681, 1835346724, 4150146227, 3993296861, 2937251920, 1002511359,
        2142515262, 4138014718,
    ];
    let memory_values: Vec<_> = (0..=15)
        .map(|i| (const_expr!(message_pointer + i), const_felt252_expr!(message[i as usize])))
        .collect();
    air_fn.memory = Felt252IdMemory::new_with_data(memory_values);
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (_, output) = registry.run_air(
        &air_fn,
        (),
        (
            const_expr!(0),
            const_expr!(9),
            (blake_state, CasmAddress::new(const_expr!(message_pointer), "blake_message_pointer")),
        ),
    );

    // Check output.
    let expected_output = [
        538881188, 2460695046, 2867837425, 2135058897, 890208416, 953468451, 1371496227,
        2159536600, 3054417061, 1384474009, 3996057645, 3268429468, 2110965500, 3522211544,
        4011799291, 2106633509,
    ];

    for (output, expected_output) in output.2.0.into_iter().zip(expected_output) {
        assert_eq!(output.calc(), const_u32_expr!(expected_output).calc());
    }
}

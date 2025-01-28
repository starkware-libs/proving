use std::array::from_fn;

use super::round::*;
use crate::airs::casm::casm_state::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint32_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;
// Macros
use crate::{const_expr, const_felt252_expr, const_u32_expr};

#[test]
fn test_blake_round() {
    let mut air_fn = Round::default();

    // Create input.
    let state = [
        1589929985, 669959787, 3341104026, 828450965, 1955226293, 542713244, 3587648250,
        2032424797, 3147641385, 3967920621, 2006879305, 2745232376, 2456599919, 130066657,
        1468412498, 325435090,
    ];
    let blake_state: [UInt32Expr; 16] = from_fn(|i| const_u32_expr!(state[i]));
    let message_pointer = 7687346;

    // Fill memory
    let messgae: [i64; 16] = [
        1190313840, 586871615, 3326317950, 2157490798, 2171729911, 4006315130, 3006051123,
        3934250148, 745259603, 1963379556, 3874654107, 2051567115, 2102274589, 1991875188,
        1621381226, 1307057221,
    ];
    let memory_values: Vec<_> = (0..=15)
        .map(|i| {
            (
                const_expr!(message_pointer + i),
                const_felt252_expr!(messgae[i as usize]),
            )
        })
        .collect();
    air_fn.memory = Felt252IdMemory::new_with_data(memory_values);
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(
        &air_fn,
        (),
        (
            const_expr!(0),
            const_expr!(5),
            (
                blake_state,
                CasmAddress::new(const_expr!(message_pointer), "blake_message_pointer"),
            ),
        ),
    );

    // Check output.
    let expected_output = [
        3963716202, 4206293977, 412788584, 1881793115, 1886140120, 51970688, 3922737378, 844204754,
        4073846804, 4289399476, 2793963234, 3884584562, 1260145169, 2821845203, 2951876740,
        745869788,
    ];

    for (output, expected_output) in output.2 .0.into_iter().zip(expected_output) {
        assert_eq!(output.calc(), const_u32_expr!(expected_output).calc());
    }

    // Check state.
    let expected_state = vec![
        (0, "input_limb_0"),     // chain index
        (5, "input_limb_1"),     // round number
        (26625, "input_limb_2"), // state
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
        (2, "round_sigma_output_limb_0"), // sigma
        (12, "round_sigma_output_limb_1"),
        (6, "round_sigma_output_limb_2"),
        (10, "round_sigma_output_limb_3"),
        (0, "round_sigma_output_limb_4"),
        (11, "round_sigma_output_limb_5"),
        (8, "round_sigma_output_limb_6"),
        (3, "round_sigma_output_limb_7"),
        (4, "round_sigma_output_limb_8"),
        (13, "round_sigma_output_limb_9"),
        (7, "round_sigma_output_limb_10"),
        (5, "round_sigma_output_limb_11"),
        (15, "round_sigma_output_limb_12"),
        (14, "round_sigma_output_limb_13"),
        (1, "round_sigma_output_limb_14"),
        (9, "round_sigma_output_limb_15"),
        (38270, "low_16_bits"), // blake message
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
        (20854, "g_output_limb_0"), // g1
        (57359, "g_output_limb_1"),
        (13995, "g_output_limb_2"),
        (18896, "g_output_limb_3"),
        (37505, "g_output_limb_4"),
        (8501, "g_output_limb_5"),
        (21594, "g_output_limb_6"),
        (34877, "g_output_limb_7"),
        (26397, "g_output_limb_0"), // g2
        (5687, "g_output_limb_1"),
        (24486, "g_output_limb_2"),
        (59901, "g_output_limb_3"),
        (37692, "g_output_limb_4"),
        (52607, "g_output_limb_5"),
        (60571, "g_output_limb_6"),
        (43297, "g_output_limb_7"),
        (52940, "g_output_limb_0"), // g3
        (35912, "g_output_limb_1"),
        (42136, "g_output_limb_2"),
        (64413, "g_output_limb_3"),
        (3776, "g_output_limb_4"),
        (57539, "g_output_limb_5"),
        (5797, "g_output_limb_6"),
        (7878, "g_output_limb_7"),
        (34433, "g_output_limb_0"), // g4
        (50343, "g_output_limb_1"),
        (56032, "g_output_limb_2"),
        (944, "g_output_limb_3"),
        (57807, "g_output_limb_4"),
        (46448, "g_output_limb_5"),
        (12355, "g_output_limb_6"),
        (5432, "g_output_limb_7"),
        (33386, "g_output_limb_0"), // g5
        (60481, "g_output_limb_1"),
        (640, "g_output_limb_2"),
        (793, "g_output_limb_3"),
        (32482, "g_output_limb_4"),
        (42632, "g_output_limb_5"),
        (4572, "g_output_limb_6"),
        (11381, "g_output_limb_7"),
        (62425, "g_output_limb_0"), // g6
        (64182, "g_output_limb_1"),
        (14562, "g_output_limb_2"),
        (59856, "g_output_limb_3"),
        (3698, "g_output_limb_4"),
        (59274, "g_output_limb_5"),
        (18961, "g_output_limb_6"),
        (19228, "g_output_limb_7"),
        (42856, "g_output_limb_0"), // g7
        (6298, "g_output_limb_1"),
        (35538, "g_output_limb_2"),
        (12881, "g_output_limb_3"),
        (63508, "g_output_limb_4"),
        (62161, "g_output_limb_5"),
        (61651, "g_output_limb_6"),
        (43057, "g_output_limb_7"),
        (57947, "g_output_limb_0"), // g8
        (28713, "g_output_limb_1"),
        (14040, "g_output_limb_2"),
        (28780, "g_output_limb_3"),
        (2740, "g_output_limb_4"),
        (65451, "g_output_limb_5"),
        (4228, "g_output_limb_6"),
        (45042, "g_output_limb_7"),
    ]
    .into();
    assert_expected_state(&state, &expected_state);
}

#[test]
fn test_blake_round2() {
    let mut air_fn = Round::default();

    // Create input.
    let state = [
        2981648577, 2100013035, 663841651, 2464560971, 3804981465, 2521887078, 1263129662,
        3279679818, 1291748021, 2308065230, 3957504572, 113619231, 622788508, 1137821987,
        2149537027, 2989138246,
    ];
    let blake_state: [UInt32Expr; 16] = from_fn(|i| const_u32_expr!(state[i]));
    let message_pointer = 8676;

    // Fill memory
    let messgae: [i64; 16] = [
        1883221824, 4159262814, 3806732234, 552650188, 2549022015, 3000021069, 2298537828,
        915357142, 1657285681, 1835346724, 4150146227, 3993296861, 2937251920, 1002511359,
        2142515262, 4138014718,
    ];
    let memory_values: Vec<_> = (0..=15)
        .map(|i| {
            (
                const_expr!(message_pointer + i),
                const_felt252_expr!(messgae[i as usize]),
            )
        })
        .collect();
    air_fn.memory = Felt252IdMemory::new_with_data(memory_values);
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (_, output) = registry.run_air(
        &air_fn,
        (),
        (
            const_expr!(0),
            const_expr!(9),
            (
                blake_state,
                CasmAddress::new(const_expr!(message_pointer), "blake_message_pointer"),
            ),
        ),
    );

    // Check output.
    let expected_output = [
        1595516873, 1627571169, 282182205, 109799459, 202420134, 3760382394, 2206057594,
        2642183819, 2101613650, 1423565011, 3526873510, 3385908489, 3382355132, 1220181296,
        2178320081, 284142126,
    ];

    for (output, expected_output) in output.2 .0.into_iter().zip(expected_output) {
        assert_eq!(output.calc(), const_u32_expr!(expected_output).calc());
    }
}

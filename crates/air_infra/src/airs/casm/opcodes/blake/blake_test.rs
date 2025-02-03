use super::decode_blake_opcode::*;
use crate::airs::casm::casm_state::*;
use crate::airs::casm::common::*;
use crate::airs::casm::opcodes::blake::blake_compress::*;
use crate::const_expr;
// Macros
use crate::const_felt252_expr;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::state::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

// TODO(Stav): Add a test that compares to a rust implementation of Blake.
#[allow(clippy::too_many_arguments)]
fn test_blake(
    offsets: [i16; 3],
    casm_state: [i32; 3],
    pointers: [i32; 3],
    state: [i64; 8],
    t: i32,
    new_state: [i64; 8],
    message: [i64; 16],
    flags: ([bool; 5], OpcodeExtension),
) -> (State, CasmStateVar) {
    let [pc_value, ap_value, fp_value] = casm_state;
    let ([dst_base_fp, op0_base_fp, op1_base_fp, op1_base_ap, _ap_update_add_1], opcode) = flags;
    let [state_pointer, new_state_pointer, messgae_pointer] = pointers;
    let pc = const_expr!(pc_value);
    let ap = const_expr!(ap_value);
    let fp = const_expr!(fp_value);

    let mut blake_opcode = BlakeCompressOpcode::default();
    let (registry, _) = AirFnRegistry::new(&blake_opcode);

    // Fill memory
    // opcode
    let mut memory_values = vec![(
        pc.clone(),
        const_felt252_expr!(
            assemble_instruction(
                offsets[0],
                offsets[1],
                offsets[2],
                DecodeBlakeOpcode::default()
                    .get_flags()
                    .non_constants_to_arr(&flags.0),
                opcode
            ),
            0
        ),
    )];

    // dst
    let dst_base = if dst_base_fp { fp_value } else { ap_value };
    memory_values.push((
        const_expr!((dst_base as i16 + offsets[0]) as u32),
        const_felt252_expr!(t),
    ));

    // op0
    let op0_base = if op0_base_fp { fp_value } else { ap_value };
    memory_values.push((
        const_expr!((op0_base as i16 + offsets[1]) as u32),
        const_felt252_expr!(state_pointer),
    ));

    // op1
    let op1_base = if op1_base_fp {
        assert!(!op1_base_ap, "Invalid configuration of flags");
        fp_value
    } else {
        assert!(op1_base_ap, "Invalid configuration of flags");
        ap_value
    };
    memory_values.push((
        const_expr!((op1_base as i16 + offsets[2]) as u32),
        const_felt252_expr!(messgae_pointer),
    ));

    for i in 0..8 {
        // new_state
        memory_values.push((
            const_expr!(new_state_pointer as u32 + i),
            const_felt252_expr!(new_state[i as usize]),
        ));
        // state
        memory_values.push((
            const_expr!(state_pointer as u32 + i),
            const_felt252_expr!(state[i as usize]),
        ));
    }
    for i in 0..16 {
        // message
        memory_values.push((
            const_expr!(messgae_pointer as u32 + i),
            const_felt252_expr!(message[i as usize]),
        ));
    }

    // new state pointer
    memory_values.push((
        const_expr!(ap_value),
        const_felt252_expr!(new_state_pointer),
    ));
    blake_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    let (state, output) = registry.run_air(
        &blake_opcode,
        (),
        CasmStateVar::new(pc.clone(), ap.clone(), fp.clone()),
    );
    (state, output)
}

#[test]
fn test_blake_opcode() {
    let state = [
        3759144919, 3571705300, 3897207279, 4099207644, 352939213, 93879281, 2823052663, 3290983512,
    ];
    let new_state = [
        3732224820, 992304330, 2505173906, 3209801007, 2076716084, 1709955699, 2814744008,
        925232990,
    ];
    let messgae = [
        3675856565, 2505499898, 2411686070, 3389252950, 3499394596, 729107608, 2054428875,
        2812783018, 494163526, 2118351834, 3071324623, 2000055100, 1663106196, 876311781,
        2518385179, 203883843,
    ];
    let [pc, ap, fp] = [3, 11, 6];
    let (state, output) = test_blake(
        [45, 83, 112],
        [pc, ap, fp],
        [456, 1465, 432453],
        state,
        64,
        new_state,
        messgae,
        ([false, true, true, false, true], OpcodeExtension::Blake),
    );

    // Check the output
    assert_eq!(output.pc().calc(), (pc + 1).to_string());
    assert_eq!(output.fp().calc(), fp.to_string());
    assert_eq!(output.ap().calc(), (ap + 1).to_string());

    // Check state
    // Total of 278 deductions.
    let expected_state = vec![
        (32813, "offset0"),
        (32851, "offset1"),
        (32880, "offset2"),
        (0, "dst_base_fp"),
        (1, "op0_base_fp"),
        (1, "op1_base_fp"),
        (0, "op1_base_ap"),
        (1, "ap_update_add_1"),
        (1, "opcode_extension"),
        (6, "mem0_base"),
        (2, "op0_id"),
        (456, "op0_limb_0"),
        (0, "op0_limb_1"),
        (0, "op0_limb_2"),
        (6, "mem1_base"),
        (3, "op1_id"),
        (325, "op1_limb_0"),
        (332, "op1_limb_1"),
        (1, "op1_limb_2"),
        (36, "ap_id"),
        (441, "ap_limb_0"),
        (2, "ap_limb_1"),
        (0, "ap_limb_2"),
        (11, "mem_dst_base"),
        (64, "low_16_bits"),
        (0, "high_16_bits"),
        (0, "low_7_ms_bits"),
        (0, "high_14_ms_bits"),
        (0, "high_5_ms_bits"),
        (1, "dst_id"),
        (65495, "low_16_bits"),
        (57359, "high_16_bits"),
        (127, "low_7_ms_bits"),
        (14339, "high_14_ms_bits"),
        (28, "high_5_ms_bits"),
        (5, "state_0_id"),
        (58836, "low_16_bits"),
        (54499, "high_16_bits"),
        (114, "low_7_ms_bits"),
        (13624, "high_14_ms_bits"),
        (26, "high_5_ms_bits"),
        (7, "state_1_id"),
        (43503, "low_16_bits"),
        (59466, "high_16_bits"),
        (84, "low_7_ms_bits"),
        (14866, "high_14_ms_bits"),
        (29, "high_5_ms_bits"),
        (9, "state_2_id"),
        (61916, "low_16_bits"),
        (62548, "high_16_bits"),
        (120, "low_7_ms_bits"),
        (15637, "high_14_ms_bits"),
        (30, "high_5_ms_bits"),
        (11, "state_3_id"),
        (27853, "low_16_bits"),
        (5385, "high_16_bits"),
        (54, "low_7_ms_bits"),
        (1346, "high_14_ms_bits"),
        (2, "high_5_ms_bits"),
        (13, "state_4_id"),
        (31729, "low_16_bits"),
        (1432, "high_16_bits"),
        (61, "low_7_ms_bits"),
        (358, "high_14_ms_bits"),
        (0, "high_5_ms_bits"),
        (15, "state_5_id"),
        (23927, "low_16_bits"),
        (43076, "high_16_bits"),
        (46, "low_7_ms_bits"),
        (10769, "high_14_ms_bits"),
        (21, "high_5_ms_bits"),
        (17, "state_6_id"),
        (27736, "low_16_bits"),
        (50216, "high_16_bits"),
        (54, "low_7_ms_bits"),
        (12554, "high_14_ms_bits"),
        (24, "high_5_ms_bits"),
        (19, "state_7_id"),
        (0, "ms_8_bits"),
        (0, "ms_8_bits"),
        (63, "xor"),
        (82, "xor"),
        (14, "xor"),
        (81, "xor"),
        (0, "round_output_limb_0"),
        (10, "round_output_limb_1"),
        (10782, "round_output_limb_2"),
        (35146, "round_output_limb_3"),
        (65004, "round_output_limb_4"),
        (40254, "round_output_limb_5"),
        (55015, "round_output_limb_6"),
        (8313, "round_output_limb_7"),
        (28405, "round_output_limb_8"),
        (15416, "round_output_limb_9"),
        (45496, "round_output_limb_10"),
        (7596, "round_output_limb_11"),
        (11636, "round_output_limb_12"),
        (18997, "round_output_limb_13"),
        (64553, "round_output_limb_14"),
        (43880, "round_output_limb_15"),
        (13407, "round_output_limb_16"),
        (27756, "round_output_limb_17"),
        (61181, "round_output_limb_18"),
        (46896, "round_output_limb_19"),
        (17650, "round_output_limb_20"),
        (29432, "round_output_limb_21"),
        (38042, "round_output_limb_22"),
        (23906, "round_output_limb_23"),
        (12806, "round_output_limb_24"),
        (30525, "round_output_limb_25"),
        (61761, "round_output_limb_26"),
        (29549, "round_output_limb_27"),
        (36086, "round_output_limb_28"),
        (10822, "round_output_limb_29"),
        (13462, "round_output_limb_30"),
        (42217, "round_output_limb_31"),
        (46937, "round_output_limb_32"),
        (40801, "round_output_limb_33"),
        (432453, "round_output_limb_34"),
        (42, "ms_8_bits"),
        (137, "ms_8_bits"),
        (238, "ms_8_bits"),
        (183, "ms_8_bits"),
        (227, "xor"),
        (196, "xor"),
        (122, "xor"),
        (62, "xor"),
        (196, "ms_8_bits"),
        (62, "ms_8_bits"),
        (255, "ms_8_bits"),
        (224, "ms_8_bits"),
        (52, "xor"),
        (59, "xor"),
        (117, "xor"),
        (222, "xor"),
        (253, "ms_8_bits"),
        (157, "ms_8_bits"),
        (68, "ms_8_bits"),
        (114, "ms_8_bits"),
        (30, "xor"),
        (185, "xor"),
        (198, "xor"),
        (239, "xor"),
        (185, "ms_8_bits"),
        (239, "ms_8_bits"),
        (229, "ms_8_bits"),
        (212, "ms_8_bits"),
        (202, "xor"),
        (92, "xor"),
        (37, "xor"),
        (59, "xor"),
        (214, "ms_8_bits"),
        (32, "ms_8_bits"),
        (148, "ms_8_bits"),
        (93, "ms_8_bits"),
        (125, "xor"),
        (66, "xor"),
        (27, "xor"),
        (125, "xor"),
        (66, "ms_8_bits"),
        (125, "ms_8_bits"),
        (169, "ms_8_bits"),
        (232, "ms_8_bits"),
        (146, "xor"),
        (235, "xor"),
        (81, "xor"),
        (149, "xor"),
        (110, "ms_8_bits"),
        (60, "ms_8_bits"),
        (50, "ms_8_bits"),
        (119, "ms_8_bits"),
        (243, "xor"),
        (92, "xor"),
        (5, "xor"),
        (75, "xor"),
        (92, "ms_8_bits"),
        (75, "ms_8_bits"),
        (241, "ms_8_bits"),
        (244, "ms_8_bits"),
        (47, "xor"),
        (173, "xor"),
        (81, "xor"),
        (191, "xor"),
        (177, "ms_8_bits"),
        (29, "ms_8_bits"),
        (241, "ms_8_bits"),
        (115, "ms_8_bits"),
        (249, "xor"),
        (64, "xor"),
        (193, "xor"),
        (110, "xor"),
        (64, "ms_8_bits"),
        (110, "ms_8_bits"),
        (108, "ms_8_bits"),
        (21, "ms_8_bits"),
        (52, "xor"),
        (44, "xor"),
        (200, "xor"),
        (123, "xor"),
        (45, "ms_8_bits"),
        (74, "ms_8_bits"),
        (140, "ms_8_bits"),
        (42, "ms_8_bits"),
        (130, "xor"),
        (161, "xor"),
        (115, "xor"),
        (96, "xor"),
        (161, "ms_8_bits"),
        (96, "ms_8_bits"),
        (123, "ms_8_bits"),
        (5, "ms_8_bits"),
        (115, "xor"),
        (218, "xor"),
        (235, "xor"),
        (101, "xor"),
        (252, "ms_8_bits"),
        (171, "ms_8_bits"),
        (52, "ms_8_bits"),
        (164, "ms_8_bits"),
        (191, "xor"),
        (200, "xor"),
        (129, "xor"),
        (15, "xor"),
        (200, "ms_8_bits"),
        (15, "ms_8_bits"),
        (93, "ms_8_bits"),
        (168, "ms_8_bits"),
        (200, "xor"),
        (149, "xor"),
        (197, "xor"),
        (167, "xor"),
        (52, "ms_8_bits"),
        (108, "ms_8_bits"),
        (183, "ms_8_bits"),
        (159, "ms_8_bits"),
        (6, "xor"),
        (131, "xor"),
        (13, "xor"),
        (243, "xor"),
        (131, "ms_8_bits"),
        (243, "ms_8_bits"),
        (108, "ms_8_bits"),
        (196, "ms_8_bits"),
        (94, "xor"),
        (239, "xor"),
        (37, "xor"),
        (55, "xor"),
        (29, "low_7_ms_bits"),
        (14237, "high_14_ms_bits"),
        (27, "high_5_ms_bits"),
        (4, "new_state_0_id"),
        (46, "low_7_ms_bits"),
        (3785, "high_14_ms_bits"),
        (7, "high_5_ms_bits"),
        (6, "new_state_1_id"),
        (117, "low_7_ms_bits"),
        (9556, "high_14_ms_bits"),
        (18, "high_5_ms_bits"),
        (8, "new_state_2_id"),
        (86, "low_7_ms_bits"),
        (12244, "high_14_ms_bits"),
        (23, "high_5_ms_bits"),
        (10, "new_state_3_id"),
        (22, "low_7_ms_bits"),
        (7922, "high_14_ms_bits"),
        (15, "high_5_ms_bits"),
        (12, "new_state_4_id"),
        (109, "low_7_ms_bits"),
        (6522, "high_14_ms_bits"),
        (12, "high_5_ms_bits"),
        (14, "new_state_5_id"),
        (74, "low_7_ms_bits"),
        (10737, "high_14_ms_bits"),
        (20, "high_5_ms_bits"),
        (16, "new_state_6_id"),
        (119, "low_7_ms_bits"),
        (3529, "high_14_ms_bits"),
        (6, "high_5_ms_bits"),
        (18, "new_state_7_id"),
    ]
    .into();
    assert_expected_state(&state, &expected_state);
}

#[test]
fn test_blake_last_block() {
    let state = [
        4071327835, 1664964879, 3980302945, 1627022125, 1767661733, 1368393883, 126117237,
        3662929314,
    ];
    let new_state = [
        3682597515, 2216389235, 312988646, 2824139482, 287550524, 1393039850, 2625350416, 483117428,
    ];
    let messgae = [
        1730312174, 3506704347, 1997875835, 3947607044, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let [pc, ap, fp] = [546, 5656, 886];

    let (_, output) = test_blake(
        [5416, 485, 15],
        [pc, ap, fp],
        [2146, 65, 7155],
        state,
        16,
        new_state,
        messgae,
        (
            [true, true, true, false, false],
            OpcodeExtension::BlakeFinalize,
        ),
    );

    // Check the output
    assert_eq!(output.pc().calc(), (pc + 1).to_string());
    assert_eq!(output.fp().calc(), fp.to_string());
    assert_eq!(output.ap().calc(), ap.to_string());
}

#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_blake_opcode_fail() {
    let state = [
        3759144919, 3571705300, 3897207279, 4099207644, 352939213, 93879281, 2823052663, 3290983512,
    ];
    let new_state = [
        3732224820, 992304331, 2505173906, 3209801007, 2076716084, 1709955699, 2814744008,
        925232990,
    ];
    let messgae = [
        3675856565, 2505499898, 2411686070, 3389252950, 3499394596, 729107608, 2054428875,
        2812783018, 494163526, 2118351834, 3071324623, 2000055100, 1663106196, 876311781,
        2518385179, 203883843,
    ];

    test_blake(
        [45, 83, 112],
        [3, 11, 6],
        [456, 1465, 432453],
        state,
        64,
        new_state,
        messgae,
        ([false, true, true, false, true], OpcodeExtension::Blake),
    );
}

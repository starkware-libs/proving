use air_infra::casm_state::CasmStateVar;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::state::State;
use air_infra::core::variables::AsProverType;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_felt252_expr};
use expect_test::expect;

use super::decode_blake_opcode::*;
use crate::casm::common::*;
use crate::casm::opcodes::blake::blake_compress_opcode::*;

#[allow(clippy::too_many_arguments)]
fn test_blake(
    offsets: [i16; 3],
    casm_state: [u32; 3],
    pointers: [u32; 3],
    state: [u32; 8],
    t: u32,
    new_state: [u32; 8],
    message: [u32; 16],
    flags: ([bool; 5], OpcodeExtension),
) -> (State, CasmStateVar) {
    let [pc_value, ap_value, fp_value] = casm_state;
    let ([dst_base_fp, op0_base_fp, op1_base_fp, op1_base_ap, _ap_update_add_1], opcode) = flags;
    let [state_pointer, new_state_pointer, message_pointer] = pointers;
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
                DecodeBlakeOpcode::default().get_flags().non_constants_to_arr(&flags.0),
                opcode
            ),
            0
        ),
    )];

    // dst
    let dst_base = if dst_base_fp { fp_value } else { ap_value };
    memory_values
        .push((const_expr!((dst_base as i16 + offsets[0]) as u32), const_felt252_expr!(t as i64)));

    // op0
    let op0_base = if op0_base_fp { fp_value } else { ap_value };
    memory_values.push((
        const_expr!((op0_base as i16 + offsets[1]) as u32),
        const_felt252_expr!(state_pointer as i64),
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
        const_felt252_expr!(message_pointer as i64),
    ));

    for i in 0..8 {
        // new_state
        memory_values.push((
            const_expr!(new_state_pointer + i),
            const_felt252_expr!(new_state[i as usize] as i64),
        ));
        // state
        memory_values
            .push((const_expr!(state_pointer + i), const_felt252_expr!(state[i as usize] as i64)));
    }
    for i in 0..16 {
        // message
        memory_values.push((
            const_expr!(message_pointer + i),
            const_felt252_expr!(message[i as usize] as i64),
        ));
    }

    // new state pointer
    memory_values.push((const_expr!(ap_value), const_felt252_expr!(new_state_pointer as i64)));
    blake_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    let (state, output) =
        registry.run_air(&blake_opcode, (), CasmStateVar::new(pc.clone(), ap.clone(), fp.clone()));
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
    let message = [
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
        message,
        ([false, true, true, false, true], OpcodeExtension::Blake),
    );

    // Check the output
    assert_eq!(output.pc().calc(), (pc + 1).to_string());
    assert_eq!(output.fp().calc(), fp.to_string());
    assert_eq!(output.ap().calc(), (ap + 1).to_string());

    // Check state
    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (32813, "offset0"),
        (32851, "offset1"),
        (32880, "offset2"),
        (0, "dst_base_fp"),
        (1, "op0_base_fp"),
        (1, "op1_base_fp"),
        (1, "ap_update_add_1"),
        (1, "opcode_extension"),
        (6, "mem0_base"),
        (2, "op0_id"),
        (456, "op0_limb_0"),
        (0, "op0_limb_1"),
        (0, "op0_limb_2"),
        (0, "op0_limb_3"),
        (0, "partial_limb_msb"),
        (6, "mem1_base"),
        (3, "op1_id"),
        (325, "op1_limb_0"),
        (332, "op1_limb_1"),
        (1, "op1_limb_2"),
        (0, "op1_limb_3"),
        (0, "partial_limb_msb"),
        (36, "ap_id"),
        (441, "ap_limb_0"),
        (2, "ap_limb_1"),
        (0, "ap_limb_2"),
        (0, "ap_limb_3"),
        (0, "partial_limb_msb"),
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
        (10782, "blake_round_output_limb_0"),
        (35146, "blake_round_output_limb_1"),
        (65004, "blake_round_output_limb_2"),
        (40254, "blake_round_output_limb_3"),
        (55015, "blake_round_output_limb_4"),
        (8313, "blake_round_output_limb_5"),
        (28405, "blake_round_output_limb_6"),
        (15416, "blake_round_output_limb_7"),
        (45496, "blake_round_output_limb_8"),
        (7596, "blake_round_output_limb_9"),
        (11636, "blake_round_output_limb_10"),
        (18997, "blake_round_output_limb_11"),
        (64553, "blake_round_output_limb_12"),
        (43880, "blake_round_output_limb_13"),
        (13407, "blake_round_output_limb_14"),
        (27756, "blake_round_output_limb_15"),
        (61181, "blake_round_output_limb_16"),
        (46896, "blake_round_output_limb_17"),
        (17650, "blake_round_output_limb_18"),
        (29432, "blake_round_output_limb_19"),
        (38042, "blake_round_output_limb_20"),
        (23906, "blake_round_output_limb_21"),
        (12806, "blake_round_output_limb_22"),
        (30525, "blake_round_output_limb_23"),
        (61761, "blake_round_output_limb_24"),
        (29549, "blake_round_output_limb_25"),
        (36086, "blake_round_output_limb_26"),
        (10822, "blake_round_output_limb_27"),
        (13462, "blake_round_output_limb_28"),
        (42217, "blake_round_output_limb_29"),
        (46937, "blake_round_output_limb_30"),
        (40801, "blake_round_output_limb_31"),
        (432453, "blake_round_output_limb_32"),
        (15156, "triple_xor_32_output_limb_0"),
        (56949, "triple_xor_32_output_limb_1"),
        (23754, "triple_xor_32_output_limb_0"),
        (15141, "triple_xor_32_output_limb_1"),
        (60306, "triple_xor_32_output_limb_0"),
        (38225, "triple_xor_32_output_limb_1"),
        (44335, "triple_xor_32_output_limb_0"),
        (48977, "triple_xor_32_output_limb_1"),
        (11316, "triple_xor_32_output_limb_0"),
        (31688, "triple_xor_32_output_limb_1"),
        (55923, "triple_xor_32_output_limb_0"),
        (26091, "triple_xor_32_output_limb_1"),
        (38344, "triple_xor_32_output_limb_0"),
        (42949, "triple_xor_32_output_limb_1"),
        (61278, "triple_xor_32_output_limb_0"),
        (14117, "triple_xor_32_output_limb_1"),
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
    "#]]
    .assert_eq(&state.to_string());
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
    let message =
        [1730312174, 3506704347, 1997875835, 3947607044, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let [pc, ap, fp] = [546, 5656, 886];

    let (_, output) = test_blake(
        [5416, 485, 15],
        [pc, ap, fp],
        [2146, 65, 7155],
        state,
        16,
        new_state,
        message,
        ([true, true, true, false, false], OpcodeExtension::BlakeFinalize),
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
    let message = [
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
        message,
        ([false, true, true, false, true], OpcodeExtension::Blake),
    );
}

mod blake_rust_tests {
    use blake2::*;

    use super::*;
    use crate::casm::opcodes::blake::create_blake_round_input::*;

    // Convert array of bytes to array of u32
    fn u8_to_u32_array(bytes: &[u8]) -> Vec<u32> {
        let mut u32_array = vec![];
        for chunk in bytes.chunks(4) {
            let mut padded_chunk = [0u8; 4];

            for (i, &b) in chunk.iter().enumerate() {
                padded_chunk[i] = b;
            }

            u32_array.push(u32::from_le_bytes(padded_chunk));
        }
        u32_array
    }

    #[test]
    fn test_empty_string_hash() {
        let mut hasher = Blake2s256::new();
        hasher.update(b"");
        let hash: [u8; 32] = hasher.finalize().into();

        let state = [
            // key_size = 0x00, output_size = 0x20
            IV[0] ^ 0x01010020,
            IV[1],
            IV[2],
            IV[3],
            IV[4],
            IV[5],
            IV[6],
            IV[7],
        ];
        let new_state = u8_to_u32_array(&hash);
        let message = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        test_blake(
            [45, 83, 112],
            [3, 11, 6],
            [456, 1465, 432453],
            state,
            0,
            new_state.try_into().expect("Expected hash size of 8 u32 elements"),
            message,
            ([false, true, true, false, true], OpcodeExtension::BlakeFinalize),
        );
    }

    #[test]
    fn test_big_string_hash() {
        let input = b"25844d502c329223298318e937d334dd3ed7dd273b71432bb157f742f33b03a55bc82437b9477cd70e08e088fc2999eabb7225";
        let input_as_u32 = u8_to_u32_array(input);
        let num_bytes = input.len();

        let mut hasher = Blake2s256::new();
        hasher.update(input);
        let hash: [u8; 32] = hasher.finalize().into();

        // First chunk
        let mut state = [
            // key_size = 0x00, output_size = 0x20
            IV[0] ^ 0x01010020,
            IV[1],
            IV[2],
            IV[3],
            IV[4],
            IV[5],
            IV[6],
            IV[7],
        ];

        let new_state = [
            1393684787, 2988713546, 1902042253, 224103376, 992369913, 3965699322, 2296366438,
            863347823,
        ];
        let message0 =
            input_as_u32[0..16].try_into().expect("Expected at least 16 u32 elements in input");

        let (..) = test_blake(
            [45, 83, 112],
            [3, 11, 6],
            [456, 1465, 432453],
            state,
            64,
            new_state,
            message0,
            ([false, true, true, false, true], OpcodeExtension::Blake),
        );

        // Second chunk
        state = new_state;
        // Pad with zeros the end of message.
        let mut message1 = [0u32; 16];
        message1[..10].copy_from_slice(&input_as_u32[16..26]);
        let new_state = u8_to_u32_array(&hash);

        let (..) = test_blake(
            [45, 83, 112],
            [3, 11, 6],
            [456, 1465, 432453],
            state,
            num_bytes as u32,
            new_state.try_into().expect("Expected hash size of 8 u32 elements"),
            message1,
            ([false, true, true, false, true], OpcodeExtension::BlakeFinalize),
        );
    }
}

use prover_types::cpu::PRIME;

use super::super::common::*;
use super::decode_inst::*;
use crate::airs::casm::casm_state::*;
// Macros
use crate::const_expr;
use crate::const_felt252_expr;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::state::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

fn test_with_matching_memory(
    flags: [bool; 15],
    is_flag_const: [bool; 15],
    offsets: [i16; 3],
    is_offset_const: [bool; 3],
    expected_state: State,
) {
    let const_offsets = offsets
        .iter()
        .enumerate()
        .map(|(i, &off)| if is_offset_const[i] { Some(off) } else { None })
        .collect::<Vec<Option<i16>>>()
        .try_into()
        .unwrap();
    let const_flags = Flags::from_arr(
        flags
            .iter()
            .enumerate()
            .map(|(i, &flag)| if is_flag_const[i] { Some(flag) } else { None })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
    );

    // Define and fill memory
    let pc = const_expr!(0);
    let memory = Felt252IdMemory::new_with_data(vec![(
        pc.clone(),
        const_felt252_expr!(
            assemble_instruction(offsets[0], offsets[1], offsets[2], flags) as u128,
            0
        ),
    )]);

    // Run and check output
    let air_fn = DecodeInstruction {
        const_offsets,
        const_flags,
        memory,
    };

    let (registry, entry) = AirFnRegistry::new(&air_fn);
    let (state, (offsets_output, flags_output)) =
        registry.run_air(&air_fn, CasmAddress::new(pc, "pc"));

    // Check entry
    compare_json(
        &entry,
        &format!(
            "{}{}.json",
            TEST_JSONS_DECODE_INSTRUCTION_DIR,
            entry.name.to_lowercase()
        ),
    );

    assert!(
        state == expected_state,
        "State {} does not match {}",
        state,
        expected_state
    );

    for (i, &offset) in offsets.iter().enumerate() {
        assert_eq!(
            offsets_output[i].calc(),
            (offset as i64).rem_euclid(PRIME as i64).to_string()
        );
    }
    for (i, flag) in flags.iter().enumerate() {
        assert_eq!(flags_output[i].calc(), (*flag as u32).to_string());
    }
}

fn init_flags_and_offsets() -> ([bool; 15], [i16; 3]) {
    let flags = Flags {
        dst_base_fp: Some(false),
        op0_base_fp: Some(true),
        op1_imm: Some(false),
        op1_base_fp: Some(true),
        op1_base_ap: Some(false),
        res_add: Some(false),
        res_mul: Some(false),
        pc_update_jump: Some(true),
        pc_update_jump_rel: Some(false),
        pc_update_jnz: Some(true),
        ap_update_add: Some(true),
        ap_update_add_1: Some(false),
        opcode_call: Some(false),
        opcode_ret: Some(false),
        opcode_assert_eq: Some(true),
    };
    let offsets = [0x4321, -0x0765, 0xcba];
    (flags.into(), offsets)
}

#[test]
fn test_no_consts() {
    let (flags, offsets) = init_flags_and_offsets();
    let is_offset_const = [false; 3];
    let is_flag_const = [false; 15];

    test_with_matching_memory(
        flags,
        is_flag_const,
        offsets,
        is_offset_const,
        vec![
            (49953, "offset0"),
            (30875, "offset1"),
            (36026, "offset2"),
            (0, "dst_base_fp"),
            (1, "op0_base_fp"),
            (0, "op1_imm"),
            (1, "op1_base_fp"),
            (0, "op1_base_ap"),
            (0, "res_add"),
            (0, "res_mul"),
            (1, "pc_update_jump"),
            (0, "pc_update_jump_rel"),
            (1, "pc_update_jnz"),
            (1, "ap_update_add"),
            (0, "ap_update_add_1"),
            (0, "opcode_call"),
            (0, "opcode_ret"),
            (1, "opcode_assert_eq"),
        ]
        .into(),
    );
}

#[test]
fn test_all_consts() {
    let (flags, offsets) = init_flags_and_offsets();
    let is_offset_const = [true; 3];
    let is_flag_const = [true; 15];

    test_with_matching_memory(
        flags,
        is_flag_const,
        offsets,
        is_offset_const,
        vec![].into(),
    );
}

#[test]
fn test_some_consts() {
    let (flags, offsets) = init_flags_and_offsets();
    let is_offset_const = [true, false, true];
    let mut is_flag_const = [true; 15];
    is_flag_const[0] = false;
    is_flag_const[2] = false;

    test_with_matching_memory(
        flags,
        is_flag_const,
        offsets,
        is_offset_const,
        vec![(30875, "offset1"), (0, "dst_base_fp"), (0, "op1_imm")].into(),
    );
}

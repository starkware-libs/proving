use super::super::common::*;
use super::decode_instruction::*;

use crate::airs::memory::felt252_id_memory::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::prover_types::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

//Macros
use crate::const_expr;
use crate::const_felt252_expr;

fn test_with_matching_memory(
    flags: [bool; 15],
    is_flag_const: [bool; 15],
    offsets: [i16; 3],
    is_offset_const: [bool; 3],
    entry_file_name: Option<&str>,
    expected_state: Vec<u32>,
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

    let registry = AirFnRegistry::new(&air_fn);
    let (state, (offsets_output, flags_output)) = registry.run_air(&air_fn, pc);

    // Check entry
    if let Some(entry_file_name) = entry_file_name {
        compare_test_json(
            registry,
            &air_fn.name(),
            &(TEST_JSONS_OPCODES_DIR.to_owned() + entry_file_name),
        );
    }

    assert_eq!(
        state.calc(),
        expected_state
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
    );

    for (i, &offset) in offsets.iter().enumerate() {
        assert_eq!(
            offsets_output[i].calc(),
            (offset as i64).rem_euclid(PRIME as i64).to_string()
        );
    }
    for (i, flag) in flags.iter().enumerate() {
        assert_eq!(flags_output[i].calc(), flag.to_string());
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
        Some("decode_inst_no_consts.json"),
        vec![
            289, 97, 3, 38, 15, 10, 203, 4, 0, 1, 0, 1, 0, 0, 0, 1, 0, 1, 1, 0, 0, 0, 1, 0,
        ],
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
        Some("decode_inst_all_consts.json"),
        vec![0],
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
        Some("decode_inst_some_consts.json"),
        vec![3, 38, 15, 0, 0, 0],
    );
}

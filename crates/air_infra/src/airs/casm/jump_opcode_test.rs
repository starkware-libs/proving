use super::common::*;
use super::jump_opcode::*;

use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::expr::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;

// Macros
use crate::const_expr;
use crate::expr;
use crate::felt252_expr;

fn test_jump_opcode(
    is_rel_jump: bool,
    op1_base_fp: bool,
    ap_update_add_1: bool,
    op1: u128,
    offset_value: i16,
    air_body_hints: Option<[&str; 3]>,
) {
    // Create the air function
    let mut jump_opcode = JumpOpcode {
        is_rel: is_rel_jump,
        flag_op1_base_fp: op1_base_fp,
        flag_ap_update_add_1: ap_update_add_1,
        memory: Memory::default(),
    };

    // Register values at opcode start
    let pc_value = 3;
    let ap_value = 11;
    let fp_value = 6;

    let pc = expr!("pc", pc_value);
    let ap = expr!("ap", ap_value);
    let fp = expr!("fp", fp_value);

    // Fill memory
    let mut memory_values = vec![(
        pc.clone(),
        felt252_expr!(
            "op",
            assemble_jump(offset_value, &jump_opcode.get_flags()) as u128,
            0
        ),
    )];
    if is_rel_jump {
        memory_values.push((const_expr!(pc_value + 1), felt252_expr!("op1", op1, 0)));
    } else if op1_base_fp {
        memory_values.push((
            const_expr!((fp_value as i16 + offset_value) as u32),
            felt252_expr!("op1", op1, 0),
        ));
    } else {
        memory_values.push((
            const_expr!((ap_value as i16 + offset_value) as u32),
            felt252_expr!("op1", op1, 0),
        ));
    }
    jump_opcode.init_memory(&Memory::new_with_data(memory_values));

    // Run air function
    let registry = AirFnRegistry::new(&jump_opcode);
    let (state, [next_pc, next_ap, next_fp]) =
        registry.run_air(&jump_opcode, [pc, ap.clone(), fp.clone()]);

    // Check output
    if is_rel_jump {
        assert_eq!(next_pc.calc(), (pc_value + op1 as u32).to_string());
    } else {
        assert_eq!(next_pc.calc(), op1.to_string());
    }
    assert_eq!(next_fp.calc(), fp.calc());
    if ap_update_add_1 {
        assert_eq!(next_ap.calc(), (ap_value + 1).to_string());
    } else {
        assert_eq!(next_ap.calc(), ap.calc());
    }

    // Check state
    let mut expected_state = vec![pc_value, ap_value, fp_value];
    if is_rel_jump {
        expected_state.push((op1 & 0x1FF) as u32);
    } else {
        expected_state.push((offset_as_u16(offset_value) & 0xF) as u32);
        expected_state.push(((offset_as_u16(offset_value) >> 4) & 0x1FF) as u32);
        expected_state.push(((offset_as_u16(offset_value) >> 13) & 0x7) as u32);
        expected_state.push((op1 & 0x1FF) as u32);
        expected_state.push(((op1 >> 9) & 0x1FF) as u32);
        expected_state.push(((op1 >> 18) & 0x1FF) as u32);
    }
    assert_eq!(
        state.calc(),
        expected_state
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    );

    // Check air body
    if let Some([check_instruction_offsets, check_instruction_name, read_call]) = air_body_hints {
        let check_instruction_call = &format!(
            "({}, {}) = {}({})",
            check_instruction_offsets,
            jump_opcode.get_flags(),
            check_instruction_name,
            "state[0]"
        );
        let entry = registry.get_air_fn_entry(&jump_opcode);
        assert_eq!(
            entry
                .air_body
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
            vec![
                &format!(
                    "deduction_tmp_0 = [{name}_input[0], {name}_input[1], {name}_input[2]]",
                    name = jump_opcode.name()
                ),
                "Deduction: deduction_tmp_0[0]",
                "Deduction: deduction_tmp_0[1]",
                "Deduction: deduction_tmp_0[2]",
                check_instruction_call,
                read_call
            ]
        );
    }
}

#[test]
fn test_abs_jump_base_ap() {
    let check_instruction_offsets =
        "[const_2147483646, const_2147483646, (((state[3] + (state[4] * const_16)) + (state[5] * const_8192)) - const_32768)]";
    let check_instruction_name = "CheckInstruction_788d5ba22ffccac";
    let read_addr_output = "((state[6] + (state[7] * const_512)) + (state[8] * const_262144))";
    let read_addr_input = "(state[1] + (((state[3] + (state[4] * const_16)) + (state[5] * const_8192)) - const_32768))";
    let read_addr_call = &format!(
        "{} = {}({})",
        read_addr_output, "ReadAddr_d86123cf8dd732a9", read_addr_input
    );
    test_jump_opcode(
        false,
        false,
        false,
        8,
        2,
        Some([
            check_instruction_offsets,
            check_instruction_name,
            read_addr_call,
        ]),
    );
}

#[test]
fn test_abs_jump_base_fp() {
    test_jump_opcode(false, true, false, 5, 10, None);
}

#[test]
fn test_abs_jump_base_ap_inc_ap() {
    test_jump_opcode(false, false, true, 12, 100, None);
}

#[test]
fn test_abs_jump_base_fp_inc_ap() {
    test_jump_opcode(false, true, true, 20, 17, None);
}

#[test]
fn test_abs_big_op1() {
    test_jump_opcode(false, false, false, 1684685, 402, None);
}

#[test]
fn test_abs_jump_negativ_offset() {
    test_jump_opcode(false, false, false, 9, -9, None);
}

#[test]
fn test_rel_jump() {
    let check_instruction_offsets = "[const_2147483646, const_2147483646, const_1]";
    let check_instruction_name = "CheckInstruction_a64ec1f5e7b083a0";
    let read_small_felt252_output = "Felt252::from_m31_(zero_extend([state[3]]))";
    let read_small_felt252_input = "(state[0] + const_1)";
    let read_small_felt252_name = "ReadSmallFelt252_cc824bd2f61c6ef6";
    let read_small_felt252_call = &format!(
        "{} = {}({})",
        read_small_felt252_output, read_small_felt252_name, read_small_felt252_input
    );
    test_jump_opcode(
        true,
        false,
        false,
        100,
        5,
        Some([
            check_instruction_offsets,
            check_instruction_name,
            read_small_felt252_call,
        ]),
    );
}

#[test]
fn test_rel_jump_inc_ap() {
    test_jump_opcode(true, false, true, 3, 5, None);
}

#[test]
fn test_rel_big_op1() {
    test_jump_opcode(true, false, false, 411, 5, None);
}

pub fn assemble_jump(op1_off: i16, flags: &Flags) -> u64 {
    let jump_op1_off = flags
        .pc_update_jump_rel
        .map(|b| if b { 1 } else { op1_off })
        .unwrap();
    assemble_instruction(-1, -1, jump_op1_off, flags.clone().into())
}

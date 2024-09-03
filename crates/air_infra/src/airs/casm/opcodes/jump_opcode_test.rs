use super::super::casm_state::*;
use super::super::common::*;
use super::jump_opcode::*;

use crate::airs::memory::felt252_id_memory::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

// Macros
use crate::const_expr;
use crate::const_felt252_expr;
use crate::expr;

fn test_jump_opcode(
    is_rel_jump: bool,
    op1_base_fp: bool,
    ap_update_add_1: bool,
    op1: u128,
    offset_value: i16,
    entry_file_name: Option<&str>,
    expected_state: Vec<u32>,
) {
    // Create the air function
    let mut jump_opcode = JumpOpcode {
        is_rel: is_rel_jump,
        op1_base_fp,
        ap_update_add_1,
        memory: Felt252IdMemory::default(),
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
        const_felt252_expr!(
            assemble_jump(offset_value, &jump_opcode.get_flags()) as u128,
            0
        ),
    )];
    if is_rel_jump {
        memory_values.push((const_expr!(pc_value + 1), const_felt252_expr!(op1, 0)));
    } else if op1_base_fp {
        memory_values.push((
            const_expr!((fp_value as i16 + offset_value) as u32),
            const_felt252_expr!(op1, 0),
        ));
    } else {
        memory_values.push((
            const_expr!((ap_value as i16 + offset_value) as u32),
            const_felt252_expr!(op1, 0),
        ));
    }
    jump_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let registry = AirFnRegistry::new(&jump_opcode);
    let (state, next_state) =
        registry.run_air(&jump_opcode, CasmStateVar::new(pc, ap.clone(), fp.clone()));

    // Check output
    if is_rel_jump {
        assert_eq!(next_state.pc.calc(), (pc_value + op1 as u32).to_string());
    } else {
        assert_eq!(next_state.pc.calc(), op1.to_string());
    }
    assert_eq!(next_state.fp.calc(), fp.calc());
    if ap_update_add_1 {
        assert_eq!(next_state.ap.calc(), (ap_value + 1).to_string());
    } else {
        assert_eq!(next_state.ap.calc(), ap.calc());
    }

    // Check state
    assert_eq!(
        state.calc(),
        expected_state
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    );

    // Check entry
    if let Some(entry_file_name) = entry_file_name {
        compare_test_json(
            registry,
            &jump_opcode.name(),
            &(TEST_JSONS_OPCODES_DIR.to_owned() + entry_file_name),
        );
    }
}

#[test]
fn test_abs_jump_base_ap() {
    test_jump_opcode(
        false,
        false,
        false,
        8,
        2,
        Some("abs_jump_base_ap.json"),
        vec![3, 11, 6, 2, 0, 4, 0, 1, 8, 0, 0],
    );
}

#[test]
fn test_abs_jump_base_fp() {
    test_jump_opcode(
        false,
        true,
        false,
        5,
        10,
        None,
        vec![3, 11, 6, 10, 0, 4, 0, 1, 5, 0, 0],
    );
}

#[test]
fn test_abs_jump_base_ap_inc_ap() {
    test_jump_opcode(
        false,
        false,
        true,
        12,
        100,
        None,
        vec![3, 11, 6, 4, 6, 4, 0, 1, 12, 0, 0],
    );
}

#[test]
fn test_abs_jump_base_fp_inc_ap() {
    test_jump_opcode(
        false,
        true,
        true,
        20,
        17,
        None,
        vec![3, 11, 6, 1, 1, 4, 0, 1, 20, 0, 0],
    );
}

#[test]
fn test_abs_big_op1() {
    test_jump_opcode(
        false,
        false,
        false,
        1684685,
        402,
        None,
        vec![3, 11, 6, 2, 25, 4, 0, 1, 205, 218, 6],
    );
}

#[test]
fn test_abs_jump_negativ_offset() {
    test_jump_opcode(
        false,
        false,
        false,
        9,
        -9,
        None,
        vec![3, 11, 6, 7, 511, 3, 0, 1, 9, 0, 0],
    );
}

#[test]
fn test_rel_jump() {
    test_jump_opcode(
        true,
        false,
        false,
        100,
        5,
        Some("rel_jump.json"),
        vec![3, 11, 6, 0, 1, 0, 0, 100, 0, 0],
    );
}

#[test]
fn test_rel_jump_inc_ap() {
    test_jump_opcode(
        true,
        false,
        true,
        3,
        5,
        None,
        vec![3, 11, 6, 0, 1, 0, 0, 3, 0, 0],
    );
}

#[test]
fn test_rel_big_op1() {
    test_jump_opcode(
        true,
        false,
        false,
        411,
        5,
        None,
        vec![3, 11, 6, 0, 1, 0, 0, 411, 0, 0],
    );
}

pub fn assemble_jump(op1_off: i16, flags: &Flags) -> u64 {
    let jump_op1_off = flags
        .pc_update_jump_rel
        .map(|b| if b { 1 } else { op1_off })
        .unwrap();
    assemble_instruction(-1, -1, jump_op1_off, flags.clone().into())
}

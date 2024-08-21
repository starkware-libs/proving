use super::super::common::*;
use super::jump_opcode::*;

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
    expected_air_body: Option<&[&str]>,
    expected_state: Vec<u32>,
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
    assert_eq!(
        state.calc(),
        expected_state
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    );

    // Check air body
    if let Some(expected_air_body) = expected_air_body {
        let entry = registry.get_air_fn_entry(&jump_opcode);
        assert_eq!(
            entry
                .air_body
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
            expected_air_body
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
        Some(&[
            "tmp_0 = [\
                JumpOpcode_c84fbe8a2f33f1ef_input[0], \
                JumpOpcode_c84fbe8a2f33f1ef_input[1], \
                JumpOpcode_c84fbe8a2f33f1ef_input[2]\
            ]",
            "Deduction: tmp_0[0]",
            "Deduction: tmp_0[1]",
            "Deduction: tmp_0[2]",
            "(\
                [\
                    const_2147483646, \
                    const_2147483646, \
                    (((state[3] + (state[4] * const_16)) + (state[5] * const_8192)) - const_32768)\
                ], [\
                    const_true, \
                    const_true, \
                    const_false, \
                    const_false, \
                    const_true, \
                    const_false, \
                    const_false, \
                    const_true, \
                    const_false, \
                    const_false, \
                    const_false, \
                    const_false, \
                    const_false, \
                    const_false, \
                    const_false\
                ]\
            ) = DecodeInstruction_a4fdc221dc5c5f46(state[0])",
            "((state[6] + (state[7] * const_512)) + (state[8] * const_262144)) = \
                ReadAddr_d86123cf8dd732a9((\
                    state[1] + \
                    (((state[3] + (state[4] * const_16)) + (state[5] * const_8192)) - const_32768)\
                ))",
        ]),
        vec![3, 11, 6, 2, 0, 4, 8, 0, 0],
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
        vec![3, 11, 6, 10, 0, 4, 5, 0, 0],
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
        vec![3, 11, 6, 4, 6, 4, 12, 0, 0],
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
        vec![3, 11, 6, 1, 1, 4, 20, 0, 0],
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
        vec![3, 11, 6, 2, 25, 4, 205, 218, 6],
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
        vec![3, 11, 6, 7, 511, 3, 9, 0, 0],
    );
}

#[test]
fn test_rel_jump() {
    test_jump_opcode(true, false, false, 100, 5, Some(&[
        "tmp_0 = [\
            JumpOpcode_35e5e7be1094296e_input[0], \
            JumpOpcode_35e5e7be1094296e_input[1], \
            JumpOpcode_35e5e7be1094296e_input[2]\
        ]",
        "Deduction: tmp_0[0]",
        "Deduction: tmp_0[1]",
        "Deduction: tmp_0[2]",
        "(\
            [\
                const_2147483646, \
                const_2147483646, \
                const_1\
            ], [\
                const_true, \
                const_true, \
                const_true, \
                const_false, \
                const_false, \
                const_false, \
                const_false, \
                const_false, \
                const_true, \
                const_false, \
                const_false, \
                const_false, \
                const_false, \
                const_false, \
                const_false\
            ]\
        ) = DecodeInstruction_d5261ee7a67207d3(state[0])",
        "Felt252::from_limbs(zero_extend([state[3]])) = ReadSmallFelt252_cc824bd2f61c6ef6((state[0] + const_1))"
    ]), vec![3, 11, 6, 100]);
}

#[test]
fn test_rel_jump_inc_ap() {
    test_jump_opcode(true, false, true, 3, 5, None, vec![3, 11, 6, 3]);
}

#[test]
fn test_rel_big_op1() {
    test_jump_opcode(true, false, false, 411, 5, None, vec![3, 11, 6, 411]);
}

pub fn assemble_jump(op1_off: i16, flags: &Flags) -> u64 {
    let jump_op1_off = flags
        .pc_update_jump_rel
        .map(|b| if b { 1 } else { op1_off })
        .unwrap();
    assemble_instruction(-1, -1, jump_op1_off, flags.clone().into())
}

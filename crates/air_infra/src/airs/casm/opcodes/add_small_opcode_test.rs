use super::super::common::*;
use super::add_small_opcode::*;

use crate::core::air_fn_registry::*;
use crate::core::expressions::expr::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;

// Macros
use crate::const_expr;
use crate::expr;
use crate::felt252_expr;

fn test_add_small(
    non_consts_flags: [bool; 6],
    offset_values: [i16; 3],
    dst: u32,
    op0: u32,
    op1: u32,
    expected_air_body: Option<&[&str]>,
    expected_state: Vec<u32>,
) {
    // Read the non-constant flags
    let [flag_dst_base_fp, flag_op0_base_fp, flag_op1_imm, flag_op1_base_fp, flag_op1_base_ap, flag_ap_update_add_1] =
        non_consts_flags;

    let [offset_dst_val, offset0_val, mut offset1_val] = offset_values;
    if flag_op1_imm {
        offset1_val = 1;
    }

    // Create the air function
    let mut add_small_opcode = AddSmallOpcode {
        is_immediate: flag_op1_imm,
        memory: Memory::default(),
    };

    // Register values at opcode start
    let pc_value = 10;
    let ap_value = 50;
    let fp_value = 100;

    let pc = expr!("pc", pc_value);
    let ap = expr!("ap", ap_value);
    let fp = expr!("fp", fp_value);

    // Cretae the non-constant flags
    let non_consts_flags = if flag_op1_imm {
        vec![flag_dst_base_fp, flag_op0_base_fp, flag_ap_update_add_1]
    } else {
        vec![
            flag_dst_base_fp,
            flag_op0_base_fp,
            flag_op1_base_fp,
            flag_op1_base_ap,
            flag_ap_update_add_1,
        ]
    };

    // Fill memory
    let mut memory_values = vec![(
        pc.clone(),
        felt252_expr!(
            "op",
            assemble_instruction(
                offset_dst_val,
                offset0_val,
                offset1_val,
                add_small_opcode
                    .get_flags()
                    .non_constants_to_arr(non_consts_flags),
            ) as u128,
            0
        ),
    )];
    if flag_dst_base_fp {
        memory_values.push((
            const_expr!((fp_value as i16 + offset_dst_val) as u32),
            felt252_expr!("dst", dst as u128, 0),
        ));
    } else {
        memory_values.push((
            const_expr!((ap_value as i16 + offset_dst_val) as u32),
            felt252_expr!("dst", dst as u128, 0),
        ));
    };
    if flag_op0_base_fp {
        memory_values.push((
            const_expr!((fp_value as i16 + offset0_val) as u32),
            felt252_expr!("op0", op0 as u128, 0),
        ));
    } else {
        memory_values.push((
            const_expr!((ap_value as i16 + offset0_val) as u32),
            felt252_expr!("op0", op0 as u128, 0),
        ));
    }
    if flag_op1_imm {
        memory_values.push((
            const_expr!(pc_value + 1),
            felt252_expr!("op1", op1 as u128, 0),
        ));
    } else if flag_op1_base_fp {
        memory_values.push((
            const_expr!((fp_value as i16 + offset1_val) as u32),
            felt252_expr!("op1", op1 as u128, 0),
        ));
    } else {
        memory_values.push((
            const_expr!((ap_value as i16 + offset1_val) as u32),
            felt252_expr!("op1", op1 as u128, 0),
        ));
    };
    add_small_opcode.init_memory(&Memory::new_with_data(memory_values));

    // Run air function

    let registry = AirFnRegistry::new(&add_small_opcode);
    let (state, [next_pc, next_ap, next_fp]) =
        registry.run_air(&add_small_opcode, [pc.clone(), ap.clone(), fp.clone()]);

    // Check output
    assert_eq!(next_fp.calc(), fp.calc());
    if flag_ap_update_add_1 {
        assert_eq!(next_ap.calc(), (ap_value + 1).to_string());
    } else {
        assert_eq!(next_ap.calc(), ap.calc());
    }
    if flag_op1_imm {
        assert_eq!(next_pc.calc(), (pc_value + 2).to_string());
    } else {
        assert_eq!(next_pc.calc(), (pc_value + 1).to_string());
    };

    assert_eq!(
        state.calc(),
        expected_state
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    );

    // Check air body
    if let Some(expected_air_body) = expected_air_body {
        let entry = registry.get_air_fn_entry(&add_small_opcode);
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
fn test_add_small_not_imm() {
    test_add_small(
        [true, false, false, false, true, false],
        [3, 5, 7],
        90125677,
        77779999,
        12345678,
        Some(&[
            "tmp_0 = [\
                AddSmallOpcode_801397eb9925538_input[0], \
                AddSmallOpcode_801397eb9925538_input[1], \
                AddSmallOpcode_801397eb9925538_input[2]\
            ]",
            "Deduction: tmp_0[0]",
            "Deduction: tmp_0[1]",
            "Deduction: tmp_0[2]",
            "(\
                [\
                    (((state[3] + (state[4] * const_512)) + const_0) - const_32768), \
                    (((state[5] + (state[6] * const_4)) + (state[7] * const_2048)) - const_32768), \
                    (((state[8] + (state[9] * const_16)) + (state[10] * const_8192)) - const_32768)\
                ], \
                [\
                    Bool::from_m31(state[11]), \
                    Bool::from_m31(state[12]), \
                    const_false, \
                    Bool::from_m31(state[13]), \
                    Bool::from_m31(state[14]), \
                    const_true, \
                    const_false, \
                    const_false, \
                    const_false, \
                    const_false, \
                    const_false, \
                    Bool::from_m31(state[15]), \
                    const_false, \
                    const_false, \
                    const_true\
                ]\
            ) = DecodeInstruction_2f4ab58cc783b3f7(state[0])",
            "Felt252::from_limbs(zero_extend([state[16], state[17], state[18]])) = \
                ReadSmallFelt252_90f301596f69aa5a((\
                    ((state[11] * state[2]) + ((const_1 - state[11]) * state[1])) + \
                    (((state[3] + (state[4] * const_512)) + const_0) - const_32768)\
                ))",
            "Felt252::from_limbs(zero_extend([state[19], state[20], state[21]])) = \
                ReadSmallFelt252_90f301596f69aa5a((\
                    ((state[12] * state[2]) + ((const_1 - state[12]) * state[1])) + \
                    (((state[5] + (state[6] * const_4)) + (state[7] * const_2048)) - const_32768)\
                ))",
            "Constraint: ((state[13] + state[14]) - const_1)",
            "Felt252::from_limbs(zero_extend([state[22], state[23], state[24]])) = \
                ReadSmallFelt252_90f301596f69aa5a((\
                    ((state[13] * state[2]) + (state[14] * state[1])) + \
                    (((state[8] + (state[9] * const_16)) + (state[10] * const_8192)) - const_32768)\
                ))",
            "Constraint: (\
                ((state[16] + (const_512 * state[17])) + (const_262144 * state[18])) - \
                (((state[19] + (const_512 * state[20])) + (const_262144 * state[21])) + \
                ((state[22] + (const_512 * state[23])) + (const_262144 * state[24]))))",
        ]),
        vec![
            10, 50, 100, 3, 64, 1, 1, 16, 7, 0, 4, 1, 0, 0, 1, 0, 365, 410, 343, 31, 362, 296, 334,
            48, 47,
        ],
    );
}

#[test]
#[should_panic]
fn test_add_mod_not_equal() {
    test_add_small(
        [false, true, true, false, false, true],
        [3, 5, 7],
        90124653,
        77779999,
        12345678,
        None,
        vec![],
    );
}

#[test]
#[should_panic]
fn test_add_mod_over_27bit() {
    test_add_small(
        [false, true, true, false, false, true],
        [3, 5, 7],
        134217728,
        134217727,
        1,
        None,
        vec![],
    );
}

#[test]
fn test_add_small_imm() {
    test_add_small(
        [true, false, true, false, true, false],
        [-3, -5, 1],
        90125677,
        77779999,
        12345678,
        Some(&[
            "tmp_0 = [\
                AddSmallOpcode_7dff5a93938b26ba_input[0], \
                AddSmallOpcode_7dff5a93938b26ba_input[1], \
                AddSmallOpcode_7dff5a93938b26ba_input[2]\
            ]",
            "Deduction: tmp_0[0]",
            "Deduction: tmp_0[1]",
            "Deduction: tmp_0[2]",
            "(\
                [\
                    (((state[3] + (state[4] * const_512)) + const_0) - const_32768), \
                    (((state[5] + (state[6] * const_4)) + (state[7] * const_2048)) - const_32768), \
                    const_1\
                ], \
                [\
                    Bool::from_m31(state[8]), \
                    Bool::from_m31(state[9]), \
                    const_true, \
                    const_false, \
                    const_false, \
                    const_true, \
                    const_false, \
                    const_false, \
                    const_false, \
                    const_false, \
                    const_false, \
                    Bool::from_m31(state[10]), \
                    const_false, \
                    const_false, \
                    const_true\
                ]\
            ) = DecodeInstruction_7dd793dc3ba867a8(state[0])",
            "Felt252::from_limbs(zero_extend([state[11], state[12], state[13]])) = \
                ReadSmallFelt252_90f301596f69aa5a((\
                    ((state[8] * state[2]) + ((const_1 - state[8]) * state[1])) + \
                    (((state[3] + (state[4] * const_512)) + const_0) - const_32768)\
                ))",
            "Felt252::from_limbs(zero_extend([state[14], state[15], state[16]])) = \
                ReadSmallFelt252_90f301596f69aa5a((\
                    ((state[9] * state[2]) + ((const_1 - state[9]) * state[1])) + \
                    (((state[5] + (state[6] * const_4)) + (state[7] * const_2048)) - const_32768)\
                ))",
            "Felt252::from_limbs(zero_extend([state[17], state[18], state[19]])) = \
                ReadSmallFelt252_90f301596f69aa5a((state[0] + const_1))",
            "Constraint: (\
                ((state[11] + (const_512 * state[12])) + (const_262144 * state[13])) - \
                (((state[14] + (const_512 * state[15])) + (const_262144 * state[16])) + \
                ((state[17] + (const_512 * state[18])) + (const_262144 * state[19]))))",
        ]),
        vec![
            10, 50, 100, 509, 63, 3, 510, 15, 1, 0, 0, 365, 410, 343, 31, 362, 296, 334, 48, 47,
        ],
    );
}

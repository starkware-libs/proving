use super::super::casm_state::*;
use super::super::common::*;
use super::mul_small_opcode::*;

use crate::airs::memory::felt252_id_memory::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;

// Macros
use crate::const_expr;
use crate::expr;
use crate::felt252_expr;

fn test_mul_small(
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
    let mut mul_small_opcode = MulSmallOpcode {
        is_imm: flag_op1_imm,
        memory: Felt252IdMemory::default(),
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
                mul_small_opcode
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
    mul_small_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function

    let registry = AirFnRegistry::new(&mul_small_opcode);
    let (state, next_state) = registry.run_air(
        &mul_small_opcode,
        CasmStateVar::new(pc.clone(), ap.clone(), fp.clone()),
    );

    // Check output
    assert_eq!(next_state.fp.calc(), fp.calc());
    if flag_ap_update_add_1 {
        assert_eq!(next_state.ap.calc(), (ap_value + 1).to_string());
    } else {
        assert_eq!(next_state.ap.calc(), ap.calc());
    }
    if flag_op1_imm {
        assert_eq!(next_state.pc.calc(), (pc_value + 2).to_string());
    } else {
        assert_eq!(next_state.pc.calc(), (pc_value + 1).to_string());
    };

    assert_eq!(
        state.calc(),
        expected_state
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    );

    if let Some(expected_air_body) = expected_air_body {
        let entry = registry.get_air_fn_entry(&mul_small_opcode.name());
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
fn test_mul_small_not_imm() {
    test_mul_small(
        [true, false, false, false, true, false],
        [3, 5, 7],
        1042584088,
        32123,
        32456,
        Some(&[
            "tmp_0 = MulSmallOpcode_is_imm_false_input",
            "Deduction: tmp_0.pc",
            "Deduction: tmp_0.ap",
            "Deduction: tmp_0.fp",
            "(\
                [\
                    (((state[3] + (state[4] * const_512)) + const_0) - const_32768), \
                    (((state[5] + (state[6] * const_4)) + (state[7] * const_2048)) - const_32768), \
                    (((state[8] + (state[9] * const_16)) + (state[10] * const_8192)) - const_32768)\
                ], [\
                    Bool::from_m31(state[11]), \
                    Bool::from_m31(state[12]), \
                    const_false, Bool::from_m31(state[13]), \
                    Bool::from_m31(state[14]), \
                    const_false, \
                    const_true, \
                    const_false, \
                    const_false, \
                    const_false, \
                    const_false, \
                    Bool::from_m31(state[15]), \
                    const_false, const_false, \
                    const_true\
                ]\
            ) = DecodeInstruction_385376571fef8ea3(state[0])",
            "Felt252::from_limbs(zero_extend([state[18], state[19], state[20], state[21]])) = \
                ReadPositive_num_bits_30((\
                    ((state[11] * state[2]) + ((const_1 - state[11]) * state[1])) + \
                    (((state[3] + (state[4] * const_512)) + const_0) - const_32768)\
                ))",
            "Felt252::from_limbs(zero_extend([state[23], state[24]])) = \
                ReadPositive_num_bits_15((\
                    ((state[12] * state[2]) + ((const_1 - state[12]) * state[1])) + \
                    (((state[5] + (state[6] * const_4)) + (state[7] * const_2048)) - const_32768)\
                ))",
            "Constraint: ((state[13] + state[14]) - const_1)",
            "Felt252::from_limbs(zero_extend([state[26], state[27]])) = \
                ReadPositive_num_bits_15((\
                    ((state[13] * state[2]) + (state[14] * state[1])) + \
                    (((state[8] + (state[9] * const_16)) + (state[10] * const_8192)) - const_32768)\
                ))",
            "Constraint: (\
                (((state[18] + (const_512 * state[19])) + (const_262144 * state[20])) + (const_134217728 * state[21])) - \
                ((state[23] + (const_512 * state[24])) * (state[26] + (const_512 * state[27]))))"
        ]),
        vec![
            10, 50, 100, 3, 64, 1, 1, 16, 7, 0, 4, 1, 0, 0, 1, 0, 0, 1, 24, 73, 393, 7, 2, 379, 62, 3, 200, 63]
    );
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evalutate to 0)")]
fn test_mul_mod_not_equal() {
    test_mul_small(
        [false, true, true, false, false, true],
        [3, 5, 7],
        1042584088,
        32123,
        32457,
        None,
        vec![],
    );
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 0: RangeCheck6 on input 64")]
fn test_mul_mod_over_15bit() {
    test_mul_small(
        [false, true, true, false, false, true],
        [3, 5, 7],
        32768,
        32768,
        1,
        None,
        vec![],
    );
}

#[test]
fn test_mul_small_imm() {
    test_mul_small(
        [true, false, true, false, true, false],
        [-3, -5, 1],
        56,
        7,
        8,
        Some(&[
            "tmp_0 = MulSmallOpcode_is_imm_true_input",
            "Deduction: tmp_0.pc",
            "Deduction: tmp_0.ap",
            "Deduction: tmp_0.fp",
            "(\
                [\
                    (((state[3] + (state[4] * const_512)) + const_0) - const_32768), \
                    (((state[5] + (state[6] * const_4)) + (state[7] * const_2048)) - const_32768), \
                    const_1\
                ], [\
                    Bool::from_m31(state[8]), \
                    Bool::from_m31(state[9]), \
                    const_true, \
                    const_false, \
                    const_false, \
                    const_false, \
                    const_true, \
                    const_false, \
                    const_false, \
                    const_false, \
                    const_false, \
                    Bool::from_m31(state[10]), \
                    const_false, \
                    const_false, \
                    const_true\
                ]\
            ) = DecodeInstruction_22fdd1d4be52f69d(state[0])",
            "Felt252::from_limbs(zero_extend([state[13], state[14], state[15], state[16]])) = \
                ReadPositive_num_bits_30((\
                    ((state[8] * state[2]) + ((const_1 - state[8]) * state[1])) + \
                    (((state[3] + (state[4] * const_512)) + const_0) - const_32768)\
                ))",
            "Felt252::from_limbs(zero_extend([state[18], state[19]])) = \
                ReadPositive_num_bits_15((\
                    ((state[9] * state[2]) + ((const_1 - state[9]) * state[1])) + \
                    (((state[5] + (state[6] * const_4)) + (state[7] * const_2048)) - const_32768)\
                ))",
            "Felt252::from_limbs(zero_extend([state[21], state[22]])) = \
                ReadPositive_num_bits_15((state[0] + const_1))",
            "Constraint: (\
                (((state[13] + (const_512 * state[14])) + (const_262144 * state[15])) + (const_134217728 * state[16])) - \
                ((state[18] + (const_512 * state[19])) * (state[21] + (const_512 * state[22]))))"
        ]),
        vec![
            10, 50, 100, 509, 63, 3, 510, 15, 1, 0, 0, 0, 1, 56, 0, 0, 0, 2, 7, 0, 3, 8, 0],
    );
}

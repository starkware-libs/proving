use super::super::common::*;
use super::assert_eq_opcode::*;

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

// [fp + offset] == [ap + offset]
#[test]
#[should_panic]
fn test_assert_not_equal_deref() {
    test_assert_equal(
        [true, false, false, false, true, false],
        1,
        4,
        2,
        None,
        None,
    );
}

// [ap + offset] == [fp + offset]
#[test]
fn test_assert_equal_deref() {
    test_assert_equal(
        [false, false, false, true, false, false],
        3,
        4,
        3,
        None,
        None,
    );
}

// [ap + offset] == imm
#[test]
#[should_panic]
fn test_assert_not_equal_imm() {
    test_assert_equal(
        [false, false, true, false, false, false],
        3,
        4,
        5,
        None,
        None,
    );
}

// [fp + offset] == imm
#[test]
fn test_assert_equal_imm() {
    test_assert_equal(
        [true, false, true, false, false, false],
        3,
        6,
        3,
        None,
        None,
    );
}

// [ap + offset] == [[fp + offset] + offset]
#[test]
#[should_panic]
fn test_assert_not_equal_double_deref() {
    test_assert_equal(
        [false, true, false, false, false, false],
        15,
        6,
        16,
        None,
        None,
    );
}

// [fp + offset] == [[ap + offset] + offset]
#[test]
fn test_assert_equal_double_deref() {
    test_assert_equal(
        [true, false, false, false, false, false],
        15,
        4,
        15,
        None,
        None,
    );
}

#[test]
fn test_assert_equal_double_deref_big_op0() {
    test_assert_equal(
        [true, false, false, false, false, false],
        15,
        1546487,
        15,
        None,
        None,
    );
}

#[test]
#[should_panic]
fn test_assert_not_equal_double_deref_big_op0() {
    test_assert_equal(
        [true, false, false, false, false, false],
        15,
        454687,
        78,
        None,
        None,
    );
}

// [ap + offset] == [fp + offset]
#[test]
fn test_assert_equal_deref_constraint_deduction() {
    let check_instruction_offsets = &format!(
        "[{}, {}, {}]",
        "(((state[3] + (state[4] * const_512)) + const_0) - const_32768)",
        "const_2147483646",
        "(((state[5] + (state[6] * const_16)) + (state[7] * const_8192)) - const_32768)"
    );
    let check_instruction_flags = vec![
        "Bool::from_m31(state[8])",
        "Bool::from_m31(state[9])",
        "Bool::from_m31(state[10])",
        "Bool::from_m31(state[11])",
    ];
    let memory_read = &format!(
        "Felt252::from_m31_{} = {}({})",
        "(zero_extend([state[12]]))",
        "ReadSmallFelt252_cc824bd2f61c6ef6",
        "(((state[8] * state[2]) + ((const_1 - state[8]) * state[1])) + \
        (((state[3] + (state[4] * const_512)) + const_0) - const_32768))"
    );
    let memory_constraints = [
        "Constraint: ((state[9] + state[10]) - const_1)",
        &format!(
            "{}({}) == {}",
            "Memory_59f18133215d0936",
            "[(((state[9] * state[2]) + (state[10] * state[1])) + (((state[5] + \
        (state[6] * const_16)) + (state[7] * const_8192)) - const_32768))]",
            "zero_extend([state[12]])"
        ),
    ];
    test_assert_equal(
        [false, false, false, true, false, false],
        15,
        4,
        15,
        Some((
            check_instruction_offsets,
            &check_instruction_flags,
            "CheckInstruction_8541c1464a1fb5a8",
        )),
        Some((&[memory_read], &memory_constraints)),
    );
}

// [fp + offset] == imm
#[test]
fn test_assert_equal_imm_constraint_deduction() {
    let check_instruction_offsets = &format!(
        "[{}, {}, {}]",
        "(((state[3] + (state[4] * const_512)) + const_0) - const_32768)",
        "const_2147483646",
        "const_1"
    );
    let check_instruction_flags = ["Bool::from_m31(state[5])", "Bool::from_m31(state[6])"];
    let memory_read = &format!(
        "Felt252::from_m31_{} = {}({})",
        "(zero_extend([state[7]]))",
        "ReadSmallFelt252_cc824bd2f61c6ef6",
        "(((state[5] * state[2]) + ((const_1 - state[5]) * state[1])) + \
        (((state[3] + (state[4] * const_512)) + const_0) - const_32768))"
    );
    let memory_constraint = &format!(
        "{}({}) == {}",
        "Memory_59f18133215d0936", "[(state[0] + const_1)]", "zero_extend([state[7]])"
    );
    test_assert_equal(
        [true, false, true, false, false, false],
        15,
        4,
        15,
        Some((
            check_instruction_offsets,
            &check_instruction_flags,
            "CheckInstruction_a40b2fddf1b31684",
        )),
        Some((&[memory_read], &[memory_constraint])),
    );
}

// [fp + offset] == [[ap + offset] + offset]
#[test]
fn test_assert_equal_double_deref_constraint_deduction() {
    let check_instruction_offsets = &format!(
        "[{}, {}, {}]",
        "(((state[3] + (state[4] * const_512)) + const_0) - const_32768)",
        "(((state[5] + (state[6] * const_4)) + (state[7] * const_2048)) - const_32768)",
        "(((state[8] + (state[9] * const_16)) + (state[10] * const_8192)) - const_32768)"
    );
    let check_instruction_flags = [
        "Bool::from_m31(state[11])",
        "Bool::from_m31(state[12])",
        "Bool::from_m31(state[13])",
    ];
    let memory_reads: [&str; 2] = [
        &format!(
            "Felt252::from_m31_{} = {}({})",
            "(zero_extend([state[14]]))",
            "ReadSmallFelt252_cc824bd2f61c6ef6",
            "(((state[11] * state[2]) + ((const_1 - state[11]) * state[1])) + \
            (((state[3] + (state[4] * const_512)) + const_0) - const_32768))"
        ),
        &format!(
            "{} = {}({})",
            "((state[15] + (state[16] * const_512)) + (state[17] * const_262144))",
            "ReadAddr_d86123cf8dd732a9",
            "(((state[12] * state[2]) + ((const_1 - state[12]) * state[1])) + \
            (((state[5] + (state[6] * const_4)) + (state[7] * const_2048)) - const_32768))"
        ),
    ];
    let memory_constraint = &format!(
        "{}({}) == {}",
        "Memory_59f18133215d0936",
        "[(((state[15] + (state[16] * const_512)) + (state[17] * const_262144)) + \
        (((state[8] + (state[9] * const_16)) + (state[10] * const_8192)) - const_32768))]",
        "zero_extend([state[14]])"
    );
    test_assert_equal(
        [true, false, false, false, false, false],
        15,
        4,
        15,
        Some((
            check_instruction_offsets,
            &check_instruction_flags,
            "CheckInstruction_ebf000a2b9b432e4",
        )),
        Some((&memory_reads, &[memory_constraint])),
    );
}

fn test_assert_equal(
    non_consts_flags: [bool; 6],
    dst: u128,
    op0: u128,
    op1: u128,
    check_instruction_body: Option<(&str, &[&str], &str)>,
    memory_body: Option<(&[&str], &[&str])>,
) {
    // Read the non-constant flags
    let [flag_dst_base_fp, flag_op0_base_fp, flag_op1_imm, flag_op1_base_fp, flag_op1_base_ap, flag_ap_update_add_1] =
        non_consts_flags;

    // Create the air function
    let double_deref = !flag_op1_imm && !flag_op1_base_fp && !flag_op1_base_ap;
    let mut assert_equal_opcode = AssertEqOpcode {
        is_double_deref: double_deref,
        is_immediate: flag_op1_imm,
        memory: Memory::default(),
    };

    // Register values at opcode start
    let pc_value = 3;
    let ap_value = 11;
    let fp_value = 6;
    let offset0_value = 3;
    let offset1_value = if double_deref { 7 } else { -1 };
    let offset2_value = if flag_op1_imm { 1 } else { 2 };

    let pc = expr!("pc", pc_value);
    let ap = expr!("ap", ap_value);
    let fp = expr!("fp", fp_value);
    let offset0 = offset_as_u16(offset0_value);
    let offset1 = offset_as_u16(offset1_value);
    let offset2 = offset_as_u16(offset2_value);

    // Cretae the non-constant flags
    let non_consts_flags = if flag_op1_imm {
        vec![flag_dst_base_fp, flag_ap_update_add_1]
    } else if double_deref {
        vec![flag_dst_base_fp, flag_op0_base_fp, flag_ap_update_add_1]
    } else {
        vec![
            flag_dst_base_fp,
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
                offset0_value,
                offset1_value,
                offset2_value,
                assert_equal_opcode
                    .get_flags()
                    .non_constants_to_arr(non_consts_flags),
            ) as u128,
            0
        ),
    )];
    if flag_dst_base_fp {
        memory_values.push((
            const_expr!((fp_value as i16 + offset0_value) as u32),
            felt252_expr!("dst", dst, 0),
        ));
    } else {
        memory_values.push((
            const_expr!((ap_value as i16 + offset0_value) as u32),
            felt252_expr!("dst", dst, 0),
        ));
    };
    if flag_op1_imm {
        memory_values.push((const_expr!(pc_value + 1), felt252_expr!("op1", op1, 0)));
    } else if double_deref {
        memory_values.push((
            const_expr!((op0 as i32 + offset2_value as i32) as u32),
            felt252_expr!("op1", op1, 0),
        ));
        if flag_op0_base_fp {
            memory_values.push((
                const_expr!((fp_value as i16 + offset1_value) as u32),
                felt252_expr!("op0", op0, 0),
            ));
        } else {
            memory_values.push((
                const_expr!((ap_value as i16 + offset1_value) as u32),
                felt252_expr!("op0", op0, 0),
            ));
        }
    } else if flag_op1_base_fp {
        memory_values.push((
            const_expr!((fp_value as i16 + offset2_value) as u32),
            felt252_expr!("op1", op1, 0),
        ));
    } else {
        memory_values.push((
            const_expr!((ap_value as i16 + offset2_value) as u32),
            felt252_expr!("op1", op1, 0),
        ));
    };
    assert_equal_opcode.init_memory(&Memory::new_with_data(memory_values));

    // Run air function

    let registry = AirFnRegistry::new(&assert_equal_opcode);
    let (state, [next_pc, next_ap, next_fp]) =
        registry.run_air(&assert_equal_opcode, [pc.clone(), ap.clone(), fp.clone()]);

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

    // Check state
    let mut expected_state = vec![
        pc_value,
        ap_value,
        fp_value,
        (offset0 & 0x1FF) as u32,
        (offset0 >> 9) as u32,
    ];
    if double_deref {
        expected_state.push((offset1 & 0x3) as u32);
        expected_state.push(((offset1 >> 2) & 0x1FF) as u32);
        expected_state.push((offset1 >> 11) as u32);
    }
    if !flag_op1_imm {
        expected_state.push((offset2 & 0xF) as u32);
        expected_state.push(((offset2 >> 4) & 0x1FF) as u32);
        expected_state.push((offset2 >> 13) as u32);
    };
    expected_state.push(flag_dst_base_fp as u32);
    if double_deref {
        expected_state.push(flag_op0_base_fp as u32);
    }
    if !double_deref && !flag_op1_imm {
        expected_state.push(flag_op1_base_fp as u32);
        expected_state.push(flag_op1_base_ap as u32);
    }
    expected_state.push(flag_ap_update_add_1 as u32);
    expected_state.push(dst as u32);
    if double_deref {
        expected_state.push((op0 & 0x1FF) as u32);
        expected_state.push(((op0 >> 9) & 0x1FF) as u32);
        expected_state.push(((op0 >> 18) & 0x1FF) as u32);
    }

    assert_eq!(
        state.calc(),
        expected_state
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    );

    // Check air body
    if let Some((check_instruction_offsets, check_instruction_flags, check_instruction_name)) =
        check_instruction_body
    {
        let (memory_reads, memory_constraints) = memory_body.unwrap();
        let entry = registry.get_air_fn_entry(&assert_equal_opcode);
        let air_body = [
            &format!(
                "tmp_0 = [{name}_input[0], {name}_input[1], {name}_input[2]]",
                name = assert_equal_opcode.name()
            ),
            "Deduction: tmp_0[0]",
            "Deduction: tmp_0[1]",
            "Deduction: tmp_0[2]",
            &format!(
                "({}, [{}]) = {}({})",
                check_instruction_offsets,
                assert_equal_opcode
                    .get_flags()
                    .to_string(check_instruction_flags.to_vec()),
                check_instruction_name,
                "state[0]"
            ),
        ];
        let mut air_body_vec = air_body.to_vec();
        air_body_vec.append(&mut memory_reads.to_vec());
        air_body_vec.append(&mut memory_constraints.to_vec());

        assert_eq!(
            entry
                .air_body
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
            air_body_vec
        );
    }
}

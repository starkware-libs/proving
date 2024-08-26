use super::super::common::*;
use super::assert_eq_opcode::*;

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

// [fp + offset] == [ap + offset]
#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_assert_not_equal_deref() {
    test_assert_equal(
        [true, false, false, false, true, false],
        1,
        4,
        2,
        None,
        vec![],
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
        vec![3, 11, 6, 3, 64, 2, 0, 4, 0, 1, 0, 0, 0, 1],
    );
}

// [ap + offset] == imm
#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_assert_not_equal_imm() {
    test_assert_equal(
        [false, false, true, false, false, false],
        3,
        4,
        5,
        None,
        vec![],
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
        vec![3, 11, 6, 3, 64, 1, 0, 0, 1],
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
        vec![],
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
        vec![
            3, 11, 6, 3, 64, 3, 1, 16, 2, 0, 4, 1, 0, 0, 0, 2, 4, 0, 0, 1,
        ],
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
        vec![
            3, 11, 6, 3, 64, 3, 1, 16, 2, 0, 4, 1, 0, 0, 0, 2, 247, 460, 5, 1,
        ],
    );
}

#[test]
#[should_panic(expected = "given value != value in memory")]
fn test_assert_not_equal_double_deref_big_op0() {
    test_assert_equal(
        [true, false, false, false, false, false],
        15,
        454687,
        78,
        None,
        vec![],
    );
}

// [ap + offset] == [fp + offset]
#[test]
fn test_assert_equal_deref_constraint_deduction() {
    test_assert_equal(
        [false, false, false, true, false, false],
        15,
        4,
        15,
        Some(&[
            "tmp_0 = [\
                AssertEqOpcode_47a894d9c977ed99_input[0], \
                AssertEqOpcode_47a894d9c977ed99_input[1], \
                AssertEqOpcode_47a894d9c977ed99_input[2]\
            ]",
            "Deduction: tmp_0[0]",
            "Deduction: tmp_0[1]",
            "Deduction: tmp_0[2]",
            "(\
                [\
                    (((state[3] + (state[4] * const_512)) + const_0) - const_32768), \
                    const_2147483646, \
                    (((state[5] + (state[6] * const_16)) + (state[7] * const_8192)) - const_32768)\
                ], \
                [Bool::from_m31(state[8]), \
                const_true, \
                const_false, \
                Bool::from_m31(state[9]), \
                Bool::from_m31(state[10]), \
                const_false, \
                const_false, \
                const_false, \
                const_false, \
                const_false, \
                const_false, \
                Bool::from_m31(state[11]), \
                const_false, \
                const_false, \
                const_true]\
            ) = DecodeInstruction_8443cf0a4db7edc5(state[0])",
            "Constraint: ((state[9] + state[10]) - const_1)",
            "() = MemVerifyEqual_9275f6b821cf1219([\
                (((state[8] * state[2]) + ((const_1 - state[8]) * state[1])) + \
                (((state[3] + (state[4] * const_512)) + const_0) - const_32768)), \
                (((state[9] * state[2]) + (state[10] * state[1])) + \
                (((state[5] + (state[6] * const_16)) + (state[7] * const_8192)) - const_32768))\
            ])",
        ]),
        vec![3, 11, 6, 3, 64, 2, 0, 4, 0, 1, 0, 0, 0, 1],
    );
}

// [fp + offset] == imm
#[test]
fn test_assert_equal_imm_constraint_deduction() {
    test_assert_equal(
        [true, false, true, false, false, false],
        15,
        4,
        15,
        Some(&[
            "tmp_0 = [\
                AssertEqOpcode_47a894d9c977ed99_input[0], \
                AssertEqOpcode_47a894d9c977ed99_input[1], \
                AssertEqOpcode_47a894d9c977ed99_input[2]]",
            "Deduction: tmp_0[0]",
            "Deduction: tmp_0[1]",
            "Deduction: tmp_0[2]",
            "(\
                [\
                    (((state[3] + (state[4] * const_512)) + const_0) - const_32768), \
                    const_2147483646, \
                    const_1\
                ], \
                [\
                    Bool::from_m31(state[5]), \
                    const_true, \
                    const_true, \
                    const_false, \
                    const_false, \
                    const_false, \
                    const_false, \
                    const_false, \
                    const_false, \
                    const_false, \
                    const_false, \
                    Bool::from_m31(state[6]), \
                    const_false, \
                    const_false, \
                    const_true]\
            ) = DecodeInstruction_70368a7eef804c24(state[0])",
            "() = MemVerifyEqual_9275f6b821cf1219([\
                (((state[5] * state[2]) + ((const_1 - state[5]) * state[1])) + \
                (((state[3] + (state[4] * const_512)) + const_0) - const_32768)), \
                (state[0] + const_1)\
            ])",
        ]),
        vec![3, 11, 6, 3, 64, 1, 0, 0, 1],
    );
}

// [fp + offset] == [[ap + offset] + offset]
#[test]
fn test_assert_equal_double_deref_constraint_deduction() {
    let expected_air_body = [
        "tmp_0 = [\
            AssertEqOpcode_649fa2975275ca5d_input[0], \
            AssertEqOpcode_649fa2975275ca5d_input[1], \
            AssertEqOpcode_649fa2975275ca5d_input[2]\
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
                const_false, \
                const_false, \
                const_false, \
                const_false, \
                const_false, \
                const_false, \
                const_false, \
                const_false, \
                Bool::from_m31(state[13]), \
                const_false, \
                const_false, \
                const_true\
            ]\
        ) = DecodeInstruction_a06f5e7da24ead84(state[0])",
        "Felt252::from_limbs(zero_extend([state[16], state[17], state[18]])) = \
            ReadPositive_dd7d1f062646f801((\
                ((state[12] * state[2]) + ((const_1 - state[12]) * state[1])) + \
                (((state[5] + (state[6] * const_4)) + (state[7] * const_2048)) - const_32768)\
            ))",
        "() = MemVerifyEqual_9275f6b821cf1219(\
            [\
                (((state[11] * state[2]) + ((const_1 - state[11]) * state[1])) + \
                (((state[3] + (state[4] * const_512)) + const_0) - const_32768)), \
                (((state[16] + (state[17] * const_512)) + (state[18] * const_262144)) + \
                (((state[8] + (state[9] * const_16)) + (state[10] * const_8192)) - const_32768))\
            ])",
    ];
    let expected_state = vec![
        3, 11, 6, 3, 64, 3, 1, 16, 2, 0, 4, 1, 0, 0, 0, 2, 4, 0, 0, 1,
    ];
    test_assert_equal(
        [true, false, false, false, false, false],
        15,
        4,
        15,
        Some(&expected_air_body),
        expected_state,
    );
}

fn test_assert_equal(
    non_consts_flags: [bool; 6],
    dst: u128,
    op0: u128,
    op1: u128,
    expected_air_body: Option<&[&str]>,
    expected_state: Vec<u32>,
) {
    // Read the non-constant flags
    let [flag_dst_base_fp, flag_op0_base_fp, flag_op1_imm, flag_op1_base_fp, flag_op1_base_ap, flag_ap_update_add_1] =
        non_consts_flags;

    // Create the air function
    let double_deref = !flag_op1_imm && !flag_op1_base_fp && !flag_op1_base_ap;
    let mut assert_equal_opcode = AssertEqOpcode {
        is_double_deref: double_deref,
        is_immediate: flag_op1_imm,
        memory: Felt252IdMemory::default(),
    };

    let offset0_value = 3;
    let offset1_value = if double_deref { 7 } else { -1 };
    let offset2_value = if flag_op1_imm { 1 } else { 2 };

    // Register values at opcode start
    let pc_value = 3;
    let ap_value = 11;
    let fp_value = 6;
    let pc = expr!("pc", pc_value);
    let ap = expr!("ap", ap_value);
    let fp = expr!("fp", fp_value);

    // Create the non-constant flags
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
    assert_equal_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

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
    assert_eq!(
        state.calc(),
        expected_state
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    );

    // Check air body
    if let Some(expected_air_body) = expected_air_body {
        let entry = registry.get_air_fn_entry(&assert_equal_opcode.name());
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

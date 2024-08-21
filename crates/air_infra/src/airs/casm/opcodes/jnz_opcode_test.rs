use super::super::common::*;
use super::jnz_opcode::*;

use crate::core::air_fn_registry::*;
use crate::core::expressions::expr::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;

use crate::const_expr;
use crate::expr;
use crate::felt252_expr;

fn build_and_test(
    [is_taken, flag_dst_base_fp, flag_ap_update_add_1]: [bool; 3],
    offset_dst: i16,
    dst_value: Felt252Expr,
    op1_value: u32,
    expected_air_body: Option<&[&str]>,
    expected_state: Vec<u32>,
) {
    let [pc_value, ap_value, fp_value] = [50, 200, 150];
    let [pc, ap, fp] = [
        expr!("pc", pc_value),
        expr!("ap", ap_value),
        expr!("fp", fp_value),
    ];

    let mut jnz_opcode = JnzOpcode {
        is_taken,
        flag_dst_base_fp,
        flag_ap_update_add_1,
        memory: Memory::default(),
    };

    // Fill memory
    let mut memory_values = vec![(
        pc.clone(),
        felt252_expr!(
            "op",
            assemble_jnz(offset_dst, &jnz_opcode.get_flags()) as u128,
            0
        ),
    )];

    memory_values.push((
        const_expr!(pc_value + 1),
        felt252_expr!("op1_imm", op1_value as u128, 0),
    ));

    if flag_dst_base_fp {
        memory_values.push((
            const_expr!((fp_value as i16 + offset_dst) as u32),
            dst_value.clone(),
        ));
    } else {
        memory_values.push((
            const_expr!((ap_value as i16 + offset_dst) as u32),
            dst_value.clone(),
        ));
    }
    let memory = Memory::new_with_data(memory_values);

    jnz_opcode.init_memory(&memory);

    // Run air function
    let registry = AirFnRegistry::new(&jnz_opcode);
    let (state, [next_pc, next_ap, next_fp]) =
        registry.run_air(&jnz_opcode, [pc, ap.clone(), fp.clone()]);

    // Check output
    if is_taken {
        assert_eq!(next_pc.calc(), (pc_value + op1_value).to_string());
    } else {
        assert_eq!(next_pc.calc(), (pc_value + 2).to_string());
    }

    if flag_ap_update_add_1 {
        assert_eq!(next_ap.calc(), (ap_value + 1).to_string());
    } else {
        assert_eq!(next_ap.calc(), ap_value.to_string());
    }

    assert_eq!(next_fp.calc(), fp_value.to_string());

    // Check state
    assert_eq!(
        state.calc(),
        expected_state
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );

    // Check air_body
    let air_body = registry.get_air_fn_entry(&jnz_opcode).air_body;

    if let Some(expected_air_body) = expected_air_body {
        assert_eq!(
            air_body
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
            expected_air_body
        );
    }
}

#[test]
fn test_not_taken_zero_match_base_ap() {
    build_and_test(
        [false, false, false],
        -13,
        felt252_expr!("dst", 0, 0),
        15,
        Some(&[
            "tmp_0 = [\
                JnzOpcode_7f048efcd2fafd3f_input[0], \
                JnzOpcode_7f048efcd2fafd3f_input[1], \
                JnzOpcode_7f048efcd2fafd3f_input[2]]",
            "Deduction: tmp_0[0]",
            "Deduction: tmp_0[1]",
            "Deduction: tmp_0[2]",
            "(\
                [\
                    (((state[3] + (state[4] * const_512)) + const_0) - const_32768), \
                    const_2147483646, \
                    const_1\
                ], [\
                    const_false, \
                    const_true, \
                    const_true, \
                    const_false, \
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
                    const_false\
                ]\
            ) = DecodeInstruction_f07b2e63ffedf789(state[0])",
            "Felt252::from_limbs([\
                state[5], state[6], state[7], state[8], state[9], state[10], state[11], \
                state[12], state[13], state[14], state[15], state[16], state[17], state[18], \
                state[19], state[20], state[21], state[22], state[23], state[24], state[25], \
                state[26], state[27], state[28], state[29], state[30], state[31], state[32]\
            ]) = ReadSmallFelt252_bb908db6c9837328((\
                state[1] + (((state[3] + (state[4] * const_512)) + const_0) - const_32768)))",
            "Constraint: ((((((((((((((((((((((((((((const_0 + \
                state[5]) + state[6]) + state[7]) + state[8]) + state[9]) + state[10]) + state[11]) + \
                state[12]) + state[13]) + state[14]) + state[15]) + state[16]) + state[17]) + state[18]) + \
                state[19]) + state[20]) + state[21]) + state[22]) + state[23]) + state[24]) + state[25]) + \
                state[26]) + state[27]) + state[28]) + state[29]) + state[30]) + state[31]) + state[32])"
        ]),
        vec![
            50, 200, 150, 499, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ],
    );
}

#[test]
fn test_taken_match_base_ap() {
    build_and_test(
        [true, false, false],
        -13,
        felt252_expr!("dst", 123, 456),
        15,
        Some(&[
            "tmp_0 = [\
                JnzOpcode_384ff84280622c61_input[0], \
                JnzOpcode_384ff84280622c61_input[1], \
                JnzOpcode_384ff84280622c61_input[2]]",
            "Deduction: tmp_0[0]",
            "Deduction: tmp_0[1]",
            "Deduction: tmp_0[2]",
            "(\
                [\
                    (((state[3] + (state[4] * const_512)) + const_0) - const_32768), \
                    const_2147483646, \
                    const_1\
                ], [\
                    const_false, \
                    const_true, \
                    const_true, \
                    const_false, \
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
                    const_false\
                ]\
            ) = DecodeInstruction_f07b2e63ffedf789(state[0])",
            "Felt252::from_limbs([\
                state[5], state[6], state[7], state[8], state[9], state[10], state[11], \
                state[12], state[13], state[14], state[15], state[16], state[17], state[18], \
                state[19], state[20], state[21], state[22], state[23], state[24], state[25], \
                state[26], state[27], state[28], state[29], state[30], state[31], state[32]\
            ]) = ReadSmallFelt252_bb908db6c9837328((\
                state[1] + (((state[3] + (state[4] * const_512)) + const_0) - const_32768)))",
            "Deduction: (const_1 // ((((((((((((((((((((((((((((const_0 + \
                state[5]) + state[6]) + state[7]) + state[8]) + state[9]) + state[10]) + state[11]) + \
                state[12]) + state[13]) + state[14]) + state[15]) + state[16]) + state[17]) + state[18]) + \
                state[19]) + state[20]) + state[21]) + state[22]) + state[23]) + state[24]) + state[25]) + \
                state[26]) + state[27]) + state[28]) + state[29]) + state[30]) + state[31]) + state[32]))",
            "Constraint: ((((((((((((((((((((((((((((((const_0 + \
                state[5]) + state[6]) + state[7]) + state[8]) + state[9]) + state[10]) + state[11]) + \
                state[12]) + state[13]) + state[14]) + state[15]) + state[16]) + state[17]) + state[18]) + \
                state[19]) + state[20]) + state[21]) + state[22]) + state[23]) + state[24]) + state[25]) + \
                state[26]) + state[27]) + state[28]) + state[29]) + state[30]) + state[31]) + state[32]) * \
                state[33]) - const_1)",
            "tmp_9 = (state[5] - const_1)",
            "tmp_10 = (state[26] - const_136)",
            "tmp_11 = (state[32] - const_256)",
            "Deduction: (const_1 // ((((((((((((((((((((((((((((const_0 + \
                (tmp_9 * tmp_9)) + state[6]) + state[7]) + state[8]) + state[9]) + state[10]) + state[11]) + \
                state[12]) + state[13]) + state[14]) + state[15]) + state[16]) + state[17]) + state[18]) + \
                state[19]) + state[20]) + state[21]) + state[22]) + state[23]) + state[24]) + state[25]) + \
                (tmp_10 * tmp_10)) + state[27]) + state[28]) + state[29]) + state[30]) + state[31]) + (tmp_11 * tmp_11)))",
            "Constraint: ((((((((((((((((((((((((((((((const_0 + \
                (tmp_9 * tmp_9)) + state[6]) + state[7]) + state[8]) + state[9]) + state[10]) + state[11]) + \
                state[12]) + state[13]) + state[14]) + state[15]) + state[16]) + state[17]) + state[18]) + \
                state[19]) + state[20]) + state[21]) + state[22]) + state[23]) + state[24]) + state[25]) + \
                (tmp_10 * tmp_10)) + state[27]) + state[28]) + state[29]) + state[30]) + state[31]) + (tmp_11 * tmp_11)) * \
                state[34]) - const_1)", 
            "Felt252::from_limbs(zero_extend([state[35]])) = ReadSmallFelt252_cc824bd2f61c6ef6((state[0] + const_1))"
        ]),
        vec![
            50, 200, 150, 499, 63, 123, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 288, 3, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 1955558780, 500077285, 15,
        ],
    );
}

#[test]
#[should_panic(expected = "0 has no inverse")]
fn test_taken_zero_mismatch_base_ap() {
    build_and_test(
        [true, false, false],
        -13,
        felt252_expr!("dst", 0, 0),
        15,
        None,
        vec![],
    );
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evalutate to 0)")]
fn test_not_taken_mismatch_base_ap() {
    build_and_test(
        [false, false, false],
        -13,
        felt252_expr!("dst", 123, 4567),
        15,
        None,
        vec![],
    );
}

#[test]
#[should_panic(expected = "0 has no inverse")]
fn test_taken_p_mismatch_base_ap() {
    build_and_test(
        [true, false, false],
        -13,
        felt252_expr!("dst", 1, 17 * u128::pow(2, 64) + u128::pow(2, 123)),
        15,
        None,
        vec![],
    );
}

pub fn assemble_jnz(offset_dst: i16, flags: &Flags) -> u64 {
    assemble_instruction(offset_dst, -1, 1, flags.clone().into())
}

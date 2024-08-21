use super::super::common::*;
use super::call_opcode::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::expr::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;

use crate::const_expr;
use crate::expr;
use crate::felt252_expr;

fn build_and_test(
    flag_op1_base_fp: bool,
    offset2_option: Option<i16>,
    op1_value: u32,
    expected_air_body: &[&str],
    expected_state: Vec<u32>,
) {
    let [pc_value, ap_value, fp_value] = [50, 200, 150];
    let [pc, ap, fp] = [
        expr!("pc", pc_value),
        expr!("ap", ap_value),
        expr!("fp", fp_value),
    ];

    let is_rel = offset2_option.is_none();
    let offset2 = offset2_option.unwrap_or(1);

    let mut call_opcode = CallOpcode {
        is_rel,
        flag_op1_base_fp,
        memory: Memory::default(),
    };

    // Fill memory
    let mut memory_values = vec![(
        pc.clone(),
        felt252_expr!(
            "op",
            assemble_call(offset2, &call_opcode.get_flags()) as u128,
            0
        ),
    )];

    if is_rel {
        memory_values.push((
            const_expr!(pc_value + 1),
            felt252_expr!("op1_imm", op1_value as u128, 0),
        ));
    } else if flag_op1_base_fp {
        memory_values.push((
            const_expr!((fp_value as i16 + offset2) as u32),
            felt252_expr!("op1_fp", op1_value as u128, 0),
        ));
    } else {
        memory_values.push((
            const_expr!((ap_value as i16 + offset2) as u32),
            felt252_expr!("op1_ap", op1_value as u128, 0),
        ));
    }
    let memory = Memory::new_with_data(memory_values);

    call_opcode.init_memory(&memory);

    // Run air function
    let registry = AirFnRegistry::new(&call_opcode);
    let (state, [next_pc, next_ap, next_fp]) =
        registry.run_air(&call_opcode, [pc, ap.clone(), fp.clone()]);

    // Check output
    if is_rel {
        assert_eq!(next_pc.calc(), (pc_value + op1_value).to_string());
    } else {
        assert_eq!(next_pc.calc(), op1_value.to_string());
    }
    assert_eq!(next_ap.calc(), (ap_value + 2).to_string());
    assert_eq!(next_fp.calc(), (ap_value + 2).to_string());

    // Check state
    assert_eq!(
        state.calc(),
        expected_state
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );

    // Check air_body
    let air_body = registry.get_air_fn_entry(&call_opcode).air_body;

    assert_eq!(
        air_body
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        expected_air_body
    );
}

#[test]
fn test_relative_call() {
    build_and_test(false, None, 500, &[
        "tmp_0 = [\
            CallOpcode_ccf475fd29f10d2b_input[0], \
            CallOpcode_ccf475fd29f10d2b_input[1], \
            CallOpcode_ccf475fd29f10d2b_input[2]\
        ]",
        "Deduction: tmp_0[0]",
        "Deduction: tmp_0[1]",
        "Deduction: tmp_0[2]",
        "(\
            [const_0, const_1, const_1], \
            [\
                const_false, \
                const_false, \
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
                const_true, \
                const_false, \
                const_false\
            ]\
        ) = DecodeInstruction_8a7cb0cfbf63f85a(state[0])",
        "Memory_59f18133215d0936([state[1]]) == zero_extend([state[2]])",
        "Memory_59f18133215d0936([(state[1] + const_1)]) == zero_extend([(state[0] + const_2)])",
        "Felt252::from_limbs(zero_extend([state[3]])) = \
            ReadSmallFelt252_cc824bd2f61c6ef6((state[0] + const_1))"
    ], vec![50, 200, 150, 500]);
}

const CALL_FP_EXPECTED_AIR_BODY: [&str; 8] = [
    "tmp_0 = [\
        CallOpcode_572bef75c2ae21ea_input[0], \
        CallOpcode_572bef75c2ae21ea_input[1], \
        CallOpcode_572bef75c2ae21ea_input[2]\
    ]",
    "Deduction: tmp_0[0]",
    "Deduction: tmp_0[1]",
    "Deduction: tmp_0[2]",
    "(\
        [\
            const_0, \
            const_1, \
            (((state[3] + (state[4] * const_16)) + (state[5] * const_8192)) - const_32768)\
        ], [\
            const_false, \
            const_false, \
            const_false, \
            const_true, \
            const_false, \
            const_false, \
            const_false, \
            const_true, \
            const_false, \
            const_false, \
            const_false, \
            const_false, \
            const_true, \
            const_false, \
            const_false\
        ]\
    ) = DecodeInstruction_48b2fb68e2c629d6(state[0])",
    "Memory_59f18133215d0936([state[1]]) == zero_extend([state[2]])",
    "Memory_59f18133215d0936([(state[1] + const_1)]) == zero_extend([(state[0] + const_1)])",
    "((state[6] + (state[7] * const_512)) + (state[8] * const_262144)) = \
        ReadAddr_d86123cf8dd732a9((\
            state[2] + \
            (((state[3] + (state[4] * const_16)) + (state[5] * const_8192)) - const_32768)\
        ))",
];

#[test]
fn test_fp_call_positive_offset2() {
    build_and_test(
        true,
        Some(5),
        600,
        &CALL_FP_EXPECTED_AIR_BODY,
        vec![50, 200, 150, 5, 0, 4, 88, 1, 0],
    );
}

#[test]
fn test_fp_call_negative_offset2() {
    build_and_test(
        true,
        Some(-5),
        400,
        &CALL_FP_EXPECTED_AIR_BODY,
        vec![50, 200, 150, 11, 511, 3, 400, 0, 0],
    );
}

const CALL_AP_EXPECTED_AIR_BODY: [&str; 8] = [
    "tmp_0 = [\
        CallOpcode_9fc0c9c42043f0cc_input[0], \
        CallOpcode_9fc0c9c42043f0cc_input[1], \
        CallOpcode_9fc0c9c42043f0cc_input[2]\
    ]",
    "Deduction: tmp_0[0]",
    "Deduction: tmp_0[1]",
    "Deduction: tmp_0[2]",
    "(\
        [\
            const_0, \
            const_1, \
            (((state[3] + (state[4] * const_16)) + (state[5] * const_8192)) - const_32768)\
        ], [\
            const_false, \
            const_false, \
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
            const_true, \
            const_false, \
            const_false\
        ]\
    ) = DecodeInstruction_d682a34433babffb(state[0])",
    "Memory_59f18133215d0936([state[1]]) == zero_extend([state[2]])",
    "Memory_59f18133215d0936([(state[1] + const_1)]) == zero_extend([(state[0] + const_1)])",
    "((state[6] + (state[7] * const_512)) + (state[8] * const_262144)) = \
        ReadAddr_d86123cf8dd732a9((\
            state[1] + \
            (((state[3] + (state[4] * const_16)) + (state[5] * const_8192)) - const_32768)\
        ))",
];

#[test]
fn test_ap_call_positive_offset2() {
    build_and_test(
        false,
        Some(10),
        1234,
        &CALL_AP_EXPECTED_AIR_BODY,
        vec![50, 200, 150, 10, 0, 4, 210, 2, 0],
    );
}

#[test]
fn test_ap_call_negative_offset2() {
    build_and_test(
        false,
        Some(-10),
        55,
        &CALL_AP_EXPECTED_AIR_BODY,
        vec![50, 200, 150, 6, 511, 3, 55, 0, 0],
    );
}

pub fn assemble_call(offset2: i16, flags: &Flags) -> u64 {
    let call_op1_off = flags
        .pc_update_jump_rel
        .map(|b| if b { 1 } else { offset2 })
        .unwrap();
    assemble_instruction(0, 1, call_op1_off, flags.clone().into())
}

use super::super::casm_state::*;
use super::super::common::*;
use super::call_opcode::*;
use crate::airs::memory::felt252_id_memory::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;

use crate::const_expr;
use crate::const_felt252_expr;
use crate::expr;

fn build_and_test(
    op1_base_fp: bool,
    offset2_option: Option<i16>,
    op1_value: ((u128, u128), i32),
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
        op1_base_fp,
        memory: Felt252IdMemory::default(),
    };

    // Fill memory
    let mut memory_values = vec![(
        pc.clone(),
        const_felt252_expr!(assemble_call(offset2, &call_opcode.get_flags()) as u128, 0),
    )];

    let op1_value_252 = const_felt252_expr!(op1_value.0 .0, op1_value.0 .1);
    if is_rel {
        memory_values.push((const_expr!(pc_value + 1), op1_value_252));
    } else if op1_base_fp {
        memory_values.push((
            const_expr!((fp_value as i16 + offset2) as u32),
            op1_value_252,
        ));
    } else {
        memory_values.push((
            const_expr!((ap_value as i16 + offset2) as u32),
            op1_value_252,
        ));
    }

    memory_values.push((
        const_expr!(ap_value),
        const_felt252_expr!(fp_value as u128, 0),
    ));
    let ret_addr = pc_value + (if is_rel { 2 } else { 1 });
    memory_values.push((
        const_expr!(ap_value + 1),
        const_felt252_expr!(ret_addr as u128, 0),
    ));

    call_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let registry = AirFnRegistry::new(&call_opcode);
    let (state, next_state) =
        registry.run_air(&call_opcode, CasmStateVar::new(pc, ap.clone(), fp.clone()));

    // Check output
    if is_rel {
        assert_eq!(
            next_state.pc.calc(),
            (pc_value as i32 + op1_value.1).to_string()
        );
    } else {
        assert_eq!(next_state.pc.calc(), op1_value.1.to_string());
    }
    assert_eq!(next_state.ap.calc(), (ap_value + 2).to_string());
    assert_eq!(next_state.fp.calc(), (ap_value + 2).to_string());

    // Check state
    assert_eq!(
        state.calc(),
        expected_state
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );

    // Check air_body
    let air_body = registry.get_air_fn_entry(&call_opcode.name()).air_body;

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
    build_and_test(
        false,
        None,
        ((500, 0), 500),
        &[
            "tmp_0 = CallOpcode_is_rel_true_op1_base_fp_false_input",
            "Deduction: tmp_0.pc",
            "Deduction: tmp_0.ap",
            "Deduction: tmp_0.fp",
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
            "() = MemVerify((state[1], Felt252::from_limbs(zero_extend([state[2]]))))",
            "() = MemVerify((\
            (state[1] + const_1), \
            Felt252::from_limbs(zero_extend([(state[0] + const_2)]))\
        ))",
            "(((\
            (state[11] * const_262144) + \
            ((state[10] * const_512) + \
            state[9])) - \
            state[7]) - \
            (const_134217728 * state[8])) = \
            ReadSmall((state[0] + const_1))",
        ],
        vec![50, 200, 150, 0, 2, 3, 1, 0, 0, 500, 0, 0],
    );
}

#[test]
fn test_relative_call_negative() {
    build_and_test(
        false,
        None,
        (
            (
                340282366920938463463374607431768211440,
                10633823966279327296825105735305134079,
            ),
            -17,
        ),
        &[
            "tmp_0 = CallOpcode_is_rel_true_op1_base_fp_false_input",
            "Deduction: tmp_0.pc",
            "Deduction: tmp_0.ap",
            "Deduction: tmp_0.fp",
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
            "() = MemVerify((state[1], Felt252::from_limbs(zero_extend([state[2]]))))",
            "() = MemVerify((\
            (state[1] + const_1), \
            Felt252::from_limbs(zero_extend([(state[0] + const_2)]))\
        ))",
            "(((\
                (state[11] * const_262144) + \
                ((state[10] * const_512) + \
                state[9])) - \
                state[7]) - \
                (const_134217728 * state[8])) = \
                ReadSmall((state[0] + const_1))",
        ],
        vec![50, 200, 150, 0, 2, 3, 1, 1, 1, 496, 511, 511],
    );
}

const CALL_FP_EXPECTED_AIR_BODY: [&str; 8] = [
    "tmp_0 = CallOpcode_is_rel_false_op1_base_fp_true_input",
    "Deduction: tmp_0.pc",
    "Deduction: tmp_0.ap",
    "Deduction: tmp_0.fp",
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
    "() = MemVerify((state[1], Felt252::from_limbs(zero_extend([state[2]]))))",
    "() = MemVerify((\
        (state[1] + const_1), \
        Felt252::from_limbs(zero_extend([(state[0] + const_1)]))\
    ))",
    "Felt252::from_limbs(zero_extend([state[10], state[11], state[12]])) = \
        ReadPositive_num_bits_27((\
            state[2] + \
            (((state[3] + (state[4] * const_16)) + (state[5] * const_8192)) - const_32768)\
        ))",
];

#[test]
fn test_fp_call_positive_offset2() {
    build_and_test(
        true,
        Some(5),
        ((600, 0), 600),
        &CALL_FP_EXPECTED_AIR_BODY,
        vec![50, 200, 150, 5, 0, 4, 0, 2, 3, 1, 88, 1, 0],
    );
}

#[test]
fn test_fp_call_negative_offset2() {
    build_and_test(
        true,
        Some(-5),
        ((400, 0), 400),
        &CALL_FP_EXPECTED_AIR_BODY,
        vec![50, 200, 150, 11, 511, 3, 0, 2, 3, 1, 400, 0, 0],
    );
}

const CALL_AP_EXPECTED_AIR_BODY: [&str; 8] = [
    "tmp_0 = CallOpcode_is_rel_false_op1_base_fp_false_input",
    "Deduction: tmp_0.pc",
    "Deduction: tmp_0.ap",
    "Deduction: tmp_0.fp",
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
    "() = MemVerify((state[1], Felt252::from_limbs(zero_extend([state[2]]))))",
    "() = MemVerify((\
        (state[1] + const_1), \
        Felt252::from_limbs(zero_extend([(state[0] + const_1)]))\
    ))",
    "Felt252::from_limbs(zero_extend([state[10], state[11], state[12]])) = \
        ReadPositive_num_bits_27((\
            state[1] + \
            (((state[3] + (state[4] * const_16)) + (state[5] * const_8192)) - const_32768)\
    ))",
];

#[test]
fn test_ap_call_positive_offset2() {
    build_and_test(
        false,
        Some(10),
        ((1234, 0), 1234),
        &CALL_AP_EXPECTED_AIR_BODY,
        vec![50, 200, 150, 10, 0, 4, 0, 2, 3, 1, 210, 2, 0],
    );
}

#[test]
fn test_ap_call_negative_offset2() {
    build_and_test(
        false,
        Some(-10),
        ((55, 0), 55),
        &CALL_AP_EXPECTED_AIR_BODY,
        vec![50, 200, 150, 6, 511, 3, 0, 2, 3, 1, 55, 0, 0],
    );
}

pub fn assemble_call(offset2: i16, flags: &Flags) -> u64 {
    let call_op1_off = flags
        .pc_update_jump_rel
        .map(|b| if b { 1 } else { offset2 })
        .unwrap();
    assemble_instruction(0, 1, call_op1_off, flags.clone().into())
}

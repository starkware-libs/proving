use super::call_opcode::*;
use super::common::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::expr::Expr;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::*;

use crate::const_expr;
use crate::expr;
use crate::felt252_expr;

fn build_and_test(
    flag_op1_base_fp: bool,
    offset2_option: Option<i16>,
    op1_value: u32,
    check_instruction_name: &str,
    next_pc_line: String,
) {
    let [pc_value, ap_value, fp_value] = [50, 200, 150];
    let [pc, ap, fp] = [
        expr!("pc", pc_value),
        expr!("ap", ap_value),
        expr!("fp", fp_value),
    ];

    let is_rel = offset2_option.is_none();
    let offset2 = offset2_option.unwrap_or(1);

    let mut expected_state = vec![pc_value, ap_value, fp_value];
    if is_rel {
        expected_state.push(op1_value);
    } else {
        let offset2_u16 = offset_as_u16(offset2);
        expected_state.push((offset2_u16 & 0xF) as u32);
        expected_state.push(((offset2_u16 >> 4) & 0x1FF) as u32);
        expected_state.push((offset2_u16 >> 13) as u32);
        expected_state.push(op1_value & 0x1FF);
        expected_state.push(op1_value >> 9);
        expected_state.push((op1_value >> 18) & 0x1FF);
    }

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

    let check_instruction_offset2 = if is_rel {
        "const_1"
    } else {
        "(((state[3] + (state[4] * const_16)) + (state[5] * const_8192)) - const_32768)"
    };
    let check_instruction_call = format!(
        "([const_0, const_1, {}], {}) = {}(state[0])",
        check_instruction_offset2,
        call_opcode.get_flags(),
        check_instruction_name,
    );

    let expected_air_body = [
        format!(
            "deduction_tmp_0 = [{name}_input[0], {name}_input[1], {name}_input[2]]",
            name = call_opcode.name()
        ),
        "Deduction: deduction_tmp_0[0]".to_string(), // state[0] = pc
        "Deduction: deduction_tmp_0[1]".to_string(), // state[1] = ap
        "Deduction: deduction_tmp_0[2]".to_string(), // state[2] = fp
        check_instruction_call,
        format!("{}([state[1]]) == zero_extend([state[2]])", memory.name()),
        format!(
            "{}([(state[1] + const_1)]) == zero_extend([(state[0] + const_{})])",
            memory.name(),
            1 + is_rel as u32
        ),
        next_pc_line,
    ];

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
    let next_pc_line = format!(
        "{} = ReadSmallFelt252_cc824bd2f61c6ef6({})",
        "Felt252::from_m31_(zero_extend([state[3]]))", "(state[0] + const_1)",
    );
    build_and_test(
        false,
        None,
        500,
        "CheckInstruction_3e8ab07a0ff6ded2",
        next_pc_line,
    );
}

#[test]
fn test_fp_call_positive_offset2() {
    let next_pc_line = format!(
        "{} = ReadAddr_d86123cf8dd732a9({})",
        "((state[6] + (state[7] * const_512)) + (state[8] * const_262144))",
        "(state[2] + (((state[3] + (state[4] * const_16)) + (state[5] * const_8192)) - const_32768))",
    );
    build_and_test(
        true,
        Some(5),
        600,
        "CheckInstruction_617ed37adc670019",
        next_pc_line,
    );
}

#[test]
fn test_fp_call_negative_offset2() {
    let next_pc_line = format!(
        "{} = ReadAddr_d86123cf8dd732a9({})",
        "((state[6] + (state[7] * const_512)) + (state[8] * const_262144))",
        "(state[2] + (((state[3] + (state[4] * const_16)) + (state[5] * const_8192)) - const_32768))",
    );
    build_and_test(
        true,
        Some(-5),
        400,
        "CheckInstruction_617ed37adc670019",
        next_pc_line,
    );
}

#[test]
fn test_ap_call_positive_offset2() {
    let next_pc_line = format!(
        "{} = ReadAddr_d86123cf8dd732a9({})",
        "((state[6] + (state[7] * const_512)) + (state[8] * const_262144))",
        "(state[1] + (((state[3] + (state[4] * const_16)) + (state[5] * const_8192)) - const_32768))",
    );
    build_and_test(
        false,
        Some(10),
        1234,
        "CheckInstruction_3510bff4c5766846",
        next_pc_line,
    );
}

#[test]
fn test_ap_call_negative_offset2() {
    let next_pc_line = format!(
        "{} = ReadAddr_d86123cf8dd732a9({})",
        "((state[6] + (state[7] * const_512)) + (state[8] * const_262144))",
        "(state[1] + (((state[3] + (state[4] * const_16)) + (state[5] * const_8192)) - const_32768))",
    );
    build_and_test(
        false,
        Some(-10),
        55,
        "CheckInstruction_3510bff4c5766846",
        next_pc_line,
    );
}

pub fn assemble_call(offset2: i16, flags: &Flags) -> u64 {
    let call_op1_off = flags
        .pc_update_jump_rel
        .map(|b| if b { 1 } else { offset2 })
        .unwrap();
    assemble_instruction(0, 1, call_op1_off, flags.clone().into())
}

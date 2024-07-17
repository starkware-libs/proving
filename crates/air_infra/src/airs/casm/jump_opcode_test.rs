use crate::const_expr;
use crate::core::air_fn_registry::*;

use crate::core::expressions::expr::Expr;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::*;
use crate::expr;
use crate::felt252_expr;

use super::common::*;
use super::jump_opcode::*;

fn test_jump_opcode(
    opcode: &Flags,
    state_vec: Option<Vec<&str>>,
    deductions_vec: Option<Vec<&str>>,
    constarint_vec: Option<Vec<&str>>,
) {
    // Register values at opcode start
    let pc_value = 3;
    let ap_value = 11;
    let fp_value = 6;
    let offset_value = 5;

    let pc = expr!("pc", pc_value);
    let ap = expr!("ap", ap_value);
    let fp = expr!("fp", fp_value);
    let op1 = 8;

    // Read opcode
    let is_rel_jump = opcode.pc_update_jump_rel.unwrap();
    let op1_base_fp = opcode.op1_base_fp.unwrap();
    let ap_update_add_1 = opcode.ap_update_add_1.unwrap();

    // Fill memory
    let mut memory_values = vec![(
        pc.clone(),
        felt252_expr!("op", assemble_jump(offset_value, opcode) as u128, 0),
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
    let memory: Memory<FeltExpr, Felt252Expr> = Memory::new_with_data(memory_values);

    // Run air function
    let func = JumpOpcode {
        is_rel: is_rel_jump,
        flag_op1_base_fp: op1_base_fp,
        flag_ap_update_add_1: ap_update_add_1,
        memory: memory.clone(),
    };
    let registry = AirFnRegistry::new(&func);
    let (state, [next_pc, next_ap, next_fp]) =
        registry.run_air(&func, [pc, ap.clone(), fp.clone()]);

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
    if let Some(state_vec) = state_vec {
        assert_eq!(state.calc(), state_vec);
    }

    // Check deductions
    if let Some(deductions) = deductions_vec {
        let lists = registry.get_compiled_air_fn(&func);
        assert_eq!(
            lists
                .deductions
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
            deductions
        );
    }

    // Check constraints
    if let Some(constraints) = constarint_vec {
        let lists = registry.get_compiled_air_fn(&func);
        assert_eq!(
            lists
                .constraints
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
            constraints
        );
    }
}

#[test]
fn test_abs_jump_base_ap() {
    let state_list = vec!["3", "11", "6", "5", "2048", "8"];
    test_jump_opcode(
        &create_flags(false, false, false),
        Some(state_list),
        None,
        None,
    );
}

#[test]
fn test_abs_jump_base_fp() {
    let state_list = vec!["3", "11", "6", "5", "2048", "8"];
    test_jump_opcode(
        &create_flags(false, true, false),
        Some(state_list),
        None,
        None,
    );
}

#[test]
fn test_abs_jump_base_ap_inc_ap() {
    let state_list = vec!["3", "11", "6", "5", "2048", "8"];
    test_jump_opcode(
        &create_flags(false, false, true),
        Some(state_list),
        None,
        None,
    );
}

#[test]
fn test_abs_jump_base_fp_inc_ap() {
    let state_list = vec!["3", "11", "6", "5", "2048", "8"];
    test_jump_opcode(
        &create_flags(false, true, true),
        Some(state_list),
        None,
        None,
    );
}

#[test]
fn test_rel_jump() {
    let state_list = vec!["3", "11", "6", "8"];
    test_jump_opcode(
        &create_flags(true, false, false),
        Some(state_list),
        None,
        None,
    );
}

#[test]
fn test_rel_jump_inc_ap() {
    let state_list = vec!["3", "11", "6", "8"];
    test_jump_opcode(
        &create_flags(true, false, true),
        Some(state_list),
        None,
        None,
    );
}

#[test]
fn test_abs_jump_deduction_constraints() {
    let deductions = vec![
    "deduction_tmp_0 = [JumpOpcode__false__false__false_input[0], JumpOpcode__false__false__false_input[1], JumpOpcode__false__false__false_input[2]]",
    "deduction_tmp_0[0]",
    "deduction_tmp_0[1]", 
    "deduction_tmp_0[2]",
    "deduction_tmp_4 = Memory__FeltExpr__Felt252Expr(state[0])",
    "deduction_tmp_5 = ((UInt32::from_felt(deduction_tmp_4.get_felt(const_2)) >> const_8) & const_15)",
    "deduction_tmp_5.low().as_felt()",
    "deduction_tmp_6 = RangeCheck4(state[3])",
    "deduction_tmp_4.get_felt(const_3)", 
    "deduction_tmp_7 = Memory__FeltExpr__Felt252Expr((state[1] + ((state[3] + (state[4] * const_16)) - const_32768)))",
    "deduction_tmp_7.get_felt(const_0)"
    ];
    let constraints = vec![
    "RangeCheck4([state[3]]) == []",
    "Memory__FeltExpr__Felt252Expr([state[0]]) == [const_4095, const_4087, (const_127 + (state[3] * const_256)), state[4], const_147]",
    "Memory__FeltExpr__Felt252Expr([(state[1] + ((state[3] + (state[4] * const_16)) - const_32768))]) == [state[5]]"
    ];
    test_jump_opcode(
        &create_flags(false, false, false),
        None,
        Some(deductions),
        Some(constraints),
    );
}

#[test]
fn test_rel_jump_deduction_constraints() {
    let deductions = vec![
        "deduction_tmp_0 = [JumpOpcode__false__false__true_input[0], JumpOpcode__false__false__true_input[1], JumpOpcode__false__false__true_input[2]]",
        "deduction_tmp_0[0]",
        "deduction_tmp_0[1]",
        "deduction_tmp_0[2]",
        "deduction_tmp_2 = Memory__FeltExpr__Felt252Expr(state[0])",
        "deduction_tmp_3 = Memory__FeltExpr__Felt252Expr((state[0] + const_1))",
        "deduction_tmp_3.get_felt(const_0)",
    ];
    let constraints = vec![
        "Memory__FeltExpr__Felt252Expr([state[0]]) == [const_4095, const_4087, const_383, const_2048, const_263]",
        "Memory__FeltExpr__Felt252Expr([(state[0] + const_1)]) == [state[3]]"
    ];
    test_jump_opcode(
        &create_flags(true, false, false),
        None,
        Some(deductions),
        Some(constraints),
    );
}

pub fn assemble_jump(op1_off: i16, flags: &Flags) -> u64 {
    let jump_op1_off = flags
        .pc_update_jump_rel
        .map(|b| if b { 1 } else { op1_off })
        .unwrap();
    assemble_instruction(-1, -1, jump_op1_off, flags.clone().into())
}

fn create_flags(is_rel: bool, fp_based: bool, ap_add_1: bool) -> Flags {
    let ap_based = if is_rel { false } else { !fp_based };
    Flags {
        dst_base_fp: Some(true),
        op0_base_fp: Some(true),
        op1_imm: Some(is_rel),
        op1_base_fp: Some(fp_based),
        op1_base_ap: Some(ap_based),
        res_add: Some(false),
        res_mul: Some(false),
        pc_update_jump: Some(!is_rel),
        pc_update_jump_rel: Some(is_rel),
        pc_update_jnz: Some(false),
        ap_update_add: Some(false),
        ap_update_add_1: Some(ap_add_1),
        opcode_call: Some(false),
        opcode_ret: Some(false),
        opcode_assert_eq: Some(false),
    }
}

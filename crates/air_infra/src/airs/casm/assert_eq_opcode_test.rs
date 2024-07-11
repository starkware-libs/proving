use crate::const_expr;
use crate::core::air_fn_registry::*;
use crate::core::expressions::expr::Expr;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::Felt252;
use crate::expr;
use crate::felt252_expr;

use super::assert_eq_op::*;
use super::common::*;

// [fp + offset] == [ap + offset]
#[test]
#[should_panic]
fn test_assert_not_equal_deref() {
    let opcode = create_flags(true, false, false, false, true, false);
    assert_equal(opcode, 1, 4, 2, None, None, None);
}

// [ap + offset] == [fp + offset]
#[test]
fn test_assert_equal_deref() {
    let opcode = create_flags(false, false, false, true, false, false);
    let state_vec = vec!["3", "11", "6", "3", "8", "2", "2048", "3", "3"];
    assert_equal(opcode, 3, 4, 3, Some(state_vec), None, None);
}

// [ap + offset] == imm
#[test]
#[should_panic]
fn test_assert_not_equal_imm() {
    let opcode = create_flags(false, false, true, false, false, false);
    assert_equal(opcode, 3, 4, 5, None, None, None);
}

// [fp + offset] == imm
#[test]
fn test_assert_equal_imm() {
    let opcode = create_flags(true, false, true, false, false, false);
    let state_vec = vec!["3", "11", "6", "3", "8", "3", "3"];
    assert_equal(opcode, 3, 6, 3, Some(state_vec), None, None);
}

// [ap + offset] == [[fp + offset] + offset]
#[test]
#[should_panic]
fn test_assert_not_equal_double_deref() {
    let opcode = create_flags(false, true, false, false, false, false);
    assert_equal(opcode, 15, 6, 16, None, None, None);
}

// [fp + offset] == [[ap + offset] + offset]
#[test]
fn test_assert_equal_double_deref() {
    let opcode = create_flags(true, false, false, false, false, false);
    // pc, ap, fp, opcode[0]
    let state_vec = vec![
        "3", "11", "6", "3", "8", "7", "128", "2", "2048", "15", "4", "15",
    ];
    assert_equal(opcode, 15, 4, 15, Some(state_vec), None, None);
}

// [ap + offset] == [fp + offset]
#[test]
fn test_assert_equal_deref_constraint_deduction() {
    let opcode = create_flags(false, false, false, true, false, false);
    let deductions = vec![
        "deduction_tmp_0 = [AssertEqOpcode__false__false__false__false__true__false_input[0], AssertEqOpcode__false__false__false__false__true__false_input[1], AssertEqOpcode__false__false__false__false__true__false_input[2]]",
        "deduction_tmp_0[0]", 
        "deduction_tmp_0[1]", 
        "deduction_tmp_0[2]", 
        "deduction_tmp_6 = Memory__FeltExpr__Felt252Expr(state[0])", 
        "deduction_tmp_6.get_felt(const_0)", 
        "deduction_tmp_7 = (UInt32::from_felt(deduction_tmp_6.get_felt(const_1)) & const_15)", 
        "deduction_tmp_7.low().as_felt()", 
        "deduction_tmp_8 = RangeCheck4(state[4])", 
        "deduction_tmp_9 = ((UInt32::from_felt(deduction_tmp_6.get_felt(const_2)) >> const_8) & const_15)", 
        "deduction_tmp_9.low().as_felt()", 
        "deduction_tmp_10 = RangeCheck4(state[5])", 
        "deduction_tmp_6.get_felt(const_3)", 
        "deduction_tmp_11 = Memory__FeltExpr__Felt252Expr((state[1] + (state[3] + (state[4] * const_4096))))", 
        "deduction_tmp_11.get_felt(const_0)", 
        "deduction_tmp_12 = Memory__FeltExpr__Felt252Expr((state[2] + (state[5] + (state[6] * const_16))))", 
        "deduction_tmp_12.get_felt(const_0)"
    ];
    let constraints = vec![
        "RangeCheck4([state[4]]) == []", 
        "RangeCheck4([state[5]]) == []", 
        "Memory__FeltExpr__Felt252Expr([state[0]]) == [state[3], (state[4] + const_4080), (const_127 + (state[5] * const_256)), state[6], const_8, const_4]", 
        "Memory__FeltExpr__Felt252Expr([(state[1] + (state[3] + (state[4] * const_4096)))]) == [state[7]]", 
        "Memory__FeltExpr__Felt252Expr([(state[2] + (state[5] + (state[6] * const_16)))]) == [state[8]]", 
        "(state[7] - state[8])"
    ];
    assert_equal(opcode, 15, 4, 15, None, Some(deductions), Some(constraints));
}

// [fp + offset] == imm
#[test]
fn test_assert_equal_imm_constraint_deduction() {
    let opcode = create_flags(true, false, true, false, false, false);
    let deductions = vec![
        "deduction_tmp_0 = [AssertEqOpcode__false__true__false__false__false__true_input[0], AssertEqOpcode__false__true__false__false__false__true_input[1], AssertEqOpcode__false__true__false__false__false__true_input[2]]", 
        "deduction_tmp_0[0]", 
        "deduction_tmp_0[1]", 
        "deduction_tmp_0[2]", 
        "deduction_tmp_4 = Memory__FeltExpr__Felt252Expr(state[0])", 
        "deduction_tmp_4.get_felt(const_0)", 
        "deduction_tmp_5 = (UInt32::from_felt(deduction_tmp_4.get_felt(const_1)) & const_15)", 
        "deduction_tmp_5.low().as_felt()", 
        "deduction_tmp_6 = RangeCheck4(state[4])", 
        "deduction_tmp_7 = Memory__FeltExpr__Felt252Expr((state[2] + (state[3] + (state[4] * const_4096))))", 
        "deduction_tmp_7.get_felt(const_0)", 
        "deduction_tmp_8 = Memory__FeltExpr__Felt252Expr((state[0] + const_32769))", 
        "deduction_tmp_8.get_felt(const_0)"
    ];
    let constraints = vec![
        "RangeCheck4([state[4]]) == []", 
        "Memory__FeltExpr__Felt252Expr([state[0]]) == [state[3], (state[4] + const_4080), const_383, const_2048, const_5, const_4]", 
        "Memory__FeltExpr__Felt252Expr([(state[2] + (state[3] + (state[4] * const_4096)))]) == [state[5]]", 
        "Memory__FeltExpr__Felt252Expr([(state[0] + const_32769)]) == [state[6]]", 
        "(state[5] - state[6])"
    ];
    assert_equal(opcode, 15, 4, 15, None, Some(deductions), Some(constraints));
}

// [fp + offset] == [[ap + offset] + offset]
#[test]
fn test_assert_equal_double_deref_constraint_deduction() {
    let opcode = create_flags(true, false, false, false, false, false);
    let deductions = vec![
        "deduction_tmp_0 = [AssertEqOpcode__false__true__false__false__false__false_input[0], AssertEqOpcode__false__true__false__false__false__false_input[1], AssertEqOpcode__false__true__false__false__false__false_input[2]]", 
        "deduction_tmp_0[0]", 
        "deduction_tmp_0[1]", 
        "deduction_tmp_0[2]", 
        "deduction_tmp_10 = Memory__FeltExpr__Felt252Expr(state[0])", 
        "deduction_tmp_10.get_felt(const_0)", 
        "deduction_tmp_11 = (UInt32::from_felt(deduction_tmp_10.get_felt(const_1)) & const_15)", 
        "deduction_tmp_11.low().as_felt()", 
        "deduction_tmp_12 = RangeCheck4(state[4])", 
        "deduction_tmp_13 = ((UInt32::from_felt(deduction_tmp_10.get_felt(const_1)) >> const_4) & const_255)", 
        "deduction_tmp_13.low().as_felt()", 
        "deduction_tmp_14 = RangeCheck8(state[5])", 
        "deduction_tmp_15 = (UInt32::from_felt(deduction_tmp_10.get_felt(const_2)) & const_255)", 
        "deduction_tmp_15.low().as_felt()", 
        "deduction_tmp_16 = RangeCheck8(state[6])", 
        "deduction_tmp_17 = ((UInt32::from_felt(deduction_tmp_10.get_felt(const_2)) >> const_8) & const_15)", 
        "deduction_tmp_17.low().as_felt()", 
        "deduction_tmp_18 = RangeCheck4(state[7])", 
        "deduction_tmp_10.get_felt(const_3)", 
        "deduction_tmp_19 = Memory__FeltExpr__Felt252Expr((state[2] + (state[3] + (state[4] * const_4096))))", 
        "deduction_tmp_19.get_felt(const_0)", 
        "deduction_tmp_20 = Memory__FeltExpr__Felt252Expr((state[1] + (state[5] + (state[6] * const_256))))", 
        "deduction_tmp_20.get_felt(const_0)", 
        "deduction_tmp_21 = Memory__FeltExpr__Felt252Expr((state[10] + (state[7] + (state[8] * const_16))))", 
        "deduction_tmp_21.get_felt(const_0)"
    ];
    let constraints = vec![
        "RangeCheck4([state[4]]) == []",
        "RangeCheck8([state[5]]) == []", 
        "RangeCheck8([state[6]]) == []", 
        "RangeCheck4([state[7]]) == []", 
        "Memory__FeltExpr__Felt252Expr([state[0]]) == [state[3], (state[4] + (state[5] * const_16)), (state[6] + (state[7] * const_256)), state[8], const_1, const_4]", 
        "Memory__FeltExpr__Felt252Expr([(state[2] + (state[3] + (state[4] * const_4096)))]) == [state[9]]", 
        "Memory__FeltExpr__Felt252Expr([(state[1] + (state[5] + (state[6] * const_256)))]) == [state[10]]", 
        "Memory__FeltExpr__Felt252Expr([(state[10] + (state[7] + (state[8] * const_16)))]) == [state[11]]", 
        "(state[9] - state[11])"
        ];
    assert_equal(opcode, 15, 4, 15, None, Some(deductions), Some(constraints));
}

fn assert_equal(
    opcode: Flags,
    op0: u128,
    op1: u128,
    op2: u128,
    state_vec: Option<Vec<&str>>,
    dedecuctions_vec: Option<Vec<&str>>,
    constarint_vec: Option<Vec<&str>>,
) {
    //  Read the opcode
    let dst_base_fp = opcode.dst_base_fp.unwrap();
    let op0_base_fp = opcode.op0_base_fp.unwrap();
    let op1_imm = opcode.op1_imm.unwrap();
    let op1_base_fp = opcode.op1_base_fp.unwrap();
    let op1_base_ap = opcode.op1_base_ap.unwrap();
    let ap_update_add_1 = opcode.ap_update_add_1.unwrap();
    let double_deref = !op1_imm && !op1_base_fp && !op1_base_ap;

    // Register values at opcode start
    let pc_value = 3;
    let ap_value = 11;
    let fp_value = 6;
    let offset0_value = 3;
    let offset1_value = if double_deref { 7 } else { -1 };
    let offset2_value = if op1_imm { 1 } else { 2 };

    let pc = expr!("pc", pc_value);
    let ap = expr!("ap", ap_value);
    let fp = expr!("fp", fp_value);
    let offset0 = offset_as_u16(offset0_value);
    let offset1 = offset_as_u16(offset1_value);
    let offset2 = offset_as_u16(offset2_value);

    // Fill memory
    let mut memory_values = vec![(
        pc.clone(),
        felt252_expr!(
            "op",
            assemble_instruction(
                offset0_value,
                offset1_value,
                offset2_value,
                opcode.clone().into()
            ) as u128,
            0
        ),
    )];
    if dst_base_fp {
        memory_values.push((
            const_expr!(fp_value + offset0 as u32),
            felt252_expr!("op0", op0, 0),
        ));
    } else {
        memory_values.push((
            const_expr!(ap_value + offset0 as u32),
            felt252_expr!("op0", op0, 0),
        ));
    };
    if op1_imm {
        memory_values.push((
            const_expr!(pc_value + offset_as_u16(1) as u32),
            felt252_expr!("op2", op2, 0),
        ));
    } else if double_deref {
        memory_values.push((
            const_expr!(op1 as u32 + offset2 as u32),
            felt252_expr!("op2", op2, 0),
        ));
        if op0_base_fp {
            memory_values.push((
                const_expr!(fp_value + offset1 as u32),
                felt252_expr!("op1", op1, 0),
            ));
        } else {
            memory_values.push((
                const_expr!(ap_value + offset1 as u32),
                felt252_expr!("op1", op1, 0),
            ));
        }
    } else if op1_base_fp {
        memory_values.push((
            const_expr!(fp_value + offset2 as u32),
            felt252_expr!("op2", op2, 0),
        ));
    } else {
        memory_values.push((
            const_expr!(ap_value + offset2 as u32),
            felt252_expr!("op2", op2, 0),
        ));
    };
    let memory: Memory<FeltExpr, Felt252Expr> = Memory::new_with_data(memory_values);

    // Run air function
    let func = AssertEqOpcode {
        flag_dst_base_fp: dst_base_fp,
        flag_op0_base_fp: op0_base_fp,
        flag_op1_imm: op1_imm,
        flag_op1_base_fp: op1_base_fp,
        flag_op1_base_ap: op1_base_ap,
        flag_ap_update_add_1: ap_update_add_1,
        memory,
    };
    let registry = AirFnRegistry::new(&func);
    let (state, [next_pc, next_ap, next_fp]) =
        registry.run_air(&func, [pc.clone(), ap.clone(), fp.clone()]);

    // Check output
    assert_eq!(next_fp.calc(), fp.calc());
    assert_eq!(next_ap.calc(), ap.calc());
    assert_eq!(next_pc.calc(), pc.calc());

    // Check state
    if let Some(state_vec) = state_vec {
        assert_eq!(state.calc(), state_vec);
    }

    // Check deducations
    if let Some(deductions) = dedecuctions_vec {
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

fn create_flags(
    flag_dst_base_fp: bool,
    flag_op0_base_fp: bool,
    flag_op1_imm: bool,
    flag_op1_base_fp: bool,
    flag_op1_base_ap: bool,
    flag_ap_update_add_1: bool,
) -> Flags {
    Flags {
        dst_base_fp: Some(flag_dst_base_fp),
        op0_base_fp: Some(flag_op0_base_fp),
        op1_imm: Some(flag_op1_imm),
        op1_base_fp: Some(flag_op1_base_fp),
        op1_base_ap: Some(flag_op1_base_ap),
        res_add: Some(false),
        res_mul: Some(false),
        pc_update_jump: Some(false),
        pc_update_jump_rel: Some(false),
        pc_update_jnz: Some(false),
        ap_update_add: Some(false),
        ap_update_add_1: Some(flag_ap_update_add_1),
        opcode_call: Some(false),
        opcode_ret: Some(false),
        opcode_assert_eq: Some(true),
    }
}

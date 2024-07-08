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
use super::ret_opcode::*;

pub fn assemble_ret() -> u64 {
    let ret_dst_off = -2;
    let ret_op0_off = -1;
    let ret_op1_off = -1;
    assemble_instruction(ret_dst_off, ret_op0_off, ret_op1_off, RET_FLAGS.into())
}

#[test]
fn test_ret_opcode() {
    let deductions = [
        "deduction_tmp_0 = [RetOpcode_input[0], RetOpcode_input[1], RetOpcode_input[2]]",
        "deduction_tmp_0[0]",
        "deduction_tmp_0[1]",
        "deduction_tmp_0[2]",
        "deduction_tmp_2 = Memory__FeltExpr__Felt252Expr(state[0])",
        "deduction_tmp_4 = Memory__FeltExpr__Felt252Expr((state[2] - const_1))",
        "deduction_tmp_4.get_felt(const_0)",
        "deduction_tmp_4.get_felt(const_1)",
        "deduction_tmp_5 = Memory__FeltExpr__Felt252Expr((state[2] - const_2))",
        "deduction_tmp_5.get_felt(const_0)",
        "deduction_tmp_5.get_felt(const_1)",
    ];

    // Register values at opcode start
    let pc_value = 3;
    let fp_value = 6;
    let ap_value = 11;

    // Old values of pc, fp saved by the last call opcode
    let saved_fp = 4;
    let saved_pc = 1;

    let pc: FeltExpr = expr!("pc", pc_value);
    let ap: FeltExpr = expr!("ap", ap_value);
    let fp: FeltExpr = expr!("fp", fp_value);

    // Fill memory
    let memory = Memory::<FeltExpr, Felt252Expr>::new_with_data(vec![
        (pc.clone(), felt252_expr!("op", assemble_ret() as u128, 0)),
        (
            const_expr!(fp_value - 1),
            felt252_expr!("saved_pc", saved_pc, 0),
        ),
        (
            const_expr!(fp_value - 2),
            felt252_expr!("saved_fp", saved_fp, 0),
        ),
    ]);

    // Run opcode and check output
    let mut func = RetOpcode::default();
    func.init_memory(&memory);
    let registry = AirFnRegistry::new(&func);

    let (state, output) = registry.run_air(&func, [pc, ap, fp]);

    let [next_pc, next_ap, next_fp] = output;
    assert_eq!(next_pc.calc(), saved_pc.to_string());
    assert_eq!(next_fp.calc(), saved_fp.to_string());
    assert_eq!(next_ap.calc(), ap_value.to_string());
    assert_eq!(state.calc(), ["3", "11", "6", "1", "0", "4", "0"]);

    let lists = registry.get_codegen_air_fn(&func);

    assert!(
        lists
            .deductions
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            == deductions
    );
}

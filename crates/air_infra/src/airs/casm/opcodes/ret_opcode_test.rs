use super::super::casm_state::*;
use super::super::common::*;
use super::ret_opcode::*;

use crate::airs::memory::felt252_id_memory::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

use crate::const_expr;
use crate::const_felt252_expr;
use crate::expr;

pub fn assemble_ret() -> u64 {
    let ret_off_0 = -2;
    let ret_off_1 = -1;
    let ret_off_2 = -1;
    assemble_instruction(ret_off_0, ret_off_1, ret_off_2, RET_FLAGS.into())
}

#[test]
fn test_ret_opcode() {
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
    let memory = Felt252IdMemory::new_with_data(vec![
        (pc.clone(), const_felt252_expr!(assemble_ret() as u128, 0)),
        (const_expr!(fp_value - 1), const_felt252_expr!(saved_pc, 0)),
        (const_expr!(fp_value - 2), const_felt252_expr!(saved_fp, 0)),
    ]);

    // Run opcode and check output
    let func = RetOpcode { memory };
    let registry = AirFnRegistry::new(&func);

    let (state, output) = registry.run_air(&func, CasmStateVar::new(pc, ap, fp));

    assert_eq!(output.pc.calc(), saved_pc.to_string());
    assert_eq!(output.fp.calc(), saved_fp.to_string());
    assert_eq!(output.ap.calc(), ap_value.to_string());
    assert_eq!(
        state.calc(),
        ["3", "11", "6", "0", "1", "1", "0", "0", "2", "4", "0", "0"]
    );

    // Check entry
    compare_test_json(
        &registry,
        &func.name(),
        &(TEST_JSONS_OPCODES_DIR.to_owned() + "ret.json"),
    );
}

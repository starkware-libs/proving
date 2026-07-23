use air_infra::casm_state::CasmStateVar;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::variables::AsProverType;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_felt252_expr};
use expect_test::expect;

use super::ret_opcode::*;
use crate::casm::common::*;

pub fn assemble_ret() -> u128 {
    let ret_off_0 = -2;
    let ret_off_1 = -1;
    let ret_off_2 = -1;
    assemble_instruction(ret_off_0, ret_off_1, ret_off_2, RET_FLAGS.into(), OpcodeExtension::Stone)
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

    let pc: FeltExpr = const_expr!(pc_value);
    let ap: FeltExpr = const_expr!(ap_value);
    let fp: FeltExpr = const_expr!(fp_value);

    // Fill memory
    let memory = Felt252IdMemory::new_with_data(vec![
        (pc.clone(), const_felt252_expr!(assemble_ret(), 0)),
        (const_expr!(fp_value - 1), const_felt252_expr!(saved_pc, 0)),
        (const_expr!(fp_value - 2), const_felt252_expr!(saved_fp, 0)),
    ]);

    // Run opcode and check output
    let func = RetOpcode { memory };
    let (registry, _) = AirFnRegistry::new(&func);

    let (state, output) = registry.run_air(&func, (), CasmStateVar::new(pc, ap, fp));

    assert_eq!(output.pc().calc(), saved_pc.to_string());
    assert_eq!(output.fp().calc(), saved_fp.to_string());
    assert_eq!(output.ap().calc(), ap_value.to_string());
    expect![[r#"
        (1, "enabler"),
        (3, "input_pc"),
        (11, "input_ap"),
        (6, "input_fp"),
        (1, "next_pc_id"),
        (1, "next_pc_limb_0"),
        (0, "next_pc_limb_1"),
        (0, "next_pc_limb_2"),
        (0, "next_pc_limb_3"),
        (0, "partial_limb_msb"),
        (2, "next_fp_id"),
        (4, "next_fp_limb_0"),
        (0, "next_fp_limb_1"),
        (0, "next_fp_limb_2"),
        (0, "next_fp_limb_3"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

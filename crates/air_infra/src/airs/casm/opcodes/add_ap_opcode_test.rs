use super::super::casm_state::*;
use super::super::common::*;
use super::add_ap_opcode::*;

use crate::airs::memory::felt252_id_memory::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

// Macros
use crate::const_expr;
use crate::const_felt252_expr;
use crate::expr;

#[test]
fn test_add_ap() {
    // build the air function
    let mut add_ap_opcode = AddAp {
        memory: Felt252IdMemory::default(),
    };

    // Register values at opcode start
    let pc_value = 30;
    let ap_value = 11;
    let fp_value = 6;
    let immediate = 299;

    let pc = expr!("pc", pc_value);
    let ap = expr!("ap", ap_value);
    let fp = expr!("fp", fp_value);

    // Fill memory
    let mut memory_values = vec![(
        pc.clone(),
        const_felt252_expr!(
            assemble_instruction(-1, -1, 1, add_ap_opcode.get_flags().into()) as u128,
            0
        ),
    )];
    memory_values.push((const_expr!(pc_value + 1), const_felt252_expr!(immediate, 0)));
    add_ap_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let registry = AirFnRegistry::new(&add_ap_opcode);
    let (state, next_state) = registry.run_air(
        &add_ap_opcode,
        CasmStateVar::new(pc, ap.clone(), fp.clone()),
    );

    // Check the output
    assert_eq!(next_state.pc.calc(), (pc_value + 2).to_string());
    assert_eq!(next_state.fp.calc(), (fp_value).to_string());
    assert_eq!(
        next_state.ap.calc(),
        (ap_value + immediate as u32).to_string()
    );

    // Check the state
    let expected_state = [30, 11, 6, 0, 1, 0, 0, 299, 0, 0];
    assert_eq!(
        state.calc(),
        expected_state
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    );

    // Check entry
    compare_test_json(
        registry,
        &add_ap_opcode.name(),
        &(TEST_JSONS_OPCODES_DIR.to_owned() + "add_ap.json"),
    );
}

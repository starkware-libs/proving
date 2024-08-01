use super::common::*;
use super::add_ap_opcode::*;

use crate::airs::casm::check_instruction;
use crate::core::memory::Memory;
use crate::core::air_fn_registry::AirFnRegistry;
use crate::core::memory::MemoryAirFn;
use crate::core::expressions::expr::*;
use crate::core::prover_types::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;

// Macros
use crate::expr;
use crate::felt252_expr;
use crate::const_expr;


#[test]
fn test_add_ap_small_imm() {
    test_add_ap(103);
}


fn test_add_ap(immediate: u128) {
    // build the air function
    let mut add_ap_opcode = AddAp {
        memory: Memory::default()
    };

    // Register values at opcode start
    let pc_value = 30;
    let ap_value = 11;
    let fp_value = 6;

    let pc = expr!("pc", pc_value);
    let ap = expr!("ap", ap_value);
    let fp = expr!("fp", fp_value);

    // Fill memory
    let mut memory_values = vec![(
        pc.clone(),
        felt252_expr!("op", assemble_instruction(-1,-1, 1, add_ap_opcode.get_flags().into()) as u128, 0),
    )];
    memory_values.push((const_expr!(pc_value + 1), felt252_expr!("imm", immediate, 0)));
    add_ap_opcode.init_memory(&Memory::new_with_data(memory_values));

    // Run air function
    let registry = AirFnRegistry::new(&add_ap_opcode);
    let (state, [next_pc, next_ap, next_fp]) =
        registry.run_air(&add_ap_opcode, [pc, ap.clone(), fp.clone()]);

    // Check the output
    assert_eq!(next_pc.calc(), (pc_value + 2 as u32).to_string());
    assert_eq!(next_fp.calc(), (fp_value).to_string());
    assert_eq!(next_ap.calc(), (ap_value + immediate as u32).to_string());

    // Check the state
    let expected_state = vec![pc_value, ap_value, fp_value, immediate as u32];
    assert_eq!(
        state.calc(),
        expected_state
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    );

    // Check the air body
    // let check_instruction_offsets = "[const_2147483646, const_2147483646, const_1]";
    // let entry = registry.get_air_fn_entry(&add_ap_opcode);
    // assert_eq!(
    //     entry
    //         .air_body
    //         .iter()
    //         .map(|x| x.to_string())
    //         .collect::<Vec<String>>(),
    //         vec!["hey"]
    // );
    
}

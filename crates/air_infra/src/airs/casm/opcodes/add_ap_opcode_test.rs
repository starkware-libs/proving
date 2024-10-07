use super::super::casm_state::*;
use super::super::common::*;
use super::add_ap_opcode::*;

use crate::airs::memory::felt252_id_memory::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

// Macros
use crate::const_expr;
use crate::const_felt252_expr;

#[test]
fn test_entry_json() {
    let (_, entry) = AirFnRegistry::new(&AddApOpcode {
        is_imm: false,
        op1_base_fp: false,
        memory: Felt252IdMemory::default(),
    });
    compare_json(
        &entry,
        &format!(
            "{}{}.json",
            TEST_JSONS_OPCODES_DIR,
            entry.name.to_lowercase()
        ),
    );

    let (_, entry) = AirFnRegistry::new(&AddApOpcode {
        is_imm: true,
        op1_base_fp: false,
        memory: Felt252IdMemory::default(),
    });
    compare_json(
        &entry,
        &format!(
            "{}{}.json",
            TEST_JSONS_OPCODES_DIR,
            entry.name.to_lowercase()
        ),
    );

    let (_, entry) = AirFnRegistry::new(&AddApOpcode {
        is_imm: false,
        op1_base_fp: true,
        memory: Felt252IdMemory::default(),
    });
    compare_json(
        &entry,
        &format!(
            "{}{}.json",
            TEST_JSONS_OPCODES_DIR,
            entry.name.to_lowercase()
        ),
    );
}

#[test]
fn test_add_ap_negative_imm() {
    // build the air function
    let mut add_ap_opcode = AddApOpcode {
        is_imm: true,
        op1_base_fp: false,
        memory: Felt252IdMemory::default(),
    };

    // Register values at opcode start
    let pc = 30;
    let ap = 11;
    let fp = 6;

    // Fill memory
    let mut memory_values = vec![(
        const_expr!(pc),
        const_felt252_expr!(
            assemble_instruction(-1, -1, 1, add_ap_opcode.get_flags().into()) as u128,
            0
        ),
    )];
    memory_values.push((const_expr!(pc + 1), const_felt252_expr!(-1i128)));
    add_ap_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (registry, _) = AirFnRegistry::new(&add_ap_opcode);
    let (state, next_state) = registry.run_air(
        &add_ap_opcode,
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check the output
    assert_eq!(next_state.pc.calc(), (pc + 2).to_string());
    assert_eq!(next_state.fp.calc(), (fp).to_string());
    assert_eq!(next_state.ap.calc(), (ap - 1).to_string());

    // Check the state
    let expected_state = [
        "30", // pc
        "11", // ap
        "6",  // fp
        "1",  // op1 id
        "1",  // op1 (sign)
        "0",  // op1 (sign)
        "0",  // op1
        "0",  // op1
        "0",  // op1
    ];
    assert_eq!(
        state.calc(),
        expected_state
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    );
}

#[test]
fn test_add_ap_deref_base_fp() {
    // build the air function
    let mut add_ap_opcode = AddApOpcode {
        is_imm: false,
        op1_base_fp: true,
        memory: Felt252IdMemory::default(),
    };

    // Register values at opcode start
    let pc = 30;
    let ap = 11;
    let fp = 6;
    let op1 = 299;
    let offset2 = 400;

    // Fill memory
    let mut memory_values = vec![(
        const_expr!(pc),
        const_felt252_expr!(
            assemble_instruction(-1, -1, offset2, add_ap_opcode.get_flags().into()) as u128,
            0
        ),
    )];
    memory_values.push((
        const_expr!((fp as i16 + offset2) as u32),
        const_felt252_expr!(op1),
    ));
    add_ap_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (registry, _) = AirFnRegistry::new(&add_ap_opcode);
    let (state, next_state) = registry.run_air(
        &add_ap_opcode,
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check the output
    assert_eq!(next_state.pc.calc(), (pc + 1).to_string());
    assert_eq!(next_state.fp.calc(), (fp).to_string());
    assert_eq!(next_state.ap.calc(), (ap + op1 as u32).to_string());

    // Check the state
    let expected_state = [
        "30",    // pc
        "11",    // ap
        "6",     // fp
        "33168", // offset2
        "1",     // op1_id
        "0",     // op1 (sign)
        "0",     // op1(sign)
        "299",   // op1
        "0",     // op1
        "0",     // op1
    ];
    assert_eq!(state.calc(), expected_state);
}

#[test]
#[should_panic(expected = "FLAG_OP1_IMM and FLAG_OP1_BASE_FP cannot be set at the same time.")]
fn test_failed_op1_src() {
    // build the air function
    let mut add_ap_opcode = AddApOpcode {
        is_imm: true,
        op1_base_fp: true,
        memory: Felt252IdMemory::default(),
    };

    // Register values at opcode start
    let pc = 30;
    let ap = 11;
    let fp = 6;
    let op1 = 299;
    let offset2 = 400;

    // Fill memory
    let mut memory_values = vec![(
        const_expr!(pc),
        const_felt252_expr!(
            assemble_instruction(-1, -1, offset2, add_ap_opcode.get_flags().into()) as u128,
            0
        ),
    )];
    memory_values.push((
        const_expr!((fp as i16 + offset2) as u32),
        const_felt252_expr!(op1),
    ));
    add_ap_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (registry, _) = AirFnRegistry::new(&add_ap_opcode);
    registry.run_air(
        &add_ap_opcode,
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
}

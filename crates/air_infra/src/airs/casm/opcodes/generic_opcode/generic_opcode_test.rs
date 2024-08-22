use super::generic_opcode::*;

use crate::airs::casm::casm_state::CasmStateVar;
use crate::airs::casm::common::*;
use crate::airs::casm::opcodes::assert_eq_opcode::*;
use crate::airs::casm::opcodes::call_opcode::*;
use crate::airs::casm::opcodes::jnz_opcode::*;
use crate::airs::casm::opcodes::jump_opcode::*;
use crate::airs::casm::opcodes::jump_opcode_test::*;
use crate::airs::casm::opcodes::ret_opcode::*;
use crate::airs::casm::opcodes::ret_opcode_test::*;
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
    let (_, entry) = AirFnRegistry::new(&GenericOpcode::default());
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
fn test_generic_call() {
    let mut generic_opcode = GenericOpcode::default();
    let mut call_opcode = CallOpcode {
        is_rel: true,
        op1_base_fp: false,
        memory: Felt252IdMemory::default(),
    };

    // Register values at opcode start
    let [pc, ap, fp] = [50, 200, 150];
    let immediate = 299;

    // Fill memory
    let mut memory_values = vec![(
        const_expr!(pc),
        const_felt252_expr!(
            assemble_instruction(0, 1, 1, call_opcode.get_flags().into()) as u128,
            0
        ),
    )];
    memory_values.push((const_expr!(pc + 1), const_felt252_expr!(immediate)));
    memory_values.push((const_expr!(ap), const_felt252_expr!(fp as i64)));
    memory_values.push((const_expr!(ap + 1), const_felt252_expr!(pc as i64 + 2)));

    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());
    call_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (mut registry, _) = AirFnRegistry::new(&generic_opcode);
    registry.add_entry(&call_opcode);
    let (state, output) = registry.run_air(
        &generic_opcode,
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
    let (_, expected_output) = registry.run_air(
        &call_opcode,
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check output
    assert_eq!(expected_output.calc(), output.calc());

    // Check state
    let expected_state = [
        50, 200, 150, 32768, 32769, 32769, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 2, 150, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 52, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 299, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 351, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 188, 30, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 960, 30, 0, 0, 0, 0,
        0, 60, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 299, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1285643938, 243381480, 0,
        0, 0, 349,
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
fn test_generic_ret() {
    let mut generic_opcode = GenericOpcode::default();
    let mut ret_opcode = RetOpcode {
        memory: Felt252IdMemory::default(),
    };
    // Register values at opcode start
    let [pc, ap, fp] = [3, 11, 6];

    // Old values of pc, fp saved by the last call opcode
    let saved_fp = 4;
    let saved_pc = 1;

    // Fill memory
    let memory_values = vec![
        (
            const_expr!(pc),
            const_felt252_expr!(assemble_ret() as u128, 0),
        ),
        (const_expr!(fp - 1), const_felt252_expr!(saved_pc)),
        (const_expr!(fp - 2), const_felt252_expr!(saved_fp)),
    ];
    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());
    ret_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (mut registry, _) = AirFnRegistry::new(&generic_opcode);
    registry.add_entry(&ret_opcode);
    let (state, output) = registry.run_air(
        &generic_opcode,
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
    let (_, expected_output) = registry.run_air(
        &ret_opcode,
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check output
    assert_eq!(expected_output.calc(), output.calc());

    // Check state
    let expected_state = [
        3, 11, 6, 32766, 32767, 32767, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 2, 4, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2029429668, 536870912, 0, 0, 0, 4,
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
fn test_generic_assert_equal() {
    let mut generic_opcode = GenericOpcode::default();
    let mut assert_equal_opcode = AssertEqOpcode {
        is_double_deref: false,
        is_imm: false,
        memory: Felt252IdMemory::default(),
    };
    let [offset0, offset1, offset2] = [3, -1, 2];
    let dst = 3;
    let op1 = 3;

    // Register values at opcode start
    let [pc, ap, fp] = [3, 11, 6];

    // Fill memory
    let mut memory_values = vec![(
        const_expr!(pc),
        const_felt252_expr!(
            assemble_instruction(
                offset0,
                offset1,
                offset2,
                assert_equal_opcode
                    .get_flags()
                    .non_constants_to_arr(&[false, true, false, false]),
            ) as u128,
            0
        ),
    )];
    memory_values.push((
        const_expr!((ap as i16 + offset0) as u32),
        const_felt252_expr!(dst as i128),
    ));
    memory_values.push((
        const_expr!((fp as i16 + offset2) as u32),
        const_felt252_expr!(op1 as i128),
    ));
    // Not in use
    memory_values.push((
        const_expr!((fp as i16 + offset1) as u32),
        const_felt252_expr!(0, 0),
    ));
    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());
    assert_equal_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (mut registry, _) = AirFnRegistry::new(&generic_opcode);
    registry.add_entry(&assert_equal_opcode);
    let (state, output) = registry.run_air(
        &generic_opcode,
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
    let (_, expected_output) = registry.run_air(
        &assert_equal_opcode,
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check output
    assert_eq!(expected_output.calc(), output.calc());

    // Check state
    let expected_state = [
        3, 11, 6, 32771, 32767, 32770, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 3, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 60231555, 1431655765, 0, 0, 0, 6,
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
fn test_generic_jump() {
    let mut generic_opcode = GenericOpcode::default();
    let mut jump_opcode = JumpOpcode {
        is_rel: false,
        is_imm: false,
        is_double_deref: false,
        memory: Felt252IdMemory::default(),
    };

    let offset_value = 10;
    let op1 = 5;

    // Register values at opcode start
    let [pc, ap, fp] = [3, 11, 6];

    // Fill memory
    let mut memory_values = vec![(
        const_expr!(pc),
        const_felt252_expr!(
            assemble_jump(
                None,
                Some(offset_value),
                jump_opcode
                    .get_flags()
                    .non_constants_to_arr(&[true, false, false]),
            ) as u128,
            0
        ),
    )];
    memory_values.push((
        const_expr!((fp as i16 + offset_value) as u32),
        const_felt252_expr!(op1 as i128),
    ));
    // Not in use
    memory_values.push((
        const_expr!((fp as i64 - 1) as u32),
        const_felt252_expr!(0, 0),
    ));
    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());
    jump_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (mut registry, _) = AirFnRegistry::new(&generic_opcode);
    registry.add_entry(&jump_opcode);
    let (state, output) = registry.run_air(
        &generic_opcode,
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
    let (_, expected_output) = registry.run_air(
        &jump_opcode,
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check output
    assert_eq!(expected_output.calc(), output.calc());

    // Check state
    let expected_state = [
        3, 11, 6, 32767, 32767, 32778, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 5, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1351207863, 1, 0, 0, 0, 4,
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
fn test_generic_jump_not_zero_taken() {
    let mut generic_opcode = GenericOpcode::default();
    let mut jnz_opcode = JnzOpcode {
        is_taken: true,
        dst_base_fp: false,
        memory: Felt252IdMemory::default(),
    };

    // Register values at opcode start
    let [pc, ap, fp] = [50, 200, 150];
    let offset_dst = -13;
    let op1 = 15;

    // Fill memory
    let mut memory_values = vec![(
        const_expr!(pc),
        const_felt252_expr!(
            assemble_instruction(
                offset_dst,
                -1,
                1,
                jnz_opcode.get_flags().non_constants_to_arr(&[false])
            ) as u128,
            0
        ),
    )];
    memory_values.push((const_expr!(pc + 1), const_felt252_expr!(op1 as i128)));
    memory_values.push((
        const_expr!((ap as i16 + offset_dst) as u32),
        const_felt252_expr!(123, 456),
    ));
    // Not in use
    memory_values.push((const_expr!(fp - 1), const_felt252_expr!(0, 0)));
    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());
    jnz_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (mut registry, _) = AirFnRegistry::new(&generic_opcode);
    registry.add_entry(&jnz_opcode);
    let (state, output) = registry.run_air(
        &generic_opcode,
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
    let (_, expected_output) = registry.run_air(
        &jnz_opcode,
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check output
    assert_eq!(expected_output.calc(), output.calc());

    // Check state
    let expected_state = [
        50, 200, 150, 32755, 32767, 32769, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 2, 123, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 288, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 15, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 500077285, 1955558780, 414, 0, 0,
        65,
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
fn test_generic_jump_not_zero_not_taken() {
    let mut generic_opcode = GenericOpcode::default();
    let mut jnz_opcode = JnzOpcode {
        is_taken: false,
        dst_base_fp: false,
        memory: Felt252IdMemory::default(),
    };

    // Register values at opcode start
    let [pc, ap, fp] = [50, 200, 150];
    let offset_dst = -13;
    let op1 = 15;

    // Fill memory
    let mut memory_values = vec![(
        const_expr!(pc),
        const_felt252_expr!(
            assemble_instruction(
                offset_dst,
                -1,
                1,
                jnz_opcode.get_flags().non_constants_to_arr(&[false])
            ) as u128,
            0
        ),
    )];
    memory_values.push((const_expr!(pc + 1), const_felt252_expr!(op1 as i128)));
    memory_values.push((
        const_expr!((ap as i16 + offset_dst) as u32),
        const_felt252_expr!(0, 0),
    ));
    // Not in use
    memory_values.push((const_expr!(fp - 1), const_felt252_expr!(0, 0)));
    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values.clone());
    jnz_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (mut registry, _) = AirFnRegistry::new(&generic_opcode);
    registry.add_entry(&jnz_opcode);
    let (state, output) = registry.run_air(
        &generic_opcode,
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
    let (_, expected_output) = registry.run_air(
        &jnz_opcode,
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );

    // Check output
    assert_eq!(expected_output.calc(), output.calc());

    // Check state
    let expected_state = [
        50, 200, 150, 32755, 32767, 32769, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 2, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 15, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1351207863, 1, 0, 0, 0, 52,
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
#[should_panic(expected = "0 has no inverse")]
fn test_generic_jump_not_zero_dst_is_p() {
    let mut generic_opcode = GenericOpcode::default();
    let jnz_opcode = JnzOpcode {
        is_taken: false,
        dst_base_fp: false,
        memory: Felt252IdMemory::default(),
    };

    // Register values at opcode start
    let [pc, ap, fp] = [50, 200, 150];
    let offset_dst = -13;
    let op1 = 15;

    // Fill memory
    let mut memory_values = vec![(
        const_expr!(pc),
        const_felt252_expr!(
            assemble_instruction(
                offset_dst,
                -1,
                1,
                jnz_opcode.get_flags().non_constants_to_arr(&[false])
            ) as u128,
            0
        ),
    )];
    memory_values.push((const_expr!(pc + 1), const_felt252_expr!(op1 as i128)));
    memory_values.push((
        const_expr!((ap as i16 + offset_dst) as u32),
        const_felt252_expr!(1, 10633823966279327296825105735305134080),
    ));
    // Not in use
    memory_values.push((const_expr!(fp - 1), const_felt252_expr!(0, 0)));
    generic_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (registry, _) = AirFnRegistry::new(&generic_opcode);
    registry.run_air(
        &generic_opcode,
        CasmStateVar::new(const_expr!(pc), const_expr!(ap), const_expr!(fp)),
    );
}

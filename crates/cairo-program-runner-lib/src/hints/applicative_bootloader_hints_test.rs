use cairo_vm::Felt252;
use cairo_vm::hint_processor::builtin_hint_processor::hint_utils::get_ptr_from_var_name;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::runners::builtin_runner::OutputBuiltinRunner;

use super::*;
use crate::test_utils::fill_ids_data_for_test;
use crate::{PROGRAM_INPUT, ProgramInput};

#[test]
fn test_aggregator_program_hash_function_to_ap() {
    let mut vm = VirtualMachine::new(false, false);
    vm.add_memory_segment();
    vm.add_memory_segment();
    let mut exec_scopes = ExecutionScopes::new();
    // Set by the input-loading hint of whichever applicative bootloader is running.
    exec_scopes.insert_value(vars::AGGREGATOR_PROGRAM_HASH_FUNCTION, 2usize);

    aggregator_program_hash_function_to_ap(&mut vm, &mut exec_scopes).unwrap();

    assert_eq!(*vm.get_integer(Relocatable::from((1, 0))).unwrap().as_ref(), Felt252::from(2));
}

#[test]
fn test_aggregator_program_hash_function_to_ap_requires_scope_variable() {
    let mut vm = VirtualMachine::new(false, false);
    vm.add_memory_segment();
    vm.add_memory_segment();
    let mut exec_scopes = ExecutionScopes::new();
    assert!(aggregator_program_hash_function_to_ap(&mut vm, &mut exec_scopes).is_err());
}

#[test]
fn test_prepare_aggregator_simple_bootloader_output_segment() {
    let mut vm = VirtualMachine::new(false, false);
    vm.add_memory_segment();
    vm.add_memory_segment();
    vm.set_fp(1);
    vm.set_ap(1);
    let ids_data = fill_ids_data_for_test(&["aggregator_output_ptr"]);
    let ap_tracking = ApTracking::new();

    // An output builtin whose pre-hint state points at a fresh segment.
    let applicative_segment = vm.add_memory_segment();
    let mut output_builtin_runner = OutputBuiltinRunner::new(true);
    output_builtin_runner.set_state(OutputBuiltinState {
        base: applicative_segment.segment_index as usize,
        base_offset: 0,
        pages: Default::default(),
        attributes: Default::default(),
    });
    vm.builtin_runners = vec![output_builtin_runner.into()];

    let fibonacci = format!(
        "{}/resources/compiled_programs/test_programs/fibonacci_compiled.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let task = serde_json::json!({
        "type": "RunProgramTask",
        "path": fibonacci,
        "program_hash_function": "blake",
    });
    let mut exec_scopes = ExecutionScopes::new();
    exec_scopes.insert_value(
        PROGRAM_INPUT,
        ProgramInput::Json(
            serde_json::json!({
                "tasks": [task],
                "fact_topologies_path": null,
                "single_page": false,
                "bootloader_config": {
                    "supported_simple_bootloader_hash_list": ["0x1", "0x2"],
                    "circuit_applicative_bootloader_program_hash": "0x3",
                    "circuit_applicative_config_commitment": "0x4",
                    "supported_cairo_verifier_program_hashes": ["0x5"],
                },
                "packed_outputs": [],
                "aggregator_task": task,
            })
            .to_string(),
        ),
    );

    prepare_aggregator_simple_bootloader_output_segment(
        &mut vm,
        &mut exec_scopes,
        &ids_data,
        &ap_tracking,
    )
    .unwrap();

    // The aggregator task's hash function is exposed for the nondet hint.
    let hash_function: usize = exec_scopes.get(vars::AGGREGATOR_PROGRAM_HASH_FUNCTION).unwrap();
    let input: ApplicativeBootloaderInput = exec_scopes.get(APPLICATIVE_BOOTLOADER_INPUT).unwrap();
    assert_eq!(hash_function, input.aggregator_task.program_hash_function as usize);

    // The simple bootloader input holds only the aggregator task.
    let simple_bootloader_input: SimpleBootloaderInput =
        exec_scopes.get(vars::SIMPLE_BOOTLOADER_INPUT).unwrap();
    assert_eq!(simple_bootloader_input.tasks.len(), 1);

    // The applicative output builtin state was saved, and the builtin points at the fresh
    // aggregator segment.
    let saved_state: OutputBuiltinState =
        exec_scopes.get(vars::APPLICATIVE_OUTPUT_BUILTIN_STATE).unwrap();
    assert_eq!(saved_state.base, applicative_segment.segment_index as usize);
    let aggregator_output_ptr =
        get_ptr_from_var_name("aggregator_output_ptr", &vm, &ids_data, &ap_tracking).unwrap();
    assert_eq!(
        vm.get_output_builtin_mut().unwrap().get_state().base,
        aggregator_output_ptr.segment_index as usize
    );
}

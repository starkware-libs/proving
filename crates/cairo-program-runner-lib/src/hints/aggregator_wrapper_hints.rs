use std::collections::HashMap;
use std::path::PathBuf;

use cairo_vm::hint_processor::builtin_hint_processor::hint_utils::get_ptr_from_var_name;
use cairo_vm::hint_processor::hint_processor_definition::HintReference;
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::vm_core::VirtualMachine;
use starknet_types_core::felt::Felt;

use super::utils::get_program_input_value;
use crate::hints::fact_topologies::{FactTopology, write_to_fact_topologies_file};
use crate::hints::types::{AggregatorWrapperInput, SimpleBootloaderInput};
use crate::hints::{SIMPLE_BOOTLOADER_INPUT, vars};

/// Implements hint: LOAD_AGGREGATOR_WRAPPER_INPUT
///
/// Loads the wrapper's program input: synthesizes the single-task `SimpleBootloaderInput` for
/// its inner bootloader run, and stores the task's program-hash function and the poseidon-preimage
///  dump path.
pub fn load_aggregator_wrapper_input(exec_scopes: &mut ExecutionScopes) -> Result<(), HintError> {
    let input: AggregatorWrapperInput = get_program_input_value(exec_scopes)?;
    exec_scopes.insert_value(
        vars::AGGREGATOR_PROGRAM_HASH_FUNCTION,
        input.task.program_hash_function as usize,
    );
    exec_scopes.insert_value(vars::POSEIDON_PREIMAGE_DUMP_PATH, input.poseidon_preimage_dump_path);
    // The inner run writes to a scratch segment, so we set single_page to `true` always.
    // The topology path is read by the aggregated-output dump hint.
    exec_scopes.insert_value(
        SIMPLE_BOOTLOADER_INPUT,
        SimpleBootloaderInput {
            fact_topologies_path: input.fact_topologies_path,
            single_page: true,
            tasks: vec![input.task],
        },
    );
    Ok(())
}

/// Implements hint: DUMP_AGGREGATOR_WRAPPER_POSEIDON_PREIMAGE
///
/// Dumps the poseidon preimage (`poseidon_preimage_start`..`poseidon_preimage_end`), which opens
/// the wrapper output's final digest, as a JSON felt array to `POSEIDON_PREIMAGE_DUMP_PATH`.
pub fn dump_aggregator_wrapper_poseidon_preimage(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    ids_data: &HashMap<String, HintReference>,
    ap_tracking: &ApTracking,
) -> Result<(), HintError> {
    let start = get_ptr_from_var_name("poseidon_preimage_start", vm, ids_data, ap_tracking)?;
    let end: Relocatable =
        get_ptr_from_var_name("poseidon_preimage_end", vm, ids_data, ap_tracking)?;
    let size = (end - start)?;
    let preimage: Vec<Felt> =
        vm.get_integer_range(start, size)?.into_iter().map(|v| v.into_owned()).collect();

    let dump_path: PathBuf = exec_scopes.get(vars::POSEIDON_PREIMAGE_DUMP_PATH)?;
    let json = serde_json::to_string_pretty(&preimage).map_err(|e| {
        HintError::CustomHint(format!("Failed to serialize the poseidon preimage: {e}").into())
    })?;
    std::fs::write(&dump_path, json).map_err(|e| {
        HintError::CustomHint(
            format!("Failed to write the poseidon preimage to {dump_path:?}: {e}").into(),
        )
    })?;

    Ok(())
}

/// Implements hint: DUMP_AGGREGATED_OUTPUT_FACT_TOPOLOGY
///
/// Writes the aggregated output's fact topology to `fact_topologies_path`, when set.
///
/// The aggregator recorded a topology for its whole output, `[n_input_tasks, task records...,
/// aggregated_output...]`, but the registered fact covers the aggregated output alone. The dump
/// therefore shrinks the recorded first page by the records prefix.
pub fn dump_aggregated_output_fact_topology(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    ids_data: &HashMap<String, HintReference>,
    ap_tracking: &ApTracking,
) -> Result<(), HintError> {
    let simple_bootloader_input: &SimpleBootloaderInput =
        exec_scopes.get_ref(vars::SIMPLE_BOOTLOADER_INPUT)?;
    let fact_topologies: Vec<FactTopology> = exec_scopes.get(vars::FACT_TOPOLOGIES)?;
    let [fact_topology] = fact_topologies.as_slice() else {
        return Err(HintError::CustomHint(
            format!("Expected exactly one fact topology; got {}.", fact_topologies.len())
                .into_boxed_str(),
        ));
    };

    let output_start = get_ptr_from_var_name("aggregator_output_start", vm, ids_data, ap_tracking)?;
    let aggregated_output_start =
        get_ptr_from_var_name("aggregated_output_start", vm, ids_data, ap_tracking)?;
    let output_end = get_ptr_from_var_name("aggregator_output_end", vm, ids_data, ap_tracking)?;
    let prefix_len = (aggregated_output_start - output_start)?;
    let aggregated_output_len = (output_end - aggregated_output_start)?;

    let mut page_sizes = fact_topology.page_sizes.clone();
    let first_page = page_sizes
        .first_mut()
        .ok_or_else(|| HintError::CustomHint("Empty page sizes.".to_string().into_boxed_str()))?;
    *first_page = first_page.checked_sub(prefix_len).ok_or_else(|| {
        HintError::CustomHint(
            format!("Page 0 ({first_page}) is shorter than the tasks prefix ({prefix_len}).")
                .into_boxed_str(),
        )
    })?;
    let total: usize = page_sizes.iter().sum();
    if total != aggregated_output_len {
        return Err(HintError::CustomHint(
            format!(
                "Adjusted page sizes sum to {total}, but the aggregated output length is \
                 {aggregated_output_len}."
            )
            .into_boxed_str(),
        ));
    }

    if let Some(path) = &simple_bootloader_input.fact_topologies_path {
        let adjusted =
            FactTopology { tree_structure: fact_topology.tree_structure.clone(), page_sizes };
        write_to_fact_topologies_file(path.as_path(), std::slice::from_ref(&adjusted))
            .map_err(Into::<HintError>::into)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::rc::Rc;

    use cairo_vm::types::program::Program;
    use cairo_vm::types::relocatable::MaybeRelocatable;

    use super::*;
    use crate::hints::types::{Cairo0Executable, HashFunc, Task, TaskSpec};
    use crate::test_utils::prepare_non_continuous_ids_data_for_test;
    use crate::{PROGRAM_INPUT, ProgramInput};

    fn wrapper_input(fact_topologies_path: Option<PathBuf>) -> AggregatorWrapperInput {
        AggregatorWrapperInput {
            task: TaskSpec {
                task: Rc::new(Task::Cairo0Program(Cairo0Executable {
                    program: Program::default(),
                    program_input: None,
                })),
                program_hash_function: HashFunc::Blake,
            },
            poseidon_preimage_dump_path: PathBuf::from("/tmp/test_dump.json"),
            fact_topologies_path,
        }
    }

    #[test]
    fn test_load_aggregator_wrapper_input() {
        let mut exec_scopes = ExecutionScopes::new();
        let input = wrapper_input(Some(PathBuf::from("/tmp/test_topologies.json")));
        exec_scopes.insert_value(PROGRAM_INPUT, ProgramInput::Value(Box::new(input)));

        load_aggregator_wrapper_input(&mut exec_scopes).expect("Hint failed");

        let hash_function: usize =
            exec_scopes.get(vars::AGGREGATOR_PROGRAM_HASH_FUNCTION).expect("hash function not set");
        assert_eq!(hash_function, HashFunc::Blake as usize);
        let simple_bl_input: SimpleBootloaderInput =
            exec_scopes.get(SIMPLE_BOOTLOADER_INPUT).expect("SimpleBootloaderInput not set");
        assert_eq!(simple_bl_input.tasks.len(), 1);
        assert!(simple_bl_input.single_page);
        assert_eq!(
            simple_bl_input.fact_topologies_path,
            Some(PathBuf::from("/tmp/test_topologies.json"))
        );
        let dump_path: PathBuf =
            exec_scopes.get(vars::POSEIDON_PREIMAGE_DUMP_PATH).expect("dump path not set");
        assert_eq!(dump_path, PathBuf::from("/tmp/test_dump.json"));
    }

    #[test]
    fn test_dump_aggregator_wrapper_poseidon_preimage() {
        let expected_preimage: Vec<Felt> = (10u64..14).map(Felt::from).collect();

        let mut vm = VirtualMachine::new(false, false);
        vm.add_memory_segment();
        vm.add_memory_segment();

        // Segment 0: the preimage. Segment 1: the two pointers, addressed fp-2..fp-1.
        let data: Vec<MaybeRelocatable> =
            expected_preimage.iter().map(|f| MaybeRelocatable::from(*f)).collect();
        vm.load_data(Relocatable::from((0, 0)), &data).expect("Failed to load data");
        vm.load_data(
            Relocatable::from((1, 0)),
            &[MaybeRelocatable::from((0, 0)), MaybeRelocatable::from((0, expected_preimage.len()))],
        )
        .expect("Failed to load pointer data");
        vm.set_fp(2);

        let ids_data = prepare_non_continuous_ids_data_for_test(&[
            ("poseidon_preimage_start", -2),
            ("poseidon_preimage_end", -1),
        ]);
        let ap_tracking = ApTracking::new();

        let dump_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let dump_path = dump_file.path().to_path_buf();
        let mut exec_scopes = ExecutionScopes::new();
        exec_scopes.insert_value(vars::POSEIDON_PREIMAGE_DUMP_PATH, dump_path.clone());

        dump_aggregator_wrapper_poseidon_preimage(
            &mut vm,
            &mut exec_scopes,
            &ids_data,
            &ap_tracking,
        )
        .expect("Hint failed");

        let dumped: Vec<String> =
            serde_json::from_str(&std::fs::read_to_string(&dump_path).expect("read dump"))
                .expect("parse dump");
        let parsed: Vec<Felt> =
            dumped.iter().map(|v| Felt::from_hex(v).expect("hex felt")).collect();
        assert_eq!(parsed, expected_preimage);
    }

    /// A VM whose output-range ids span a `prefix_len`-word records prefix and an
    /// `aggregated_len`-word aggregated output.
    fn topology_vm(
        prefix_len: usize,
        aggregated_len: usize,
    ) -> (VirtualMachine, HashMap<String, HintReference>) {
        let mut vm = VirtualMachine::new(false, false);
        vm.add_memory_segment();
        vm.add_memory_segment();
        vm.load_data(
            Relocatable::from((1, 0)),
            &[
                MaybeRelocatable::from((0, 0)),
                MaybeRelocatable::from((0, prefix_len)),
                MaybeRelocatable::from((0, prefix_len + aggregated_len)),
            ],
        )
        .expect("Failed to load pointer data");
        vm.set_fp(3);
        let ids_data = prepare_non_continuous_ids_data_for_test(&[
            ("aggregator_output_start", -3),
            ("aggregated_output_start", -2),
            ("aggregator_output_end", -1),
        ]);
        (vm, ids_data)
    }

    fn topology_scopes(
        fact_topologies_path: Option<std::path::PathBuf>,
        fact_topologies: Vec<FactTopology>,
    ) -> ExecutionScopes {
        let mut exec_scopes = ExecutionScopes::new();
        exec_scopes.insert_value(
            SIMPLE_BOOTLOADER_INPUT,
            SimpleBootloaderInput { fact_topologies_path, single_page: true, tasks: vec![] },
        );
        exec_scopes.insert_value(vars::FACT_TOPOLOGIES, fact_topologies);
        exec_scopes
    }

    #[test]
    fn test_dump_aggregated_output_fact_topology() {
        // Prefix of 7 words; aggregated output of 12 words paginated as [10, 2] past the prefix.
        let (mut vm, ids_data) = topology_vm(7, 12);
        let dump_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let mut exec_scopes = topology_scopes(
            Some(dump_file.path().to_path_buf()),
            vec![FactTopology { tree_structure: vec![2, 1, 0, 2], page_sizes: vec![17, 2] }],
        );

        dump_aggregated_output_fact_topology(
            &mut vm,
            &mut exec_scopes,
            &ids_data,
            &ApTracking::new(),
        )
        .expect("Hint failed");

        let dumped: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(dump_file.path()).expect("read dump"))
                .expect("parse dump");
        assert_eq!(
            dumped,
            serde_json::json!({
                "fact_topologies": [{"tree_structure": [2, 1, 0, 2], "page_sizes": [10, 2]}]
            })
        );
    }

    #[test]
    fn test_dump_aggregated_output_fact_topology_rejects_a_length_mismatch() {
        // The adjusted pages sum to 5, but the aggregated output holds 6 words.
        let (mut vm, ids_data) = topology_vm(7, 6);
        let mut exec_scopes = topology_scopes(
            None,
            vec![FactTopology { tree_structure: vec![1, 0], page_sizes: vec![12] }],
        );

        let error = dump_aggregated_output_fact_topology(
            &mut vm,
            &mut exec_scopes,
            &ids_data,
            &ApTracking::new(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("aggregated output length"));
    }
}

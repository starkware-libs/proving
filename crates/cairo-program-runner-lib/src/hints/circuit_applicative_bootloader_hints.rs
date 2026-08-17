//! Hints of the circuit-unpacking applicative bootloader and its circuit-tree unpacker
//! (`starkware/cairo/bootloaders/circuit_applicative_bootloader/`), plus the mock circuit
//! verifier's input hint.

use std::any::Any;
use std::collections::HashMap;

use cairo_vm::Felt252;
use cairo_vm::hint_processor::builtin_hint_processor::hint_utils::{
    get_ptr_from_var_name, insert_value_from_var_name,
};
use cairo_vm::hint_processor::hint_processor_definition::HintReference;
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::types::relocatable::MaybeRelocatable;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::runners::builtin_runner::OutputBuiltinState;
use cairo_vm::vm::vm_core::VirtualMachine;
use leaf_proof_format::N_DIGEST_WORDS;
use num_traits::ToPrimitive;

use super::fact_topologies::{
    FactTopology, GPS_FACT_TOPOLOGY, add_consecutive_output_pages, write_to_fact_topologies_file,
};
use super::types::{CircuitApplicativeBootloaderInput, MockCircuitVerifierInput, PackedNode};
use super::utils::get_program_input_value;
use super::{SimpleBootloaderInput, vars};

/// Scope variable holding the current packed-output tree node (a [`PackedNode`]).
const NODE: &str = "node";

fn felt_from_decimal_str(s: &str) -> Result<Felt252, HintError> {
    Felt252::from_dec_str(s)
        .map_err(|e| HintError::CustomHint(format!("Invalid decimal felt '{s}': {e:?}").into()))
}

/// Loads a list of u32 words into a fresh memory segment and returns its base.
fn load_words_segment(
    vm: &mut VirtualMachine,
    words: &[u32],
) -> Result<MaybeRelocatable, HintError> {
    let base = vm.add_memory_segment();
    let data: Vec<MaybeRelocatable> =
        words.iter().map(|w| MaybeRelocatable::from(Felt252::from(*w))).collect();
    vm.load_data(base, &data).map_err(HintError::Memory)?;
    Ok(MaybeRelocatable::from(base))
}

/// Implements hint: %{ LOAD_CIRCUIT_APPLICATIVE_BOOTLOADER_INPUT %}
///
/// The hint is used to:
/// 1. Load the CircuitApplicativeBootloaderInput.
/// 2. Create a segment for the aggregator output (ids.aggregator_output_ptr).
/// 3. Save the applicative output builtin state.
/// 4. Point the output builtin at the aggregator segment.
/// 5. Prepare the simple bootloader input holding only the aggregator task.
/// 6. Set the `aggregator_program_hash_function` scope variable.
pub fn load_circuit_applicative_bootloader_input(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    ids_data: &HashMap<String, HintReference>,
    ap_tracking: &ApTracking,
) -> Result<(), HintError> {
    let input: CircuitApplicativeBootloaderInput = get_program_input_value(exec_scopes)?;

    let new_segment_base = vm.add_memory_segment();
    insert_value_from_var_name(
        "aggregator_output_ptr",
        new_segment_base,
        vm,
        ids_data,
        ap_tracking,
    )?;

    let simple_bootloader_input = SimpleBootloaderInput {
        tasks: vec![input.aggregator_task.clone()],
        fact_topologies_path: None,
        single_page: true,
    };
    // Read back by the `nondet %{ aggregator_program_hash_function %}` hint.
    exec_scopes.insert_value(
        vars::AGGREGATOR_PROGRAM_HASH_FUNCTION,
        input.aggregator_task.program_hash_function as usize,
    );
    exec_scopes.insert_value(vars::CIRCUIT_APPLICATIVE_BOOTLOADER_INPUT, input);
    exec_scopes.insert_value(vars::SIMPLE_BOOTLOADER_INPUT, simple_bootloader_input);

    let output_builtin = vm.get_output_builtin_mut()?;
    let applicative_output_builtin_state = output_builtin.get_state();
    output_builtin.new_state(new_segment_base.segment_index as usize, 0, true);
    exec_scopes
        .insert_value(vars::APPLICATIVE_OUTPUT_BUILTIN_STATE, applicative_output_builtin_state);

    Ok(())
}

/// Implements hint: %{ CIRCUIT_APPLICATIVE_SETUP_VERIFIER_RUN %}
///
/// The hint is used to:
/// 1. Save the aggregator's fact topologies (resetting the list).
/// 2. Create a segment for the verifier task output (ids.verifier_output_ptr).
/// 3. Point the output builtin at it.
/// 4. Prepare the simple bootloader input holding only the circuit verifier task.
pub fn circuit_applicative_setup_verifier_run(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    ids_data: &HashMap<String, HintReference>,
    ap_tracking: &ApTracking,
) -> Result<(), HintError> {
    let fact_topologies: Vec<FactTopology> = exec_scopes.get(vars::FACT_TOPOLOGIES)?;
    exec_scopes.insert_value(vars::AGGREGATOR_FACT_TOPOLOGIES, fact_topologies);
    exec_scopes.insert_value(vars::FACT_TOPOLOGIES, Vec::<FactTopology>::new());

    let new_segment_base = vm.add_memory_segment();
    insert_value_from_var_name("verifier_output_ptr", new_segment_base, vm, ids_data, ap_tracking)?;

    let input: &CircuitApplicativeBootloaderInput =
        exec_scopes.get_ref(vars::CIRCUIT_APPLICATIVE_BOOTLOADER_INPUT)?;
    let simple_bootloader_input = SimpleBootloaderInput {
        tasks: vec![input.verifier_task.clone()],
        fact_topologies_path: None,
        single_page: true,
    };
    exec_scopes.insert_value(vars::SIMPLE_BOOTLOADER_INPUT, simple_bootloader_input);

    let output_builtin = vm.get_output_builtin_mut()?;
    output_builtin.new_state(new_segment_base.segment_index as usize, 0, true);

    Ok(())
}

/// Implements hint: %{ CIRCUIT_APPLICATIVE_SETUP_UNPACK %}
///
/// The hint is used to:
/// 1. Restore the applicative output builtin state.
/// 2. Allocate the bootloader-tasks-output segment (ids.bootloader_tasks_output_ptr).
/// 3. Build the unpacker config from the input's supported circuit hashes (ids.config).
/// 4. Set the packed-output root as the current `node` scope variable.
pub fn circuit_applicative_setup_unpack(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    ids_data: &HashMap<String, HintReference>,
    ap_tracking: &ApTracking,
) -> Result<(), HintError> {
    let output_builtin_state: OutputBuiltinState =
        exec_scopes.get(vars::APPLICATIVE_OUTPUT_BUILTIN_STATE)?;
    vm.get_output_builtin_mut()?.set_state(output_builtin_state);

    let tasks_output_base = vm.add_memory_segment();
    insert_value_from_var_name(
        "bootloader_tasks_output_ptr",
        tasks_output_base,
        vm,
        ids_data,
        ap_tracking,
    )?;

    let input: &CircuitApplicativeBootloaderInput =
        exec_scopes.get_ref(vars::CIRCUIT_APPLICATIVE_BOOTLOADER_INPUT)?;
    let supported_circuit_hashes = input.supported_circuit_hashes.clone();
    let packed_output = input.packed_output.clone();

    // ids.config = CircuitUnpackerConfig { n_supported_circuit_hashes: felt,
    // supported_circuit_hashes: felt* }: the input's circuit hashes list, flattened.
    let n_supported_circuit_hashes = supported_circuit_hashes.len();
    for circuit_hash in &supported_circuit_hashes {
        if circuit_hash.len() != N_DIGEST_WORDS {
            return Err(HintError::CustomHint(
                format!(
                    "Supported circuit hash has {} words; expected {N_DIGEST_WORDS}.",
                    circuit_hash.len()
                )
                .into(),
            ));
        }
    }
    let flat_circuit_hashes: Vec<u32> =
        supported_circuit_hashes.iter().flatten().copied().collect();
    let supported_circuit_hashes_ptr = load_words_segment(vm, &flat_circuit_hashes)?;
    let config_base = vm.add_memory_segment();
    vm.load_data(
        config_base,
        &[
            MaybeRelocatable::from(Felt252::from(n_supported_circuit_hashes)),
            supported_circuit_hashes_ptr,
        ],
    )
    .map_err(HintError::Memory)?;
    insert_value_from_var_name("config", config_base, vm, ids_data, ap_tracking)?;

    // The unpacker's node hints (CIRCUIT_UNPACK_*) read the current node from this scope; enter
    // a fresh scope so the bootloader's final CIRCUIT_UNPACK_EXIT_SCOPE balances it.
    let scope: HashMap<String, Box<dyn Any>> =
        HashMap::from([(NODE.to_string(), Box::new(packed_output) as Box<dyn Any>)]);
    exec_scopes.enter_scope(scope);

    Ok(())
}

fn get_node(exec_scopes: &ExecutionScopes) -> Result<&PackedNode, HintError> {
    exec_scopes.get_ref::<PackedNode>(NODE)
}

/// The subtasks list of a `Composite` node.
fn composite_subtasks(node: &PackedNode) -> Result<&Vec<PackedNode>, HintError> {
    match node {
        PackedNode::Composite { subtasks, .. } => Ok(subtasks),
        _ => Err(HintError::CustomHint("Packed node is not a Composite with subtasks.".into())),
    }
}

/// A leaf `Composite` holds its single `Plain` preimage reveal; an internal fold node holds its
/// two child `Composite`s, or one for a self-fold (the single-leaf tree's root pass).
fn node_is_leaf(node: &PackedNode) -> Result<bool, HintError> {
    let subtasks = composite_subtasks(node)?;
    match subtasks.as_slice() {
        [PackedNode::Plain { .. }] => Ok(true),
        [PackedNode::Composite { .. }] | [_, _] => Ok(false),
        subtasks => Err(HintError::CustomHint(
            format!(
                "Packed Composite node with unsupported number of subtasks: {}.",
                subtasks.len()
            )
            .into(),
        )),
    }
}

/// Implements hint: %{ CIRCUIT_UNPACK_SET_IS_LEAF %}
pub fn circuit_unpack_set_is_leaf(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    ids_data: &HashMap<String, HintReference>,
    ap_tracking: &ApTracking,
) -> Result<(), HintError> {
    let is_leaf = node_is_leaf(get_node(exec_scopes)?)?;
    insert_value_from_var_name("is_leaf", Felt252::from(is_leaf as u64), vm, ids_data, ap_tracking)
}

/// Implements hint: %{ CIRCUIT_UNPACK_SET_IS_SELF_FOLD %}
pub fn circuit_unpack_set_is_self_fold(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    ids_data: &HashMap<String, HintReference>,
    ap_tracking: &ApTracking,
) -> Result<(), HintError> {
    let is_self_fold = matches!(
        composite_subtasks(get_node(exec_scopes)?)?.as_slice(),
        [PackedNode::Composite { .. }]
    );
    insert_value_from_var_name(
        "is_self_fold",
        Felt252::from(is_self_fold as u64),
        vm,
        ids_data,
        ap_tracking,
    )
}

fn enter_subtask_scope(exec_scopes: &mut ExecutionScopes, index: usize) -> Result<(), HintError> {
    let subtask = composite_subtasks(get_node(exec_scopes)?)?
        .get(index)
        .ok_or_else(|| {
            HintError::CustomHint(format!("Packed node has no subtask {index}.").into())
        })?
        .clone();
    let scope: HashMap<String, Box<dyn Any>> =
        HashMap::from([(NODE.to_string(), Box::new(subtask) as Box<dyn Any>)]);
    exec_scopes.enter_scope(scope);
    Ok(())
}

/// Implements hint: %{ CIRCUIT_UNPACK_SET_CIRCUIT_HASH_INDEX %}
///
/// Sets ids.circuit_hash_index to the index, in the config's supported-circuit-hashes list, of
/// the circuit hash the current node carries (`Composite::circuit_hash`). The list is read from
/// Cairo memory through ids.config (no hint-scope state), and hashes are matched across all eight
/// words.
pub fn circuit_unpack_set_circuit_hash_index(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    ids_data: &HashMap<String, HintReference>,
    ap_tracking: &ApTracking,
) -> Result<(), HintError> {
    let node_circuit_hash: [u32; N_DIGEST_WORDS] = match get_node(exec_scopes)? {
        PackedNode::Composite { circuit_hash, .. } => *circuit_hash,
        _ => {
            return Err(HintError::CustomHint(
                "Packed node is not a Composite with a circuit hash.".into(),
            ));
        }
    };
    let node_circuit_hash_felts = node_circuit_hash.map(Felt252::from);

    // ids.config = CircuitUnpackerConfig { n_supported_circuit_hashes: felt,
    // supported_circuit_hashes: felt* }.
    let config_ptr = get_ptr_from_var_name("config", vm, ids_data, ap_tracking)?;
    let n_supported_circuit_hashes = vm.get_integer(config_ptr)?.to_usize().ok_or_else(|| {
        HintError::CustomHint("n_supported_circuit_hashes does not fit a usize.".into())
    })?;
    let supported_circuit_hashes_ptr = vm.get_relocatable((config_ptr + 1)?)?;

    let mut circuit_hash_index = None;
    for index in 0..n_supported_circuit_hashes {
        let circuit_hash_ptr = (supported_circuit_hashes_ptr + index * N_DIGEST_WORDS)?;
        let words = vm.get_integer_range(circuit_hash_ptr, N_DIGEST_WORDS)?;
        if words
            .iter()
            .zip(&node_circuit_hash_felts)
            .all(|(word, node_word)| word.as_ref() == node_word)
        {
            circuit_hash_index = Some(index);
            break;
        }
    }
    let circuit_hash_index = circuit_hash_index.ok_or_else(|| {
        HintError::CustomHint(
            format!(
                "Packed node circuit hash {node_circuit_hash:?} is not in the supported circuit \
                 hashes list."
            )
            .into(),
        )
    })?;
    insert_value_from_var_name(
        "circuit_hash_index",
        Felt252::from(circuit_hash_index),
        vm,
        ids_data,
        ap_tracking,
    )
}

/// Implements hint: %{ CIRCUIT_UNPACK_ENTER_SUBTASK_0 %}
pub fn circuit_unpack_enter_subtask_0(exec_scopes: &mut ExecutionScopes) -> Result<(), HintError> {
    enter_subtask_scope(exec_scopes, 0)
}

/// Implements hint: %{ CIRCUIT_UNPACK_ENTER_SUBTASK_1 %}
pub fn circuit_unpack_enter_subtask_1(exec_scopes: &mut ExecutionScopes) -> Result<(), HintError> {
    enter_subtask_scope(exec_scopes, 1)
}

/// Implements hint: %{ CIRCUIT_UNPACK_EXIT_SCOPE %}
pub fn circuit_unpack_exit_scope(exec_scopes: &mut ExecutionScopes) -> Result<(), HintError> {
    exec_scopes.exit_scope().map_err(HintError::FromScopeError)
}

/// Implements hint: %{ CIRCUIT_UNPACK_SET_LEAF_DATA %}
///
/// Sets ids.preimage / ids.preimage_len to the current leaf's raw `Plain.output_preimage` felts
/// (loaded into a fresh segment).
pub fn circuit_unpack_set_leaf_data(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    ids_data: &HashMap<String, HintReference>,
    ap_tracking: &ApTracking,
) -> Result<(), HintError> {
    let node = get_node(exec_scopes)?;
    let PackedNode::Plain { output_preimage } = &composite_subtasks(node)?[0] else {
        return Err(HintError::CustomHint("Leaf subtask is not a Plain preimage reveal.".into()));
    };

    let preimage: Vec<Felt252> =
        output_preimage.iter().map(|s| felt_from_decimal_str(s)).collect::<Result<_, _>>()?;

    let preimage_base = vm.add_memory_segment();
    let data: Vec<MaybeRelocatable> = preimage.iter().map(|f| MaybeRelocatable::from(*f)).collect();
    vm.load_data(preimage_base, &data).map_err(HintError::Memory)?;
    insert_value_from_var_name("preimage", preimage_base, vm, ids_data, ap_tracking)?;
    insert_value_from_var_name(
        "preimage_len",
        Felt252::from(preimage.len() as u64),
        vm,
        ids_data,
        ap_tracking,
    )?;

    Ok(())
}

/// Implements hint: %{ CIRCUIT_APPLICATIVE_WRITE_FACT_TOPOLOGY %}
///
/// The hint is used to:
/// 1. Build the final fact topology from the aggregator's (page 0 resized for the removed tasks
///    output and the added header).
/// 2. Configure the output builtin pages.
/// 3. Dump the topologies file.
///
/// It reads ids.output_start, ids.bootloader_tasks_output_ptr and ids.tasks_output_end.
pub fn circuit_applicative_write_fact_topology(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    ids_data: &HashMap<String, HintReference>,
    ap_tracking: &ApTracking,
) -> Result<(), HintError> {
    let aggregator_fact_topologies: Vec<FactTopology> =
        exec_scopes.get(vars::AGGREGATOR_FACT_TOPOLOGIES)?;
    if aggregator_fact_topologies.len() != 1 {
        return Err(HintError::CustomHint(
            "Expected exactly one fact topology for the aggregator task".into(),
        ));
    }
    let aggregator_fact_topology = aggregator_fact_topologies.first().unwrap();
    let original_first_page_length = aggregator_fact_topology.page_sizes[0];
    // The header contains the modified aggregator program hash and the config commitment.
    let header_size = 1 + 1;

    let tasks_output_end = get_ptr_from_var_name("tasks_output_end", vm, ids_data, ap_tracking)?;
    let tasks_output_start =
        get_ptr_from_var_name("bootloader_tasks_output_ptr", vm, ids_data, ap_tracking)?;
    let bootloader_tasks_output_length = tasks_output_end.offset - tasks_output_start.offset;

    let first_page_length =
        original_first_page_length - bootloader_tasks_output_length + header_size;

    let fact_topology = FactTopology {
        tree_structure: aggregator_fact_topology.tree_structure.clone(),
        page_sizes: vec![first_page_length]
            .into_iter()
            .chain(aggregator_fact_topology.page_sizes[1..].to_vec())
            .collect(),
    };

    let output_start = get_ptr_from_var_name("output_start", vm, ids_data, ap_tracking)?;
    let output_builtin = vm.get_output_builtin_mut()?;
    output_builtin.add_attribute(GPS_FACT_TOPOLOGY.into(), fact_topology.tree_structure.clone());
    let output_start = (output_start + fact_topology.page_sizes[0])?;
    let _ = add_consecutive_output_pages(
        &fact_topology.page_sizes[1..],
        output_builtin,
        1, // Starting page ID.
        output_start,
    )?;

    let input: &CircuitApplicativeBootloaderInput =
        exec_scopes.get_ref(vars::CIRCUIT_APPLICATIVE_BOOTLOADER_INPUT)?;
    if let Some(path) = &input.fact_topologies_path {
        write_to_fact_topologies_file(path.as_path(), std::slice::from_ref(&fact_topology))
            .map_err(Into::<HintError>::into)?;
    }

    Ok(())
}

/// Implements hint: %{ MOCK_CIRCUIT_VERIFIER_LOAD_INPUT %}
///
/// Sets ids.circuit_hash / ids.output_values (eight u32 words each, loaded into fresh segments)
/// and ids.n_steps from the MockCircuitVerifierInput program input.
pub fn load_mock_circuit_verifier_input(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    ids_data: &HashMap<String, HintReference>,
    ap_tracking: &ApTracking,
) -> Result<(), HintError> {
    let input: MockCircuitVerifierInput = get_program_input_value(exec_scopes)?;
    let circuit_hash_ptr = load_words_segment(vm, &input.circuit_hash)?;
    let output_values_ptr = load_words_segment(vm, &input.output_values)?;
    insert_value_from_var_name("circuit_hash", circuit_hash_ptr, vm, ids_data, ap_tracking)?;
    insert_value_from_var_name("output_values", output_values_ptr, vm, ids_data, ap_tracking)?;
    insert_value_from_var_name("n_steps", Felt252::from(input.n_steps), vm, ids_data, ap_tracking)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use cairo_vm::hint_processor::builtin_hint_processor::hint_utils::get_integer_from_var_name;
    use cairo_vm::types::program::Program;
    use cairo_vm::types::relocatable::Relocatable;
    use cairo_vm::vm::runners::builtin_runner::OutputBuiltinRunner;

    use super::*;
    use crate::hints::types::{Cairo0Executable, HashFunc, Task, TaskSpec};
    use crate::test_utils::fill_ids_data_for_test;
    use crate::{PROGRAM_INPUT, ProgramInput};

    /// A VM with the program/execution segments and fp/ap set past `n_ids` ids slots.
    fn vm_with_ids(names: &[&str]) -> (VirtualMachine, HashMap<String, HintReference>, ApTracking) {
        let mut vm = VirtualMachine::new(false, false);
        vm.add_memory_segment();
        vm.add_memory_segment();
        vm.set_fp(names.len());
        vm.set_ap(names.len());
        (vm, fill_ids_data_for_test(names), ApTracking::new())
    }

    /// Attaches an output builtin whose current state points at a fresh segment; returns that
    /// segment's base.
    fn add_output_builtin(vm: &mut VirtualMachine) -> Relocatable {
        let output_segment = vm.add_memory_segment();
        let mut output_builtin_runner = OutputBuiltinRunner::new(true);
        output_builtin_runner.set_state(OutputBuiltinState {
            base: output_segment.segment_index as usize,
            base_offset: 0,
            pages: Default::default(),
            attributes: Default::default(),
        });
        vm.builtin_runners = vec![output_builtin_runner.into()];
        output_segment
    }

    fn scopes_with_node(node: PackedNode) -> ExecutionScopes {
        let mut exec_scopes = ExecutionScopes::new();
        exec_scopes.insert_value(NODE, node);
        exec_scopes
    }

    fn sample_digest(seed: u32) -> [u32; N_DIGEST_WORDS] {
        std::array::from_fn(|i| seed + i as u32)
    }

    /// A supported-circuit-hashes list entry (the input carries them as plain word lists).
    fn sample_digest_words(seed: u32) -> Vec<u32> {
        sample_digest(seed).to_vec()
    }

    fn leaf_node(circuit_hash_seed: u32) -> PackedNode {
        PackedNode::Composite {
            circuit_hash: sample_digest(circuit_hash_seed),
            subtasks: vec![PackedNode::Plain {
                output_preimage: vec!["7".to_string(), "11".to_string(), "13".to_string()],
            }],
        }
    }

    fn internal_node(circuit_hash_seed: u32, left: PackedNode, right: PackedNode) -> PackedNode {
        PackedNode::Composite {
            circuit_hash: sample_digest(circuit_hash_seed),
            subtasks: vec![left, right],
        }
    }

    fn dummy_task() -> TaskSpec {
        TaskSpec {
            task: Rc::new(Task::Cairo0Program(Cairo0Executable {
                program: Program::default(),
                program_input: None,
            })),
            program_hash_function: HashFunc::Blake,
        }
    }

    fn sample_input() -> CircuitApplicativeBootloaderInput {
        CircuitApplicativeBootloaderInput {
            aggregator_task: dummy_task(),
            verifier_task: dummy_task(),
            packed_output: internal_node(10, leaf_node(20), leaf_node(20)),
            supported_circuit_hashes: vec![sample_digest_words(10), sample_digest_words(20)],
            fact_topologies_path: None,
        }
    }

    #[test]
    fn test_set_is_leaf() {
        for (node, expected) in
            [(leaf_node(20), 1u64), (internal_node(10, leaf_node(20), leaf_node(20)), 0)]
        {
            let (mut vm, ids_data, ap_tracking) = vm_with_ids(&["is_leaf"]);
            let mut exec_scopes = scopes_with_node(node);
            circuit_unpack_set_is_leaf(&mut vm, &mut exec_scopes, &ids_data, &ap_tracking).unwrap();
            assert_eq!(
                get_integer_from_var_name("is_leaf", &vm, &ids_data, &ap_tracking).unwrap(),
                Felt252::from(expected)
            );
        }
    }

    #[test]
    fn test_set_is_leaf_rejects_malformed_nodes() {
        // Not a Composite.
        let (mut vm, ids_data, ap_tracking) = vm_with_ids(&["is_leaf"]);
        let mut exec_scopes = scopes_with_node(PackedNode::Plain { output_preimage: vec![] });
        assert!(
            circuit_unpack_set_is_leaf(&mut vm, &mut exec_scopes, &ids_data, &ap_tracking).is_err()
        );

        // A Composite with an unsupported number of subtasks.
        let mut exec_scopes = scopes_with_node(PackedNode::Composite {
            circuit_hash: sample_digest(10),
            subtasks: vec![leaf_node(20), leaf_node(20), leaf_node(20)],
        });
        assert!(
            circuit_unpack_set_is_leaf(&mut vm, &mut exec_scopes, &ids_data, &ap_tracking).is_err()
        );
    }

    #[test]
    fn test_enter_subtask_scopes_and_exit() {
        let left = leaf_node(20);
        let right = leaf_node(30);
        let mut exec_scopes = scopes_with_node(internal_node(10, left.clone(), right.clone()));

        circuit_unpack_enter_subtask_0(&mut exec_scopes).unwrap();
        assert_eq!(*get_node(&exec_scopes).unwrap(), left);
        circuit_unpack_exit_scope(&mut exec_scopes).unwrap();

        circuit_unpack_enter_subtask_1(&mut exec_scopes).unwrap();
        assert_eq!(*get_node(&exec_scopes).unwrap(), right);
        circuit_unpack_exit_scope(&mut exec_scopes).unwrap();

        assert_eq!(*get_node(&exec_scopes).unwrap(), internal_node(10, left, right));
    }

    #[test]
    fn test_enter_subtask_missing_index_fails() {
        // A leaf has a single subtask, so subtask 1 does not exist.
        let mut exec_scopes = scopes_with_node(leaf_node(20));
        assert!(circuit_unpack_enter_subtask_1(&mut exec_scopes).is_err());
    }

    /// Sets up ids.config -> { n_supported_circuit_hashes, supported_circuit_hashes } in VM
    /// memory.
    fn load_config(
        vm: &mut VirtualMachine,
        ids_data: &HashMap<String, HintReference>,
        ap_tracking: &ApTracking,
        supported_circuit_hashes: &[Vec<u32>],
    ) {
        let flat: Vec<u32> = supported_circuit_hashes.iter().flatten().copied().collect();
        let circuit_hashes_ptr = load_words_segment(vm, &flat).unwrap();
        let config_base = vm.add_memory_segment();
        vm.load_data(
            config_base,
            &[
                MaybeRelocatable::from(Felt252::from(supported_circuit_hashes.len())),
                circuit_hashes_ptr,
            ],
        )
        .unwrap();
        insert_value_from_var_name("config", config_base, vm, ids_data, ap_tracking).unwrap();
    }

    #[test]
    fn test_set_circuit_hash_index_matches_full_hash() {
        let (mut vm, ids_data, ap_tracking) = vm_with_ids(&["config", "circuit_hash_index"]);
        load_config(
            &mut vm,
            &ids_data,
            &ap_tracking,
            &[sample_digest_words(10), sample_digest_words(20)],
        );
        let mut exec_scopes = scopes_with_node(leaf_node(20));
        circuit_unpack_set_circuit_hash_index(&mut vm, &mut exec_scopes, &ids_data, &ap_tracking)
            .unwrap();
        assert_eq!(
            get_integer_from_var_name("circuit_hash_index", &vm, &ids_data, &ap_tracking).unwrap(),
            Felt252::from(1)
        );
    }

    #[test]
    fn test_set_circuit_hash_index_rejects_unsupported_hash() {
        let (mut vm, ids_data, ap_tracking) = vm_with_ids(&["config", "circuit_hash_index"]);
        // The second supported circuit hash shares its first word with the node's but differs in
        // the rest; matching is across all eight words, so the lookup must fail.
        let mut almost = sample_digest_words(20);
        almost[7] += 1;
        load_config(&mut vm, &ids_data, &ap_tracking, &[sample_digest_words(10), almost]);
        let mut exec_scopes = scopes_with_node(leaf_node(20));
        assert!(
            circuit_unpack_set_circuit_hash_index(
                &mut vm,
                &mut exec_scopes,
                &ids_data,
                &ap_tracking
            )
            .is_err()
        );
    }

    #[test]
    fn test_set_leaf_data() {
        let (mut vm, ids_data, ap_tracking) = vm_with_ids(&["preimage", "preimage_len"]);
        let mut exec_scopes = scopes_with_node(leaf_node(20));
        circuit_unpack_set_leaf_data(&mut vm, &mut exec_scopes, &ids_data, &ap_tracking).unwrap();

        assert_eq!(
            get_integer_from_var_name("preimage_len", &vm, &ids_data, &ap_tracking).unwrap(),
            Felt252::from(3)
        );
        let preimage_ptr = get_ptr_from_var_name("preimage", &vm, &ids_data, &ap_tracking).unwrap();
        let preimage: Vec<Felt252> = vm
            .get_integer_range(preimage_ptr, 3)
            .unwrap()
            .into_iter()
            .map(|f| *f.as_ref())
            .collect();
        assert_eq!(preimage, vec![Felt252::from(7), Felt252::from(11), Felt252::from(13)]);
    }

    #[test]
    fn test_set_leaf_data_rejects_malformed_leaves() {
        let (mut vm, ids_data, ap_tracking) = vm_with_ids(&["preimage", "preimage_len"]);

        // The leaf's subtask is not a Plain preimage reveal.
        let mut exec_scopes = scopes_with_node(internal_node(10, leaf_node(20), leaf_node(20)));
        assert!(
            circuit_unpack_set_leaf_data(&mut vm, &mut exec_scopes, &ids_data, &ap_tracking)
                .is_err()
        );

        // A non-decimal felt in the preimage.
        let mut exec_scopes = scopes_with_node(PackedNode::Composite {
            circuit_hash: sample_digest(20),
            subtasks: vec![PackedNode::Plain { output_preimage: vec!["0xabc".to_string()] }],
        });
        assert!(
            circuit_unpack_set_leaf_data(&mut vm, &mut exec_scopes, &ids_data, &ap_tracking)
                .is_err()
        );
    }

    #[test]
    fn test_load_mock_circuit_verifier_input() {
        let (mut vm, ids_data, ap_tracking) =
            vm_with_ids(&["circuit_hash", "output_values", "n_steps"]);
        let mut exec_scopes = ExecutionScopes::new();
        exec_scopes.insert_value(
            PROGRAM_INPUT,
            ProgramInput::Json(
                serde_json::json!({
                    "n_steps": 17,
                    "circuit_hash": sample_digest(10),
                    "output_values": sample_digest(20),
                })
                .to_string(),
            ),
        );
        load_mock_circuit_verifier_input(&mut vm, &mut exec_scopes, &ids_data, &ap_tracking)
            .unwrap();

        assert_eq!(
            get_integer_from_var_name("n_steps", &vm, &ids_data, &ap_tracking).unwrap(),
            Felt252::from(17)
        );
        for (name, seed) in [("circuit_hash", 10u32), ("output_values", 20)] {
            let ptr = get_ptr_from_var_name(name, &vm, &ids_data, &ap_tracking).unwrap();
            let words: Vec<Felt252> = vm
                .get_integer_range(ptr, N_DIGEST_WORDS)
                .unwrap()
                .into_iter()
                .map(|f| *f.as_ref())
                .collect();
            let expected: Vec<Felt252> =
                sample_digest(seed).iter().map(|&w| Felt252::from(w)).collect();
            assert_eq!(words, expected, "words mismatch for {name}");
        }
    }

    #[test]
    fn test_load_mock_circuit_verifier_input_requires_ids() {
        // Each insert fails cleanly when its ids variable is absent.
        for present_ids in [&[][..], &["circuit_hash"][..], &["circuit_hash", "output_values"][..]]
        {
            let (mut vm, ids_data, ap_tracking) = vm_with_ids(present_ids);
            let mut exec_scopes = ExecutionScopes::new();
            exec_scopes.insert_value(
                PROGRAM_INPUT,
                ProgramInput::Json(
                    serde_json::json!({
                        "n_steps": 17,
                        "circuit_hash": sample_digest(10),
                        "output_values": sample_digest(20),
                    })
                    .to_string(),
                ),
            );
            assert!(
                load_mock_circuit_verifier_input(
                    &mut vm,
                    &mut exec_scopes,
                    &ids_data,
                    &ap_tracking
                )
                .is_err(),
                "expected failure with ids {present_ids:?}"
            );
        }
    }

    #[test]
    fn test_load_circuit_applicative_bootloader_input() {
        let (mut vm, ids_data, ap_tracking) = vm_with_ids(&["aggregator_output_ptr"]);
        let output_segment = add_output_builtin(&mut vm);

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
                    "aggregator_task": task,
                    "verifier_task": task,
                    "packed_output": {"Composite": {
                        "circuit_hash": sample_digest(20),
                        "subtasks": [{"Plain": {"output_preimage": ["7"]}}],
                    }},
                    "supported_circuit_hashes": [sample_digest(10), sample_digest(20)],
                })
                .to_string(),
            ),
        );

        load_circuit_applicative_bootloader_input(
            &mut vm,
            &mut exec_scopes,
            &ids_data,
            &ap_tracking,
        )
        .unwrap();

        // The aggregator task's hash function is exposed for the nondet hint.
        let hash_function: usize = exec_scopes.get(vars::AGGREGATOR_PROGRAM_HASH_FUNCTION).unwrap();
        assert_eq!(hash_function, HashFunc::Blake as usize);

        // The simple bootloader input holds only the aggregator task.
        let simple_bootloader_input: SimpleBootloaderInput =
            exec_scopes.get(vars::SIMPLE_BOOTLOADER_INPUT).unwrap();
        assert_eq!(simple_bootloader_input.tasks.len(), 1);

        // The applicative output builtin state was saved and the builtin points at the new
        // aggregator segment.
        let saved_state: OutputBuiltinState =
            exec_scopes.get(vars::APPLICATIVE_OUTPUT_BUILTIN_STATE).unwrap();
        assert_eq!(saved_state.base, output_segment.segment_index as usize);
        let aggregator_output_ptr =
            get_ptr_from_var_name("aggregator_output_ptr", &vm, &ids_data, &ap_tracking).unwrap();
        assert_eq!(
            vm.get_output_builtin_mut().unwrap().get_state().base,
            aggregator_output_ptr.segment_index as usize
        );
    }

    #[test]
    fn test_setup_verifier_run() {
        let (mut vm, ids_data, ap_tracking) = vm_with_ids(&["verifier_output_ptr"]);
        add_output_builtin(&mut vm);
        let mut exec_scopes = ExecutionScopes::new();
        exec_scopes.insert_value(
            vars::FACT_TOPOLOGIES,
            vec![FactTopology { tree_structure: vec![1, 0], page_sizes: vec![3] }],
        );
        exec_scopes.insert_value(vars::CIRCUIT_APPLICATIVE_BOOTLOADER_INPUT, sample_input());

        circuit_applicative_setup_verifier_run(&mut vm, &mut exec_scopes, &ids_data, &ap_tracking)
            .unwrap();

        // The aggregator's fact topologies moved aside and the live list was reset.
        let aggregator_fact_topologies: Vec<FactTopology> =
            exec_scopes.get(vars::AGGREGATOR_FACT_TOPOLOGIES).unwrap();
        assert_eq!(aggregator_fact_topologies.len(), 1);
        let fact_topologies: Vec<FactTopology> = exec_scopes.get(vars::FACT_TOPOLOGIES).unwrap();
        assert!(fact_topologies.is_empty());

        // The simple bootloader input holds only the verifier task, and the output builtin points
        // at the fresh verifier segment.
        let simple_bootloader_input: SimpleBootloaderInput =
            exec_scopes.get(vars::SIMPLE_BOOTLOADER_INPUT).unwrap();
        assert_eq!(simple_bootloader_input.tasks.len(), 1);
        let verifier_output_ptr =
            get_ptr_from_var_name("verifier_output_ptr", &vm, &ids_data, &ap_tracking).unwrap();
        assert_eq!(
            vm.get_output_builtin_mut().unwrap().get_state().base,
            verifier_output_ptr.segment_index as usize
        );
    }

    #[test]
    fn test_setup_unpack() {
        let (mut vm, ids_data, ap_tracking) =
            vm_with_ids(&["bootloader_tasks_output_ptr", "config"]);
        add_output_builtin(&mut vm);
        let applicative_segment = vm.add_memory_segment();
        let input = sample_input();
        let mut exec_scopes = ExecutionScopes::new();
        exec_scopes.insert_value(
            vars::APPLICATIVE_OUTPUT_BUILTIN_STATE,
            OutputBuiltinState {
                base: applicative_segment.segment_index as usize,
                base_offset: 0,
                pages: Default::default(),
                attributes: Default::default(),
            },
        );
        exec_scopes.insert_value(vars::CIRCUIT_APPLICATIVE_BOOTLOADER_INPUT, input.clone());

        circuit_applicative_setup_unpack(&mut vm, &mut exec_scopes, &ids_data, &ap_tracking)
            .unwrap();

        // The applicative output builtin state was restored.
        assert_eq!(
            vm.get_output_builtin_mut().unwrap().get_state().base,
            applicative_segment.segment_index as usize
        );

        // ids.config = { n_supported_circuit_hashes, supported_circuit_hashes (flattened) }.
        let config_ptr = get_ptr_from_var_name("config", &vm, &ids_data, &ap_tracking).unwrap();
        assert_eq!(*vm.get_integer(config_ptr).unwrap().as_ref(), Felt252::from(2));
        let circuit_hashes_ptr = vm.get_relocatable((config_ptr + 1u32).unwrap()).unwrap();
        let words: Vec<Felt252> = vm
            .get_integer_range(circuit_hashes_ptr, 2 * N_DIGEST_WORDS)
            .unwrap()
            .into_iter()
            .map(|f| *f.as_ref())
            .collect();
        let expected: Vec<Felt252> = [sample_digest(10), sample_digest(20)]
            .iter()
            .flatten()
            .map(|&w| Felt252::from(w))
            .collect();
        assert_eq!(words, expected);

        // The packed-output root node scope was entered.
        assert_eq!(*get_node(&exec_scopes).unwrap(), input.packed_output);
    }

    #[test]
    fn test_setup_unpack_rejects_malformed_circuit_hashes() {
        let (mut vm, ids_data, ap_tracking) =
            vm_with_ids(&["bootloader_tasks_output_ptr", "config"]);
        add_output_builtin(&mut vm);
        let mut input = sample_input();
        input.supported_circuit_hashes = vec![vec![1, 2, 3]];
        let mut exec_scopes = ExecutionScopes::new();
        exec_scopes.insert_value(
            vars::APPLICATIVE_OUTPUT_BUILTIN_STATE,
            vm.get_output_builtin_mut().unwrap().get_state(),
        );
        exec_scopes.insert_value(vars::CIRCUIT_APPLICATIVE_BOOTLOADER_INPUT, input);
        assert!(
            circuit_applicative_setup_unpack(&mut vm, &mut exec_scopes, &ids_data, &ap_tracking)
                .is_err()
        );
    }

    #[test]
    fn test_write_fact_topology() {
        let (mut vm, ids_data, ap_tracking) =
            vm_with_ids(&["output_start", "bootloader_tasks_output_ptr", "tasks_output_end"]);
        let output_segment = add_output_builtin(&mut vm);

        // The unpacked tasks output spans 4 cells; the aggregator's single-page topology gets its
        // first page resized by -4 (removed tasks output) + 2 (added header).
        let tasks_segment = vm.add_memory_segment();
        insert_value_from_var_name(
            "output_start",
            output_segment,
            &mut vm,
            &ids_data,
            &ap_tracking,
        )
        .unwrap();
        insert_value_from_var_name(
            "bootloader_tasks_output_ptr",
            tasks_segment,
            &mut vm,
            &ids_data,
            &ap_tracking,
        )
        .unwrap();
        insert_value_from_var_name(
            "tasks_output_end",
            (tasks_segment + 4u32).unwrap(),
            &mut vm,
            &ids_data,
            &ap_tracking,
        )
        .unwrap();

        let fact_topologies_file = tempfile::NamedTempFile::new().unwrap();
        let mut input = sample_input();
        input.fact_topologies_path = Some(fact_topologies_file.path().to_path_buf());
        let mut exec_scopes = ExecutionScopes::new();
        exec_scopes.insert_value(vars::CIRCUIT_APPLICATIVE_BOOTLOADER_INPUT, input);
        exec_scopes.insert_value(
            vars::AGGREGATOR_FACT_TOPOLOGIES,
            vec![FactTopology { tree_structure: vec![1, 0], page_sizes: vec![10, 3] }],
        );

        circuit_applicative_write_fact_topology(&mut vm, &mut exec_scopes, &ids_data, &ap_tracking)
            .unwrap();

        let written: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(fact_topologies_file.path()).unwrap())
                .unwrap();
        assert_eq!(written["fact_topologies"][0]["page_sizes"], serde_json::json!([10 - 4 + 2, 3]));
    }

    #[test]
    fn test_write_fact_topology_requires_single_aggregator_topology() {
        let (mut vm, ids_data, ap_tracking) =
            vm_with_ids(&["output_start", "bootloader_tasks_output_ptr", "tasks_output_end"]);
        let mut exec_scopes = ExecutionScopes::new();
        exec_scopes.insert_value(vars::AGGREGATOR_FACT_TOPOLOGIES, Vec::<FactTopology>::new());
        assert!(
            circuit_applicative_write_fact_topology(
                &mut vm,
                &mut exec_scopes,
                &ids_data,
                &ap_tracking
            )
            .is_err()
        );
    }

    #[test]
    fn test_set_circuit_hash_index_rejects_non_composite_node() {
        let (mut vm, ids_data, ap_tracking) = vm_with_ids(&["config", "circuit_hash_index"]);
        load_config(&mut vm, &ids_data, &ap_tracking, &[sample_digest_words(10)]);
        let mut exec_scopes = scopes_with_node(PackedNode::Plain { output_preimage: vec![] });
        assert!(
            circuit_unpack_set_circuit_hash_index(
                &mut vm,
                &mut exec_scopes,
                &ids_data,
                &ap_tracking
            )
            .is_err()
        );
    }

    #[test]
    fn test_set_leaf_data_rejects_non_plain_reveal() {
        let (mut vm, ids_data, ap_tracking) = vm_with_ids(&["preimage", "preimage_len"]);
        // A single-subtask Composite whose subtask is not the Plain preimage reveal.
        let mut exec_scopes = scopes_with_node(PackedNode::Composite {
            circuit_hash: sample_digest(20),
            subtasks: vec![leaf_node(10)],
        });
        assert!(
            circuit_unpack_set_leaf_data(&mut vm, &mut exec_scopes, &ids_data, &ap_tracking)
                .is_err()
        );
    }

    #[test]
    fn test_setup_hints_require_ids() {
        // Each setup hint fails cleanly when its ids variable is absent.
        let (mut vm, ids_data, ap_tracking) = vm_with_ids(&[]);
        add_output_builtin(&mut vm);
        let mut exec_scopes = ExecutionScopes::new();
        exec_scopes.insert_value(vars::CIRCUIT_APPLICATIVE_BOOTLOADER_INPUT, sample_input());
        exec_scopes.insert_value(
            vars::FACT_TOPOLOGIES,
            vec![FactTopology { tree_structure: vec![1, 0], page_sizes: vec![3] }],
        );
        exec_scopes.insert_value(
            vars::APPLICATIVE_OUTPUT_BUILTIN_STATE,
            vm.get_output_builtin_mut().unwrap().get_state(),
        );
        assert!(
            circuit_applicative_setup_verifier_run(
                &mut vm,
                &mut exec_scopes,
                &ids_data,
                &ap_tracking
            )
            .is_err()
        );
        assert!(
            circuit_applicative_setup_unpack(&mut vm, &mut exec_scopes, &ids_data, &ap_tracking)
                .is_err()
        );

        let mut exec_scopes = ExecutionScopes::new();
        exec_scopes
            .insert_value(PROGRAM_INPUT, ProgramInput::Json(serde_json::json!({}).to_string()));
        // Malformed program input also fails cleanly.
        assert!(
            load_circuit_applicative_bootloader_input(
                &mut vm,
                &mut exec_scopes,
                &ids_data,
                &ap_tracking
            )
            .is_err()
        );
    }

    #[test]
    fn test_set_leaf_data_requires_ids() {
        // The last insert (preimage_len) fails when its ids variable is absent.
        let (mut vm, ids_data, ap_tracking) = vm_with_ids(&["preimage"]);
        let mut exec_scopes = scopes_with_node(leaf_node(20));
        assert!(
            circuit_unpack_set_leaf_data(&mut vm, &mut exec_scopes, &ids_data, &ap_tracking)
                .is_err()
        );
    }

    #[test]
    fn test_write_fact_topology_unwritable_path_fails() {
        let (mut vm, ids_data, ap_tracking) =
            vm_with_ids(&["output_start", "bootloader_tasks_output_ptr", "tasks_output_end"]);
        let output_segment = add_output_builtin(&mut vm);
        let tasks_segment = vm.add_memory_segment();
        insert_value_from_var_name(
            "output_start",
            output_segment,
            &mut vm,
            &ids_data,
            &ap_tracking,
        )
        .unwrap();
        insert_value_from_var_name(
            "bootloader_tasks_output_ptr",
            tasks_segment,
            &mut vm,
            &ids_data,
            &ap_tracking,
        )
        .unwrap();
        insert_value_from_var_name(
            "tasks_output_end",
            (tasks_segment + 4u32).unwrap(),
            &mut vm,
            &ids_data,
            &ap_tracking,
        )
        .unwrap();

        // A directory is not a writable topologies file path.
        let dir = tempfile::tempdir().unwrap();
        let mut input = sample_input();
        input.fact_topologies_path = Some(dir.path().to_path_buf());
        let mut exec_scopes = ExecutionScopes::new();
        exec_scopes.insert_value(vars::CIRCUIT_APPLICATIVE_BOOTLOADER_INPUT, input);
        exec_scopes.insert_value(
            vars::AGGREGATOR_FACT_TOPOLOGIES,
            vec![FactTopology { tree_structure: vec![1, 0], page_sizes: vec![10, 3] }],
        );
        assert!(
            circuit_applicative_write_fact_topology(
                &mut vm,
                &mut exec_scopes,
                &ids_data,
                &ap_tracking
            )
            .is_err()
        );
    }
}

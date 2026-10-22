use std::collections::HashMap;

use cairo_vm::hint_processor::builtin_hint_processor::hint_utils::{
    get_integer_from_var_name, get_ptr_from_var_name, insert_value_from_var_name,
};
use cairo_vm::hint_processor::hint_processor_definition::HintReference;
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::types::relocatable::MaybeRelocatable;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::vm_core::VirtualMachine;
use num_traits::ToPrimitive;

use super::types::{BOOTLOADER_CONFIG_SIZE, ConcatAggregatorInput};
use super::utils::get_program_input_value;

const TASKS_OUTPUTS: &str = "tasks_outputs";

/// Implements hint: CONCAT_AGGREGATOR_PARSE_TASKS_OUTPUTS
///
/// Parses `program_input["bootloader_output"]` — `[bootloader_config (BOOTLOADER_CONFIG_SIZE
/// felts), n_tasks, per task: (task_output_size, program_hash, output...)]` — into the per-task
/// `[program_hash, output...]` records, stored in the `tasks_outputs` scope variable with their
/// count in `ids.n_tasks`. Rejects an output a record overruns or that isn't fully consumed.
pub fn concat_aggregator_parse_task(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    ids_data: &HashMap<String, HintReference>,
    ap_tracking: &ApTracking,
) -> Result<(), HintError> {
    let concat_aggregator_input: ConcatAggregatorInput = get_program_input_value(exec_scopes)?;

    let mut iter =
        concat_aggregator_input.bootloader_output.iter().map(|felt| MaybeRelocatable::from(*felt));
    let mut next_item = || {
        iter.next()
            .ok_or_else(|| HintError::CustomHint("Unexpected end of bootloader output.".into()))
    };

    let extract_usize = |mr: MaybeRelocatable| -> Result<usize, HintError> {
        match mr {
            MaybeRelocatable::Int(felt) => felt
                .to_usize()
                .ok_or_else(|| HintError::CustomHint("Failed to convert value to usize.".into())),
            _ => Err(HintError::CustomHint("Expected Int variant.".into())),
        }
    };

    for _ in 0..BOOTLOADER_CONFIG_SIZE {
        next_item()?;
    }

    let n_tasks = extract_usize(next_item()?)?;

    let tasks_outputs = (0..n_tasks)
        .map(|_| -> Result<Vec<MaybeRelocatable>, HintError> {
            let task_size = extract_usize(next_item()?)?;
            (0..task_size.saturating_sub(1)).map(|_| next_item()).collect()
        })
        .collect::<Result<Vec<_>, _>>()?;

    if iter.next().is_some() {
        return Err(HintError::CustomHint("Bootloader output wasn't fully consumed.".into()));
    }
    if tasks_outputs.is_empty() {
        return Err(HintError::CustomHint("No tasks found in the bootloader output.".into()));
    }

    insert_value_from_var_name("n_tasks", tasks_outputs.len(), vm, ids_data, ap_tracking)?;
    exec_scopes.insert_value(TASKS_OUTPUTS, tasks_outputs);
    Ok(())
}

/// Implements hint:
/// %{
///     task_index = len(tasks_outputs) - ids.n_tasks
///     segments.load_data(ptr=ids.output_ptr, data=tasks_outputs[task_index])
///     ids.output_size = len(tasks_outputs[task_index]) + size_offset
/// %}
/// Where `size_offset` is in {0, 1}.
pub fn concat_aggregator_get_handle_task_output(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    ids_data: &HashMap<String, HintReference>,
    ap_tracking: &ApTracking,
    size_offset: usize,
) -> Result<(), HintError> {
    let tasks_outputs: &Vec<Vec<MaybeRelocatable>> = exec_scopes.get_ref(TASKS_OUTPUTS)?;
    let n_tasks = get_integer_from_var_name("n_tasks", vm, ids_data, ap_tracking)?
        .to_usize()
        .ok_or_else(|| HintError::CustomHint("Failed to convert value to usize.".into()))?;
    let task_index = tasks_outputs.len() - n_tasks;
    let output_ptr = get_ptr_from_var_name("output_ptr", vm, ids_data, ap_tracking)?;
    vm.segments.load_data(output_ptr, &tasks_outputs[task_index])?;

    insert_value_from_var_name(
        "output_size",
        tasks_outputs[task_index].len() + size_offset,
        vm,
        ids_data,
        ap_tracking,
    )
}

#[cfg(test)]
mod tests {
    use cairo_vm::Felt252;

    use super::*;
    use crate::test_utils::fill_ids_data_for_test;
    use crate::{PROGRAM_INPUT, ProgramInput};

    #[test]
    fn test_concat_aggregator_parse_task() {
        // We expect concat_aggregator_parse_task to set n_tasks and set the hint variable
        // TASKS_OUTPUTS to the per-task outputs parsed from bootloader_output.
        // bootloader_output is laid out as
        // [bootloader_config (BOOTLOADER_CONFIG_SIZE=1 felt), n_tasks, (task_output_size,
        // *task_output) per task]: the leading config felt (111) is irrelevant to the tasks'
        // outputs, then n_tasks=2 tasks of sizes 3 and 2 are parsed. Every felt is a hex string;
        // the second task's single output felt is wider than a `u64`.
        let input_json = r#"{"bootloader_output": ["0x6f", "0x2", "0x3", "0x64", "0x65", "0x2", "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcd"]}"#;

        let trace_enabled = false;
        let disable_trace_padding = false;
        let mut vm = VirtualMachine::new(trace_enabled, disable_trace_padding);
        // Segment 0 (program) and segment 1 (execution, where fp lives and `n_tasks` is written).
        vm.segments.add();
        vm.segments.add();
        vm.set_fp(1);

        let mut exec_scopes = ExecutionScopes::new();
        exec_scopes.insert_value(PROGRAM_INPUT, ProgramInput::Json(input_json.to_string()));

        let ids_data = fill_ids_data_for_test(&["n_tasks"]);
        let ap_tracking = ApTracking::new();

        concat_aggregator_parse_task(&mut vm, &mut exec_scopes, &ids_data, &ap_tracking)
            .expect("Hint failed unexpectedly");

        // n_tasks is written to the `n_tasks` id.
        assert_eq!(
            get_integer_from_var_name("n_tasks", &vm, &ids_data, &ap_tracking).unwrap().to_usize(),
            Some(2)
        );

        let tasks_outputs: Vec<Vec<MaybeRelocatable>> =
            exec_scopes.get(TASKS_OUTPUTS).expect("tasks_outputs not found in execution scope");
        assert_eq!(
            tasks_outputs,
            vec![
                vec![
                    MaybeRelocatable::from(Felt252::from(100)),
                    MaybeRelocatable::from(Felt252::from(101)),
                ],
                vec![MaybeRelocatable::from(Felt252::from_dec_str("32164469228446645568324754197800141613077674933089001409933681519550835661").unwrap())],
            ]
        );
    }
}

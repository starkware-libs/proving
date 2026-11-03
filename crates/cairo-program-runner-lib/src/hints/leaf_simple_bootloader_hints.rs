//! The leaf simple bootloader's input hint and the hint that hands its output hash function to
//! the program. Its preimage dump hint is the privacy simple bootloader's.

use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::vm_core::VirtualMachine;

use super::privacy_simple_bootloader_hints::insert_privacy_simple_bootloader_input;
use super::utils::{get_program_input_value, scope_usize_to_ap};
use crate::hints::types::{HashFunc, LeafSimpleBootloaderInput};
use crate::hints::vars;

/// Implements hint: LOAD_LEAF_SIMPLE_BOOTLOADER_INPUT
///
/// Loads the [`LeafSimpleBootloaderInput`] into the execution scopes: the inner
/// `SimpleBootloaderInput`, the preimage dump path and the output hash function. Rejects pedersen
/// and any number of tasks but one.
pub fn load_leaf_simple_bootloader_input(
    exec_scopes: &mut ExecutionScopes,
) -> Result<(), HintError> {
    let input: LeafSimpleBootloaderInput = get_program_input_value(exec_scopes)?;
    if input.output_hash_func == HashFunc::Pedersen {
        return Err(HintError::CustomHint(
            "The leaf simple bootloader hashes its output with blake or poseidon, not pedersen."
                .into(),
        ));
    }
    let n_tasks = input.privacy_input.simple_bootloader_input.tasks.len();
    if n_tasks != 1 {
        return Err(HintError::CustomHint(
            format!("The leaf simple bootloader runs exactly one task, got {n_tasks}.").into(),
        ));
    }
    insert_privacy_simple_bootloader_input(exec_scopes, input.privacy_input);
    exec_scopes.insert_value(vars::OUTPUT_HASH_FUNC, input.output_hash_func as usize);
    Ok(())
}

/// Implements nondet %{ output_hash_func %}
/// Compiles to: memory[ap] = to_felt_or_relocatable(output_hash_func)
///
/// The scope variable is set by `load_leaf_simple_bootloader_input`.
pub fn output_hash_func_to_ap(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
) -> Result<(), HintError> {
    scope_usize_to_ap(vm, exec_scopes, vars::OUTPUT_HASH_FUNC)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use cairo_vm::Felt252;
    use cairo_vm::types::relocatable::Relocatable;

    use super::*;
    use crate::hints::types::SimpleBootloaderInput;
    use crate::{PROGRAM_INPUT, ProgramInput};

    /// A leaf input over `n_tasks` fibonacci tasks, with `output_hash_func` as given, or left out
    /// for `None`.
    fn input_json(n_tasks: usize, output_hash_func: Option<&str>) -> String {
        let task_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/fibonacci.json");
        let task = serde_json::json!({
            "type": "RunProgramTask",
            "path": task_path.to_str().unwrap(),
            "program_hash_function": "blake",
        });
        let mut input = serde_json::json!({
            "tasks": vec![task; n_tasks],
            "single_page": true,
            "fact_topologies_path": "/tmp/topologies.json",
            "output_preimage_dump_path": "/tmp/preimage.json",
        });
        if let Some(output_hash_func) = output_hash_func {
            input["output_hash_func"] = output_hash_func.into();
        }
        input.to_string()
    }

    fn load(input_json: String) -> Result<ExecutionScopes, HintError> {
        let mut exec_scopes = ExecutionScopes::new();
        exec_scopes.insert_value(PROGRAM_INPUT, ProgramInput::Json(input_json));
        load_leaf_simple_bootloader_input(&mut exec_scopes)?;
        Ok(exec_scopes)
    }

    #[test]
    fn test_load_leaf_simple_bootloader_input() {
        let exec_scopes = load(input_json(1, Some("poseidon"))).unwrap();

        let simple_bl_input: SimpleBootloaderInput =
            exec_scopes.get(vars::SIMPLE_BOOTLOADER_INPUT).unwrap();
        assert_eq!(simple_bl_input.tasks.len(), 1);
        assert!(simple_bl_input.single_page);
        assert_eq!(
            simple_bl_input.fact_topologies_path,
            Some(PathBuf::from("/tmp/topologies.json"))
        );
        let dump_path: PathBuf = exec_scopes.get(vars::OUTPUT_PREIMAGE_DUMP_PATH).unwrap();
        assert_eq!(dump_path, PathBuf::from("/tmp/preimage.json"));
        let output_hash_func: usize = exec_scopes.get(vars::OUTPUT_HASH_FUNC).unwrap();
        assert_eq!(output_hash_func, HashFunc::Poseidon as usize);
        // The program's `POSEIDON_HASH` constant.
        assert_eq!(output_hash_func, 1);
    }

    #[test]
    fn test_output_hash_func_to_ap() {
        let mut exec_scopes = load(input_json(1, Some("poseidon"))).unwrap();
        let mut vm = VirtualMachine::new(false, false);
        vm.add_memory_segment();
        vm.add_memory_segment();

        output_hash_func_to_ap(&mut vm, &mut exec_scopes).unwrap();

        assert_eq!(*vm.get_integer(Relocatable::from((1, 0))).unwrap().as_ref(), Felt252::from(1));
    }

    #[test]
    fn test_blake_output_hash_func_is_the_programs_blake_hash() {
        let exec_scopes = load(input_json(1, Some("blake"))).unwrap();
        let output_hash_func: usize = exec_scopes.get(vars::OUTPUT_HASH_FUNC).unwrap();
        assert_eq!(output_hash_func, HashFunc::Blake as usize);
        // The program's `BLAKE_HASH` constant.
        assert_eq!(output_hash_func, 2);
    }

    #[test]
    fn test_output_hash_func_is_required() {
        let err = load(input_json(1, None)).unwrap_err();
        assert!(err.to_string().contains("output_hash_func"), "{err}");
    }

    #[test]
    fn test_pedersen_output_hash_func_is_rejected() {
        let err = load(input_json(1, Some("pedersen"))).unwrap_err();
        assert!(err.to_string().contains("not pedersen"), "{err}");
    }

    #[test]
    fn test_two_tasks_are_rejected() {
        let err = load(input_json(2, Some("blake"))).unwrap_err();
        assert!(err.to_string().contains("exactly one task, got 2"), "{err}");
    }
}

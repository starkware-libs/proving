use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use air_infra::core::air_fn::AirFn;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::compiled_structs::{
    CompiledAirFn, CompiledAirVar, ConstraintEvalStep, TraceGenStep,
};
use itertools::Itertools;
use tempfile::tempdir;
use xshell::{cmd, Shell};

use super::framework_gen::generate_component_structs;
use crate::code_gen::simd_prover_gen::generate_simd_claim_provers;

pub fn project_root() -> PathBuf {
    std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
}

pub fn reformat_rust_code(code_text: String) -> String {
    // Since rustfmt is used with nightly features, it takes 2 runs to reach a fixed point.
    reformat_rust_code_inner(reformat_rust_code_inner(code_text))
}

pub fn reformat_rust_code_inner(code_text: String) -> String {
    let shell = Shell::new().unwrap();
    shell.set_var("RUSTUP_TOOLCHAIN", "nightly-2024-01-04");
    let rustfmt_toml = project_root().join("rustfmt.toml");
    let mut stdout = cmd!(shell, "rustfmt --config-path {rustfmt_toml}")
        .stdin(code_text)
        .read()
        .unwrap();
    if !stdout.ends_with('\n') {
        stdout.push('\n');
    }
    stdout
}

// Generates the prover & verifier code.
pub fn dump_component_code(air_fn: &impl AirFn, folder_path: &Path) {
    let registry = AirFnRegistry::new(air_fn);

    let lists = registry.get_compiled_air_fn(air_fn);
    let air_entry = registry.get_air_fn_entry(air_fn);
    let claim_provers = generate_simd_claim_provers(&lists);
    let eval_tokens = generate_component_structs(&air_entry.name, lists, &air_entry);

    // Write the generated code to files.
    let text = reformat_rust_code(claim_provers.to_string().unwrap());
    fs::write(folder_path.join("prover.rs"), text).unwrap();
    let text = reformat_rust_code(eval_tokens.to_string().unwrap());
    fs::write(folder_path.join("component.rs"), text).unwrap();
}

pub fn assert_generated_code_unchanged(air_fn: &impl AirFn, folder_path: &Path) {
    let temp_dir = tempdir().expect("Could not open temporary folder!");
    let temp_dir = temp_dir.path();
    dump_component_code(air_fn, temp_dir);
    let generated_file_paths = fs::read_dir(temp_dir)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_file() && path.file_name().unwrap().to_str().unwrap().ends_with(".rs") {
                Some(path)
            } else {
                None
            }
        })
        .collect_vec();

    for path in generated_file_paths {
        let generated_code = fs::read_to_string(&path).unwrap();
        let exisitng_file_path = folder_path.join(path.file_name().unwrap());
        let existing_code = fs::read_to_string(&exisitng_file_path).unwrap();

        pretty_assertions::assert_eq!(
            generated_code,
            existing_code,
            r#"
            Generated code in {}.
            is different from the code in {}. 
            Run the following  to update the code:
            '$ FIX_CODE=1 cargo test'"#,
            path.display(),
            exisitng_file_path.display()
        );
    }
}

pub fn fn_calls_from_constraints(constraints: &[ConstraintEvalStep]) -> Vec<String> {
    constraints
        .iter()
        .filter_map(|constraint| {
            if let ConstraintEvalStep::LookupConstraint { fn_name, .. } = constraint {
                Some(fn_name.to_string())
            } else {
                None
            }
        })
        .collect()
}

pub fn fn_calls_from_deductions(deductions: &[TraceGenStep]) -> Vec<String> {
    deductions
        .iter()
        .filter_map(|deduction| {
            if let TraceGenStep::Lookup { fn_name, .. } = deduction {
                Some(fn_name.to_string())
            } else {
                None
            }
        })
        .collect()
}

pub fn unique_constraint_function_calls(constraints: &[ConstraintEvalStep]) -> Vec<String> {
    let function_calls = fn_calls_from_constraints(constraints);
    let mut seen_functions = HashSet::new();
    function_calls
        .into_iter()
        .filter(|fn_name| seen_functions.insert(fn_name.to_string()))
        .collect()
}

pub fn unique_deduction_function_calls(deductions: &[TraceGenStep]) -> Vec<String> {
    let function_calls = fn_calls_from_deductions(deductions);
    let mut seen_functions = HashSet::new();
    function_calls
        .into_iter()
        .filter(|fn_name| seen_functions.insert(fn_name.to_string()))
        .collect()
}

pub fn n_function_calls(constraints: &[ConstraintEvalStep]) -> usize {
    constraints
        .iter()
        .filter(|c| matches!(c, ConstraintEvalStep::LookupConstraint { .. }))
        .count()
}

pub fn callee_lookup_length(lists: &CompiledAirFn) -> usize {
    expression_n_cells(&lists.input) + expression_n_cells(&lists.output)
}

pub fn n_logup_columns(lists: &CompiledAirFn) -> usize {
    let n_function_calls = unique_constraint_function_calls(&lists.constraints).len();
    n_function_calls + callee_lookup_length(lists)
}

fn expression_n_cells(expr: &CompiledAirVar) -> usize {
    match expr {
        CompiledAirVar::State(..) => 1,
        CompiledAirVar::Var(..) => 1,
        CompiledAirVar::BinaryOp(..) => 1,
        CompiledAirVar::UnaryOp(..) => 1,
        CompiledAirVar::Tuple(vars) => vars.iter().map(expression_n_cells).sum(),
        CompiledAirVar::Array(vars) => vars.iter().map(expression_n_cells).sum(),
        _ => unimplemented!(),
    }
}

pub fn n_trace_cells(deductions: &[TraceGenStep]) -> usize {
    deductions
        .iter()
        .filter(|c| matches!(c, TraceGenStep::Deduction(_)))
        .count()
}

/// To run in FIX mode - '$ FIX_CODE=1 cargo test'
#[cfg(test)]
pub fn compare_contents_or_fix_with_path(air_fn: &impl AirFn, folder_path: &Path) {
    let is_fix_mode = std::env::var("FIX_CODE") == Ok("1".to_string());
    if is_fix_mode {
        dump_component_code(air_fn, folder_path);
    } else {
        assert_generated_code_unchanged(air_fn, folder_path);
    }
}

#[cfg(test)]
mod tests {
    use genco::lang::rust;
    use genco::quote;

    use super::reformat_rust_code;

    #[test]
    fn test_reformat_rust_code() {
        let mut code = rust::Tokens::new();
        code.extend(quote! {
            fn foo() {
                println!("Hello, world!");
            }
        });
        let code_text = reformat_rust_code(code.to_string().expect("Could not format Rust code."));
        println!("{}", code_text);
    }
}

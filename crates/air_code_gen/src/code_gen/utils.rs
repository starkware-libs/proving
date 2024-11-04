use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use compiled_casm_air::compiled_structs::{
    CompiledAirFn, CompiledAirVar, ConstraintEvalStep, LookupData, TraceGenStep,
};
use genco::lang::rust;
use genco::quote;
use itertools::Itertools;
use tempfile::tempdir;
use xshell::{cmd, Shell};

use super::framework_gen::generate_component_code;
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
    let rustfmt_toml = project_root().join("../../rustfmt.toml");
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
pub fn dump_component_code(air_fn: CompiledAirFn, folder_path: &Path) {
    let claim_provers = generate_simd_claim_provers(&air_fn);
    let eval_tokens = generate_component_code(air_fn);

    // Write the generated code to files.
    let text = reformat_rust_code(claim_provers.to_string().unwrap());
    fs::write(folder_path.join("prover.rs"), text).unwrap();
    let text = reformat_rust_code(eval_tokens.to_string().unwrap());
    fs::write(folder_path.join("component.rs"), text).unwrap();

    // Generate mod.rs, if it does not exist.
    let mod_rs_path = folder_path.join("mod.rs");
    if !std::path::Path::new(&mod_rs_path).exists() {
        let mod_rs_code: rust::Tokens = quote! {
            pub mod component;
            pub mod prover;

            pub use component::{RelationElements, Claim, InteractionClaim, Component, Eval};
            pub use prover::{ClaimGenerator, InputType, InteractionClaimGenerator};
        };
        let text = reformat_rust_code(mod_rs_code.to_string().unwrap());
        fs::write(mod_rs_path, text).unwrap();
    }
}

pub fn assert_generated_code_unchanged(air_fn: CompiledAirFn, folder_path: &Path) {
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
        if path.file_name().unwrap() == "mod.rs" {
            continue;
        }

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

pub fn relation_calls_from_constraints(constraints: &[ConstraintEvalStep]) -> Vec<String> {
    constraints
        .iter()
        .filter_map(|constraint| {
            if let ConstraintEvalStep::LookupData(LookupData { relation_name, .. }) = constraint {
                Some(relation_name.to_string())
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
            if let TraceGenStep::LookupCall { fn_name, .. } = deduction {
                Some(fn_name.to_string())
            } else {
                None
            }
        })
        .collect()
}

pub fn unique_constraint_relations(constraints: &[ConstraintEvalStep]) -> Vec<String> {
    let function_calls = relation_calls_from_constraints(constraints);
    let mut seen_functions = HashSet::new();
    function_calls
        .into_iter()
        .filter(|fn_name| seen_functions.insert(fn_name.to_string()))
        .sorted()
        .collect()
}

pub fn unique_deduction_function_calls(deductions: &[TraceGenStep]) -> Vec<String> {
    let function_calls = fn_calls_from_deductions(deductions);
    let mut seen_functions = HashSet::new();
    function_calls
        .into_iter()
        .filter(|fn_name| seen_functions.insert(fn_name.to_string()))
        .sorted()
        .collect()
}

pub fn unique_relation_calls(deductions: &[TraceGenStep]) -> Vec<String> {
    let mut seen_relations = HashSet::new();
    deductions.iter().for_each(|d| {
        if let TraceGenStep::LookupData(LookupData { relation_name, .. }) = d {
            seen_relations.insert(relation_name.to_string());
        }
    });
    seen_relations.into_iter().sorted().collect()
}

pub fn n_function_calls(constraints: &[ConstraintEvalStep]) -> usize {
    constraints
        .iter()
        .filter(|c| matches!(c, ConstraintEvalStep::LookupData { .. }))
        .count()
}

/// Computes width of the lookup 'yield'. Used as 'n_alpha_powers'.
///
/// Assumption: lists.input, lists.output are either 1-trace-cell sized, or structs,
/// arrays, tuples of variables that size.
pub fn callee_lookup_length(lists: &CompiledAirFn) -> usize {
    let n_felts = |expr| -> usize {
        match expr {
            CompiledAirVar::Var(..) => 1,
            CompiledAirVar::State(_) => 1,
            CompiledAirVar::BinaryOp(..) => 1,
            CompiledAirVar::UnaryOp(..) => 1,
            CompiledAirVar::Tuple(vec) => vec.len(),
            CompiledAirVar::Array(vec) => vec.len(),
            CompiledAirVar::Struct { r#type: _, fields } => fields.len(),
            _ => panic!("Unexpected I/O type!"),
        }
    };
    n_felts(lists.input.clone()) + n_felts(lists.output.clone())
}

pub fn n_logup_columns(lists: &CompiledAirFn) -> usize {
    let n_function_calls = unique_constraint_relations(&lists.constraints).len();
    n_function_calls + callee_lookup_length(lists)
}

pub fn block_doc(msg: &str) -> rust::Tokens {
    quote! {
        $['\n']$("//")$msg.$['\n']
    }
}

/// To run in FIX mode - '$ FIX_CODE=1 cargo test'
#[cfg(test)]
pub fn compare_contents_or_fix_with_path(air_fn: CompiledAirFn, folder_path: &Path) {
    let component_name = air_fn.name.to_lowercase();
    let folder_path = folder_path.join(component_name + "/");
    fs::create_dir_all(&folder_path).ok();
    let is_fix_mode = std::env::var("FIX_CODE") == Ok("1".to_string());
    if is_fix_mode {
        dump_component_code(air_fn, &folder_path);
    } else {
        assert_generated_code_unchanged(air_fn, &folder_path);
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

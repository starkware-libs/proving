use std::fs;
use std::path::{Path, PathBuf};

use air_infra::core::air_fn::AirFn;
use air_infra::core::air_fn_registry::AirFnRegistry;
use itertools::Itertools;
use tempfile::tempdir;
use xshell::{cmd, Shell};

use crate::code_gen::component_gen::generate_component;
use crate::code_gen::cpu_prover_gen::generate_cpu_prover_component;
use crate::code_gen::simd_prover_gen::generate_simd_prover_component;
use crate::code_gen::simd_trace_gen::generate_simd_trace_writer_code;
use crate::code_gen::test_utils_gen::generate_test_air_code;
use crate::code_gen::trace_gen::generate_trace_writer_code;

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

pub fn dump_component_code(air_fn: &impl AirFn, folder_path: &Path) {
    let resigtry = AirFnRegistry::new(air_fn);

    let lists = resigtry.get_compiled_air_fn(air_fn);
    let air_entry = resigtry.get_air_fn_entry(air_fn);
    let trace_tokens = generate_trace_writer_code(&air_entry.name, &lists.input, &lists.deductions);
    let simd_trace_tokens = generate_simd_trace_writer_code(
        &air_entry.name,
        &lists.input.clone(),
        &lists.deductions.clone(),
    );
    let cpu_prover_tokens =
        generate_cpu_prover_component(&air_entry.name, &lists.constraints.clone());
    let simd_prover_tokens =
        generate_simd_prover_component(&air_entry.name, &lists.constraints.clone());
    let component_tokens = generate_component(&air_entry.name, lists, &air_entry);
    let test_utils_tokens = generate_test_air_code(&air_entry.name);

    // Write the generated code to files.
    let text = reformat_rust_code(trace_tokens.to_string().unwrap());
    fs::write(folder_path.join("trace.rs"), text).unwrap();
    let text = reformat_rust_code(simd_trace_tokens.to_string().unwrap());
    fs::write(folder_path.join("simd_trace.rs"), text).unwrap();
    let text = reformat_rust_code(cpu_prover_tokens.to_string().unwrap());
    fs::write(folder_path.join("cpu_prover.rs"), text).unwrap();
    let text = reformat_rust_code(simd_prover_tokens.to_string().unwrap());
    fs::write(folder_path.join("simd_prover.rs"), text).unwrap();
    let text = reformat_rust_code(component_tokens.to_string().unwrap());
    fs::write(folder_path.join("component.rs"), text).unwrap();
    let text = reformat_rust_code(test_utils_tokens.to_string().unwrap());
    fs::write(folder_path.join("test_utils.rs"), text).unwrap();
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

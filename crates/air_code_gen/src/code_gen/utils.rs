use std::fs;
use std::path::{Path, PathBuf};

use compiled_casm_air::compiled_structs::{CompiledAirFn, CompiledAirVar, TraceType};
use genco::lang::rust;
use genco::quote;
use tempfile::tempdir;
use xshell::{cmd, Shell};

use super::constraints::{generate_constraints_code, generate_inline_code};
use super::trace_gen::RustProverGen;

pub fn project_root() -> PathBuf {
    std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
}

pub fn reformat_rust_code(code_text: String) -> String {
    // Since rustfmt is used with nightly features, it takes 2 runs to reach a fixed point.
    reformat_rust_code_inner(reformat_rust_code_inner(code_text))
}

pub fn reformat_rust_code_inner(code_text: String) -> String {
    let shell = Shell::new().unwrap();
    shell.set_var("RUSTUP_TOOLCHAIN", "nightly-2024-12-16");
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
pub fn dump_component_code(
    air_fn: &CompiledAirFn,
    constraints_folder_path: &Path,
    witness_folder_path: &Path,
) {
    let (constraints_code, witness_code) = if air_fn.r#type == TraceType::Inline {
        (generate_inline_code(air_fn), quote!())
    } else {
        (
            generate_constraints_code(air_fn),
            RustProverGen::new(air_fn.clone()).generate_witness_code(),
        )
    };

    // Write the generated code to files.
    let file_name = &format!("{}.rs", air_fn.name);

    // TODO(Gali): handle witness sub-routines.
    if air_fn.r#type != TraceType::Inline {
        let witness_code = reformat_rust_code(witness_code.to_string().unwrap());
        fs::write(witness_folder_path.join(file_name), witness_code).unwrap();
    }
    let constraints_code = reformat_rust_code(constraints_code.to_string().unwrap());
    let suffix = get_constraints_folder_path_suffix(&air_fn.r#type, file_name);
    fs::write(constraints_folder_path.join(suffix), constraints_code).unwrap();
}

pub fn assert_generated_code_unchanged(
    air_fn: CompiledAirFn,
    constraints_folder_path: &Path,
    witness_folder_path: &Path,
) {
    let air_fn_name = air_fn.name.clone();
    let temp_dir = tempdir().expect("Could not open temporary folder!");
    let temp_dir = temp_dir.path();
    let temp_witness_folder_path = temp_dir.join("witness");
    let temp_constraints_folder_path = temp_dir.join("constraints");
    fs::create_dir_all(&temp_constraints_folder_path).ok();
    fs::create_dir_all(temp_constraints_folder_path.join("subroutines")).ok();
    fs::create_dir_all(&temp_witness_folder_path).ok();
    dump_component_code(
        &air_fn,
        &temp_constraints_folder_path,
        &temp_witness_folder_path,
    );

    let rust_file_name = &format!("{}.rs", air_fn_name);
    let suffix = &get_constraints_folder_path_suffix(&air_fn.r#type, rust_file_name);
    let mut files_to_compare = vec![(
        constraints_folder_path.join(suffix),
        temp_constraints_folder_path.join(suffix),
    )];
    if air_fn.r#type != TraceType::Inline {
        files_to_compare.push((
            witness_folder_path.join(rust_file_name),
            temp_witness_folder_path.join(rust_file_name),
        ));
    }
    for (existing_code_path, generated_code_path) in files_to_compare {
        let existing_code = fs::read_to_string(&existing_code_path).unwrap();
        let generated_code = fs::read_to_string(&generated_code_path).unwrap();
        pretty_assertions::assert_eq!(
            existing_code,
            generated_code,
            r#"
            Generated code in {}.
            is different from the code in {}.
            Run the following  to update the code:
            '$ FIX_CODE=1 cargo test'"#,
            generated_code_path.display(),
            existing_code_path.display(),
        );
    }
}

fn get_constraints_folder_path_suffix(r#type: &TraceType, file_name: &String) -> String {
    if r#type == &TraceType::Inline {
        format!("subroutines/{}", file_name)
    } else {
        file_name.clone()
    }
}

// Removes trailing zeroes from a comma-separated sequence of M31 elements.
// Used to reduce 0 multiplications in the extension field.
pub fn remove_trailing_zeroes(felts: &[CompiledAirVar]) -> Vec<CompiledAirVar> {
    let mut felts = felts.to_vec();
    while felts
        .last()
        .is_some_and(|f| f.eq(&CompiledAirVar::Const("M31".to_string(), "0".to_string())))
    {
        felts.pop();
    }
    felts
}

pub fn get_variable_name(ty: &str, val: &str) -> String {
    format!("{ty}_{val}")
        .replace([',', '<', '['], "_")
        .replace("::", "_")
        .replace(['>', ']', ' ', ':'], "")
        .replace("__", "_")
}

/// Replaces plain generics `<...>` with Turobofish `::<...>`.
/// Used where a function call is needed.
/// E.g. "BigUint<x, y, z>" -> "BigUint::<x, y, z>".
pub fn replace_generics_with_turbofish(ty: &str) -> String {
    ty.replace('<', "::<")
}

pub fn block_doc(msg: &str) -> rust::Tokens {
    quote! {
        $['\n']$("// ")$msg.$['\n']
    }
}

/// To run in FIX mode - '$ FIX_CODE=1 cargo test'
#[cfg(test)]
pub fn compare_contents_or_fix_with_path(
    air_fn: CompiledAirFn,
    constraints_folder_path: &Path,
    witness_folder_path: &Path,
) {
    fs::create_dir_all(witness_folder_path).ok();
    fs::create_dir_all(constraints_folder_path).ok();
    let is_fix_mode = std::env::var("FIX_CODE") == Ok("1".to_string());
    if is_fix_mode {
        dump_component_code(&air_fn, constraints_folder_path, witness_folder_path);
    } else {
        assert_generated_code_unchanged(air_fn, constraints_folder_path, witness_folder_path);
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

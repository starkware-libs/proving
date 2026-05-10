use std::fs;

use air_common::TraceType;
use air_compile::compiled_structs::CompiledAirFn;
use eval_air_fn_constraints::SampleEvaluation;
use tempfile::tempdir;

use crate::supported_components::{AutogenCodeFile, AutogenCodeType};
use crate::utils::{
    add_file_to_module, format_air_fn_code, generate_air_fn_code, generated_code_path,
};

/// To run in FIX mode - '$ FIX_CODE=1 cargo test'
pub fn compare_contents_or_fix_with_path(
    air_fn: &CompiledAirFn,
    sample_evaluation: Option<&SampleEvaluation>,
    job: &AutogenCodeFile,
) {
    fs::create_dir_all(&job.dest_dir).ok();
    let is_fix_mode = std::env::var("FIX_CODE") == Ok("1".to_string());
    if is_fix_mode {
        dump_component_code(air_fn, sample_evaluation, job);
    } else {
        assert_generated_code_unchanged(air_fn, sample_evaluation, job);
    }
}

fn dump_component_code(
    air_fn: &CompiledAirFn,
    sample_evaluation: Option<&SampleEvaluation>,
    job: &AutogenCodeFile,
) {
    // TODO(Gali): handle witness sub-routines.
    if air_fn.r#type == TraceType::Inline && job.code_type == AutogenCodeType::WITNESS {
        return;
    }

    let raw_code = generate_air_fn_code(air_fn, sample_evaluation, job.code_type);
    let code = format_air_fn_code(raw_code, &job.code_type);
    let dest_path = generated_code_path(air_fn, &job.dest_dir, &job.code_type);
    add_file_to_module(dest_path.as_path(), code, &job.code_type);
}

fn assert_generated_code_unchanged(
    air_fn: &CompiledAirFn,
    sample_evaluation: Option<&SampleEvaluation>,
    job: &AutogenCodeFile,
) {
    let temp_dir = tempdir().expect("Could not open temporary folder!");
    let temp_dir = temp_dir.path();
    let new_code_path = temp_dir.join(&air_fn.name);

    let raw_code = generate_air_fn_code(air_fn, sample_evaluation, job.code_type);
    let generated_code = format_air_fn_code(raw_code, &job.code_type);
    fs::write(&new_code_path, &generated_code).expect("Couldn't write temp file");

    let existing_code_path = generated_code_path(air_fn, &job.dest_dir, &job.code_type);
    let existing_code = fs::read_to_string(&existing_code_path)
        .unwrap_or_else(|e| panic!("Cannot read {}: {e}", existing_code_path.display()));
    pretty_assertions::assert_eq!(
        existing_code,
        generated_code,
        r#"
        Generated code in {}.
        is different from the code in {}.
        Run the following  to update the code:
        '$ FIX_CODE=1 cargo test'"#,
        new_code_path.display(),
        existing_code_path.display(),
    );
}

use std::fs;
use std::path::Path;

use air_common::TraceType;
use air_compile::compiled_structs::CompiledAirFn;
use eval_air_fn_constraints::SampleEvaluation;
use tempfile::tempdir;

use super::supported_components::{AutogenCodeFile, AutogenCodeType};
use crate::utils::{
    add_file_to_module, format_air_fn_code, generate_air_fn_code, generated_code_path, load_air_fns,
};

#[test]
fn add_ap_cairo_code_gen() {
    let codegen_jobs = [
        AutogenCodeFile {
            air_fn_name: "add_ap_opcode".to_string(),
            source_path: "../compiled_casm_air/compiled_jsons/opcodes/add_ap_opcode.json".into(),
            dest_dir: "../code_gen_regression/verifier/src/components".into(),
            code_type: AutogenCodeType::CAIRO,
        },
        AutogenCodeFile {
            air_fn_name: "read_small".to_string(),
            source_path: "../compiled_casm_air/compiled_jsons/subroutines/read_small.json".into(),
            dest_dir: "../code_gen_regression/verifier/src/components".into(),
            code_type: AutogenCodeType::CAIRO,
        },
        AutogenCodeFile {
            air_fn_name: "decode_instruction_d2a10".to_string(),
            source_path:
                "../compiled_casm_air/compiled_jsons/subroutines/decode_instruction_d2a10.json"
                    .into(),
            dest_dir: "../code_gen_regression/verifier/src/components".into(),
            code_type: AutogenCodeType::CAIRO,
        },
    ];

    let (compiled_air_fns, sample_evaluations) =
        load_air_fns(Path::new("../compiled_casm_air/"), &codegen_jobs);

    for job in codegen_jobs {
        let air_fn = compiled_air_fns
            .get(&job.air_fn_name)
            .unwrap_or_else(|| panic!("Missing AirFn {}", job.air_fn_name));
        let sample_evaluation = sample_evaluations.get(&job.air_fn_name);
        compare_contents_or_fix_with_path(air_fn, sample_evaluation, &job);
    }
}

#[test]
fn code_gen_regression() {
    let components_to_check = [
        "../compiled_casm_air/compiled_jsons/builtins/mul_mod_builtin.json",
        "../compiled_casm_air/compiled_jsons/builtins/range_check_builtin.json",
        "../compiled_casm_air/compiled_jsons/lookups/partial_ec_mul_window_bits_18.json",
        "../compiled_casm_air/compiled_jsons/lookups/pedersen_points_table_window_bits_18.json",
        "../compiled_casm_air/compiled_jsons/lookups/range_check_20.json",
        "../compiled_casm_air/compiled_jsons/lookups/range_check_9_9.json",
        "../compiled_casm_air/compiled_jsons/lookups/range_check_7_2_5.json",
        "../compiled_casm_air/compiled_jsons/lookups/triple_xor_32.json",
        "../compiled_casm_air/compiled_jsons/lookups/verify_bitwise_xor_8.json",
        "../compiled_casm_air/compiled_jsons/lookups/verify_instruction.json",
        "../compiled_casm_air/compiled_jsons/opcodes/jnz_opcode_taken.json",
    ];

    let inline_air_fns_to_check = [
        "../compiled_casm_air/compiled_jsons/subroutines/decode_instruction_d2a10.json",
        "../compiled_casm_air/compiled_jsons/subroutines/double_karatsuba_1454b.json",
        "../compiled_casm_air/compiled_jsons/subroutines/ec_add.json",
        "../compiled_casm_air/compiled_jsons/subroutines/mem_verify.json",
        "../compiled_casm_air/compiled_jsons/subroutines/verify_add_252.json",
        "../compiled_casm_air/compiled_jsons/subroutines/verify_mul_252.json",
    ];

    // Generate witness code only for component AirFns
    let witness_jobs = components_to_check
        .iter()
        .map(|path_str| {
            let path = Path::new(path_str);
            let air_fn_name = path
                .file_stem()
                .expect("Invalid filename")
                .to_str()
                .expect("Invalid filename");

            AutogenCodeFile {
                air_fn_name: air_fn_name.to_string(),
                source_path: path.into(),
                dest_dir: "../code_gen_regression/witness/src/components".into(),
                code_type: AutogenCodeType::WITNESS,
            }
        })
        .collect::<Vec<_>>();

    // Generate constraint evaluation code for both component and inline AirFns
    let constraint_jobs = inline_air_fns_to_check
        .iter()
        .chain(components_to_check.iter())
        .map(|path_str| {
            let path = Path::new(path_str);
            let air_fn_name = path
                .file_stem()
                .expect("Invalid filename")
                .to_str()
                .expect("Invalid filename");
            AutogenCodeFile {
                air_fn_name: air_fn_name.to_string(),
                source_path: path.into(),
                dest_dir: "../code_gen_regression/cairo_air/src/components".into(),
                code_type: AutogenCodeType::AIR,
            }
        })
        .collect::<Vec<_>>();

    let jobs = [constraint_jobs, witness_jobs].concat();

    let (compiled_air_fns, sample_evaluations) =
        load_air_fns(Path::new("../compiled_casm_air/"), &jobs);

    for job in jobs {
        let air_fn = compiled_air_fns
            .get(&job.air_fn_name)
            .unwrap_or_else(|| panic!("Missing AirFn {}", job.air_fn_name));
        let sample_evaluation = sample_evaluations.get(&job.air_fn_name);
        compare_contents_or_fix_with_path(air_fn, sample_evaluation, &job);
    }
}

/// To run in FIX mode - '$ FIX_CODE=1 cargo test'
fn compare_contents_or_fix_with_path(
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
    let code = format_air_fn_code(raw_code, job.code_type);
    let dest_path = generated_code_path(air_fn, &job.dest_dir, job.code_type);
    add_file_to_module(dest_path.as_path(), code, job.code_type);
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
    let generated_code = format_air_fn_code(raw_code, job.code_type);
    fs::write(&new_code_path, &generated_code).expect("Couldn't write temp file");

    let existing_code_path = generated_code_path(air_fn, &job.dest_dir, job.code_type);
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

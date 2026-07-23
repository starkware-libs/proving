use std::path::Path;

use airs::casm::casm_registry::create_casm_registry_ordered_by_stwo_cairo;
use expect_test::expect_file;
use itertools::Itertools;

use crate::rust::claims::generate_claims_rust_file;
use crate::rust::claims_generator::generate_claim_generator_file;
use crate::rust::components::generate_components_rust_file;
use crate::rust::provers::generate_provers_rust_file;
use crate::supported_components::{AutogenCodeFile, AutogenCodeType};
use crate::test_utils::compare_contents_or_fix_with_path;
use crate::utils::{STWO_CAIRO_AIR_CONFIG, load_air_fns, reformat_rust_code};

#[test]
fn code_gen_regression() {
    let components_to_check = [
        "../../outputs/compiled_casm_air/compiled_jsons/builtins/mul_mod_builtin.json",
        "../../outputs/compiled_casm_air/compiled_jsons/builtins/range_check_builtin.json",
        "../../outputs/compiled_casm_air/compiled_jsons/lookups/partial_ec_mul_window_bits_18.json",
        "../../outputs/compiled_casm_air/compiled_jsons/lookups/\
         pedersen_points_table_window_bits_18.json",
        "../../outputs/compiled_casm_air/compiled_jsons/lookups/range_check_20.json",
        "../../outputs/compiled_casm_air/compiled_jsons/lookups/range_check_9_9.json",
        "../../outputs/compiled_casm_air/compiled_jsons/lookups/range_check_7_2_5.json",
        "../../outputs/compiled_casm_air/compiled_jsons/lookups/triple_xor_32.json",
        "../../outputs/compiled_casm_air/compiled_jsons/lookups/verify_bitwise_xor_8.json",
        "../../outputs/compiled_casm_air/compiled_jsons/lookups/verify_instruction.json",
        "../../outputs/compiled_casm_air/compiled_jsons/opcodes/jnz_opcode_taken.json",
    ];

    let inline_air_fns_to_check = [
        "../../outputs/compiled_casm_air/compiled_jsons/subroutines/decode_instruction_1af1f.json",
        "../../outputs/compiled_casm_air/compiled_jsons/subroutines/double_karatsuba_9cdb9.json",
        "../../outputs/compiled_casm_air/compiled_jsons/subroutines/ec_add.json",
        "../../outputs/compiled_casm_air/compiled_jsons/subroutines/mem_verify.json",
        "../../outputs/compiled_casm_air/compiled_jsons/subroutines/verify_add_252.json",
        "../../outputs/compiled_casm_air/compiled_jsons/subroutines/verify_mul_252.json",
    ];

    // Generate witness code only for component AirFns
    let witness_jobs = components_to_check
        .iter()
        .map(|path_str| {
            let path = Path::new(path_str);
            let air_fn_name =
                path.file_stem().expect("Invalid filename").to_str().expect("Invalid filename");

            AutogenCodeFile {
                air_fn_name: air_fn_name.to_string(),
                source_path: path.into(),
                dest_dir: "../../test_data/code_gen_regression/witness/src/components".into(),
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
            let air_fn_name =
                path.file_stem().expect("Invalid filename").to_str().expect("Invalid filename");
            AutogenCodeFile {
                air_fn_name: air_fn_name.to_string(),
                source_path: path.into(),
                dest_dir: "../../test_data/code_gen_regression/cairo_air/src/components".into(),
                code_type: AutogenCodeType::AIR(STWO_CAIRO_AIR_CONFIG),
            }
        })
        .collect::<Vec<_>>();

    let jobs = [constraint_jobs, witness_jobs].concat();

    let (compiled_air_fns, sample_evaluations) =
        load_air_fns(Path::new("../../outputs/compiled_casm_air/"), &jobs);

    for job in jobs {
        let air_fn = compiled_air_fns
            .get(&job.air_fn_name)
            .unwrap_or_else(|| panic!("Missing AirFn {}", job.air_fn_name));
        let sample_evaluation = sample_evaluations.get(&job.air_fn_name);
        compare_contents_or_fix_with_path(air_fn, sample_evaluation, &job);
    }
}

#[test]
fn test_generate_claims_generator() {
    let compiled_registry = create_casm_registry_ordered_by_stwo_cairo();
    let generated_code = generate_claim_generator_file(&compiled_registry);
    let code_string = generated_code.to_string().unwrap();
    let formatted_code = reformat_rust_code(code_string);
    expect_file!["../../../../test_data/code_gen_regression/witness/src/claims_generator.rs"]
        .assert_eq(&formatted_code);
}

#[test]
fn test_generate_claims_rust() {
    let compiled_registry = create_casm_registry_ordered_by_stwo_cairo();
    let generated_code = generate_claims_rust_file(&compiled_registry);
    let code_string = generated_code.to_string().unwrap();
    let formatted_code = reformat_rust_code(code_string);
    expect_file!["../../../../test_data/code_gen_regression/cairo_air/src/claims.rs"]
        .assert_eq(&formatted_code);
}

#[test]
fn test_generate_components_rust() {
    let compiled_registry = create_casm_registry_ordered_by_stwo_cairo();
    let generated_code = generate_components_rust_file(&compiled_registry);
    let code_string = generated_code.to_string().unwrap();
    let formatted_code = reformat_rust_code(code_string);
    expect_file!["../../../../test_data/code_gen_regression/cairo_air/src/components.rs"]
        .assert_eq(&formatted_code);
}

#[test]
fn test_generate_provers_rust() {
    let compiled_registry = create_casm_registry_ordered_by_stwo_cairo();
    let generated_code = generate_provers_rust_file(&compiled_registry.keys().collect_vec());
    let code_string = generated_code.to_string().unwrap();
    let formatted_code = reformat_rust_code(code_string);
    expect_file!["../../../../test_data/code_gen_regression/cairo_air/src/provers.rs"]
        .assert_eq(&formatted_code);
}
